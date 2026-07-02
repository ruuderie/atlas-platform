# CI/CD Security Hardening: Woodpecker → Kubernetes RBAC

This document covers the full security architecture for the Woodpecker CI pipeline's interaction
with the K3s cluster. It reflects the **current live state** as of 2026-07-02.

---

## What Was Wrong (Before Hardening)

The Woodpecker deploy step mounted the host's K3s admin kubeconfig directly:

```yaml
volumes:
  - /etc/rancher/k3s/k3s.yaml:/kubeconfig.yaml:ro
```

That file contains a **cluster-admin certificate** — the highest privilege level in Kubernetes.
Every pipeline run had the ability to delete any namespace, read all secrets cluster-wide, and
modify RBAC to escalate further. A compromised `uat` push would have been a full cluster breach.

---

## Current Architecture (Phase 1 — Live)

### ServiceAccount model

A `woodpecker-deployer` ServiceAccount lives in `atlas-uat` namespace. It is the **single identity**
used by the CI pipeline for all deployments. It has cross-namespace access via RoleBindings in
each environment namespace — but only the verbs it actually needs.

```
atlas-uat   ← woodpecker-deployer SA lives here
atlas-dev   ← RoleBinding grants atlas-uat SA access here
atlas-prod  ← (when provisioned) same pattern
```

### Role permissions (per namespace)

```yaml
rules:
  - apiGroups: ["apps"]
    resources: ["deployments"]
    verbs: ["get", "list", "watch", "create", "patch", "update"]
  - apiGroups: ["apps"]
    resources: ["replicasets"]
    verbs: ["get", "list"]
  - apiGroups: [""]
    resources: ["pods", "pods/log"]
    verbs: ["get", "list"]
  - apiGroups: [""]
    resources: ["configmaps", "secrets", "services"]
    verbs: ["get", "list", "create", "update", "patch"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["ingresses"]
    # watch+delete required so the deployer can apply ingress-sidecar-role which
    # grants those same verbs (Kubernetes RBAC escalation prevention).
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: ["cert-manager.io"]
    resources: ["certificates"]
    verbs: ["get", "list", "watch", "delete"]
  - apiGroups: ["rbac.authorization.k8s.io"]
    resources: ["roles", "rolebindings"]
    verbs: ["get", "list", "create", "update", "patch", "escalate", "bind"]
  - apiGroups: [""]
    resources: ["namespaces"]
    verbs: ["get"]
```

### Verified RBAC boundaries

| Action | Allowed? |
|---|---|
| `patch deployments` in `atlas-uat` | ✅ Yes |
| `patch deployments` in `atlas-dev` | ✅ Yes |
| `get secrets` in `kube-system` | ❌ No |
| `delete namespaces` | ❌ No |
| `get/patch roles` or `rolebindings` in own namespace | ✅ Yes (for ingress-sidecar SA bootstrap) |
| `get/create serviceaccounts` in own namespace | ✅ Yes (for ingress-sidecar SA bootstrap) |
| `get/patch roles` or `rolebindings` in other namespaces | ❌ No |

### Pipeline kubeconfig construction

The pipeline builds an ephemeral kubeconfig at runtime from three Woodpecker CI secrets:

```bash
mkdir -p /tmp/kube
export KUBECONFIG=/tmp/kube/config
echo "$KUBE_DEPLOY_CA" | base64 -d > /tmp/kube/ca.crt
kubectl config set-cluster atlas \
  --server="$KUBE_SERVER" \
  --certificate-authority=/tmp/kube/ca.crt \
  --embed-certs=true
kubectl config set-credentials woodpecker-deployer \
  --token="$KUBE_DEPLOY_TOKEN"
kubectl config set-context atlas \
  --cluster=atlas --user=woodpecker-deployer \
  --namespace=atlas-uat
kubectl config use-context atlas
```

No host files mounted. No cluster-admin credentials. The token is scoped to exactly what the
pipeline needs and nothing more.

### Strict branch → namespace gate

