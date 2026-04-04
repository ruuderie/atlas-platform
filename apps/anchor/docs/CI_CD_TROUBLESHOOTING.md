# CI/CD Debugging and Resolution Journey

This document captures the entire diagnostic journey of stabilizing our Woodpecker CI pipeline. It details the exact challenges encountered during our first fully successful multi-tenant deployment, the underlying causes, and the engineering solutions implemented.

## 1. Rust Pipeline Strict Failures (Ecosystem Edition Changes)
**Issue:** The pipeline initially threw an unexpected Cargo error immediately upon attempting `cargo test`: `feature edition2024 is required`.
**Cause:** Foundational dependencies (like `getrandom`) recently updated their manifest requirements to require Rust compiler 1.85+. Our original pipeline YAML was strictly referencing `rust:1.76-slim` for standard CI execution.
**Correction:** Upgraded to `rust:1.85-slim`.

## 2. Leptos Web-Application Desync
**Issue:** Even after updating the compiler, front-end dependencies like `icu-*` forced another crash requesting `rustc 1.86`. 
**Cause:** Our Rust architecture is physically built using **Leptos**. Leptos strictly requires Rust Nightly because it leverages bleeding-edge feature flags (`#![feature(...)]`) that physically do not exist on the standard Stable toolchain.
**Correction:** Completely replaced Woodpecker's `rust` references with `rustlang/rust:nightly-bookworm`. This securely aligns the Woodpecker testing phase natively with our production `Dockerfile` builder phase, completely resolving all compiler desync.

## 3. WebAuthn Test Environmental Isolation
**Issue:** Testing failed throwing `module ssr not found` inside `src/auth.rs`.
**Cause:** `cargo test --workspace` intrinsically runs sequentially without specific feature flags. Because `get_webauthn()` acts entirely within the server bounds (`#[cfg(feature = "ssr")]`), the module did not exist in the isolated testing context.
**Correction:** Safely gated the WebAuthn tests inside a `#[cfg(all(test, feature = "ssr"))]` to verify token signatures exclusively if the module boots into Server Mode.

## 4. Stale Unit-Test Initializations
**Issue:** Missing fields `metadata` and `overrides` mapped to `resume_engine::ResumeEntry`.
**Cause:** The new database migration fields `metadata` and `overrides` were recently mapped into the core `ResumeEntry` struct definition, but the mock structs mapped below it inside the unit test matrix were missed.
**Correction:** Manually defined `metadata: None` and `overrides: None` on the 3 test objects.

## 5. WebAssembly UUID Javascript Translation Crash
**Issue:** The `uuid` crate abruptly crashed the pipeline right at the last Leptos WebAssembly client build (`wasm32-unknown-unknown`), demanding an entropy parameter.
**Cause:** By design, WebAssembly executes natively isolated inside the end user's browser, meaning it does not have an operating system to parse entropy (`/dev/urandom`) from. 
**Correction:** We injected the `js` feature into the `uuid` crate inside `Cargo.toml`. This authorizes WebAssembly to bridge securely backwards and tap directly into `window.crypto.getRandomValues()` natively via JS-bindings!

## 6. Kubectl Permission Denied Volume Loop
**Issue:** The exact step that pushes updates to K3s generated `error loading /kubeconfig.yaml: permission denied`.
**Cause:** By default, standard CI containers like `bitnami/kubectl:latest` execute safely under a designated non-root user map (`uid 1001`). NixOS inherently isolates `/etc/rancher/k3s/k3s.yaml` to strict `chmod 600` root-level access. Woodpecker successfully mounted it to the container, but the virtual user was bounced.
**Correction:** Swapped from `bitnami/kubectl` to `rancher/k3s:latest`. This is the identical official container, executing completely as `root`, parsing the identical permission tree natively.

## 7. Podman Network Namespacing Deflection
**Issue:** Kubernetes connection repeatedly threw `connection refused` pointing strictly toward `127.0.0.1:6443`.
**Cause:** The K3s API naturally listens on the NixOS system's loopback (`127.0.0.1`). When `kubectl` ran inside Woodpecker's remote container, it queried `127.0.0.1`... inside of its own sealed internal container space where nothing was running.
**Correction (Attempt 1):** Injected `network_mode: host` to merge interfaces. However, Woodpecker's YAML strictly schema-checks for untrusted modifications and threw out the file, reporting `Pipeline Definition Not Found`.
**Final Correction:** Removed the schema bypass and natively pointed `kubectl` out of the sandbox to the automatic internal gateway map utilizing `--server=https://host.containers.internal:6443`. Podman seamlessly catches this packet and drops it completely flush back into the NixOS system.

---
**Summary:** The pipeline executes end-to-end identically to our local development environment without altering system permissions.

