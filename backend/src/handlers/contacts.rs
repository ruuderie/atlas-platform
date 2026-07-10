use axum::{
    extract::{Extension, Path, Json, Query},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    ColumnTrait, ActiveModelTrait, Order,
};
use crate::entities::atlas_contact;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

// ── Notes / Activities — kept via legacy entities (no atlas_ equivalent yet) ──
use crate::entities::{note, activity, user};
use crate::models::note::{NoteModel, CreateNoteInput};
use crate::models::activity::{ActivityModel, CreateActivityInput};
use crate::handlers::notes::get_user_tenant_id;

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/contacts",       get(get_contacts).post(create_contact))
        .route("/api/contacts/{id}",  get(get_contact).put(update_contact).delete(delete_contact))
        .route("/api/contacts/{id}/notes",      get(get_contact_notes).post(create_contact_note))
        .route("/api/contacts/{id}/activities", get(get_contact_activities).post(create_contact_activity))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct ContactListParams {
    pub search:   Option<String>,
    /// Filter to contacts that belong to a given account
    pub account_id: Option<Uuid>,
    /// "primary" | "all" — when "primary" only returns is_primary = true contacts
    pub role:     Option<String>,
    pub page:     Option<u64>,
    pub per_page: Option<u64>,
}
impl ContactListParams {
    fn offset(&self) -> u64 { (self.page.unwrap_or(1).max(1) - 1) * self.limit() }
    fn limit(&self)  -> u64 { self.per_page.unwrap_or(50).min(200).max(1) }
}

#[derive(Deserialize, Clone)]
pub struct CreateContactDto {
    pub first_name:  Option<String>,
    pub last_name:   Option<String>,
    /// Convenience field — set full_name directly if known (import path)
    pub full_name:   Option<String>,
    pub title:       Option<String>,
    pub department:  Option<String>,
    pub email:       Option<String>,
    pub phone:       Option<String>,
    pub whatsapp:    Option<String>,
    pub telegram:    Option<String>,
    pub linkedin_url: Option<String>,
    /// Must match an atlas_accounts.id — required for all new contacts
    pub account_id:  Option<Uuid>,
}

#[derive(Deserialize, Clone)]
pub struct UpdateContactDto {
    pub first_name:   Option<String>,
    pub last_name:    Option<String>,
    pub full_name:    Option<String>,
    pub title:        Option<String>,
    pub department:   Option<String>,
    pub email:        Option<String>,
    pub phone:        Option<String>,
    pub whatsapp:     Option<String>,
    pub telegram:     Option<String>,
    pub linkedin_url: Option<String>,
    pub twitter:      Option<String>,
    pub instagram:    Option<String>,
    pub is_primary:   Option<bool>,
}

// ── Response ──────────────────────────────────────────────────────────────────

/// Rich contact record from `atlas_contacts`.
///
/// `email_verified` and `phone_verified` are DB-managed booleans set by the
/// import pipeline (MillionVerifier output); they are read-only from the API.
#[derive(Serialize, Clone)]
pub struct ContactResponse {
    pub id:            String,
    pub account_id:    String,

    // Name
    pub first_name:    Option<String>,
    pub last_name:     Option<String>,
    /// Display-safe full name — prefer over concatenating first + last
    pub full_name:     Option<String>,
    pub preferred_name: Option<String>,

    // Professional context
    pub title:         Option<String>,
    pub department:    Option<String>,
    pub is_primary:    bool,

    // Contact channels
    pub email:         Option<String>,
    pub email_verified: bool,
    pub phone:         Option<String>,
    pub phone_verified: bool,
    pub whatsapp:      Option<String>,
    pub telegram:      Option<String>,
    pub linkedin_url:  Option<String>,
    pub twitter:       Option<String>,
    pub instagram:     Option<String>,
    pub avatar_url:    Option<String>,

    // Import
    pub data_source:   Option<String>,

    // Timestamps
    pub created_at:    String,
    pub updated_at:    String,
}

