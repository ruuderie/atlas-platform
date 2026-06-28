//! # Atlas Ingress Sidecar
//!
//! Creates and removes Kubernetes Ingress resources for provisioned tenants.
//!
//! ## Routes
//!   GET  /health                     — readiness probe
//!   POST /api/ingress/provision      — create/update Ingress
//!   POST /api/ingress/deprovision    — delete Ingress
//!
//! ## Domain strategy
//!
//! | Domain pattern          | TLS secret          | cert-manager annotation  |
//! |-------------------------|---------------------|--------------------------|
//! | *.dev.atlas.oply.co     | wildcard-tls-dev    | none (pre-existing cert) |
//! | *.uat.atlas.oply.co     | wildcard-tls-uat    | none (pre-existing cert) |
//! | *.atlas.oply.co         | wildcard-tls-prod   | none (pre-existing cert) |
//! | anything else           | {slug}-tls (custom) | letsencrypt-http (HTTP-01)|
//!
//! ## App routing
//!   "property_management" / "folio" → folio (port 80)
//!   "anchor"                         → anchor-app (port 80)
//!   "network_instance"               → network-instance (port 80)

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use k8s_openapi::api::networking::v1::{
    HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend,
    IngressRule, IngressServiceBackend, IngressSpec, IngressTLS,
    ServiceBackendPort,
};
use kube::api::ObjectMeta;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, net::SocketAddr};

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ProvisionPayload {
    tenant_slug: String,
    domain:      String,
    /// Canonical app type: "property_management" | "anchor" | "network_instance"
    #[serde(default = "default_app_slug")]
    app_slug:    String,
}

fn default_app_slug() -> String { "property_management".to_string() }

#[derive(Debug, Deserialize)]
struct DeprovisionPayload {
    tenant_slug: String,
    domain:      String,
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    /// None in local/outside-cluster mode → mock/dry-run
    client: Option<kube::Client>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Maps app_slug → k8s Service name
fn service_for_app(app_slug: &str) -> &'static str {
    match app_slug {
        "property_management" | "folio" => "folio",
        "anchor"                         => "anchor-app",
        "network_instance"               => "network-instance",
        _                                => "folio",
    }
}

/// Returns (tls_secret_name, needs_cert_manager_annotation).
///
/// Wildcard subdomains reference a pre-existing shared TLS Secret.
/// Custom domains get the cert-manager annotation so HTTP-01 auto-provisions a cert.
fn tls_config(domain: &str, tenant_slug: &str) -> (String, bool) {
    if domain.ends_with(".dev.atlas.oply.co") {
        ("wildcard-tls-dev".to_string(),  false)
    } else if domain.ends_with(".uat.atlas.oply.co") {
        ("wildcard-tls-uat".to_string(),  false)
    } else if domain.ends_with(".atlas.oply.co") {
        ("wildcard-tls-prod".to_string(), false)
    } else {
        // Custom client domain — cert-manager provisions via HTTP-01
        (format!("{}-tls", tenant_slug.replace('.', "-")), true)
    }
}

