# Platform Admin — Domain & TLS Operator Runbook

> This is the day-to-day reference for managing domains and TLS on app instances
> from platform-admin. For the full technical deep-dive, see
> [`tls_and_custom_domains.md`](./tls_and_custom_domains.md).

---

## Adding a Domain to an App Instance

### Step 1 — Open the instance in platform-admin

1. Go to **Internal Instances** in the left sidebar
2. Find and click the instance (e.g. `folio1`)
3. Click the **Domain & SSL** tab

### Step 2 — Assign the domain

Enter the domain in the input field and click **Save & Provision**.

**Two cases:**

#### Case A — Platform subdomain (`*.atlas.oply.co`)
```
folio2.atlas.oply.co
pm.atlas.oply.co
```
- No DNS action needed — you own this domain
- TLS is covered by the wildcard cert (`wildcard-tls-prod`)
- **Active within seconds** of provisioning

#### Case B — Custom client domain (anything else)
```
pm.acmerealty.com
tracker.mybusiness.io
app.clientco.co.uk
```
- Platform-admin will show a DNS record to give the client:

  | Type | Host | Value | TTL |
  |---|---|---|---|
  | `A` | `pm.acmerealty.com` | `<CLUSTER_IP>` | Auto |

- Send this record to the client. Once they add it, **cert-manager auto-issues TLS within ~60 seconds**
- Tell Cloudflare clients: set Proxy Status to **grey cloud (DNS only)** — Cloudflare proxying breaks cert issuance

### Step 3 — Verify

After provisioning (and DNS propagates for custom domains):

```bash
# Should show: issuer=C = US, O = Let's Encrypt
echo | openssl s_client -connect yourdomain.com:443 -servername yourdomain.com 2>/dev/null \
  | openssl x509 -noout -issuer -subject
```

---

## TLS Issues — What to Do

### Symptom: browser shows "Your connection is not private" or cert warning

**Check what cert is being served:**
```bash
echo | openssl s_client -connect yourdomain.com:443 -servername yourdomain.com 2>/dev/null \
  | openssl x509 -noout -issuer
```

| Issuer shown | Meaning | Fix |
|---|---|---|
| `O=Acme Co, CN=Kubernetes Ingress Controller Fake Certificate` | wildcard-tls-prod not ready yet | Wait, or use Re-Provision button |
| `O=Let's Encrypt` | ✅ Real cert, browser trust issue may be cache | Hard-refresh browser |

### Symptom: `*.atlas.oply.co` domain showing fake cert

The `wildcard-tls-prod` Secret may not be ready. Check on the server:
```bash
ssh root@<server> "kubectl get certificate wildcard-tls-prod -n atlas-dev"
```

If `READY = False`, cert-manager is still issuing. Wait 2-3 minutes and check again.
If it's been stuck >5 minutes, delete and recreate (cert-manager will retry):
```bash
ssh root@<server> "
  kubectl delete certificate wildcard-tls-prod -n atlas-dev
  kubectl apply -f - <<'EOF'
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: wildcard-tls-prod
  namespace: atlas-dev
spec:
  secretName: wildcard-tls-prod
  dnsNames: [\"*.atlas.oply.co\", \"atlas.oply.co\"]
  issuerRef:
    name: letsencrypt-cloudflare
    kind: ClusterIssuer
EOF"
```

### Symptom: Custom domain showing fake cert (DNS is set correctly)

The `letsencrypt-http` ClusterIssuer may be missing. Check:
```bash
ssh root@<server> "kubectl get clusterissuer letsencrypt-http"
```
If not found: `kubectl apply -f k8s/cluster-setup/cluster-issuers.yaml`

Then use the **Re-Provision Ingress** button in platform-admin → Domain & SSL tab.

### Re-Provision button

The **Re-Provision Ingress** button (visible in the Domain & SSL tab) re-fires the
sidecar provisioning call. Use it when:
- Domain was assigned before the ingress-sidecar was deployed
- You get a fake cert and want to retry without re-saving the domain
- The wildcard-tls-prod cert just finished issuing and nginx needs to re-read it

---

## Certificate Renewals

**You don't need to do anything.** cert-manager renews all certificates automatically
60 days before expiry. The current wildcard cert is valid until:

| Cert | Expires | Auto-renews |
|---|---|---|
| `*.atlas.oply.co` (dev) | Oct 2, 2026 | Aug 2, 2026 |
| `*.dev.atlas.oply.co` | Sep 26, 2026 | Jul 27, 2026 |
| `*.uat.atlas.oply.co` | ~Oct 2026 | ~Aug 2026 |

---

## Cluster State Reference

One server running K3s with three namespaces:

| Namespace | Environment | Wildcard cert |
|---|---|---|
| `atlas-dev` | Development / current live | `wildcard-tls-prod` (`*.atlas.oply.co`) ✅ |
| `atlas-uat` | UAT / staging | `wildcard-tls-uat` (`*.uat.atlas.oply.co`) ✅ |
| `atlas-platform` | Future production | Not yet deployed |

All ClusterIssuers are ready:

| Issuer | Solver | Used for |
|---|---|---|
| `letsencrypt-cloudflare` | DNS-01 (Cloudflare) | Wildcard certs |
| `letsencrypt-http` | HTTP-01 (nginx) | Custom client domains |
| `letsencrypt-prod` | HTTP-01 (nginx) | Original platform certs |

---

## Quick Reference — SSH Commands

```bash
# Connect to server
ssh root@<server>

# Check all TLS certs and their status
kubectl get certificate -A

# Check a specific domain's cert live
echo | openssl s_client -connect DOMAIN:443 -servername DOMAIN 2>/dev/null \
  | openssl x509 -noout -issuer -subject -dates

# See what's causing a cert to not issue
kubectl describe certificate CERT-NAME -n NAMESPACE | grep -A5 'Message\|Reason'

# See cert-manager activity
kubectl logs -n cert-manager deploy/cert-manager --tail=50

# List all dynamically-provisioned tenant ingresses
kubectl get ingress -A | grep tenant-
```

---

## Related Docs

- [`tls_and_custom_domains.md`](./tls_and_custom_domains.md) — Full technical reference (DNS routing rules, architecture diagram, all domain types)
- [`adding_a_new_app.md`](./adding_a_new_app.md) — Full checklist for launching a new product app (CI/CD + DB registration + domain)
- [`k8s/cluster-setup/README.md`](../../../k8s/cluster-setup/README.md) — One-time cluster bootstrap (only needed for a brand new cluster)
