# CI/CD Security Hardening: Woodpecker → Kubernetes RBAC

This document covers the full security roadmap for locking down how the Woodpecker CI pipeline
interacts with the K3s cluster. It is split into two phases: what is **done now** (Phase 1) and
what to do when the platform scales to a point where the investment is justified (Phase 2).

---

## Why This Matters

### The Current Problem (Pre-Phase 1)

The Woodpecker deploy step mounted the host's K3s admin kubeconfig directly:

```yaml
volumes:
  - /etc/rancher/k3s/k3s.yaml:/kubeconfig.yaml:ro
```

That file contains a **cluster-admin certificate** — the highest privilege level in Kubernetes.
Every single pipeline run, every step container, had the ability to:

- Delete any namespace, including `kube-system`
- Read all Secrets across the entire cluster (database passwords, SMTP tokens, Sealed Secret keys)
- Modify RBAC rules to escalate further

If someone pushed malicious code that got merged to `uat`, or if the Woodpecker agent itself was
compromised, the blast radius was the entire cluster and every secret in it.

### What Good Looks Like

The CI pipeline should only be able to do exactly what it needs to deploy:
- `patch` and `get` `deployments` in `atlas-platform`
- `get` and `list` `pods` and `replicasets` in `atlas-platform` (for rollout status)
- Nothing else. Anywhere.

---

## Phase 1 — Scoped ServiceAccount (DONE)

> **Status: Implemented.** Applied to the live cluster on 2026-05-09.
> Manifest committed to `k8s/base/woodpecker-rbac.yaml`.

### What was done

Created a `woodpecker-deployer` ServiceAccount in the `atlas-platform` namespace with a tightly
scoped Role:

```yaml
rules:
  - apiGroups: ["apps"]
    resources: ["deployments"]
    verbs: ["get", "list", "patch", "update"]
  - apiGroups: ["apps"]
    resources: ["replicasets"]
    verbs: ["get", "list"]
  - apiGroups: [""]
    resources: ["pods", "pods/log"]
    verbs: ["get", "list"]
```

Verified with `kubectl auth can-i`:

| Action | Allowed? |
|---|---|
| `patch deployments` in `atlas-platform` | ✅ Yes |
| `delete namespaces` | ❌ No |
| `get secrets` in `kube-system` | ❌ No |
| `get secrets` in `atlas-platform` | ❌ No |

### How to wire the pipeline to use this token (TODO)

The pipeline still uses the volume-mounted kubeconfig. To fully activate Phase 1:

1. Extract the token and CA from the cluster:
   ```bash
   # Token
   kubectl get secret woodpecker-deployer-token -n atlas-platform \
     -o jsonpath='{.data.token}' | base64 -d

   # CA cert (already base64-encoded, store as-is)
   kubectl get secret woodpecker-deployer-token -n atlas-platform \
     -o jsonpath='{.data.ca\.crt}'

   # Server (use the public IP, not 127.0.0.1)
   # Value: https://69.164.248.38:6443
   ```

2. Add three new Woodpecker CI secrets at `ci.oply.co`:
   - `KUBE_DEPLOY_TOKEN` — the decoded bearer token
   - `KUBE_DEPLOY_CA` — the base64-encoded CA cert
   - `KUBE_SERVER` — `https://69.164.248.38:6443`

3. Replace the volume mount in `.woodpecker.yml`'s `deploy_platform_k8s` step:

   ```yaml
   # Remove this:
   volumes:
     - /etc/rancher/k3s/k3s.yaml:/kubeconfig.yaml:ro

   # Add this to the commands block instead:
   - |
     mkdir -p /tmp/kube
     echo "${KUBE_DEPLOY_CA}" | base64 -d > /tmp/kube/ca.crt
     kubectl config set-cluster atlas \
       --server="${KUBE_SERVER}" \
       --certificate-authority=/tmp/kube/ca.crt
     kubectl config set-credentials woodpecker-deployer \
       --token="${KUBE_DEPLOY_TOKEN}"
     kubectl config set-context atlas \
       --cluster=atlas --user=woodpecker-deployer \
       --namespace=atlas-platform
     kubectl config use-context atlas
     export KUBECONFIG=/root/.kube/config
   ```

4. Add the three secrets to the `environment:` block of `deploy_platform_k8s`:
   ```yaml
   environment:
     KUBE_DEPLOY_TOKEN:
       from_secret: KUBE_DEPLOY_TOKEN
     KUBE_DEPLOY_CA:
       from_secret: KUBE_DEPLOY_CA
     KUBE_SERVER:
       from_secret: KUBE_SERVER
   ```

> **Note:** The `kubectl apply -k` step that applies config/secrets still needs broader access
> (it reads/writes ConfigMaps and Secrets). Until Phase 2, keep a **separate restricted secret**
> for that step or accept that config-apply still uses the admin kubeconfig for now. The image
> rollout step is the higher-risk operation.

---

## Phase 2 — Full Kubernetes Backend for the Woodpecker Agent

> **Status: Future work. Implement when any of these are true:**
> - The platform moves to multi-node K3s (i.e., worker nodes are added)
> - CI build times become a bottleneck and you want parallel job scheduling across nodes
> - You need per-pipeline resource quotas (e.g., cap WASM builds to 8 CPU / 16GB RAM)
> - A security audit requires zero host-level access for CI workloads
> - The team grows and you need namespace-isolated CI environments per team

