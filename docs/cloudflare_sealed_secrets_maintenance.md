# Cloudflare Edge & R2 Secrets Maintenance (GitOps)

This document explains the updated, GitOps-compliant secret management workflow for Cloudflare Edge and R2 credentials in the Atlas Platform.

## Architecture

We have fully migrated away from injecting Cloudflare and R2 secrets via the Woodpecker CI pipeline (`sed`). Instead, we are using **Bitnami Sealed Secrets**.

1. **The Controller:** The `sealed-secrets` controller runs inside your remote NixOS K3s cluster. It was declaratively installed via `NixForge/flake.nix`.
2. **The Encryption:** Secrets are encrypted on your local Mac using the cluster's public key. The resulting `SealedSecret` file is safe to commit to GitHub.
3. **The Decryption:** When Woodpecker runs `kubectl apply -k k8s/overlays/<env>`, the `SealedSecret` is deployed to the cluster. The `sealed-secrets` controller uses its private key (which never leaves the cluster) to decrypt the file back into a standard Kubernetes `Secret`.

---

## How to Update or Add New Secrets

If you ever need to rotate your Cloudflare API Token or R2 Access Keys, follow these steps:

### 1. Fetch the Public Certificate from the Server
You need the public key to encrypt new secrets. Make sure you can SSH into the manager node, and run this locally:
```bash
ssh -o StrictHostKeyChecking=no root@69.164.248.38 'kubectl -n kube-system get secret -l sealedsecrets.bitnami.com/sealed-secrets-key=active -o jsonpath="{.items[0].data.tls\.crt}" | base64 -d' > pub-cert.pem
```

### 2. Create a Temporary Plain Text Secret File
Create a local `secret-plain.yaml` file containing the raw credentials:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: cloudflare-edge-secrets
  namespace: atlas-platform
type: Opaque
stringData:
  R2_ACCESS_KEY_ID: "your_new_key_here"
  R2_SECRET_ACCESS_KEY: "your_new_secret_here"
  R2_ENDPOINT: "your_new_endpoint_here"
  CLOUDFLARE_ZONE_ID: "your_zone_id_here"
  CLOUDFLARE_API_TOKEN: "your_new_api_token_here"
  TLS_PROVIDER: "cloudflare"
```

### 3. Encrypt the File
Run `kubeseal` (installed via Homebrew) using the public certificate you downloaded:
```bash
kubeseal --cert pub-cert.pem --format yaml < secret-plain.yaml > k8s/overlays/<env>/sealed-secret.yaml
```

### 4. Clean Up and Commit
> [!CAUTION]
> **CRITICAL SECURITY STEP:** You must immediately delete the temporary plain text files before committing to Git!
```bash
rm secret-plain.yaml pub-cert.pem
```

Finally, commit the updated `k8s/overlays/<env>/sealed-secret.yaml` to Git and push. Woodpecker CI will automatically deploy the new encrypted state to the target environment.
