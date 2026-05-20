use axum::{
    routing::post,
    Router,
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use kube::api::ObjectMeta;
use k8s_openapi::api::networking::v1::{
    Ingress, IngressSpec, IngressRule, IngressBackend,
    HTTPIngressRuleValue, HTTPIngressPath, IngressTLS, IngressServiceBackend, ServiceBackendPort
};
use std::net::SocketAddr;

#[derive(Deserialize)]
struct IngressPayload {
    tenant_slug: String,
    domain: String,
}

#[derive(Clone)]
struct AppState {
    client: Option<kube::Client>,
}

fn build_ingress(namespace: &str, tenant_slug: &str, domain: &str) -> Ingress {
    Ingress {
        metadata: ObjectMeta {
            name: Some(format!("tenant-{}", tenant_slug)),
            namespace: Some(namespace.to_string()),
            annotations: Some([
                ("cert-manager.io/cluster-issuer".to_string(), "letsencrypt-prod".to_string()),
            ].into_iter().collect()),
            ..Default::default()
        },
        spec: Some(IngressSpec {
            ingress_class_name: Some("nginx".to_string()),
            tls: Some(vec![
                IngressTLS {
                    hosts: Some(vec![domain.to_string()]),
                    secret_name: Some(format!("tenant-{}-tls", tenant_slug)),
                }
            ]),
            rules: Some(vec![
                IngressRule {
                    host: Some(domain.to_string()),
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![
                            HTTPIngressPath {
                                path: Some("/".to_string()),
                                path_type: "Prefix".to_string(),
                                backend: IngressBackend {
                                    service: Some(IngressServiceBackend {
                                        name: "anchor-app".to_string(),
                                        port: Some(ServiceBackendPort {
                                            number: Some(80),
                                            name: None,
                                        }),
                                    }),
                                    resource: None,
                                },
                            }
                        ],
                    }),
                }
            ]),
            ..Default::default()
        }),
        status: None,
    }
}

async fn provision_ingress(
    State(state): State<AppState>,
    Json(payload): Json<IngressPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        "Received provision request for tenant_slug='{}', domain='{}'",
        payload.tenant_slug, payload.domain
    );

    let client = match &state.client {
        Some(c) => c,
        None => {
            tracing::info!("Mock Mode: Successfully (dry-run) provisioned ingress for tenant='{}', domain='{}'", payload.tenant_slug, payload.domain);
            return Ok(Json(json!({
                "status": "success",
                "message": format!("Mock Mode: Ingress tenant-{} created successfully", payload.tenant_slug)
            })));
        }
    };

    let namespace = std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "default".to_string());
    let ingresses: kube::Api<Ingress> = kube::Api::namespaced(client.clone(), &namespace);
    
    let ingress_name = format!("tenant-{}", payload.tenant_slug);
    let ingress_obj = build_ingress(&namespace, &payload.tenant_slug, &payload.domain);

    match ingresses.create(&kube::api::PostParams::default(), &ingress_obj).await {
        Ok(_) => {
            tracing::info!("Successfully created Ingress {}", ingress_name);
        }
        Err(kube::Error::Api(err)) if err.code == 409 => {
            tracing::info!("Ingress {} already exists. Updating/replacing...", ingress_name);
            let patch = kube::api::Patch::Apply(&ingress_obj);
            let patch_params = kube::api::PatchParams::apply("ingress-sidecar").force();
            ingresses.patch(&ingress_name, &patch_params, &patch)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to patch Ingress {}: {}", ingress_name, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("Failed to patch ingress: {e}") })))
                })?;
            tracing::info!("Successfully patched Ingress {}", ingress_name);
        }
        Err(e) => {
            tracing::error!("Failed to create Ingress {}: {}", ingress_name, e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("Failed to create ingress: {e}") }))));
        }
    }

    Ok(Json(json!({
        "status": "success",
        "message": format!("Ingress {} created successfully", ingress_name)
    })))
}

async fn deprovision_ingress(
    State(state): State<AppState>,
    Json(payload): Json<IngressPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::info!(
        "Received deprovision request for tenant_slug='{}', domain='{}'",
        payload.tenant_slug, payload.domain
    );

    let client = match &state.client {
        Some(c) => c,
        None => {
            tracing::info!("Mock Mode: Successfully (dry-run) deprovisioned ingress for tenant='{}'", payload.tenant_slug);
            return Ok(Json(json!({
                "status": "success",
                "message": format!("Mock Mode: Ingress tenant-{} deleted successfully", payload.tenant_slug)
            })));
        }
    };

    let namespace = std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "default".to_string());
    let ingresses: kube::Api<Ingress> = kube::Api::namespaced(client.clone(), &namespace);
    
    let ingress_name = format!("tenant-{}", payload.tenant_slug);
    match ingresses.delete(&ingress_name, &kube::api::DeleteParams::default()).await {
        Ok(_) => {
            tracing::info!("Successfully deleted Ingress {}", ingress_name);
        }
        Err(kube::Error::Api(err)) if err.code == 404 => {
            tracing::info!("Ingress {} not found, nothing to delete.", ingress_name);
        }
        Err(e) => {
            tracing::error!("Failed to delete Ingress {}: {}", ingress_name, e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("Failed to delete ingress: {e}") }))));
        }
    }

    Ok(Json(json!({
        "status": "success",
        "message": format!("Ingress {} deleted successfully", ingress_name)
    })))
}

#[tokio::main]
async fn main() {
    // Install the ring crypto provider before any TLS code runs.
    // Required because both `kube` and `reqwest` use rustls but neither
    // enables a provider feature, so we must select one explicitly.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls ring crypto provider");

    tracing_subscriber::fmt::init();
    tracing::info!("Starting Ingress Sidecar on port 8085...");

    let client = match kube::Client::try_default().await {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::warn!("Failed to initialize Kubernetes client (outside cluster?): {}. Running in mock mode.", e);
            None
        }
    };

    let state = AppState { client };

    let app = Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .route("/api/ingress/provision", post(provision_ingress))
        .route("/api/ingress/deprovision", post(deprovision_ingress))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8085));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
