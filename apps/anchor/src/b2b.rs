use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServiceRecord {
    pub id: uuid::Uuid,
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
        "SELECT id, title, payload->>'description' as description, payload->'deliverables' as deliverables, payload->>'price_range' as price_range, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'service' AND status = 'published' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    } else {
        "SELECT id, title, payload->>'description' as description, payload->'deliverables' as deliverables, payload->>'price_range' as price_range, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'service' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
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
    let payload = serde_json::json!({
        "description": description,
        "deliverables": deliv_json,
        "price_range": price_range
    });
    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload, status, display_order) VALUES ($1, 'service', $2, $3, $4, $5)")
        .bind(tenant.0)
        .bind(title)
        .bind(payload)
        .bind(if is_visible { "published" } else { "hidden" })
        .bind(display_order)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateService, "/api")]
pub async fn update_service(
    id: uuid::Uuid,
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
    let payload = serde_json::json!({
        "description": description,
        "deliverables": deliv_json,
        "price_range": price_range
    });
    sqlx::query("UPDATE app_content SET title = $1, payload = $2, status = $3, display_order = $4 WHERE id = $5 AND tenant_id IS NOT DISTINCT FROM $6 AND collection_type = 'service'")
        .bind(title)
        .bind(payload)
        .bind(if is_visible { "published" } else { "hidden" })
        .bind(display_order)
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteService, "/api")]
pub async fn delete_service(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1 AND collection_type = 'service' AND tenant_id IS NOT DISTINCT FROM $2")
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
    pub id: uuid::Uuid,
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
        "SELECT id, payload->>'client_name' as client_name, payload->>'problem' as problem, payload->>'solution' as solution, payload->>'roi_impact' as roi_impact, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'case_study' AND status = 'published' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    } else {
        "SELECT id, payload->>'client_name' as client_name, payload->>'problem' as problem, payload->>'solution' as solution, payload->>'roi_impact' as roi_impact, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'case_study' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
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
    let payload = serde_json::json!({
        "client_name": client_name,
        "problem": problem,
        "solution": solution,
        "roi_impact": roi_impact
    });
    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload, status, display_order) VALUES ($1, 'case_study', 'Case Study', $2, $3, $4)")
        .bind(tenant.0)
        .bind(payload)
        .bind(if is_visible { "published" } else { "hidden" })
        .bind(display_order)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateCaseStudy, "/api")]
pub async fn update_case_study(
    id: uuid::Uuid,
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
    let payload = serde_json::json!({
        "client_name": client_name,
        "problem": problem,
        "solution": solution,
        "roi_impact": roi_impact
    });
    sqlx::query("UPDATE app_content SET payload = $1, status = $2, display_order = $3 WHERE id = $4 AND tenant_id IS NOT DISTINCT FROM $5 AND collection_type = 'case_study'")
        .bind(payload)
        .bind(if is_visible { "published" } else { "hidden" })
        .bind(display_order)
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteCaseStudy, "/api")]
pub async fn delete_case_study(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1 AND collection_type = 'case_study' AND tenant_id IS NOT DISTINCT FROM $2")
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
    pub id: uuid::Uuid,
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
        "SELECT id, title, payload->>'url' as url, payload->>'image_url' as image_url, payload->>'description' as description, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'highlight' AND status = 'published' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
    } else {
        "SELECT id, title, payload->>'url' as url, payload->>'image_url' as image_url, payload->>'description' as description, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'highlight' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
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
    let payload = serde_json::json!({
        "url": url,
        "image_url": image_url,
        "description": description
    });
    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload, status, display_order) VALUES ($1, 'highlight', $2, $3, $4, $5)")
        .bind(tenant.0)
        .bind(title)
        .bind(payload)
        .bind(if is_visible { "published" } else { "hidden" })
        .bind(display_order)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateHighlight, "/api")]
pub async fn update_highlight(
    id: uuid::Uuid,
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
    let payload = serde_json::json!({
        "url": url,
        "image_url": image_url,
        "description": description
    });
    sqlx::query("UPDATE app_content SET title = $1, payload = $2, status = $3, display_order = $4 WHERE id = $5 AND tenant_id IS NOT DISTINCT FROM $6 AND collection_type = 'highlight'")
        .bind(title)
        .bind(payload)
        .bind(if is_visible { "published" } else { "hidden" })
        .bind(display_order)
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteHighlight, "/api")]
pub async fn delete_highlight(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1 AND collection_type = 'highlight' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}
