# Multi-Tenant CI/CD Deployment Guide

This guide details how to effectively replicate the `buildwithruud-uat` deployment engine to create and map entirely new instances of the `anchor-app` for different tenants, domains, and environments (UAT, DEV, PROD).

## How The Architecture Works
Our GitOps deployment relies on the exact configuration living across three critical zones:
1. **The NixOS Server**: Binds the bare-metal TCP traffic incoming on ports 80/443 over to the K3s Ingress Controller securely using Nginx Passthrough.
2. **Kubernetes Configuration (Kustomize)**: Defines the sealed Namespace, Database, Secret tokens, and Ingress routing that tells K3s exactly what to do.
3. **Woodpecker CI Pipeline**: Compiles the source code into a new image and specifically triggers a `rollout restart` for the designated tenant instance.

---

## Step 1: Duplicate the Kubernetes Configurations

Your platform manages configuration using `kustomization.yaml`. To create a brand new environment (e.g. `client-prod`):

1. **Create the Folder Space:**
   Navigate into `/k8s/databases/` and `/k8s/apps/` and duplicate the existing tenant configurations.
   ```bash
   cp -r k8s/databases/buildwithruud k8s/databases/client-prod
   cp -r k8s/apps/buildwithruud-uat k8s/apps/client-prod
   ```

2. **Patch the Target Resources:**
   Inside your newly created `/client-prod/` folders, open the `kustomization.yaml` files and execute a Find & Replace for the namespace. 
   Change `namespace: buildwithruud-uat` to `namespace: client-prod`.
   Make sure you uniquely change the hostname in the Ingress file (e.g., `client.domain.com`).

3. **Inject the Decoupled Database Secret:**
   Because our architecture strictly manages sensitive data outside of Git tracking for supreme security, you must manually generate the `app-secrets` object directly into your new namespace.
   First, access your database (`psql`) and run `CREATE DATABASE client_prod OWNER your_admin_user;` and set a strong database password.
   Then, directly securely push the parsed secret to Kubernetes:
   ```bash
   kubectl create secret generic app-secrets \
     --from-literal=DATABASE_URL="postgres://your_admin_user:THE_PASSWORD@10.42.0.1:5432/client_prod" \
     -n client-prod
   ```

4. **Deploy the Environment Shell:**
   From your local terminal (connected to your server via `kubectl`), execute the kustomize apply sequentially to lock in the Postgres Database first, then the App instance:
   ```bash
   kubectl apply -k k8s/databases/client-prod/
   kubectl apply -k k8s/apps/client-prod/
   ```
   *Your Kubernetes cluster now has a sealed application instance and isolated database waiting dynamically for the Woodpecker image push.*
---

## Step 2: Configure Woodpecker Pipeline Routing

Now that the system is ready to host the tenant, you must explicitly tell Woodpecker *when* and *where* to deploy to it.

Open `.woodpecker/release.yml`. By default, there is a `deploy_uat` step designed to fire exclusively when commits are pushed to the `feature/mvp` branch.

To map your new Production tenant:
1. **Copy the Deploy Step:** Duplicate the entire `deploy_uat` step and rename it `deploy_client_prod`.
2. **Assign the Branch Condition:** Add a `when` condition to tell Woodpecker only to trigger this step on production commits (e.g., `when: branch: [main, production]`).
3. **Adjust the Rollout Flag:** Change the namespace flag in the `commands:` block!
   
   ```yaml
   deploy_client_prod:
     image: rancher/k3s:latest
     volumes:
       - /etc/rancher/k3s/k3s.yaml:/kubeconfig.yaml:ro
     environment:
       KUBECONFIG: /kubeconfig.yaml
     commands:
       # Ensure you target the correct namespace (-n)
       - kubectl --server=https://host.containers.internal:6443 --insecure-skip-tls-verify rollout restart deployment/anchor-app -n client-prod
     when:
       event: [push, tag]
       branch: [main, production]
   ```

---

## Step 3: NixOS Nginx TCP Passthrough (Domain Resolution)

For custom domains outside the standard `*.anchor.oply.co` wildcard, you must instruct the host server to formally accept the SSL traffic and transparently bounce it into the Kubernetes Ingress Controller.

Open `nix_sample/flake.nix`. Under `services.nginx.appendHttpConfig`, add a downstream TCP mapping.
```nginx
# For TCP/SSL Transparent Handover
server {
    listen 443;
    server_name client.domain.com; # Target Client Domain
    proxy_pass 127.0.0.1:8443;     # Nginx Ingress TCP Port
    ssl_preread on;
}

# For Standard HTTP Fallback
server {
    listen 80;
    server_name client.domain.com;
    proxy_pass http://127.0.0.1:8080;
}
```
Apply the configuration securely by rebooting the manager node from your deployment terminal:
`colmena apply --on manager`

## Final Result
1. The user pushes code to `main`.
2. Woodpecker generates the Docker Artifact and detects the `when` condition perfectly matches the `deploy_client_prod` logic.
3. K3s natively refreshes the isolated container running inside the `client-prod` namespace.
4. Traffic smoothly routes via `client.domain.com` straight down to the dedicated application.
