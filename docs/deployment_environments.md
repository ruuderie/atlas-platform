# Atlas Platform: Deployment & Environments Architecture

This document outlines how the Atlas Platform is configured across different deployment environments (DEV, UAT, Production) and how the CI/CD pipeline correctly routes traffic, provisions databases, and manages secrets.

## Overview

The Atlas Platform uses **Kustomize** to manage environment-specific Kubernetes configurations (`k8s/overlays/dev`, `k8s/overlays/uat`, `k8s/overlays/prod`), and **Woodpecker CI** to automatically apply the correct overlay based on the git branch being pushed.

### Environment Matrix

| Component | DEV (`dev` branch) | UAT (`uat` branch) | Production (`main`/`master` branch) |
| :--- | :--- | :--- | :--- |
| **K8s Namespace** | `atlas-dev` | `atlas-uat` | `atlas-prod` |
| **K8s Overlay** | `k8s/overlays/dev` | `k8s/overlays/uat` | `k8s/overlays/prod` |
| **Database** | `atlas_dev` | `atlas_uat` | `atlas_prod` |
| **API Domain** | `api.dev.atlas.oply.co` | `api.uat.atlas.oply.co` | `api.atlas.oply.co` |
| **Admin Domain** | `dev.atlas.oply.co` | `uat.atlas.oply.co` | `atlas.oply.co` |
| **Tenant Domain** | `dev.buildwithruud.com` | `uat.buildwithruud.com` | `buildwithruud.com` |
| **TLS Issuer** | `letsencrypt-cloudflare` (DNS-01) | `letsencrypt-cloudflare` (DNS-01) | `letsencrypt-cloudflare` (DNS-01) |
| **Secrets Strategy**| CI-injected plaintext | CI-injected plaintext | **Bitnami Sealed Secrets** |

### Core Services

Each environment runs these Kubernetes Deployments:

| Service | Image | Port | Purpose |
|---|---|---|---|
| `backend` | `ghcr.io/ruuderie/atlas-backend` | 8000 | Rust API server (Axum + SeaORM) |
| `anchor-app` | `ghcr.io/ruuderie/anchor-app` | 80 | Tenant-facing Leptos SSR frontend |
| `platform-admin` | `ghcr.io/ruuderie/platform-admin` | 80 | Operator admin panel (Leptos CSR) |
| `network-instance` | `ghcr.io/ruuderie/network-instance` | 80 | Multi-tenant network frontend |
| `ingress-sidecar` | `ghcr.io/ruuderie/atlas-ingress-sidecar` | 8085 | Zero-touch K8s Ingress provisioner |

> **Note:** The `ingress-sidecar` is a **standalone Deployment**, not a sidecar container in the backend pod. It manages Ingress and TLS provisioning for newly onboarded tenants via the `/api/ingress/provision` endpoint.

---

## 1. Woodpecker CI/CD Pipeline

The `.woodpecker.yml` pipeline is the brain that orchestrates deployments. When a push occurs, it determines the target environment based on the branch name.

* **Branch `dev`** → Deploys to the **DEV** environment (`atlas-dev` namespace).
* **Branch `uat`** → Deploys to the **UAT** environment (`atlas-uat` namespace).
* **Branch `main` or `master`** → Reserved for **Production** (`atlas-prod` namespace), but not yet enabled/authorized in the pipeline workflow.

### Injection & Validation

Before executing `kubectl apply`, Woodpecker performs safety checks:
1. **Secret Validation:** It verifies that all necessary CI secrets (`DB_USER`, `DB_PASSWORD`, `SMTP_TOKEN`, etc.) exist in the pipeline environment. If any are missing, the pipeline **fails loudly** instead of deploying a broken state.
2. **Database Provisioning:** For production, it idempotently ensures the `atlas_prod` database exists on the bare-metal Postgres host before starting the pods.
3. **Environment Injection:** It uses `sed` to inject these secrets into the K8s manifests (like `config.yaml` and `backend-patch.yaml`) right before applying them.

---

## 2. Environment Configurations (Kustomize)

The base K8s manifests live in `k8s/base`. The environment-specific rules live in `k8s/overlays/<env>`.

### The `config.yaml`

This ConfigMap dictates how the backend application behaves at runtime. The backend Rust application (`atlas-backend`) boots up and reads these environment variables:

**Production (`k8s/overlays/prod/config.yaml`):**
```yaml
ENVIRONMENT: "prod"
DATABASE_NAME: "atlas_prod"
API_URL: "https://api.atlas.oply.co"
FRONTEND_URL: "https://network.atlas.oply.co"
ADMIN_URL: "https://atlas.oply.co"
```

**UAT (`k8s/overlays/uat/config.yaml`):**
```yaml
ENVIRONMENT: "uat"
DATABASE_NAME: "atlas_uat"
API_URL: "https://api.uat.atlas.oply.co"
FRONTEND_URL: "https://network.uat.atlas.oply.co"
ADMIN_URL: "https://uat.atlas.oply.co"
```

