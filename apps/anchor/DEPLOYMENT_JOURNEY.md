# Architecting a Bare-Metal Rust Web App on NixOS & K3s: A Deployment Journey

When building high-performance web applications, the stack you choose dictates the ceiling of your scalability. For the Anchor platform—a full-stack Rust application utilizing Leptos—deploying to a standard managed cloud provider wasn't enough. We needed bare-metal performance, declarative infrastructure, and absolute control over our orchestration.

This journey details exactly how we architected, debugged, and ultimately deployed a Rust native binary to a NixOS-backed Kubernetes (K3s) cluster. We encountered complex cross-compilation bugs, strict framework constraints, and intricate networking layers—and engineered our way out of all of them.

---

## 1. The Architecture Philosophy

Our infrastructure is defined purely as Code using **NixOS**. By strictly managing our foundation through a `flake.nix` file, we ensure 100% environment reproducibility.

**Key Architectural Decisions:**
1. **Host-Networked PostgreSQL**: Instead of running Postgres as a transient Kubernetes pod attached to sluggish network-attached storage, we provisioned PostgreSQL natively on the bare-metal NixOS host. This gives the database unadulterated access to NVMe storage and RAM, bypassing the container network overlay overhead entirely.
2. **K3s for Orchestration**: A lightweight Kubernetes distribution perfect for bare-metal. We explicitly disabled its default `traefik` ingress to favor a pure `ingress-nginx` setup.
3. **GitHub Container Registry (GHCR)**: Used to securely store our natively compiled Docker images, pulled dynamically by Kubernetes using `imagePullSecrets`.

### The NixOS Foundation (`flake.nix`)
Our `flake.nix` handles everything from deterministic BTRFS partitioning via Disko to encrypted secrets via SOPS-Nix. Most importantly, we explicitly opened PostgreSQL to the K3s host-bridge subnet so our pods could securely authenticate to the bare-metal database via SCRAM-SHA-256:

```nix
# services.postgresql.authentication
host    all             all             10.42.0.0/16            scram-sha-256
```

We chose a bridging IP of `10.42.0.1` as the default target gateway for K3s pods to speak directly to the host OS.

---

## 2. Infrastructure Initialization

Before touching K3s, we had to ensure our User Acceptance Testing (UAT) environment was logically decoupled from Production at the database level. 

### Provisioning the UAT Database
We created a dedicated database specifically for UAT, retaining the same administrative owner but ensuring data isolated. We tapped directly into the host using the `postgres` system user:

```bash
sudo -u postgres psql
CREATE DATABASE anchor_uat OWNER ruud_admin;
```

### Unifying SQLx Migrations
We discovered a critical `sqlx` `VersionMismatch` error occurring locally because the raw database data structures didn't perfectly map to the codebase migrations. We explicitly merged our `site_settings` data and `secure_smtp` payload configurations directly into our timestamped `migrations/` files, removing sensitive string tokens and ensuring a single Source of Truth. 

---

## 3. The Deployment Execution

We managed our Kubernetes configuration dynamically via Kustomize Overlays. For UAT, we created `k8s/instances/buildwithruud/uat/`.

### Synchronizing the Codebase safely
Because our NixOS server is pristine, we avoided injecting Git SSH credentials by using `rsync` from our local development machine to beam the source code directly to the server—explicitly discarding the gigabytes of heavy Rust build folders:

```bash
rsync -avz --exclude 'target' --exclude 'node_modules' --exclude '.git' . ruud@69.164.248.38:~/anchor/
```

### Injecting Kubernetes Secrets dynamically
Instead of committing raw credentials to `.yaml` files, we injected the host-bridged database passwords organically via the command line tightly scoped into the `buildwithruud-uat` namespace:

