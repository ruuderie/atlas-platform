use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use uuid::Uuid;
use tracing;

/// Middleware that generates a unique `request_id` for every incoming HTTP request
/// and injects it into the request extensions so handlers can use it for structured logging.
pub async fn request_id_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let request_id = Uuid::new_v4();
    
    // Inject into extensions so handlers can access it
    req.extensions_mut().insert(request_id);
    
    // Also log it at the very start of the request
    tracing::info!(
        event = "http.request.started",
        request_id = %request_id,
        method = %req.method(),
        path = %req.uri().path()
    );
    
    let response = next.run(req).await;
    
    response
}

/// Helper to extract request_id from extensions (use in handlers)
pub fn get_request_id(req: &Request<Body>) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}