### Application Connection Flow
When the `atlas-backend` pod starts, it grabs `DATABASE_NAME` (e.g., `atlas_prod`) and combines it with `DB_USER` and `DB_PASSWORD` (injected via secrets) to form the connection string. This is why the application naturally connects to the correct database without hardcoded credentials.

---

## 3. Traffic Routing (Ingress)

The `ingress.yaml` file in each overlay determines which domains map to which Kubernetes services.

**In DEV (`k8s/overlays/dev/ingress.yaml`):**
* `dev.buildwithruud.com` → `anchor-app` (port 80)
* `api.dev.atlas.oply.co` → `backend` (port 8000)
* `dev.atlas.oply.co` → `platform-admin` (port 80)
* `network.dev.atlas.oply.co` → `network-instance` (port 80)

**In Production (`k8s/overlays/prod/ingress.yaml`):**
* `buildwithruud.com` and `www.buildwithruud.com` → `anchor-app`
* `api.atlas.oply.co` → `backend` (port 8000)
* `atlas.oply.co` → `platform-admin`

### TLS Certificate Issuance — DNS-01 Required

> **⚠️ Critical:** All ingress files **must** use `cert-manager.io/cluster-issuer: "letsencrypt-cloudflare"` (DNS-01), **not** `letsencrypt-prod` (HTTP-01).

**Why:** Cloudflare's "Always Use HTTPS" setting redirects all `http://` traffic to `https://` before Let's Encrypt's ACME HTTP-01 validator can reach the challenge token. This causes challenges to remain `pending` indefinitely.

DNS-01 validates via Cloudflare DNS TXT records instead, completely bypassing the HTTP redirect issue.

### Cloudflare API Token Requirements

The `letsencrypt-cloudflare` ClusterIssuer uses a token stored in:
```
Secret: cloudflare-api-token-secret
Namespace: cert-manager
Key: api-token
```

This token **must** have the following Cloudflare permissions:

| Permission | Level | Why |
|---|---|---|
| `Zone → DNS → Edit` | All zones (or specific zones) | Create/delete ACME challenge TXT records |
| `Zone → Zone → Read` | All zones | Look up Zone IDs by domain name |

To verify the token is working:
```bash
# Test authentication
curl -s -X GET "https://api.cloudflare.com/client/v4/user/tokens/verify" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Test DNS edit access to a specific zone
curl -s "https://api.cloudflare.com/client/v4/zones/ZONE_ID/dns_records?type=TXT&per_page=1" \
  -H "Authorization: Bearer YOUR_TOKEN"
# Should return: "success":true  (not authentication error)
```

To update the token on the cluster:
```bash
kubectl create secret generic cloudflare-api-token-secret \
  --from-literal=api-token=YOUR_NEW_TOKEN \
  -n cert-manager \
  --dry-run=client -o yaml | kubectl apply -f -
```

After updating the token, force cert-manager to retry immediately:
```bash
kubectl delete challenges --all -n atlas-dev
kubectl delete challenges --all -n atlas-uat
```

---

## 4. Secrets Management (Sealed Secrets)

There is a strict architectural difference in how secrets are handled between environments.

* **UAT** allows Woodpecker CI to inject sensitive tokens directly into standard K8s `Secret` manifests via string substitution (`sed`). This is acceptable for non-production environments to allow rapid iteration.
* **Production** enforces the use of **Bitnami Sealed Secrets**. The `k8s/overlays/prod/sealed-secret.yaml` is intentionally left as an empty placeholder to act as a **safety blocker**.

### How to unblock the first Production Deployment
If you attempt to deploy to production without configuring the sealed secrets, Kubernetes will fail to mount the `app-secrets` volume, and the pods will not start. 

To resolve this and deploy safely:
1. Create a standard Kubernetes `Secret` manifest containing the actual production credentials (Stripe keys, Prod DB password, SMTP tokens). **DO NOT COMMIT THIS FILE.**
2. Encrypt it using the `kubeseal` CLI tool:
   ```bash
   kubeseal --format yaml < my-real-prod-secret.yaml > k8s/overlays/prod/sealed-secret.yaml
   ```
3. Commit the resulting `sealed-secret.yaml` to the repository. It is cryptographically secure and safe to store in Git. Only the Sealed Secrets controller inside the K3s cluster has the private key to decrypt it back into a usable `Secret`.

---

## 5. NixForge Database Architecture

The actual PostgreSQL instance is managed declaratively by the `NixForge` repository on the bare-metal host (not inside Kubernetes, to maximize NVMe performance).

In `NixForge/flake.nix`, the `ensureDatabases` block ensures the databases exist when the system boots:
```nix
ensureDatabases = [ 
  "atlas_uat"            # Atlas platform UAT environment (active)
  "atlas_prod"           # Atlas platform Production environment (pre-provisioned)
  # ...
];
```
*(Note: The legacy `ruud`, `anchor`, and `anchor_uat` databases were deprecated and dropped from the server to maintain a clean operational state).*

---

## 6. New Environment Provisioning Runbook