fn build_ingress(
    namespace:   &str,
    tenant_slug: &str,
    domain:      &str,
    app_slug:    &str,
) -> Ingress {
    let service_name           = service_for_app(app_slug);
    let (tls_secret, add_cert) = tls_config(domain, tenant_slug);

    let mut annotations: BTreeMap<String, String> = BTreeMap::new();
    annotations.insert(
        "nginx.ingress.kubernetes.io/ssl-redirect".to_string(),
        "true".to_string(),
    );
    if add_cert {
        annotations.insert(
            "cert-manager.io/cluster-issuer".to_string(),
            "letsencrypt-http".to_string(),
        );
    }

    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert("app.atlas.io/managed-by".to_string(), "ingress-sidecar".to_string());
    labels.insert("app.atlas.io/tenant".to_string(),     tenant_slug.to_string());

    Ingress {
        metadata: ObjectMeta {
            name:        Some(format!("tenant-{}", tenant_slug.replace('.', "-"))),
            namespace:   Some(namespace.to_string()),
            annotations: Some(annotations),
            labels:      Some(labels),
            ..Default::default()
        },
        spec: Some(IngressSpec {
            ingress_class_name: Some("nginx".to_string()),
            tls: Some(vec![IngressTLS {
                hosts:       Some(vec![domain.to_string()]),
                secret_name: Some(tls_secret),
            }]),
            rules: Some(vec![IngressRule {
                host: Some(domain.to_string()),
                http: Some(HTTPIngressRuleValue {
                    paths: vec![HTTPIngressPath {
                        path:      Some("/".to_string()),
                        path_type: "Prefix".to_string(),
                        backend: IngressBackend {
                            service: Some(IngressServiceBackend {
                                name: service_name.to_string(),
                                port: Some(ServiceBackendPort {
                                    number: Some(80),
                                    name:   None,
                                }),
                            }),
                            resource: None,
                        },
                    }],
                }),
            }]),
            ..Default::default()
        }),
        status: None,
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> &'static str { "ok" }

async fn provision_ingress(
    State(state): State<AppState>,
    Json(payload): Json<ProvisionPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        event       = "provision.request",
        tenant_slug = %payload.tenant_slug,
        domain      = %payload.domain,
        app_slug    = %payload.app_slug,
    );

    let Some(client) = &state.client else {
        tracing::info!(
            event = "provision.dry_run",
            tenant_slug = %payload.tenant_slug,
            domain      = %payload.domain,
        );
        return Ok(Json(json!({
            "status":  "success",
            "message": format!("Mock mode: Ingress tenant-{} (dry-run)", payload.tenant_slug)
        })));
    };

    let namespace = std::env::var("KUBERNETES_NAMESPACE")
        .unwrap_or_else(|_| "atlas-dev".to_string());
    let ingresses: kube::Api<Ingress> = kube::Api::namespaced(client.clone(), &namespace);

    let ingress_name = format!("tenant-{}", payload.tenant_slug.replace('.', "-"));
    let ingress_obj  = build_ingress(&namespace, &payload.tenant_slug, &payload.domain, &payload.app_slug);

    // Server-side apply — creates or updates idempotently
    let patch        = kube::api::Patch::Apply(&ingress_obj);
    let patch_params = kube::api::PatchParams::apply("ingress-sidecar").force();

    match ingresses.patch(&ingress_name, &patch_params, &patch).await {
        Ok(_) => {
            tracing::info!(event = "provision.success", ingress = %ingress_name, domain = %payload.domain);
            Ok(Json(json!({
                "status":  "success",
                "message": format!("Ingress {} applied for {}", ingress_name, payload.domain)
            })))
        }
        Err(e) => {
            tracing::error!(event = "provision.error", ingress = %ingress_name, error = %e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": format!("Failed to apply ingress: {e}") })),
            ))
        }
    }
}

async fn deprovision_ingress(
    State(state): State<AppState>,
    Json(payload): Json<DeprovisionPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        event       = "deprovision.request",
        tenant_slug = %payload.tenant_slug,
        domain      = %payload.domain,
    );

    let Some(client) = &state.client else {
        tracing::info!(event = "deprovision.dry_run", tenant_slug = %payload.tenant_slug);
        return Ok(Json(json!({
            "status":  "success",
            "message": format!("Mock mode: Ingress tenant-{} removed (dry-run)", payload.tenant_slug)
        })));
    };

    let namespace = std::env::var("KUBERNETES_NAMESPACE")
        .unwrap_or_else(|_| "atlas-dev".to_string());
    let ingresses: kube::Api<Ingress> = kube::Api::namespaced(client.clone(), &namespace);
    let ingress_name = format!("tenant-{}", payload.tenant_slug.replace('.', "-"));

    match ingresses.delete(&ingress_name, &kube::api::DeleteParams::default()).await {
        Ok(_) => {
            tracing::info!(event = "deprovision.success", ingress = %ingress_name);
            Ok(Json(json!({ "status": "success", "message": format!("Ingress {} deleted", ingress_name) })))
        }
        Err(kube::Error::Api(err)) if err.code == 404 => {
            tracing::warn!(event = "deprovision.not_found", ingress = %ingress_name);
            Ok(Json(json!({ "status": "success", "message": "Already gone" })))
        }
        Err(e) => {
            tracing::error!(event = "deprovision.error", ingress = %ingress_name, error = %e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": format!("Failed to delete ingress: {e}") })),
            ))
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Install the ring crypto provider — required because kube uses rustls
    // but doesn't select a provider, so we must select one explicitly.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls ring crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ingress_sidecar=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Ingress Sidecar on port 8085...");

    let client = match kube::Client::try_default().await {
        Ok(c)  => {
            tracing::info!("Kubernetes client initialized");
            Some(c)
        }
        Err(e) => {
            tracing::warn!(
                "k8s client unavailable (outside cluster?): {}. Running in dry-run mode.", e
            );
            None
        }
    };

    let state = AppState { client };

    let app = Router::new()
        .route("/health",                  get(health))
        .route("/api/ingress/provision",   post(provision_ingress))
        .route("/api/ingress/deprovision", post(deprovision_ingress))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8085));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
