# Atlas Platform — Cluster Bootstrap

These resources are **applied once by an admin** and are NOT managed by CI/CD.
They are cluster-scoped (no namespace) and represent the foundational cert infrastructure.

## Apply Order

```bash
# 1. Install cert-manager (if not already present)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.5/cert-manager.yaml
kubectl wait --namespace cert-manager --for=condition=ready pod --selector=app.kubernetes.io/instance=cert-manager --timeout=120s

# 2. Create the Cloudflare API token secret (needed by letsencrypt-cloudflare ClusterIssuer)
#    Get token from: https://dash.cloudflare.com/profile/api-tokens
#    Required permissions: Zone:DNS:Edit for atlas.oply.co
kubectl create secret generic cloudflare-api-token \
  --from-literal=api-token=<YOUR_CF_TOKEN> \
  --namespace cert-manager

# 3. Apply ClusterIssuers
kubectl apply -f cluster-issuers.yaml
```

## ClusterIssuers

| Name | Solver | Used for |
|---|---|---|
| `letsencrypt-cloudflare` | DNS-01 via Cloudflare API | Wildcard certs (`*.atlas.oply.co`, `*.dev.atlas.oply.co`) |
| `letsencrypt-http` | HTTP-01 via Nginx Ingress | Custom tenant domains (e.g. `folio.client.com`) |

## TLS Flow Summary

```
Domain type                → Secret              → ClusterIssuer           → Action needed
*.dev.atlas.oply.co        → wildcard-tls-dev    → letsencrypt-cloudflare  → CI deploys Certificate in dev overlay
*.uat.atlas.oply.co        → wildcard-tls-uat    → letsencrypt-cloudflare  → CI deploys Certificate in uat overlay  
*.atlas.oply.co            → wildcard-tls-prod   → letsencrypt-cloudflare  → CI deploys Certificate in dev+prod overlay
custom domain (any)        → {slug}-tls          → letsencrypt-http        → Auto on Ingress create (sidecar does this)
```

After bootstrap, **zero manual steps are needed** to add new `*.atlas.oply.co` apps
or custom tenant domains — the sidecar and cert-manager handle everything.