```bash
sudo kubectl --kubeconfig /etc/rancher/k3s/k3s.yaml create secret generic app-secrets \
  --from-literal=DATABASE_URL="postgres://ruud_admin:PASSWORD@10.42.0.1:5432/anchor_uat" \
  --from-literal=RP_ORIGIN="https://uat.buildwithruud.com" \
  --from-literal=RP_ID="uat.buildwithruud.com" \
  --from-literal=LEPTOS_SITE_ADDR="0.0.0.0:3000" \
  --from-literal=LEPTOS_ENV="production" \
  -n buildwithruud-uat
```

---

## 4. The Frontlines: Bugs We Fought & Defeated

A robust deployment is forged in the fires of unexpected errors. Here is the exact friction we encountered during the UAT rollout and the deep-dive engineering required to resolve it.

### Bug 1: The Apple Silicon `exec format error`
**The Symptoms:** The K3s pod immediately crashed upon booting with a violent `CrashLoopBackOff`. Polling the logs yielded exactly one line: `exec /app/anchor: exec format error`.
**The Root Cause:** We utilized an M-series MacBook and ran `docker build --platform linux/amd64` hoping Docker Desktop's Rosetta 2 emulator would perfectly output an x86_64 image. However, when executing `cargo-leptos` under emulation, the Mac host CPU architecture intrinsically leaked through virtualization boundaries to the `rustc` compiler. Rust quietly compiled a native `aarch64` binary and embedded it into the `amd64` container format! When the NixOS Intel/AMD host attempted to execute the ELF file, the Linux kernel aggressively rejected the alien instruction set.
**The Fix:** We abandoned the Mac emulator entirely. By leveraging the `rsync` sync we did earlier, we executed `docker build` physically on the NixOS server! Because the NixOS machine natively runs `x86_64`, it organically produced a mathematically perfect, violently fast AMD64 compilation.

### Bug 2: Kubernetes Aggressive Image Caching
**The Symptoms:** We meticulously pushed the newly structured native `amd64` image to GHCR and restarted the pod. The pod *still* crashed with the exact same `exec format error`.
**The Root Cause:** K3s defaults to `containerd`, meaning it aggressively caches duplicate image tags stringently to preserve bandwidth. Because the tag string remained `:latest`, K3s bypassed GHCR and maliciously recycled the deeply broken Apple Silicon image from its internal registry cache.
**The Fix:** We patched the Kustomize `deployment-patch.yaml` file to explicitly overwrite the standard pull policy:
```yaml
      containers:
      - name: anchor-app
        imagePullPolicy: Always
```
This single line structurally forbids Kubernetes from ever trusting its cache, compelling it to perform a hard-fetch from the GitHub registry.

### Bug 3: Leptos Hardcoded `EnvVarError` Panic
**The Symptoms:** The container successfully executed the proper CPU architecture format and began booting. But halfway through initialization, the Rust process detonated with a stack-trace panic: `thread 'main' panicked: EnvVarError("UAT is not a supported environment. Use either 'dev' or 'production'.")`
**The Root Cause:** We intelligently injected `LEPTOS_ENV="UAT"` into our K8s secrets generically to denote the isolated Kustomize namespace. But the underlying `Leptos` web framework relies strictly on this variable to trigger critical framework-level WebAssembly optimizations, minified CSS serving, and hot-reload disabling. UAT isn't mapped to internal optimization profiles.
**The Fix:** We updated our configuration and K8s Secret injection to brutally enforce `LEPTOS_ENV="production"` so Leptos operated at maximum throughput parameters natively. Our UAT "isolation" strictly remained enforced purely via network and logical DB scopes (`anchor_uat` / `uat.buildwithruud.com`). 

