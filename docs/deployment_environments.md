# Atlas Platform: Deployment & Environments Architecture

This document outlines how the Atlas Platform is configured across different deployment environments (UAT vs. Production) and how the CI/CD pipeline correctly routes traffic, provisions databases, and manages secrets.

## Overview

The Atlas Platform uses **Kustomize** to manage environment-specific Kubernetes configurations (`k8s/overlays/uat` and `k8s/overlays/prod`), and **Woodpecker CI** to automatically apply the correct overlay based on the git branch being pushed.

### Environment Matrix

| Component | UAT (`uat` branch) | Production (`main`/`master` branch) |
| :--- | :--- | :--- |
| **K8s Overlay** | `k8s/overlays/uat` | `k8s/overlays/prod` |
| **Database** | `atlas_uat` | `atlas_prod` |
| **API Domain** | `api.uat.atlas.oply.co` | `api.atlas.oply.co` |
| **Admin Domain** | `uat.atlas.oply.co` | `atlas.oply.co` |
| **Tenant Domains** | `uat.buildwithruud.com`, `uat.oplystusa.com` | `buildwithruud.com`, `oplystusa.com` (+ `www` variants) |
| **Secrets Strategy**| CI-injected plaintext overrides (speed/flexibility) | **Bitnami Sealed Secrets** (strict security) |

---

## 1. Woodpecker CI/CD Pipeline

The `.woodpecker.yml` pipeline is the brain that orchestrates deployments. When a push occurs, it determines the target environment based on the branch name.

* **Branch `main` or `master`** → Deploys to **Production**.
* **Any other branch (e.g., `uat`)** → Deploys to **UAT**.

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

**In Production (`k8s/overlays/prod/ingress.yaml`):**
* `atlas.oply.co` routes to the `platform-admin` service.
* `api.atlas.oply.co` routes to the `backend` service (port 8000).
* `buildwithruud.com` and `www.buildwithruud.com` route to the `anchor-app` service.

The production ingress specifies `cert-manager.io/cluster-issuer: "letsencrypt-prod"` to automatically provision trusted SSL certificates for all these bare domains.

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
