use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServiceRecord {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub deliverables: Vec<String>,
    pub price_range: Option<String>,
    pub is_visible: bool,
    pub display_order: i32,
}

#[server(GetServices, "/api")]
pub async fn get_services(public_only: bool) -> Result<Vec<ServiceRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let query_str = if public_only {
        "SELECT id, title, description, deliverables, price_range, is_visible, display_order FROM services WHERE is_visible = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    } else {
        "SELECT id, title, description, deliverables, price_range, is_visible, display_order FROM services WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    };

    let rows = sqlx::query(query_str).fetch_all(&state.pool).await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let deliverables_val: serde_json::Value = row.get("deliverables");
            let deliverables = match deliverables_val.as_array() {
                Some(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                None => vec![],
            };

            ServiceRecord {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                deliverables,
                price_range: row.try_get("price_range").unwrap_or(None),
                is_visible: row.get("is_visible"),
                display_order: row.get("display_order"),
            }
        })
        .collect();
    Ok(items)
}

#[server(AddService, "/api")]
pub async fn add_service(
    title: String,
    description: String,
    deliverables: Vec<String>,
    price_range: Option<String>,
    is_visible: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let deliv_json = serde_json::to_value(deliverables).unwrap_or(serde_json::json!([]));
    sqlx::query("INSERT INTO services (tenant_id, title, description, deliverables, price_range, is_visible, display_order) VALUES ($$7, $$1, $$2, $$3, $$4, $$5, $$6)")
        .bind(title).bind(description).bind(deliv_json).bind(price_range).bind(is_visible).bind(display_order)
        .bind(tenant.0)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateService, "/api")]
pub async fn update_service(
    id: i32,
    title: String,
    description: String,
    deliverables: Vec<String>,
    price_range: Option<String>,
    is_visible: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let deliv_json = serde_json::to_value(deliverables).unwrap_or(serde_json::json!([]));
    sqlx::query("UPDATE services SET title = $1, description = $2, deliverables = $3, price_range = $4, is_visible = $5, display_order = $$6 WHERE id = $$7 AND tenant_id IS NOT DISTINCT FROM $$8")
        .bind(title).bind(description).bind(deliv_json).bind(price_range).bind(is_visible).bind(display_order).bind(id)
        .bind(tenant.0)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteService, "/api")]
pub async fn delete_service(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM services WHERE id = $$1 AND tenant_id IS NOT DISTINCT FROM $$2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------
// Case Studies
// ---------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CaseStudyRecord {
    pub id: i32,
    pub client_name: String,
    pub problem: String,
    pub solution: String,
    pub roi_impact: String,
    pub is_visible: bool,
    pub display_order: i32,
}

#[server(GetCaseStudies, "/api")]
pub async fn get_case_studies(public_only: bool) -> Result<Vec<CaseStudyRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let query_str = if public_only {
        "SELECT id, client_name, problem, solution, roi_impact, is_visible, display_order FROM case_studies WHERE is_visible = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    } else {
        "SELECT id, client_name, problem, solution, roi_impact, is_visible, display_order FROM case_studies WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    };

    let rows = sqlx::query(query_str).fetch_all(&state.pool).await?;
    let items = rows
        .into_iter()
        .map(|row| CaseStudyRecord {
            id: row.get("id"),
            client_name: row.get("client_name"),
            problem: row.get("problem"),
            solution: row.get("solution"),
            roi_impact: row.get("roi_impact"),
            is_visible: row.get("is_visible"),
            display_order: row.get("display_order"),
        })
        .collect();
    Ok(items)
}

#[server(AddCaseStudy, "/api")]
pub async fn add_case_study(
    client_name: String,
    problem: String,
    solution: String,
    roi_impact: String,
    is_visible: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("INSERT INTO case_studies (tenant_id, client_name, problem, solution, roi_impact, is_visible, display_order) VALUES ($$7, $$1, $$2, $$3, $$4, $$5, $$6)")
        .bind(client_name).bind(problem).bind(solution).bind(roi_impact).bind(is_visible).bind(display_order).execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateCaseStudy, "/api")]
pub async fn update_case_study(
    id: i32,
    client_name: String,
    problem: String,
    solution: String,
    roi_impact: String,
    is_visible: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("UPDATE case_studies SET client_name = $1, problem = $2, solution = $3, roi_impact = $4, is_visible = $5, display_order = $$6 WHERE id = $$7 AND tenant_id IS NOT DISTINCT FROM $$8")
        .bind(client_name).bind(problem).bind(solution).bind(roi_impact).bind(is_visible).bind(display_order).bind(id).execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteCaseStudy, "/api")]
pub async fn delete_case_study(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM case_studies WHERE id = $$1 AND tenant_id IS NOT DISTINCT FROM $$2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------
// Highlights Gallery
// ---------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HighlightRecord {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub is_visible: bool,
    pub display_order: i32,
}

#[server(GetHighlights, "/api")]
pub async fn get_highlights(public_only: bool) -> Result<Vec<HighlightRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let query_str = if public_only {
        "SELECT id, title, url, image_url, description, is_visible, display_order FROM highlights WHERE is_visible = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    } else {
        "SELECT id, title, url, image_url, description, is_visible, display_order FROM highlights WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    };

    let rows = sqlx::query(query_str).fetch_all(&state.pool).await?;
    let items = rows
        .into_iter()
        .map(|row| HighlightRecord {
            id: row.get("id"),
            title: row.get("title"),
            url: row.get("url"),
            image_url: row.try_get("image_url").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            is_visible: row.get("is_visible"),
            display_order: row.get("display_order"),
        })
        .collect();
    Ok(items)
}

#[server(AddHighlight, "/api")]
pub async fn add_highlight(
    title: String,
    url: String,
    image_url: Option<String>,
    description: Option<String>,
    is_visible: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("INSERT INTO highlights (tenant_id, title, url, image_url, description, is_visible, display_order) VALUES ($$7, $$1, $$2, $$3, $$4, $$5, $$6)")
        .bind(title).bind(url).bind(image_url).bind(description).bind(is_visible).bind(display_order).execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateHighlight, "/api")]
pub async fn update_highlight(
    id: i32,
    title: String,
    url: String,
    image_url: Option<String>,
    description: Option<String>,
    is_visible: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("UPDATE highlights SET title = $1, url = $2, image_url = $3, description = $4, is_visible = $5, display_order = $$6 WHERE id = $$7 AND tenant_id IS NOT DISTINCT FROM $$8")
        .bind(title).bind(url).bind(image_url).bind(description).bind(is_visible).bind(display_order).bind(id).execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteHighlight, "/api")]
pub async fn delete_highlight(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM highlights WHERE id = $$1 AND tenant_id IS NOT DISTINCT FROM $$2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}