impl From<atlas_contact::Model> for ContactResponse {
    fn from(m: atlas_contact::Model) -> Self {
        // Build a display name: prefer full_name, fall back to first + last
        let full_name = m.full_name.clone().or_else(|| {
            match (&m.first_name, &m.last_name) {
                (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                (Some(f), None)    => Some(f.clone()),
                (None, Some(l))    => Some(l.clone()),
                _                  => None,
            }
        });

        Self {
            id:             m.id.to_string(),
            account_id:     m.account_id.to_string(),
            first_name:     m.first_name,
            last_name:      m.last_name,
            full_name,
            preferred_name: m.preferred_name,
            title:          m.title,
            department:     m.department,
            is_primary:     m.is_primary,
            email:          m.email,
            email_verified: m.email_verified,
            phone:          m.phone,
            phone_verified: m.phone_verified,
            whatsapp:       m.whatsapp,
            telegram:       m.telegram,
            linkedin_url:   m.linkedin_url,
            twitter:        m.twitter,
            instagram:      m.instagram,
            avatar_url:     m.avatar_url,
            data_source:    m.data_source,
            created_at:     m.created_at.to_rfc3339(),
            updated_at:     m.updated_at.to_rfc3339(),
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_contacts(
    Extension(db): Extension<DatabaseConnection>,
    Query(params): Query<ContactListParams>,
) -> impl IntoResponse {
    tracing::info!(
        "Fetching atlas_contacts (search={:?} account={:?} page={:?})",
        params.search, params.account_id, params.page
    );

    let mut query = atlas_contact::Entity::find()
        .filter(atlas_contact::Column::IsDuplicate.eq(false))
        .order_by(atlas_contact::Column::FullName, Order::Asc);

    if let Some(account_id) = params.account_id {
        query = query.filter(atlas_contact::Column::AccountId.eq(account_id));
    }

    if let Some(ref role) = params.role {
        if role == "primary" {
            query = query.filter(atlas_contact::Column::IsPrimary.eq(true));
        }
    }

    let all: Vec<atlas_contact::Model> = match query.all(&db).await {
        Ok(v)  => v,
        Err(e) => {
            tracing::error!("Error fetching atlas_contacts: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(Vec::<ContactResponse>::new()));
        }
    };

    // In-memory search across name / email / phone / whatsapp
    let filtered: Vec<atlas_contact::Model> = if let Some(ref term) = params.search {
        if term.is_empty() {
            all
        } else {
            let t = term.to_lowercase();
            all.into_iter().filter(|c| {
                c.full_name.as_deref().map(|n| n.to_lowercase().contains(&t)).unwrap_or(false)
                    || c.first_name.as_deref().map(|n| n.to_lowercase().contains(&t)).unwrap_or(false)
                    || c.last_name.as_deref().map(|n| n.to_lowercase().contains(&t)).unwrap_or(false)
                    || c.email.as_deref().map(|e| e.to_lowercase().contains(&t)).unwrap_or(false)
                    || c.phone.as_deref().map(|p| p.contains(&t)).unwrap_or(false)
                    || c.whatsapp.as_deref().map(|w| w.contains(&t)).unwrap_or(false)
                    || c.title.as_deref().map(|ti| ti.to_lowercase().contains(&t)).unwrap_or(false)
            }).collect()
        }
    } else {
        all
    };

    let offset = params.offset() as usize;
    let limit  = params.limit()  as usize;
    let page: Vec<ContactResponse> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(ContactResponse::from)
        .collect();

    (StatusCode::OK, JsonResponse(page))
}

pub async fn get_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Fetching atlas_contact: {}", id);

    match atlas_contact::Entity::find_by_id(id).one(&db).await {
        Ok(Some(c)) => (StatusCode::OK, JsonResponse(Some(ContactResponse::from(c)))),
        Ok(None)    => (StatusCode::NOT_FOUND, JsonResponse(None)),
        Err(err) => {
            tracing::error!("Error fetching atlas_contact: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None))
        }
    }
}

pub async fn create_contact(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<CreateContactDto>,
) -> impl IntoResponse {
    // Derive full_name from parts if not explicitly provided
    let full_name = payload.full_name.clone().or_else(|| {
        match (&payload.first_name, &payload.last_name) {
            (Some(f), Some(l)) => Some(format!("{} {}", f.trim(), l.trim())),
            (Some(f), None)    => Some(f.trim().to_string()),
            (None, Some(l))    => Some(l.trim().to_string()),
            _                  => None,
        }
    });

    let new_contact = atlas_contact::ActiveModel {
        id:           Set(Uuid::new_v4()),
        tenant_id:    Set(Uuid::nil()),
        account_id:   Set(payload.account_id.unwrap_or(Uuid::nil())),
        first_name:   Set(payload.first_name),
        last_name:    Set(payload.last_name),
        full_name:    Set(full_name),
        title:        Set(payload.title),
        department:   Set(payload.department),
        email:        Set(payload.email),
        phone:        Set(payload.phone),
        whatsapp:     Set(payload.whatsapp),
        telegram:     Set(payload.telegram),
        linkedin_url: Set(payload.linkedin_url),
        is_primary:   Set(false),
        email_verified: Set(false),
        phone_verified: Set(false),
        is_duplicate: Set(false),
        created_at:   Set(Utc::now()),
        updated_at:   Set(Utc::now()),
        ..Default::default()
    };

    match new_contact.insert(&db).await {
        Ok(c)    => (StatusCode::CREATED, JsonResponse(Some(ContactResponse::from(c)))),
        Err(err) => {
            tracing::error!("Error creating atlas_contact: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None::<ContactResponse>))
        }
    }
}

pub async fn update_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateContactDto>,
) -> impl IntoResponse {
    let existing = match atlas_contact::Entity::find_by_id(id).one(&db).await {
        Ok(Some(c)) => c,
        Ok(None)    => return (StatusCode::NOT_FOUND, JsonResponse(None::<ContactResponse>)),
        Err(err) => {
            tracing::error!("Error fetching atlas_contact for update: {:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None));
        }
    };

    let mut active: atlas_contact::ActiveModel = existing.into();

    if let Some(v) = payload.first_name   { active.first_name   = Set(Some(v)); }
    if let Some(v) = payload.last_name    { active.last_name    = Set(Some(v)); }
    if let Some(v) = payload.full_name    { active.full_name    = Set(Some(v)); }
    if let Some(v) = payload.title        { active.title        = Set(Some(v)); }
    if let Some(v) = payload.department   { active.department   = Set(Some(v)); }
    if let Some(v) = payload.email        { active.email        = Set(Some(v)); }
    if let Some(v) = payload.phone        { active.phone        = Set(Some(v)); }
    if let Some(v) = payload.whatsapp     { active.whatsapp     = Set(Some(v)); }
    if let Some(v) = payload.telegram     { active.telegram     = Set(Some(v)); }
    if let Some(v) = payload.linkedin_url { active.linkedin_url = Set(Some(v)); }
    if let Some(v) = payload.twitter      { active.twitter      = Set(Some(v)); }
    if let Some(v) = payload.instagram    { active.instagram    = Set(Some(v)); }
    if let Some(v) = payload.is_primary   { active.is_primary   = Set(v); }
    active.updated_at = Set(Utc::now());

    match active.update(&db).await {
        Ok(c)    => (StatusCode::OK, JsonResponse(Some(ContactResponse::from(c)))),
        Err(err) => {
            tracing::error!("Error updating atlas_contact: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None))
        }
    }
}

pub async fn delete_contact(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match atlas_contact::Entity::delete_by_id(id).exec(&db).await {
        Ok(_)    => StatusCode::NO_CONTENT,
        Err(err) => {
            tracing::error!("Error deleting atlas_contact: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ── Notes / Activities — delegate to legacy entities ──────────────────────────

pub async fn create_contact_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> impl IntoResponse {
    let tenant_id = match get_user_tenant_id(&db, current_user.id).await {
        Ok(t)  => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None::<NoteModel>)),
    };

    let new_note = note::ActiveModel {
        id:          Set(Uuid::new_v4()),
        content:     Set(input.content),
        created_by:  Set(current_user.id),
        entity_type: Set("Contact".to_string()),
        entity_id:   Set(id),
        tenant_id:   Set(Some(tenant_id)),
        is_private:  Set(false),
        created_at:  Set(Utc::now()),
        updated_at:  Set(Utc::now()),
    };

    match new_note.insert(&db).await {
        Ok(n)    => (StatusCode::CREATED, JsonResponse(Some(NoteModel::from(n)))),
        Err(err) => {
            tracing::error!("Error creating contact note: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None))
        }
    }
}

pub async fn get_contact_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let notes = note::Entity::find()
        .filter(note::Column::EntityType.eq("Contact"))
        .filter(note::Column::EntityId.eq(id))
        .all(&db)
        .await
        .unwrap_or_default();

    let models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    (StatusCode::OK, JsonResponse(models))
}

pub async fn create_contact_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateActivityInput>,
) -> impl IntoResponse {
    let new_activity = activity::ActiveModel {
        id:            Set(Uuid::new_v4()),
        contact_id:    Set(Some(id)),
        activity_type: Set(input.activity_type),
        title:         Set(input.title),
        description:   Set(input.description),
        status:        Set(input.status),
        due_date:      Set(input.due_date),
        completed_at:  Set(None),
        created_by:    Set(current_user.id),
        assigned_to:   Set(input.assigned_to),
        created_at:    Set(Utc::now()),
        updated_at:    Set(Utc::now()),
        ..Default::default()
    };

    match new_activity.insert(&db).await {
        Ok(a)    => (StatusCode::CREATED, JsonResponse(Some(ActivityModel::from(a)))),
        Err(err) => {
            tracing::error!("Error creating contact activity: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None::<ActivityModel>))
        }
    }
}

pub async fn get_contact_activities(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let activities = activity::Entity::find()
        .filter(activity::Column::ContactId.eq(Some(id)))
        .all(&db)
        .await
        .unwrap_or_default();

    let models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    (StatusCode::OK, JsonResponse(models))
}