The deploy step enforces a hard mapping — any unlisted branch fails immediately:

```bash
case "$CI_COMMIT_BRANCH" in
  uat)          NAMESPACE="atlas-uat" ;;
  dev)          NAMESPACE="atlas-dev" ;;
  main|master)
    echo "ERROR: Production deployment is not yet enabled."
    exit 1
    ;;
  *)
    echo "ERROR: Branch '$CI_COMMIT_BRANCH' is not authorised to deploy."
    exit 1
    ;;
esac
```

Additionally, `main` and `master` are **excluded from the global pipeline trigger** in
`.woodpecker.yml`, so the deploy step never runs at all on those branches until production
is explicitly enabled.

### What the CI pipeline does NOT manage

These resources are applied **once by a cluster admin** and are never touched by CI:

| Resource | Why admin-only |
|---|---|
| `Namespace` (`atlas-uat`, `atlas-dev`) | SA lacks permission to patch namespaces |
| `woodpecker-rbac.yaml` | ✅ **Bootstrapped via `bootstrap_rbac` CI step (2026-07-02).** The step uses the host K3s admin kubeconfig (mounted read-only) with the server IP rewritten from `127.0.0.1` to the container gateway IP. TLS verification is skipped for this step only (SAN mismatch — K3s cert doesn't cover the bridge IP). The step is safe to remove once both `bootstrap_rbac` and `deploy_platform_k8s` have passed in the same pipeline run. |
| `ghcr-login-secret` | Registry credentials — rotated out-of-band |
| `cloudflare-api-token-secret` | Cloudflare API token in `cert-manager` namespace — must have `Zone:DNS:Edit` + `Zone:Zone:Read` permissions |

The kustomize overlays intentionally exclude these resources. The CI SA only manages:
ConfigMaps, Secrets (app-secrets only), Services, Deployments, Ingresses, and the `ingress-sidecar` RBAC (via `k8s/base/ingress-sidecar-rbac.yaml`).

### Ingress Sidecar Architecture

The `ingress-sidecar` is a standalone Rust binary (`backend/src/bin/ingress_sidecar.rs`) deployed as its own Kubernetes Deployment. It runs on port `8085` and exposes:

| Endpoint | Purpose |
|---|---|
| `GET /health` | Readiness probe |
| `POST /api/ingress/provision` | Creates K8s Ingress + cert-manager TLS for a new tenant domain |
| `POST /api/ingress/deprovision` | Deletes the Ingress for an offboarded tenant |

Its `ingress-sidecar-sa` ServiceAccount is bound to a Role that only allows `get/list/create/patch/delete` on `ingresses` within its own namespace. It uses **`rustls` with the `ring` crypto provider** — the provider must be explicitly installed at startup via `rustls::crypto::ring::default_provider().install_default()` before any TLS connections are made.

> **Why standalone?** Embedding it as a sidecar in the backend pod caused every backend rollout to fail with `ImagePullBackOff` if the sidecar image hadn't been built yet. Standalone deployments have independent lifecycles.

---

## Automated Domain Provisioning (Enabled by RBAC Bootstrap)

With the `woodpecker-deployer` RBAC bootstrapped into both `atlas-dev` and `atlas-uat`, the
full automated domain provisioning loop is now operational. This is the feature the RBAC was
blocking: **platform-admin can create and configure tenant instances with live domains and
auto-issued TLS certificates, entirely from the UI.**

### What platform-admin can now do

From the **Internal Instances** page (`/internal-instances`), a platform operator can:

1. **Create a new instance** — enters a name, optional custom domain, app type (Folio / Anchor /
   Meridian), and purpose label. If no domain is provided, one is auto-generated as
   `{name}.dev.atlas.oply.co` (covered by the platform wildcard cert — no DNS action needed).

2. **Assign or change a domain** on an existing instance — from the instance's Config tab,
   entering either a platform subdomain or a custom client domain and clicking "Save & Provision".

3. **Get exact DNS instructions** — for custom client domains, the UI immediately renders the
   exact CNAME/A record the client must add, with a Cloudflare-specific note to set DNS-only
   (grey cloud) mode so cert-manager's HTTP-01 challenge can complete.

### Two SSL paths

| Domain type | Example | TLS handling | Operator action |
|---|---|---|---|
| Platform wildcard | `demo.dev.atlas.oply.co` | `*.dev.atlas.oply.co` wildcard cert — already active | None — works immediately |
| Custom client domain | `app.clientco.com` | cert-manager HTTP-01 via Let's Encrypt | Client adds CNAME/A record; cert auto-issues within ~60 s |

### Provisioning flow (end to end)

```
platform-admin UI
  → POST /api/admin/instances/:id/domain  (backend REST)
  → ingress-sidecar POST /api/ingress/provision
  → kubectl creates Ingress + cert-manager Certificate in namespace
  → K3s Traefik picks up Ingress (watches the API server)
  → Let's Encrypt issues cert (HTTP-01 challenge via Traefik)
  → HTTPS live at the domain
```

The `woodpecker-deployer` RBAC grants the ingress-sidecar's ServiceAccount permission to
`create/patch/delete` Ingress objects and `delete` cert-manager Certificates (for deprovision
cleanup). Without the `bootstrap_rbac` step having applied `k8s/base/woodpecker-rbac.yaml`
to both namespaces, every attempt to provision a domain would have returned `403 Forbidden`.

### What is NOT automated (operator handles once per cluster)

| Item | Why manual |
|---|---|
| Cloudflare wildcard DNS (`*.dev.atlas.oply.co → cluster IP`) | Set once at registrar; doesn't change |
| `cloudflare-api-token-secret` in `cert-manager` namespace | Security — rotated out-of-band |
| Namespace creation | SA lacks namespace CRUD permission (intentional) |

---

## Environment Namespace Map

| Branch | Namespace | Database | Status |
|---|---|---|---|
| `uat` | `atlas-uat` | `atlas_uat` | ✅ Active |
| `dev` | `atlas-dev` | `atlas_dev` | ✅ Active |
| `main`/`master` | `atlas-prod` *(not yet created)* | `atlas_prod` | 🔒 Blocked — not live |

### Admin prerequisites for each namespace

When provisioning a new environment namespace (run once, by a human with cluster-admin):

```bash
# 1. Create namespace
kubectl create namespace <ns>

# 2. Copy pull secret
kubectl get secret ghcr-login-secret -n atlas-uat -o json \
  | sed 's/"namespace": "atlas-uat"/"namespace": "<ns>"/' \
  | kubectl create -f -

# 3. Apply RBAC (SA lives in atlas-uat, RoleBinding grants cross-namespace access)
kubectl apply -f k8s/base/woodpecker-rbac.yaml   # with namespace: <ns> in kustomize
```

The `KUBE_DEPLOY_TOKEN` Woodpecker secret does not change — the same SA token from
`atlas-uat` covers all namespaces it has RoleBindings in.

---

## Impact on New Projects

The `woodpecker-deployer` SA is **namespace-scoped**. A new product in its own namespace
gets a `403 Forbidden` until a RoleBinding is applied there.

For a new project namespace, apply the same RBAC pattern (Role + RoleBinding referencing
the `atlas-uat` SA). No new token is needed — the existing `KUBE_DEPLOY_TOKEN` will work
once the RoleBinding exists.

> **This is intentional.** A pipeline for project A should never be able to roll out pods
> in project B's namespace. The scoping enforces that boundary automatically.

---

## Phase 2 — Full Kubernetes Backend for the Woodpecker Agent

> **Status: Future work. Implement when any of these are true:**
> - The platform moves to multi-node K3s (worker nodes added)
> - CI build times become a bottleneck requiring parallel scheduling
> - Per-pipeline resource quotas are needed (e.g., cap WASM builds to 8 CPU)
> - A security audit requires zero host-level access for CI workloads

Currently the Woodpecker agent is a **systemd service on the bare-metal host** (`WOODPECKER_BACKEND = "docker"`).
Each pipeline step runs as a Podman container on the host.

With the Kubernetes backend (`WOODPECKER_BACKEND = "kubernetes"`), the agent submits each
pipeline step as a Kubernetes Job. Steps run as pods with a proper ServiceAccount.

### Changes required in NixForge `flake.nix`

```nix
# Current agent config
services.woodpecker-agents.agents."dagger-runner" = {
  environment = {
    WOODPECKER_SERVER = "127.0.0.1:9000";
    WOODPECKER_BACKEND = "docker";                         # ← change
    DOCKER_HOST = "unix:///run/podman/podman.sock";        # ← remove
    WOODPECKER_HEALTHCHECK_ADDR = ":3001";
  };
  extraGroups = [ "podman" ];                             # ← remove
};

# Target agent config
services.woodpecker-agents.agents."dagger-runner" = {
  environment = {
    WOODPECKER_SERVER = "127.0.0.1:9000";
    WOODPECKER_BACKEND = "kubernetes";                    # ← new
    WOODPECKER_BACKEND_K8S_NAMESPACE = "woodpecker-agents";
    WOODPECKER_BACKEND_K8S_STORAGE_CLASS = "";            # use emptyDir
    WOODPECKER_BACKEND_K8S_PULL_SECRET_NAMES = "ghcr-login-secret";
    WOODPECKER_BACKEND_K8S_PRIVILEGED_PLUGINS = "woodpeckerci/plugin-docker-buildx";
    WOODPECKER_HEALTHCHECK_ADDR = ":3001";
    KUBECONFIG = "/etc/rancher/k3s/k3s.yaml";            # agent needs this to submit Jobs
  };
};
```

### Deployment steps (when the time comes)

1. Deploy NixForge change: `colmena apply --on manager`
2. Apply new K8s manifests: `kubectl apply -k k8s/ci/`
3. Verify the agent reconnects: `kubectl get jobs -n woodpecker-agents -w`
4. Validate a full pipeline run completes successfully

---

## Reliability Improvements

| Improvement | Status | Effect |
|---|---|---|
| Scoped ServiceAccount token (no host kubeconfig mount) | ✅ Done | Eliminates cluster-admin blast radius |
| Strict branch → namespace gate | ✅ Done | Only `uat`/`dev` branches can deploy; `main` explicitly blocked |
| Scoped image rollout (path-gated) | ✅ Done | Only services with changed source files get new images; config-only commits skip rollouts |
| Isolated namespaces per environment | ✅ Done | UAT (`atlas-uat`) and DEV (`atlas-dev`) are fully isolated |
| Ingress-sidecar as standalone Deployment | ✅ Done | Backend rollouts no longer blocked by sidecar image availability |
| TLS via DNS-01 (Cloudflare) | ✅ Done | Certs issue correctly behind Cloudflare's HTTPS redirect |
| `bootstrap_rbac` CI step | ✅ Done (2026-07-02) | `woodpecker-deployer` RBAC applied to `atlas-dev` + `atlas-uat`; enables automated domain provisioning from platform-admin |
| Automated domain provisioning from platform-admin UI | ✅ Enabled (2026-07-02) | Platform operators can create tenant instances and assign domains with auto-TLS from the browser; no kubectl access required |
| Version chip in platform-admin sidebar | ✅ Done (pending build) | Shows running SHA + environment badge without SSH |
| Full K8s backend for agent | ⏳ Future | Security + parallelism |

---

## Related Documentation

- [`adding_a_new_app.md`](./architecture/adding_a_new_app.md) — checklist for adding new services to the pipeline
- [`deployment_environments.md`](./deployment_environments.md) — environment config reference
- [NixForge `flake.nix`](../../NixForge/flake.nix) — bare-metal server config (Woodpecker server + agent, PostgreSQL databases)
- [`k8s/base/woodpecker-rbac.yaml`](../k8s/base/woodpecker-rbac.yaml) — RBAC manifest (admin-applied, not managed by CI)