### What it means

Currently the Woodpecker agent is a **systemd service on the bare-metal host** (`WOODPECKER_BACKEND = "docker"`).
Each pipeline step runs as a container via Podman on the host.

With the Kubernetes backend (`WOODPECKER_BACKEND = "kubernetes"`), the agent submits each
pipeline **step as a Kubernetes Job**. Steps run as pods inside the cluster with a proper
`ServiceAccount`, not as host-privileged Podman containers.

### Benefits vs. current setup

| | Docker Backend (current) | Kubernetes Backend (future) |
|---|---|---|
| Steps run as | Podman containers on host | K8s Jobs (pods) |
| K8s access | Mounted host kubeconfig (cluster-admin) | ServiceAccount token (scoped) |
| Resource limits | None enforced | CPU/Memory quotas via K8s LimitRange |
| Parallelism | Sequential on one host | Scheduled across all nodes |
| Privileged builds | Via Podman socket | Via `securityContext.privileged: true` on Job pod |
| Audit trail | Woodpecker logs only | Woodpecker logs + K8s audit log |

### Changes required

#### In NixForge `flake.nix`

```nix
# Current agent config (lines ~703-714)
services.woodpecker-agents.agents."dagger-runner" = {
  environment = {
    WOODPECKER_SERVER = "127.0.0.1:9000";
    WOODPECKER_BACKEND = "docker";                          # ← change
    DOCKER_HOST = "unix:///run/podman/podman.sock";         # ← remove
    WOODPECKER_HEALTHCHECK_ADDR = ":3001";
  };
  extraGroups = [ "podman" ];                              # ← remove
};

# Target agent config
services.woodpecker-agents.agents."dagger-runner" = {
  environment = {
    WOODPECKER_SERVER = "127.0.0.1:9000";
    WOODPECKER_BACKEND = "kubernetes";                     # ← new
    WOODPECKER_BACKEND_K8S_NAMESPACE = "woodpecker-agents"; # ← new
    WOODPECKER_BACKEND_K8S_STORAGE_CLASS = "";             # ← use emptyDir
    WOODPECKER_BACKEND_K8S_PULL_SECRET_NAMES = "ghcr-login-secret";
    WOODPECKER_BACKEND_K8S_PRIVILEGED_PLUGINS = "woodpeckerci/plugin-docker-buildx";
    WOODPECKER_HEALTHCHECK_ADDR = ":3001";
    KUBECONFIG = "/etc/rancher/k3s/k3s.yaml";             # ← agent needs this to submit Jobs
  };
};
```

#### New K8s manifests required

1. **`woodpecker-agents` namespace** — isolated from `atlas-platform`
2. **`ServiceAccount` + `Role` for agent** — ability to create/delete Jobs, Pods in `woodpecker-agents`
3. **`ghcr-login-secret`** in `woodpecker-agents` — so step pods can pull private images
4. **`LimitRange`** in `woodpecker-agents` — cap CPU/memory per step pod

#### One gotcha: Docker-in-Docker for image builds

The `woodpeckerci/plugin-docker-buildx` steps need **privileged mode** to run Docker inside the
pod. K3s must have `--allow-privileged=true` (it does by default) and the Job pod spec must set:

```yaml
securityContext:
  privileged: true
```

Woodpecker handles this automatically for plugins listed in `WOODPECKER_BACKEND_K8S_PRIVILEGED_PLUGINS`.

#### Deployment steps (when the time comes)

1. Deploy NixForge change: `colmena apply --on manager`
2. Apply new K8s manifests: `kubectl apply -k k8s/ci/`
3. Verify the agent reconnects and submits a test Job: `kubectl get jobs -n woodpecker-agents -w`
4. Validate a full pipeline run completes successfully
5. Remove the `k3s.yaml` volume mount from `.woodpecker.yml` (agent now uses its own ServiceAccount)

---

## Reliability Improvements (Separate from Security)

These changes directly reduce the "go check the git hash and SSH to verify" toil:

| Improvement | Status | Effect |
|---|---|---|
| `CI_COMMIT_CHANGED_FILES` in pipeline | ✅ Done | Pipeline correctly scopes which pods update per push — no more stale deployments or `ImagePullBackOff` |
| Stable image tags in base manifests | ✅ Done | Manifests always contain real pullable SHAs, never placeholder strings |
| Version chip in platform-admin sidebar | ✅ Done (pending build) | Shows running SHA + colour-coded UAT/PROD/DEV badge without SSH |
| Scoped ServiceAccount | ✅ Done (RBAC) | Security only — no direct reliability impact |
| Full K8s backend | ⏳ Future | Security + parallelism — no direct reliability impact |

The version chip is the direct answer to the "which version is live" question. Once the
`platform-admin` build ships, you'll see the exact 8-char SHA and environment name in the sidebar
footer on every page of the admin. No SSH required.

---

## Related Documentation

- [`adding_a_new_app.md`](./adding_a_new_app.md) — checklist for adding new services to the pipeline
- [`deployment_environments.md`](./deployment_environments.md) — UAT vs prod environment config
- [NixForge `flake.nix`](../../NixForge/flake.nix) — bare-metal server configuration (Woodpecker server + agent)
