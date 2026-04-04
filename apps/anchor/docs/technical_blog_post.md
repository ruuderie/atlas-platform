# Architecting a Multi-Tenant SaaS with Zero-Touch Kubernetes Ingress

Scaling a single-tenant application into a dynamic, multi-tenant B2B platform presents fascinating infrastructural challenges. For the **Anchor Platform**, the goal was clear: allow clients to purchase isolated instances of the application on-demand, support custom domains without manual server restarts, and guarantee absolute data isolation.

Here is a deep dive into the architectural decisions that enabled us to achieve a **Zero-Touch GitOps Provisioning System** using NixOS, K3s, and Rust.

## 1. The Challenge: Bottlenecking at the Edge

Traditionally, managing routing for multiple tenants looks like this:
1. A client buys a new instance and wants to use `client.com`.
2. A sysadmin SSH's into the server, edits the Nginx configuration, adds an SSL block, and restarts the web server.

In our setup, the edge layer is managed declaratively by **NixOS (`flake.nix`)**. Rebuilding the entire operating system configuration every time a tenant signs up is an unacceptable operational bottleneck.

## 2. The Solution: SNI TCP Passthrough

We decoupled the edge router from the routing logic. Instead of NixOS trying to manage certificates for every possible domain, we implemented **SNI (Server Name Indication) Passthrough**. 

In `flake.nix`, our Nginx server is configured with a raw TCP stream module on Port 443:
```nginx
stream {
  map $ssl_preread_server_name $https_backend {
    ci.oply.co         127.0.0.1:8443;
    grafana.oply.co    127.0.0.1:8443;
    default            127.0.0.1:32443; # Pipe everything else into K3s encrypted!
  }
}
```

If traffic hits the server for our core internal tools (CI, Grafana), Nginx handles it. But if traffic arrives for `buildwithruud.com`, `tesla.anchor.oply.co`, or any other unknown domain, Nginx acts as a blind tunnel. It streams the raw encrypted bytes directly into our internal Kubernetes orchestrator on port `32443`.

## 3. Dynamic Kubernetes Ingress & SSL

Inside our K3s cluster, we run **ingress-nginx** and **cert-manager**. 

Because `ingress-nginx` is listening to that `32443` tunnel, it receives the request exactly as the browser sent it. It reads the Host header (`client.com`) and dynamically routes it to the specific client's isolated namespace (`tenant-client`). 

Simultaneously, `cert-manager` instantly recognizes a new domain, automatically solves the Let's Encrypt ACME challenge, and provisions an SSL certificate internally. 

The result? **Zero-Touch Host Networking**. A new client can point their custom domain to our IP, and Kubernetes will handle secure routing instantaneously. NixOS never even knows the domain exists!

## 4. Securing Orchestration: The GitOps Provisioning Engine

To actually provision the isolated namespace, we needed an orchestration engine. Our Superadmin application is built in Rust using Leptos/Axum. 

Giving a public-facing web app elevated `cluster-admin` privileges to create namespaces is a severe security anti-pattern. If compromised, an attacker gains cluster ownership.

**The GitOps Broker:**
Instead of raw Kubernetes API calls, our Rust backend utilizes GitHub as a secure broker.
1. The user purchases an instance.
2. The Rust backend generates a new client Kustomize folder dynamically (cloning our `tenant-template`).
3. The Rust backend uses the GitHub API to commit this folder directly to the `main` branch.
4. **Woodpecker CI** natively detects the commit and securely applies the `kubectl` changes behind the firewall.

## 5. The Multi-Environment Sandbox

Every enterprise client receives a complete Software Development Lifecycle (SDLC) structure automatically:
*   `instances/client/prod/` binds to `client.anchor.oply.co`
*   `instances/client/uat/` binds to `client-uat.anchor.oply.co`

Each environment consists of completely isolated Rust App Pods and isolated Postgres Databases. We control versioning simply by targeting Docker tags (e.g., the UAT folder deploys `anchor-app:latest`, while the Prod folder deploys `anchor-app:stable`).

## Conclusion

By shifting imperative infrastructure into declarative GitOps, decoupling edge SSL termination via SNI Passthrough, and isolating tenant state at the namespace level, the Anchor Platform is now a hyper-scalable, secure SaaS product capable of handling thousands of custom domains seamlessly.