## 8. NGINX 503 Service Unavailable (Kustomize Secret Overwrite)
**Issue:** After preparing the project for CI/CD, navigating to `https://uat.buildwithruud.com/` threw a Cloudflare 503 error. The `anchor-app` pods were stuck in a continuous `CrashLoopBackOff`, reporting a fatal PostgreSQL authentication error for user `ruud_admin`.
**Cause:** In `k8s/instances/buildwithruud/uat/config.yaml`, the `app-secrets` block was tracked in Git as a `kind: Secret` containing a non-functional `DATABASE_URL` holding the literal string `<PLACEHOLDER>`. When `kubectl apply -k .` was manually issued to prepare the cluster for CI/CD transitions, Kubernetes declaratively evaluated the file and ruthlessly overwrote the *real* cluster secret with the dummy placeholder, permanently locking the pod out of the database.
**Correction:** 
1. Regenerated a new, highly secure password for `ruud_admin` manually via `psql`.
2. Refactored `config.yaml` from `kind: Secret` into a non-sensitive `kind: ConfigMap` (named `app-config`), structurally removing the `DATABASE_URL` from Git completely.
This completely isolates all stateful passwords from Kustomize tracking, ensuring subsequent file applies or deployments never erase database credentials.

---

## 9. Understanding SOPS & Age Encryption Strategies

Because we are decoupling Secrets from Kustomize via **SOPS**, it is critical to understand the cryptographic lifecycle of **Age** keypairs for our Multi-Tenant clusters.

### Handling and Storing Private Keys
When you generate an Age key (`age-keygen`), you receive a **Public Key** (`age1...`) and a **Private Key** (`AGE-SECRET-KEY-1...`).
- **Public Key:** Committed safely to the source code natively inside a `.sops.yaml` configuration file. This allows *anyone* or *any pipeline* to encrypt data without seeing the core secret. 
- **Private Key:** Stored inside **Woodpecker CI Secrets** UI (e.g., as `SOPS_AGE_KEY`), injected exactly at deployment runtime. Alternatively, it can be securely saved in an external cold-storage vault (like 1Password) by platform admins for emergency rollback or local dev decryption. **Never commit the Private Key to Git.**

### The "Lost Key" Data Scenario
**Question:** If I lose the private key, is the data permanently lost?
**Answer:** Technically, yes—the `.enc.yaml` files within the Git repository become completely mathematically unrecoverable. 
**However, the Application still runs!** The Kubernetes cluster and its running databases physically hold the decrypted, plaintext data in their live memory and volumes. You do not lose your live application state or databases. If a key is completely lost, you simply extract the live secret from Kubernetes (`kubectl get secret -o yaml`), generate a brand-new Age key, and re-encrypt a fresh `.enc.yaml` file into your repository.

### Safely Rotating Keys
If a developer leaves or a key is suspected of being compromised, rotating keys is incredibly simple because of the SOPS manifest syntax.
1. Generate the new Age key pair (`age-keygen`).
2. Update `.sops.yaml` with the new Public Key alongside the old Public Key.
3. Run `sops updatekeys k8s/instances/**/secret.enc.yaml`. SOPS seamlessly decrypts the secret using your old private key and simultaneously re-encrypts it using the new public key. 
4. Delete the old key from `.sops.yaml` and upload the new Private Key to Woodpecker CI. 

### Multi-Tenant Architecture Structure
SOPS handles complex multi-tenant segmentation perfectly via the `.sops.yaml` creation rules structure. 
You do **not** need a separate key for every tenant unless strict cryptographic isolation is mandated. The optimal standard is **One Key Per Major Environment**.
```yaml
creation_rules:
  # UAT and DEV share one key
  - path_regex: k8s/instances/.*/(uat|dev)/.*\.enc\.yaml$
    key_groups:
    - age:
      - age1_uat_dev_public_key_here
  # PROD uses a highly restricted key
  - path_regex: k8s/instances/.*/prod/.*\.enc\.yaml$
    key_groups:
    - age:
      - age1_prod_public_key_here
```
This isolates blast radiuses: A compromise of the Dev pipeline key physically cannot decrypt the Production client configurations. 

---

## 10. SOPS Decryption on Stripped K3s Containers
**Issue:** When attempting to install the `sops` binary dynamically within our Woodpecker CI deployment step using native package managers (`apt-get update`), the shell crashed heavily reporting `/bin/sh: apt-get: not found`. Even when migrating to download it physically via `wget`, it threw an enigmatic TLS rejection `wget: not an http or ftp url`.
**Cause:** The target execution image, `rancher/k3s:latest`, structurally relies on incredibly minimal, hyper-hardened `BusyBox` layers. It completely strips standard Linux package managers (`apt`, `apk`) to prevent cross-contamination and dramatically drops the binary size. Most notably, its built-in internal `wget` executable is rigorously compiled statically to completely exclude TLS/HTTPS support! This violently prevented securely pulling the compiled SOPS Go-binary directly from GitHub.
**Correction:** 
Instead of hacking TLS constraints inside stripped cluster interfaces, the architecture was fully decoupled sequentially using Woodpecker multi-image bindings:
1. Created an isolated preceding pipeline step (`decrypt_secrets_uat`) leveraging a generic `alpine:latest` container. Because this image possesses native root trust, it installed a fully-fledged `curl` dynamically, pulled the literal statically linked AMD64 SOPS binary perfectly out of Github over TLS, explicitly handled the `secret.enc.yaml` cryptographic decryption, and saved the parsed credential variables out transparently inside the shared CI-volume (`/woodpecker/src/`).
2. Dropped the existing deployment step back completely into the trusted `rancher/k3s:latest` layer to ingest that natively processed YAML file organically via `kubectl apply -f`, immediately wiping the unencrypted ghost state from the deployment disk before continuing.
