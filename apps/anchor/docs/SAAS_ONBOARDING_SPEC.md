# SaaS Tenant Onboarding & Automated Provisioning Specification

To scale from manual GitOps deployment to a fully automated B2B SaaS platform (e.g., self-serve for construction contractors), we must introduce an automated orchestration layer. 

Currently, deploying a new tenant involves manual Git copies, `flake.nix` modifications, and Woodpecker changes (as documented in `MULTI_TENANT_DEPLOYMENT_GUIDE.md`). This architecture specifies how to automate that exact flow entirely.

---

## 1. Architectural Paradigm Shift: The Central Control Plane
Right now, `anchor-app` is built as a single-tenant platform. To onboard users securely, we need to separate the **Control Plane** from the **Tenant Data Planes**.

### A. Central Management Dashboard (`manage.anchor.oply.co`)
This is the only app you deploy manually. It serves as your primary marketing site and onboarding wizard. It uses a global PostgreSQL database to track:
- `tenants` (ID, Company Name, Subdomain, Custom Domain)
- `subscriptions` (Stripe Customer ID, Plan Tier, Status)
- `infrastructure_state` (Provisioning status: Pending, Active, Failed)

### B. Tenant Data Planes (`contractor.anchor.oply.co`)
These are the isolated instances spun up *by* the Control Plane. They remain completely siloed (their own database, their own K8s namespace) exactly as we designed them natively.

---

## 2. The Onboarding Flow (UX)

When a new construction contractor visits your marketing site, they enter the following pipeline:

### Step 1: Authentication & Identity
- **Action:** User signs up using the existing Passkey/WebAuthn flow natively built in Leptos.
- **Backend:** Creates a user in the Central Control Plane database.

### Step 2: Tenant Configuration
- **Action:** User is prompted for their "Workspace." 
- **Inputs:** 
  - Company Name (e.g., "Apex Construction")
  - Desired Subdomain (`apex.anchor.oply.co`)
  - (Optional - Premium) Custom Domain Request (`www.apexbuilds.com`)

### Step 3: Billing & Package Selection
- **Action:** User selects a tier (e.g., "Pro Contractor" or "Enterprise Sandbox").
- **Integration:** The backend redirects to a Stripe Checkout Session or accepts a promotional Code.
- **Webhook Validation:** Once payment successfully clears, Stripe fires a webhook to your Axum backend. The user is marked `Active`.

---

## 3. The Automated Provisioning Engine (GitOps Trigger)

Once the backend receives the `Active` signal from Stripe, it must orchestrate the creation of the tenant. Because our infrastructure relies on GitOps (Woodpecker), the most elegant way to automate this is to make your Rust backend **commit code just like a developer**.

### The GitOps Orchestrator Algorithm
1. **Template Cloner:** The backend maintains a `k8s/templates/tenant/` folder containing baseline `kustomization.yaml`, `deployment.yaml`, and `ingress.yaml` files holding placeholder variables (e.g., `{{TENANT_ID}}`, `{{DOMAIN}}`).
2. **File Generation:** The backend dynamically generates a concrete folder (e.g., `k8s/apps/apex-prod/`) by replacing the placeholders with "apex".
3. **Commit & Push:** Using a Rust git crate (like `git2`), the backend natively commits the new environment folders to the GitHub repository directly.
4. **CI/CD Execution:** Woodpecker CI catches the automated webhook push, recognizes the new folder, boots up the K3s deploy step, and natively commands Kubernetes to spin up the cluster exactly as it did for `buildwithruud-uat`.

---

## 4. Solving the Dynamic Networking Bottleneck (NixOS)

Currently, we manually modify the NixOS `flake.nix` file to add Nginx TCP Passthrough blocks for custom domains. This requires a hard manual server reboot (`colmena apply`), which **blocks automation**.

### The Fix: Wildcard Passthrough & K8s Cert-Manager
To automate networking without ever touching NixOS again:
1. **NixOS Wildcard TCP Redirect:** Configure `flake.nix` so that Nginx explicitly handles routing for `ci.oply.co` and `manage.anchor.oply.co` as usual. However, for **any other unknown domain**, Nginx blindly streams the raw port 443 TCP traffic directly into the K3s Ingress Controller.
2. **Dynamic K8s Ingress:** Once K3s receives the raw traffic, the internal Kubernetes NGINX controller dynamically reads the requested SNI host (e.g., `apex.anchor.oply.co`) and routes it to the specific tenant pod.
3. **Cert-Manager:** Deploy Kubernetes `cert-manager`. When the backend GitOps engine pushes the new tenant `ingress.yaml` to the cluster, Cert-Manager immediately detects it and automatically requests a Let's Encrypt SSL certificate for the contractor's sub-domain instantly. 

---

## 5. Enterprise Features: The "Sandbox" Environment
If a user upgrades to the Enterprise tier, the GitOps Orchestrator duplicates Step 3 but targets the `sandbox` branch.
1. It creates `k8s/apps/apex-sandbox`.
2. It assigns a secondary database `postgres_apex_dev`.
3. It modifies Woodpecker pipelines so that whenever the contractor wants to test massive changes to their portfolio layout, they execute it in the `apex-sandbox` namespace without crashing their live client-facing portal.

## Technology Stack Summary Needed
1. **Stripe API** (for Checkout and Webhook listeners in Axum).
2. **git2-rs** (for allowing the Central Control Plane to commit scaling scripts to GitHub).
3. **cert-manager** (Helm chart in K3s for zero-touch SSL).
4. **Wildcard DNS** (`*.anchor.oply.co` pointed to the NixOS IP natively in Cloudflare).
