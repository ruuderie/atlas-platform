# Local Kubernetes (OrbStack) Implementation Plan

Since you'd like to test the entire stack locally in OrbStack Kubernetes before committing to the Nix server approach, we will restructure your `k8s/` manifests to support a **Base + Overlay** model. This is standard Kubernetes practice and allows you to test identically sized components locally, while overriding only environment-specific details (like database URLs or hostnames).

## 1. Directory Restructuring

We will restructure your existing `k8s/` directory as follows:

```
k8s/
├── base/                       # Shared component configurations
│   ├── backend.yaml
│   ├── directory-instance.yaml
│   ├── platform-admin.yaml
│   └── kustomization.yaml
└── overlays/
    └── local/                  # OrbStack local development environment
        ├── kustomization.yaml
        ├── config.yaml         # Local ConfigMaps and Secrets
        ├── postgres.yaml       # Local Postgres running IN OrbStack (for dev)
        └── ingress.yaml        # Exposes your apps via *.orb.local domains
```

## 2. Changes Needed for Local OrbStack

### Add Local Postgres (`overlays/local/postgres.yaml`)
Instead of connecting to your Mac's host or the remote Nix server, we'll deploy a lightweight Postgres pod directly into OrbStack. This gives you a clean slate Database for local K8s testing that perfectly mimics a generic cluster setup.

### Add Kubernetes Ingress (`overlays/local/ingress.yaml`)
Your `docker-compose.yml` uses Caddy as a reverse proxy. In Kubernetes, we use an **Ingress**. OrbStack automatically installs an Ingress Controller, which maps to your Mac's port 80/443. We will expose:
- `admin.atlas.orb.local` -> routes to `platform-admin`
- `directory.atlas.orb.local` -> routes to `directory-instance`
- `api.atlas.orb.local` -> routes to `backend`

### Update Configurations (`overlays/local/config.yaml`)
We will configure the `app-config` ConfigMap to point to our local Postgres service (`postgres` instead of `10.42.0.1`) and update the `API_URL` environment variables to point to the local Ingress domains.

## 3. Workflow

Once implemented, your local workflow will be:

1. **Start OrbStack**: Ensure OrbStack Kubernetes is running (`kubectl config use-context orbstack`).
2. **Deploy**: Run `kubectl apply -k k8s/overlays/local`
3. **Test**: Open `http://directory.atlas.orb.local` in your browser.

When you're happy with the local setup and ready to move to the Nix server, we would simply create `k8s/overlays/prod/`, point the database config to your Nix server's host Postgres (`10.42.0.1`), and apply it to the server.

---
**Do you approve of this plan?** If so, I will execute the file restructuring and create the local K8s manifests for you.