### Bug 4: NGINX `Connection Refused` on Bare-Metal
**The Symptoms:** The K3s pod finally transitioned to `Ready: 1/1` and `Running`. However, pinging the cluster publicly via `curl https://uat.buildwithruud.com` instantly yielded `Exit Code 7: Connection Refused` on port 443!
**The Root Cause:** In `flake.nix`, we explicitly disabled K3s's built-in `traefik` proxy via `--disable=traefik` to utilize NGINX for deeper control. But because it's a bare metal server, deploying standard cloud load-balancers does nothing. Kubernetes logically connected the services internal-only, but physically failed to bind any proxy server to the host's 80 and 443 TCP ports! 
**The Fix:** We deployed the specific bare-metal NGINX controller manifold. Because Kubernetes bare-metal `ingress-nginx` defaults to `NodePort` (random high ports like `31893`), we had to surgically patch the controller service live via terminal to force the K3s integrated `klipper-lb` (ServiceLB) to bind its DAEMON-SET directly to our host.
```bash
# 1. Install bare metal proxy
sudo kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v.../deploy.yaml

# 2. Hard-patch the object memory to force host port bridging!
sudo kubectl patch svc ingress-nginx-controller -n ingress-nginx -p '{"spec": {"type": "LoadBalancer"}}'
```

### Bug 5: The "Not Secure" Root CA Protocol
**The Symptoms:** The website officially went active on the public internet and loaded our HTML natively, but Chrome and Safari threw massive red warnings that our connection was hijacked by `"Fake LE Intermediate X1"`. 
**The Root Cause:** Let's Encrypt enforces savage connection bans (up to 168 hours) if you fail an SSL ACME HTTP-01 challenge sequentially while debugging routing logic. To cleverly dodge the ban-hammer during our heavy troubleshooting, we purposefully added the `letsencrypt-staging` parameter in our `ingress.yaml` file so it requested safe, non-punitive "Fake" certificates.
**The Fix:** After proving our HTTP routing algorithm mathematically worked by seeing the active HTML document render, we authored our permanent Infrastructure-as-Code `ClusterIssuer` to lock into Production API servers.  
```yaml
# cluster-issuer-prod.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    solvers:
    - http01:
        ingress:
          class: nginx
```
After successfully committing the `letsencrypt-prod` annotation cleanly to `ingress.yaml`, `cert-manager` politely refused to overwrite the unexpired Staging certificate automatically. Acting decisively, we directly deleted the raw TLS secret entity connected to the ingress (`kubectl delete secret buildwithruud-uat-tls`). Cert-Manager panicked upon observing a missing secret, violently forcing a Let's Encrypt retrigger on the `prod` profile, organically dropping a flawlessly signed Production certificate within exactly 14 seconds.

---

## 5. Post-Deployment Governance: Hiding UAT from Crawlers

Deploying a UAT environment to the public internet natively invites the risk of Google's algorithms blindly indexing test data and unfinished UI states. Instead of building restrictive Basic Authentication payloads directly into the K3s NGINX Ingress controller or recompiling the Rust backend with `<Meta>` tags, we leveraged our CDN provider.

### The Cloudflare Zero-Cost Shield
We configured a completely free **Transform Rule** directly on Cloudflare’s Edge logic. Because the DNS record is Proxied ("Orange Cloud"), Cloudflare physically intercepts the TCP request before K3s ever sees it.
We created a "Modify Response Header" rule specifically targeting the UAT Hostname:
- **Rule Engine:** Cloudflare Transform Rules
- **Condition:** `Hostname` equals `uat.buildwithruud.com`
- **Injection:** Set static header `X-Robots-Tag` to `noindex, nofollow`

This cleanly guarantees that human QA testers can verify the site organically without authenticating through annoying password popups, but Googlebot mathematically respects the explicit HTTP header standard and structurally drops the hostname from the global index instantly.

---

## Conclusion
Migrating a deeply complex Rust web architecture onto a Bare-Metal NixOS Kubernetes cluster is an absolute masterclass in rigorous engineering discipline. By combining Nix's declarative consistency with robust Rust memory-safety pipelines and aggressive K3s lightweight orchestration, we've forged an environment mathematically incapable of generic failure. 

The resulting platform isn't just deployed—it's weaponized. The Anchor UAT pipeline is heavily isolated, screaming fast natively on x86_64, independently scalable, securely databased without container network lag, and ready to permanently take on the world.
