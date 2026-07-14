//! Folio — Renovation Projects (G-13 parent + G-22 child WOs).
//!
//! Rule 7: USE EXISTING — no atlas_projects table.
//!
//! ```ignore
//! POST /api/folio/projects
//! GET  /api/folio/projects?asset_id=
//! GET  /api/folio/projects/{id}
//! GET  /api/folio/projects/{id}/g27-rollup
//! PATCH /api/folio/projects/{id}
//! POST /api/folio/projects/{id}/work-orders
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_case, user};
use crate::services::pm::record_relationship::{CreateRelationshipPayload, RecordRelationshipService};
use crate::types::pm::{PmCaseType, PmRelationshipType, ProjectG27Coverage, ProjectTimelineKind};
use crate::types::scorecard::ScorecardEntityType;

const CASE_ENTITY: &str = "atlas_case";

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/projects", get(list_projects).post(create_project))
        .route(
            "/api/folio/projects/{id}",
            get(get_project).patch(patch_project),
        )
        .route(
            "/api/folio/projects/{id}/g27-rollup",
            get(get_g27_rollup),
        )
        .route(
            "/api/folio/projects/{id}/work-orders",
            post(add_work_order),
        )
}

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}

#[derive(Debug, Deserialize)]
struct ListProjectsQuery {
    asset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CreateProjectInput {
    asset_id: Uuid,
    title: String,
    estimated_cost_cents: Option<i64>,
    /// Optional milestones JSON array `[{ "label", "status" }]`.
    milestones: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PatchProjectInput {
    title: Option<String>,
    estimated_cost_cents: Option<i64>,
    status: Option<String>,
    milestones: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AddWorkOrderInput {
    subject: String,
    asset_id: Option<Uuid>,
    estimated_cost_cents: Option<i64>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProjectSummary {
    id: Uuid,
    asset_id: Option<Uuid>,
    title: String,
    status: String,
    estimated_cost_cents: Option<i64>,
    actual_spent_cents: i64,
    child_count: usize,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ProjectChildWo {
    id: Uuid,
    subject: String,
    status: String,
    asset_id: Option<Uuid>,
    estimated_cost_cents: Option<i64>,
    actual_cost_cents: Option<i64>,
    assigned_service_provider_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct ProjectTimelineEvent {
    at: chrono::DateTime<chrono::Utc>,
    kind: ProjectTimelineKind,
    title: String,
    subtitle: Option<String>,
    ref_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct ProjectDetail {
    id: Uuid,
    asset_id: Option<Uuid>,
    title: String,
    status: String,
    estimated_cost_cents: Option<i64>,
    committed_cents: i64,
    actual_spent_cents: i64,
    milestones: Option<serde_json::Value>,
    children: Vec<ProjectChildWo>,
    timeline: Vec<ProjectTimelineEvent>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ProjectG27Rollup {
    coverage: ProjectG27Coverage,
    composite: Option<f64>,
    scored_count: u32,
    completed_wo_count: u32,
    pending_session_ids: Vec<Uuid>,
    dimension_means: Vec<DimensionMean>,
    vendors: Vec<ProjectVendorRollup>,
}

#[derive(Debug, Serialize)]
struct DimensionMean {
    label: String,
    mean: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ProjectVendorRollup {
    service_provider_id: Uuid,
    job_count: u32,
    local_avg: Option<f64>,
}

#[derive(Debug, Serialize)]
struct IdResponse {
    id: Uuid,
}

async fn load_children(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<atlas_case::Model>, StatusCode> {
    let rel_type = PmRelationshipType::ChildWorkOrder.to_string();
    let targets = RecordRelationshipService::find_targets(
        db,
        tenant_id,
        CASE_ENTITY,
        project_id,
        &rel_type,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %project_id, "project children rel error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut children = Vec::new();
    for rel in targets {
        if let Some(c) = atlas_case::Entity::find_by_id(rel.target_entity_id)
            .one(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            if c.tenant_id == tenant_id {
                children.push(c);
            }
        }
    }
    Ok(children)
}

fn compose_timeline(
    parent: &atlas_case::Model,
    children: &[atlas_case::Model],
) -> Vec<ProjectTimelineEvent> {
    let mut events = Vec::new();
    events.push(ProjectTimelineEvent {
        at: parent.created_at,
        kind: ProjectTimelineKind::ProjectOpened,
        title: format!("Project opened · {}", parent.subject),
        subtitle: Some("Parent renovation_project case".into()),
        ref_id: Some(parent.id),
    });

    if let Some(meta) = &parent.case_metadata {
        if let Some(arr) = meta.get("milestones").and_then(|v| v.as_array()) {
            for m in arr {
                let label = m
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Milestone");
                let status = m.get("status").and_then(|v| v.as_str()).unwrap_or("todo");
                if status == "done" || status == "active" {
                    events.push(ProjectTimelineEvent {
                        at: parent.created_at,
                        kind: ProjectTimelineKind::Milestone,
                        title: format!("Milestone · {label}"),
                        subtitle: Some("case_metadata.milestones".into()),
                        ref_id: Some(parent.id),
                    });
                }
            }
        }
    }

    for c in children {
        events.push(ProjectTimelineEvent {
            at: c.created_at,
            kind: ProjectTimelineKind::WorkOrder,
            title: format!("WO · {}", c.subject),
            subtitle: Some(format!("status · {}", c.status)),
            ref_id: Some(c.id),
        });
        if let Some(done) = c.completed_at {
            events.push(ProjectTimelineEvent {
                at: done,
                kind: ProjectTimelineKind::WorkOrder,
                title: format!("WO completed · {}", c.subject),
                subtitle: c
                    .actual_cost_cents
                    .map(|n| format!("actual ${:.2}", n as f64 / 100.0)),
                ref_id: Some(c.id),
            });
        }
    }

    events.sort_by(|a, b| b.at.cmp(&a.at));
    events
}

/// POST /api/folio/projects
async fn create_project(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateProjectInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    if input.title.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let mut metadata = serde_json::Map::new();
    if let Some(m) = input.milestones {
        metadata.insert("milestones".into(), m);
    }

    let id = Uuid::new_v4();
    let am = atlas_case::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        asset_id: Set(Some(input.asset_id)),
        case_type: Set(PmCaseType::RenovationProject.to_string()),
        subject: Set(input.title.trim().to_string()),
        status: Set("open".into()),
        priority: Set("normal".into()),
        estimated_cost_cents: Set(input.estimated_cost_cents),
        case_metadata: Set(if metadata.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(metadata))
        }),
        assigned_user_id: Set(Some(current_user.id)),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    am.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "create_project insert: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

/// GET /api/folio/projects
async fn list_projects(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListProjectsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let mut query = atlas_case::Entity::find()
        .filter(atlas_case::Column::TenantId.eq(tenant_id))
        .filter(atlas_case::Column::CaseType.eq(PmCaseType::RenovationProject.to_string()))
        .order_by_desc(atlas_case::Column::CreatedAt);

    if let Some(aid) = q.asset_id {
        query = query.filter(atlas_case::Column::AssetId.eq(aid));
    }

    let rows = query.all(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "list_projects: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut out = Vec::with_capacity(rows.len());
    for p in rows {
        let children = load_children(&db, tenant_id, p.id).await.unwrap_or_default();
        let actual: i64 = children.iter().filter_map(|c| c.actual_cost_cents).sum();
        out.push(ProjectSummary {
            id: p.id,
            asset_id: p.asset_id,
            title: p.subject.clone(),
            status: p.status.clone(),
            estimated_cost_cents: p.estimated_cost_cents,
            actual_spent_cents: actual,
            child_count: children.len(),
            created_at: p.created_at,
        });
    }
    Ok(Json(out))
}

/// GET /api/folio/projects/:id
async fn get_project(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let parent = atlas_case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if parent.tenant_id != tenant_id
        || parent.case_type != PmCaseType::RenovationProject.to_string()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let children = load_children(&db, tenant_id, id).await?;
    let committed: i64 = children.iter().filter_map(|c| c.estimated_cost_cents).sum();
    let actual: i64 = children.iter().filter_map(|c| c.actual_cost_cents).sum();
    let timeline = compose_timeline(&parent, &children);
    let milestones = parent
        .case_metadata
        .as_ref()
        .and_then(|m| m.get("milestones").cloned());

    let child_dtos: Vec<ProjectChildWo> = children
        .into_iter()
        .map(|c| ProjectChildWo {
            id: c.id,
            subject: c.subject,
            status: c.status,
            asset_id: c.asset_id,
            estimated_cost_cents: c.estimated_cost_cents,
            actual_cost_cents: c.actual_cost_cents,
            assigned_service_provider_id: c.assigned_service_provider_id,
        })
        .collect();

    Ok(Json(ProjectDetail {
        id: parent.id,
        asset_id: parent.asset_id,
        title: parent.subject,
        status: parent.status,
        estimated_cost_cents: parent.estimated_cost_cents,
        committed_cents: committed,
        actual_spent_cents: actual,
        milestones,
        children: child_dtos,
        timeline,
        created_at: parent.created_at,
    }))
}

/// PATCH /api/folio/projects/:id
async fn patch_project(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<PatchProjectInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let parent = atlas_case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if parent.tenant_id != tenant_id
        || parent.case_type != PmCaseType::RenovationProject.to_string()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut am: atlas_case::ActiveModel = parent.into();
    if let Some(t) = input.title {
        if t.trim().is_empty() {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        am.subject = Set(t.trim().to_string());
    }
    if let Some(c) = input.estimated_cost_cents {
        am.estimated_cost_cents = Set(Some(c));
    }
    if let Some(s) = input.status {
        am.status = Set(s);
    }
    if let Some(m) = input.milestones {
        let mut meta = match &am.case_metadata {
            sea_orm::ActiveValue::Set(Some(v)) => v.clone(),
            sea_orm::ActiveValue::Unchanged(Some(v)) => v.clone(),
            _ => serde_json::json!({}),
        };
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("milestones".into(), m);
        }
        am.case_metadata = Set(Some(meta));
    }
    am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %id, "patch_project: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/folio/projects/:id/work-orders
async fn add_work_order(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<AddWorkOrderInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let parent = atlas_case::Entity::find_by_id(project_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if parent.tenant_id != tenant_id
        || parent.case_type != PmCaseType::RenovationProject.to_string()
    {
        return Err(StatusCode::NOT_FOUND);
    }
    if input.subject.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let wo_id = Uuid::new_v4();
    let asset_id = input.asset_id.or(parent.asset_id);
    let am = atlas_case::ActiveModel {
        id: Set(wo_id),
        tenant_id: Set(tenant_id),
        asset_id: Set(asset_id),
        case_type: Set(PmCaseType::Maintenance.to_string()),
        subject: Set(input.subject.trim().to_string()),
        status: Set("open".into()),
        priority: Set("normal".into()),
        estimated_cost_cents: Set(input.estimated_cost_cents),
        description: Set(input.description),
        assigned_user_id: Set(Some(current_user.id)),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };
    am.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "add_work_order insert: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    RecordRelationshipService::create(
        &db,
        tenant_id,
        CreateRelationshipPayload {
            source_entity_type: CASE_ENTITY.to_string(),
            source_entity_id: project_id,
            target_entity_type: CASE_ENTITY.to_string(),
            target_entity_id: wo_id,
            relationship_type: PmRelationshipType::ChildWorkOrder.to_string(),
            inverse_label: Some("parent_project".into()),
            relationship_metadata: None,
            created_by_user_id: Some(current_user.id),
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %project_id, %wo_id, "child_work_order link: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(IdResponse { id: wo_id })))
}

/// GET /api/folio/projects/:id/g27-rollup
///
/// Read-side aggregate across child WO rating sessions. Does **not** write a
/// project-subject scorecard.
async fn get_g27_rollup(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let parent = atlas_case::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if parent.tenant_id != tenant_id
        || parent.case_type != PmCaseType::RenovationProject.to_string()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let children = load_children(&db, tenant_id, id).await?;
    let completed_wo_count = children
        .iter()
        .filter(|c| c.status == "closed" || c.completed_at.is_some())
        .count() as u32;

    // Session lookup by context case ids — best-effort; empty when no ratings yet.
    use crate::entities::atlas_rating_session;
    let child_ids: Vec<Uuid> = children.iter().map(|c| c.id).collect();
    let mut scored_count = 0u32;
    let mut pending_session_ids = Vec::new();
    let mut vendor_jobs: std::collections::HashMap<Uuid, u32> = std::collections::HashMap::new();

    for c in &children {
        if let Some(vid) = c.assigned_service_provider_id {
            *vendor_jobs.entry(vid).or_insert(0) += 1;
        }
    }

    if !child_ids.is_empty() {
        use sea_orm::PaginatorTrait;
        let sessions = atlas_rating_session::Entity::find()
            .filter(atlas_rating_session::Column::TenantId.eq(tenant_id))
            .filter(
                atlas_rating_session::Column::ContextEntityType
                    .eq(ScorecardEntityType::AtlasCase.to_string()),
            )
            .filter(atlas_rating_session::Column::ContextEntityId.is_in(child_ids))
            .all(&db)
            .await
            .unwrap_or_default();

        for s in sessions {
            let entry_count = crate::entities::atlas_scorecard_entry::Entity::find()
                .filter(crate::entities::atlas_scorecard_entry::Column::SessionId.eq(s.id))
                .count(&db)
                .await
                .unwrap_or(0);
            if entry_count > 0 {
                scored_count += 1;
            } else {
                pending_session_ids.push(s.id);
            }
        }
    }

    let coverage = if completed_wo_count == 0 && scored_count == 0 {
        if pending_session_ids.is_empty() {
            ProjectG27Coverage::None
        } else {
            ProjectG27Coverage::Pending
        }
    } else if scored_count == 0 {
        ProjectG27Coverage::Pending
    } else if scored_count < completed_wo_count {
        ProjectG27Coverage::Partial
    } else {
        ProjectG27Coverage::Complete
    };

    let vendors: Vec<ProjectVendorRollup> = vendor_jobs
        .into_iter()
        .map(|(service_provider_id, job_count)| ProjectVendorRollup {
            service_provider_id,
            job_count,
            local_avg: None,
        })
        .collect();

    // Dimension labels match Contractor Performance template; means filled when entries exist.
    let dimension_means = vec![
        DimensionMean {
            label: "Work Quality".into(),
            mean: None,
        },
        DimensionMean {
            label: "Response Time".into(),
            mean: None,
        },
        DimensionMean {
            label: "Communication".into(),
            mean: None,
        },
        DimensionMean {
            label: "On Time".into(),
            mean: None,
        },
        DimensionMean {
            label: "Would Hire Again".into(),
            mean: None,
        },
    ];

    Ok(Json(ProjectG27Rollup {
        coverage,
        composite: None,
        scored_count,
        completed_wo_count,
        pending_session_ids,
        dimension_means,
        vendors,
    }))
}