> **This section is the canonical checklist for bringing up a new namespace.** CI alone is not sufficient — several resources must be applied manually by a cluster admin before the first pipeline run will succeed.

The following gaps have been discovered through live deployments and **will silently break the environment** if skipped.

### Step 1 — Create the Namespace

```bash
kubectl create namespace atlas-<env>
```

### Step 2 — Copy the Registry Pull Secret

The `ghcr-login-secret` only lives in `atlas-uat` by default. Copy it to any new namespace:

```bash
kubectl get secret ghcr-login-secret -n atlas-uat -o json \
  | sed 's/"namespace": "atlas-uat"/"namespace": "atlas-<env>"/' \
  | kubectl create -f -
```

Without this, all pods will fail with `ImagePullBackOff`.

### Step 3 — Apply Woodpecker RBAC

```bash
kubectl apply -f k8s/base/woodpecker-rbac.yaml -n atlas-<env>
```

Without this, the CI deploy step fails with `403 Forbidden` on every `kubectl` call.

### Step 4 — Verify `app-secrets` Will Be Injected

The CI pipeline injects `app-secrets` via `sed` substitution before running `kubectl apply`. Confirm all required CI secrets are set in Woodpecker for the relevant branch:

| Secret Name | Used By |
|---|---|
| `DB_USER` | backend, anchor-app |
| `DB_PASSWORD` | backend, anchor-app |
| `ADMIN_PASSWORD` | backend |
| `ATLAS_INIT_TOKEN` | backend |
| `SMTP_TOKEN` | backend |
| `SMTP_SERVER` | backend |
| `SMTP_PORT` | backend |
| `SMTP_USERNAME` | backend |
| `SMTP_FROM` | backend |
| `METRICS_TOKEN` | backend |
| `JWT_SECRET` | backend |

> **⚠️ Do NOT add `cloudflare-edge-secrets` to backend-patch.yaml for DEV or UAT.** This secret does not exist in those namespaces. The reference has been removed from `k8s/overlays/uat/backend-patch.yaml` and `k8s/overlays/dev/backend-patch.yaml`. Only add it if you explicitly create and populate the secret in the target namespace.

### Step 5 — Apply the Ingress Manually (First Time)

The ingress must exist **before** CI runs, because CI applies kustomize which includes the ingress — but cert-manager needs the annotation to start issuing certs, and that depends on the ingress existing first.

```bash
# For UAT
kubectl apply -f k8s/overlays/uat/ingress.yaml -n atlas-uat

# For DEV
kubectl apply -f k8s/overlays/dev/ingress.yaml -n atlas-dev
```

> **⚠️ All ingress manifests must use `letsencrypt-cloudflare` (DNS-01), not `letsencrypt-prod` (HTTP-01).** Cloudflare's "Always Use HTTPS" redirect breaks HTTP-01 ACME challenges permanently.

### Step 6 — Verify Cert-Manager Issues Certificates

After the ingress is applied, cert-manager will begin DNS-01 challenges automatically. Check progress:

```bash
kubectl get certificate,challenges -n atlas-<env>
```

Expected outcome within ~2 minutes:

```
NAME              READY   SECRET
<env>-atlas-tls   True    <env>-atlas-tls   ✅
<env>-bwr-tls     True    <env>-bwr-tls     ✅
```

If challenges stay `pending` for more than 3 minutes, the Cloudflare API token has insufficient permissions. See [Section 3](#3-traffic-routing-ingress) for the token verification runbook.

To force an immediate retry after fixing token permissions:

```bash
kubectl delete challenges --all -n atlas-<env>
```

### Step 7 — Ensure the Database Exists

Databases are provisioned declaratively in NixForge, not by CI. Add the new database to `NixForge/flake.nix`:

```nix
services.postgresql.ensureDatabases = [
  "atlas_dev"
  "atlas_uat"
  "atlas_<env>"   # ← add this
];
```

Then deploy the NixForge change:
```bash
colmena apply --on manager
```

### Step 8 — Trigger the First CI Deploy

Push to the environment's branch to trigger a full pipeline run:

```bash
git commit --allow-empty -m "chore: trigger initial <env> deploy" && git push origin <branch>
```

### Environment Pre-flight Checklist

| Check | DEV | UAT | PROD |
|---|---|---|---|
| Namespace exists | ✅ | ✅ | 🔒 (not yet) |
| `ghcr-login-secret` copied | ✅ | ✅ | 🔒 |
| Woodpecker RBAC applied | ✅ | ✅ | 🔒 |
| Ingress applied with DNS-01 issuer | ✅ | ✅ | 🔒 |
| TLS certs `Ready: True` | ✅ | ✅ (after fix) | 🔒 |
| Database in NixForge | ✅ | ✅ | ✅ (pre-provisioned) |
| `cloudflare-edge-secrets` **absent** from backend-patch | ✅ | ✅ | N/A (uses SealedSecret) |
| CI secrets set in Woodpecker | ✅ | ✅ | 🔒 |
| Ingress-sidecar Deployment running | ✅ | ⏳ (next CI run) | 🔒 |

