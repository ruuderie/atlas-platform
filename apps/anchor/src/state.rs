use axum::extract::FromRef;
use leptos_axum::LeptosOptions;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TenantContext(pub Option<Uuid>);

#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: PgPool,
}
