# TLS & Custom Domains — Atlas Platform

> [!IMPORTANT]
> This document is the authoritative reference for how TLS certificates are issued
> and how any domain — platform subdomain or fully custom client domain — routes to
> an Atlas app instance. Read this before assigning domains via platform-admin.

---

## How it works — one sentence

When a domain is assigned to an app instance in platform-admin, the backend calls the
**ingress-sidecar**, which creates a Kubernetes `Ingress` resource. cert-manager sees the
Ingress and auto-issues a TLS certificate within ~60 seconds. No manual steps.

---

## Domain routing rules (ingress-sidecar `tls_config`)

The sidecar (`backend/src/bin/ingress_sidecar.rs`) decides TLS strategy based on the domain:

| Domain pattern | TLS Secret | cert-manager needed? | Cert type |
|---|---|---|---|
| `*.dev.atlas.oply.co` | `wildcard-tls-dev` | No | Existing wildcard |
| `*.uat.atlas.oply.co` | `wildcard-tls-uat` | No | Existing wildcard |
| `*.atlas.oply.co` | `wildcard-tls-prod` | No (secret must exist) | Wildcard via DNS-01 |
| **Anything else** | `{slug}-tls` | **Yes — HTTP-01 auto** | Per-domain via Let's Encrypt |

**"Anything else"** means literally any domain:
- `folio.clientco.com`
- `pm.mybrokeragefirm.io`
- `tracker.example.org`
- `mycompany.co.uk`

As long as the client points their DNS (A or CNAME record) at the cluster IP, cert-manager
issues a real Let's Encrypt cert within ~60 seconds of the Ingress being created.
Platform-admin shows the exact DNS record to give the client.

---

## Platform-admin flow (what you actually do)

### Adding a platform subdomain (e.g. `folio2.atlas.oply.co`)

1. Open Internal Instances → select the instance
2. Domain & SSL tab → enter `folio2.atlas.oply.co` → **Save & Provision**
3. Done. No DNS action, no cert action. TLS is live within seconds once the
   wildcard Secret exists in the cluster (see One-Time Bootstrap below).

### Adding a custom client domain (e.g. `pm.acmerealty.com`)

1. Open Internal Instances → select the instance
2. Domain & SSL tab → enter `pm.acmerealty.com` → **Save & Provision**
3. Platform-admin displays the DNS record to send to the client:

   | Type | Name (Host) | Value (Target) | TTL |
   |---|---|---|---|
   | `A` | `pm.acmerealty.com` | `69.164.248.38` | Auto |
   
   Or a CNAME if the client prefers:
   | `CNAME` | `pm` | `cluster.atlas.oply.co` | Auto |

4. Once the client adds the record and DNS propagates (~60s to minutes), cert-manager
   auto-issues the cert. **No further action required.**

> [!TIP]
> **Cloudflare clients:** Tell them to set Proxy Status to **DNS-only (grey cloud)**.
> cert-manager handles TLS — Cloudflare proxying breaks the HTTP-01 ACME challenge.

---

## Certificate issuers — what's on the cluster

Two ClusterIssuers are required (defined in `k8s/cluster-setup/cluster-issuers.yaml`):

| ClusterIssuer | Solver | Used for |
|---|---|---|
| `letsencrypt-cloudflare` | DNS-01 via Cloudflare API | Wildcard certs (`*.atlas.oply.co`) |
| `letsencrypt-http` | HTTP-01 via Nginx Ingress | **All custom domains** |

These are cluster-scoped resources applied **once by admin** (see Bootstrap below).
CI does not manage them — they persist across deployments.

---

## One-Time Cluster Bootstrap

This must be done **once per cluster** (dev + prod separately). After that, zero
cert management is ever needed.

### Step 1 — Create a Cloudflare API Token

Go to [dash.cloudflare.com/profile/api-tokens](https://dash.cloudflare.com/profile/api-tokens):

1. **Create Token** → **Edit zone DNS** template
2. Zone Resources: Include → Specific zone → `atlas.oply.co`
3. Copy the token (shown once)

### Step 2 — Apply to the cluster

```bash
# SSH into the server, then:

# Create the Cloudflare secret in cert-manager namespace
kubectl create secret generic cloudflare-api-token \
  --from-literal=api-token=<YOUR_CF_TOKEN> \
  --namespace cert-manager

# Apply both ClusterIssuers
kubectl apply -f k8s/cluster-setup/cluster-issuers.yaml

# Apply the wildcard Certificate (or let CI do it on next push)
kubectl apply -f k8s/overlays/dev/wildcard-cert.yaml    # for dev cluster
kubectl apply -f k8s/overlays/prod/wildcard-cert.yaml   # for prod cluster

# Watch it issue (~60-120 seconds)
kubectl get certificate wildcard-tls-prod -n atlas-dev -w
```

When `READY = True`, every `*.atlas.oply.co` subdomain you assign in platform-admin
gets HTTPS automatically.

---

## Reprovision button (escape hatch)

The **Domain & SSL** tab in platform-admin has a **Re-Provision Ingress** button.
Use it when:
- Domain was assigned before the ingress-sidecar was deployed
- `wildcard-tls-prod` didn't exist yet when the instance was first provisioned
- You get the Nginx fake cert and want to re-trigger without re-saving the domain

The button calls `POST /api/admin/app-instances/:id/reprovision-domain` which does
an idempotent server-side apply of the Ingress via the sidecar.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Nginx fake cert (`O=Acme Co`) | `wildcard-tls-prod` Secret doesn't exist | Run one-time bootstrap |
| Fake cert on custom domain | `letsencrypt-http` ClusterIssuer doesn't exist | Apply `cluster-issuers.yaml` |
| Custom domain cert pending >5min | DNS not propagated yet, or Cloudflare proxy is on | Wait, or disable Cloudflare proxy |
| `Certificate` resource in `False` state | ClusterIssuer config error | Check cert-manager logs: `kubectl logs -n cert-manager deploy/cert-manager` |
| TLS works but HTTP redirects to wrong domain | `ssl-redirect: true` annotation missing on Ingress | Use "Re-Provision" button to re-apply |

### Check cert-manager on server

```bash
# See all Certificate resources and their status
kubectl get certificate -A

# See pending ACME challenges
kubectl get challenge -A

# cert-manager logs (filter by your domain)
kubectl logs -n cert-manager deploy/cert-manager --tail=100 | grep "yourdomain.com"
```

---

## Architecture reference

```
platform-admin UI
  └─ POST /api/admin/app-instances/:id/domain  (or /reprovision-domain)
       └─ IngressProvisioner (backend/src/services/ingress_provisioner.rs)
            └─ POST /api/ingress/provision  →  ingress-sidecar
                 └─ kubectl server-side apply → K8s Ingress
                      └─ cert-manager watches Ingress annotation
                           ├─ *.atlas.oply.co  → references wildcard-tls-prod Secret (pre-issued)
                           └─ custom domain    → HTTP-01 ACME challenge → Let's Encrypt → TLS Secret
```

**Source files:**
- Sidecar: [`backend/src/bin/ingress_sidecar.rs`](../../../backend/src/bin/ingress_sidecar.rs)
- Provisioner service: [`backend/src/services/ingress_provisioner.rs`](../../../backend/src/services/ingress_provisioner.rs)
- ClusterIssuers: [`k8s/cluster-setup/cluster-issuers.yaml`](../../../k8s/cluster-setup/cluster-issuers.yaml)
- Wildcard cert (dev): [`k8s/overlays/dev/wildcard-cert.yaml`](../../../k8s/overlays/dev/wildcard-cert.yaml)
- Wildcard cert (prod): [`k8s/overlays/prod/wildcard-cert.yaml`](../../../k8s/overlays/prod/wildcard-cert.yaml)
- Platform-admin UI: [`apps/platform-admin/src/pages/internal_instances/config.rs`](../../../apps/platform-admin/src/pages/internal_instances/config.rs)
