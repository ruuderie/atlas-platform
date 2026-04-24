use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResumeProfile {
    pub id: i32,
    pub name: String,
    pub full_name: String,
    pub objective: Option<String>,
    pub is_public: bool,
    pub target_role: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_location: Option<String>,
    pub contact_link: Option<String>,
    pub category_visibility: serde_json::Value,
    pub category_order: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "entry_category_enum", rename_all = "lowercase")
)]
pub enum ResumeCategory {
    Work,
    Education,
    Certification,
    Skill,
    Project,
    Language,
    Volunteer,
    Extracurricular,
    Hobby,
}

impl std::fmt::Display for ResumeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Work => "work",
            Self::Education => "education",
            Self::Certification => "certification",
            Self::Skill => "skill",
            Self::Project => "project",
            Self::Language => "language",
            Self::Volunteer => "volunteer",
            Self::Extracurricular => "extracurricular",
            Self::Hobby => "hobby",
        };
        write!(f, "{}", text)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BaseResumeEntry {
    pub id: i32,
    pub category: ResumeCategory,
    pub title: String,
    pub subtitle: Option<String>,
    pub date_range: Option<String>,
    pub bullets: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResumeEntry {
    pub id: i32,
    pub profile_id: i32,
    pub category: ResumeCategory,
    pub title: String,
    pub subtitle: Option<String>,
    pub date_range: Option<String>,
    pub bullets: Vec<String>,
    pub display_order: i32,
    pub is_visible: bool,
    pub slug: Option<String>,
    pub published_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub overrides: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileEntryMapping {
    pub entry_id: i32,
    pub overrides: Option<serde_json::Value>,
}

#[server(GetResumeProfiles, "/api")]
pub async fn get_entry_collections() -> Result<Vec<ResumeProfile>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows = sqlx::query("SELECT id, name, full_name, objective, is_public, target_role, contact_email, contact_phone, contact_location, contact_link, category_visibility, category_order FROM entry_collections WHERE tenant_id = $1 ORDER BY id ASC")
        .bind(tenant_id)
        .fetch_all(&state.pool)
        .await?;

    let profiles = rows
        .into_iter()
        .map(|row| ResumeProfile {
            id: row.get("id"),
            name: row.get("name"),
            full_name: row.get("full_name"),
            objective: row.get("objective"),
            is_public: row.try_get("is_public").unwrap_or(false),
            target_role: row.get("target_role"),
            contact_email: row.get("contact_email"),
            contact_phone: row.get("contact_phone"),
            contact_location: row.get("contact_location"),
            contact_link: row.get("contact_link"),
            category_visibility: row.get("category_visibility"),
            category_order: row.get("category_order"),
        })
        .collect();

    Ok(profiles)
}

#[server(GetResumeEntries, "/api")]
pub async fn get_tenant_entries(
    profile_id: Option<i32>,
) -> Result<Vec<ResumeEntry>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let target_id = match profile_id {
        Some(id) => id,
        None => {
            let row = sqlx::query(
                "SELECT id FROM entry_collections WHERE is_public = true AND tenant_id = $1 ORDER BY id ASC LIMIT 1",
            )
            .bind(tenant_id)
            .fetch_optional(&state.pool)
            .await?;
            if let Some(r) = row {
                r.get("id")
            } else {
                return Ok(vec![]);
            }
        }
    };

    let rows = sqlx::query("SELECT e.id, pe.profile_id, e.category, e.title, e.subtitle, e.date_range, e.bullets, pe.display_order, pe.is_visible, e.slug, CAST(e.published_at AS TEXT) as published_at, e.metadata, pe.overrides FROM tenant_entries e JOIN collection_entries pe ON e.id = pe.entry_id WHERE pe.profile_id = $1 AND e.tenant_id = $2 ORDER BY pe.display_order ASC")
        .bind(target_id)
        .bind(tenant_id)
        .fetch_all(&state.pool)
        .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let bullets_val: serde_json::Value = row.get("bullets");
            let bullets: Vec<String> = serde_json::from_value(bullets_val).unwrap_or_default();
            ResumeEntry {
                id: row.get("id"),
                profile_id: row.get("profile_id"),
                category: row.get("category"),
                title: row.get("title"),
                subtitle: row.get("subtitle"),
                date_range: row.get("date_range"),
                bullets,
                display_order: row.get("display_order"),
                is_visible: row.get("is_visible"),
                slug: row.try_get("slug").unwrap_or(None),
                published_at: row.try_get("published_at").unwrap_or(None),
                metadata: row.try_get("metadata").unwrap_or(None),
                overrides: row.try_get("overrides").unwrap_or(None),
            }
        })
        .collect();

    Ok(items)
}

#[server(GetAllBaseEntries, "/api")]
pub async fn get_all_base_entries() -> Result<Vec<BaseResumeEntry>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows = sqlx::query("SELECT id, category, title, subtitle, date_range, bullets, metadata FROM tenant_entries ORDER BY id DESC")
        .fetch_all(&state.pool)
        .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let bullets_val: serde_json::Value = row.get("bullets");
            let bullets: Vec<String> = serde_json::from_value(bullets_val).unwrap_or_default();
            BaseResumeEntry {
                id: row.get("id"),
                category: row.get("category"),
                title: row.get("title"),
                subtitle: row.get("subtitle"),
                date_range: row.get("date_range"),
                bullets,
                metadata: row.try_get("metadata").unwrap_or(None),
            }
        })
        .collect();

    Ok(items)
}

#[server(GetEntryProfileMappings, "/api")]
pub async fn get_entry_profile_mappings(entry_id: i32) -> Result<Vec<i32>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows = sqlx::query("SELECT profile_id FROM collection_entries WHERE entry_id = $1")
        .bind(entry_id)
        .fetch_all(&state.pool)
        .await?;

    let ids: Vec<i32> = rows.into_iter().map(|row| row.get("profile_id")).collect();
    Ok(ids)
}

#[server(GetProfileEntryMappings, "/api")]
pub async fn get_profile_entry_mappings(
    profile_id: i32,
) -> Result<Vec<ProfileEntryMapping>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows =
        sqlx::query("SELECT entry_id, overrides FROM collection_entries WHERE profile_id = $1")
            .bind(profile_id)
            .fetch_all(&state.pool)
            .await?;

    let mappings: Vec<ProfileEntryMapping> = rows
        .into_iter()
        .map(|row| ProfileEntryMapping {
            entry_id: row.get("entry_id"),
            overrides: row.try_get("overrides").unwrap_or(None),
        })
        .collect();
    Ok(mappings)
}

#[server(AddResumeProfile, "/api")]
pub async fn add_resume_profile(
    name: String,
    full_name: String,
    objective: Option<String>,
    is_public: bool,
    target_role: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    contact_location: Option<String>,
    contact_link: Option<String>,
    category_visibility: serde_json::Value,
    category_order: serde_json::Value,
    active_entries: Vec<ProfileEntryMapping>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let row = sqlx::query("INSERT INTO entry_collections (name, full_name, objective, is_public, target_role, contact_email, contact_phone, contact_location, contact_link, category_visibility, category_order) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id")
        .bind(name).bind(full_name).bind(objective).bind(is_public).bind(target_role)
        .bind(contact_email).bind(contact_phone).bind(contact_location).bind(contact_link).bind(category_visibility).bind(category_order)
        .fetch_one(&state.pool).await?;

    let pid: i32 = row.get("id");

    for mapping in active_entries {
        sqlx::query("INSERT INTO collection_entries (profile_id, entry_id, display_order, is_visible, overrides) VALUES ($1, $2, 0, true, $3)")
            .bind(pid).bind(mapping.entry_id).bind(mapping.overrides).execute(&state.pool).await?;
    }

    Ok(())
}

#[server(UpdateResumeProfile, "/api")]
pub async fn update_resume_profile(
    id: i32,
    name: String,
    full_name: String,
    objective: Option<String>,
    is_public: bool,
    target_role: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    contact_location: Option<String>,
    contact_link: Option<String>,
    category_visibility: serde_json::Value,
    category_order: serde_json::Value,
    active_entries: Vec<ProfileEntryMapping>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query("UPDATE entry_collections SET name = $1, full_name = $2, objective = $3, is_public = $4, target_role = $5, contact_email = $6, contact_phone = $7, contact_location = $8, contact_link = $9, category_visibility = $10, category_order = $11 WHERE id = $12")
        .bind(name).bind(full_name).bind(objective).bind(is_public).bind(target_role)
        .bind(contact_email).bind(contact_phone).bind(contact_location).bind(contact_link).bind(category_visibility).bind(category_order).bind(id)
        .execute(&state.pool).await?;

    let current_mappings =
        sqlx::query("SELECT entry_id FROM collection_entries WHERE profile_id = $1")
            .bind(id)
            .fetch_all(&state.pool)
            .await?;
    let current_eids: Vec<i32> = current_mappings
        .into_iter()
        .map(|r| r.get("entry_id"))
        .collect();

    let active_eids: Vec<i32> = active_entries.iter().map(|m| m.entry_id).collect();

    for eid in &current_eids {
        if !active_eids.contains(eid) {
            sqlx::query(
                "DELETE FROM collection_entries WHERE profile_id = $1 AND entry_id = $2",
            )
            .bind(id)
            .bind(eid)
            .execute(&state.pool)
            .await?;
        }
    }

    for mapping in active_entries {
        if current_eids.contains(&mapping.entry_id) {
            sqlx::query("UPDATE collection_entries SET overrides = $1 WHERE profile_id = $2 AND entry_id = $3")
                .bind(mapping.overrides).bind(id).bind(mapping.entry_id).execute(&state.pool).await?;
        } else {
            sqlx::query("INSERT INTO collection_entries (profile_id, entry_id, display_order, is_visible, overrides) VALUES ($1, $2, 0, true, $3) ON CONFLICT DO NOTHING")
                .bind(id).bind(mapping.entry_id).bind(mapping.overrides).execute(&state.pool).await?;
        }
    }

    Ok(())
}

#[server(DeleteResumeProfile, "/api")]
pub async fn delete_resume_profile(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    sqlx::query("DELETE FROM entry_collections WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(AddBaseEntry, "/api")]
pub async fn add_base_entry(
    category: ResumeCategory,
    title: String,
    subtitle: Option<String>,
    date_range: Option<String>,
    bullets: Vec<String>,
    metadata: Option<serde_json::Value>,
    active_profiles: Vec<i32>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let bullets_json = serde_json::to_value(&bullets).unwrap_or(serde_json::json!([]));

    let row = sqlx::query("INSERT INTO tenant_entries (category, title, subtitle, date_range, bullets, metadata) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id")
        .bind(category).bind(title).bind(subtitle).bind(date_range).bind(bullets_json).bind(metadata)
        .fetch_one(&state.pool).await?;

    let entry_id: i32 = row.get("id");

    for pid in active_profiles {
        sqlx::query("INSERT INTO collection_entries (profile_id, entry_id, display_order, is_visible) VALUES ($1, $2, 0, true)")
            .bind(pid).bind(entry_id).execute(&state.pool).await?;
    }

    Ok(())
}

#[server(UpdateBaseEntry, "/api")]
pub async fn update_base_entry(
    id: i32,
    category: ResumeCategory,
    title: String,
    subtitle: Option<String>,
    date_range: Option<String>,
    bullets: Vec<String>,
    metadata: Option<serde_json::Value>,
    active_profiles: Vec<i32>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let bullets_json = serde_json::to_value(&bullets).unwrap_or(serde_json::json!([]));

    sqlx::query("UPDATE tenant_entries SET category = $1, title = $2, subtitle = $3, date_range = $4, bullets = $5, metadata = $6 WHERE id = $7")
        .bind(category).bind(title).bind(subtitle).bind(date_range).bind(bullets_json).bind(metadata).bind(id)
        .execute(&state.pool).await?;

    let current_mappings =
        sqlx::query("SELECT profile_id FROM collection_entries WHERE entry_id = $1")
            .bind(id)
            .fetch_all(&state.pool)
            .await?;
    let current_pids: Vec<i32> = current_mappings
        .into_iter()
        .map(|r| r.get("profile_id"))
        .collect();

    for pid in &current_pids {
        if !active_profiles.contains(pid) {
            sqlx::query(
                "DELETE FROM collection_entries WHERE entry_id = $1 AND profile_id = $2",
            )
            .bind(id)
            .bind(pid)
            .execute(&state.pool)
            .await?;
        }
    }

    for pid in active_profiles {
        if !current_pids.contains(&pid) {
            sqlx::query("INSERT INTO collection_entries (profile_id, entry_id, display_order, is_visible) VALUES ($1, $2, 0, true) ON CONFLICT DO NOTHING")
                .bind(pid).bind(id).execute(&state.pool).await?;
        }
    }

    Ok(())
}

#[server(DeleteBaseEntry, "/api")]
pub async fn delete_base_entry(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    sqlx::query("DELETE FROM tenant_entries WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(SetProfileEntry, "/api")]
pub async fn set_profile_entry(
    profile_id: i32,
    entry_id: i32,
    active: bool,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    if active {
        sqlx::query("INSERT INTO collection_entries (profile_id, entry_id, display_order, is_visible) VALUES ($1, $2, 0, true) ON CONFLICT (profile_id, entry_id) DO NOTHING")
            .bind(profile_id).bind(entry_id).execute(&state.pool).await?;
    } else {
        sqlx::query("DELETE FROM collection_entries WHERE profile_id = $1 AND entry_id = $2")
            .bind(profile_id)
            .bind(entry_id)
            .execute(&state.pool)
            .await?;
    }
    Ok(())
}

#[server(UpdateProfileEntryVisibility, "/api")]
pub async fn update_profile_entry_visibility(
    profile_id: i32,
    entry_id: i32,
    display_order: i32,
    is_visible: bool,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query("UPDATE collection_entries SET display_order = $1, is_visible = $2 WHERE profile_id = $3 AND entry_id = $4")
        .bind(display_order).bind(is_visible).bind(profile_id).bind(entry_id)
        .execute(&state.pool).await?;

    Ok(())
}

pub fn latex_escape(s: &str) -> String {
    s.replace("&", "\\&")
        .replace("%", "\\%")
        .replace("$", "\\$")
        .replace("#", "\\#")
        .replace("_", "\\_")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("~", "\\textasciitilde{}")
        .replace("^", "\\textasciicircum{}")
}

pub fn generate_latex_string(profile: &ResumeProfile, entries: &[ResumeEntry]) -> String {
    let target_role_str = profile.target_role.clone().unwrap_or_default();
    let objective_str = profile.objective.clone().unwrap_or_default();

    let mut contact_parts = Vec::new();
    if let Some(ref email) = profile.contact_email {
        if !email.is_empty() {
            contact_parts.push(latex_escape(email));
        }
    }
    if let Some(ref phone) = profile.contact_phone {
        if !phone.is_empty() {
            contact_parts.push(latex_escape(phone));
        }
    }
    if let Some(ref loc) = profile.contact_location {
        if !loc.is_empty() {
            contact_parts.push(latex_escape(loc));
        }
    }
    if let Some(ref link) = profile.contact_link {
        if !link.is_empty() {
            contact_parts.push(latex_escape(link));
        }
    }
    let contact_str = contact_parts.join(" \\textbar{} ");

    let mut tex_content = format!(
        r#"
\documentclass[11pt,letterpaper]{{article}}
\usepackage[utf8]{{inputenc}}
\usepackage[left=0.5in,top=0.5in,right=0.5in,bottom=0.5in]{{geometry}}
\usepackage{{enumitem}}
\usepackage{{titlesec}}
\usepackage{{charter}}

\titleformat{{\section}}
  {{\normalfont\Large\bfseries}}
  {{}}{{0em}}
  {{}}[\titlerule]
\titlespacing*{{\section}}{{0pt}}{{2ex}}{{1ex}}

\begin{{document}}
\pagestyle{{empty}}
\begin{{center}}
    {{\Huge \textbf{{{}}}}} \\
    \vspace{{0.1in}}
    {{\Large \textsc{{{}}}}} \\
    \vspace{{0.05in}}
    {{\normalsize \texttt{{{}}}}}
\end{{center}}
\vspace{{0.1in}}
"#,
        latex_escape(&profile.full_name),
        latex_escape(&target_role_str),
        contact_str
    );

    if !objective_str.is_empty() {
        tex_content.push_str(&format!(
            "\\section*{{Executive Summary}}\n\\noindent {}\n\n",
            latex_escape(&objective_str)
        ));
    }

    let is_cat_visible = |cat: &ResumeCategory| -> bool {
        profile
            .category_visibility
            .get(cat.to_string())
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    };

    let mut ordered_cats = Vec::new();
    if let Some(arr) = profile.category_order.as_array() {
        for v in arr {
            if let Some(s) = v.as_str() {
                ordered_cats.push(s.to_string());
            }
        }
    }
    if ordered_cats.is_empty() {
        ordered_cats = vec![
            "work".to_string(),
            "education".to_string(),
            "certification".to_string(),
            "project".to_string(),
            "skill".to_string(),
            "volunteer".to_string(),
            "extracurricular".to_string(),
            "language".to_string(),
            "hobby".to_string(),
        ];
    }

    let mut categories_to_render = Vec::new();
    for c in ordered_cats {
        match c.as_str() {
            "work" => categories_to_render.push((ResumeCategory::Work, "Experience")),
            "education" => categories_to_render.push((ResumeCategory::Education, "Education")),
            "certification" => {
                categories_to_render.push((ResumeCategory::Certification, "Certifications"))
            }
            "project" => categories_to_render.push((ResumeCategory::Project, "Projects")),
            "skill" => categories_to_render.push((ResumeCategory::Skill, "Skills")),
            "language" => categories_to_render.push((ResumeCategory::Language, "Languages")),
            "volunteer" => categories_to_render.push((ResumeCategory::Volunteer, "Volunteering")),
            "extracurricular" => {
                categories_to_render.push((ResumeCategory::Extracurricular, "Extracurriculars"))
            }
            "hobby" => categories_to_render.push((ResumeCategory::Hobby, "Hobbies")),
            _ => {}
        }
    }

    for (cat_enum, section_title) in categories_to_render {
        if !is_cat_visible(&cat_enum) {
            continue;
        }

        let cat_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.category == cat_enum && e.is_visible)
            .collect();

        if cat_entries.is_empty() {
            continue;
        }

        tex_content.push_str(&format!("\\section*{{{}}}\n", section_title));

        for entry in cat_entries {
            let mut resolved_title = entry.title.clone();
            let mut resolved_subtitle = entry.subtitle.clone().unwrap_or_default();
            let mut resolved_date = entry.date_range.clone().unwrap_or_default();
            let mut resolved_bullets = entry.bullets.clone();

            if let Some(overrides) = &entry.overrides {
                if let Some(val) = overrides.get("title").and_then(|v| v.as_str()) {
                    if !val.trim().is_empty() {
                        resolved_title = val.to_string();
                    }
                }
                if let Some(val) = overrides.get("subtitle").and_then(|v| v.as_str()) {
                    if !val.trim().is_empty() {
                        resolved_subtitle = val.to_string();
                    }
                }
                if let Some(val) = overrides.get("date_range").and_then(|v| v.as_str()) {
                    if !val.trim().is_empty() {
                        resolved_date = val.to_string();
                    }
                }
                if let Some(arr) = overrides.get("bullets").and_then(|v| v.as_array()) {
                    let ov_bullets: Vec<String> = arr
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();
                    if !ov_bullets.is_empty() {
                        resolved_bullets = ov_bullets;
                    }
                }
            }

            let title = latex_escape(&resolved_title);
            let date = latex_escape(&resolved_date);
            let subtitle = latex_escape(&resolved_subtitle);

            tex_content.push_str(&format!(
                "\\noindent \\textbf{{{}}} \\hfill {} \\\\\n",
                title, date
            ));

            if !subtitle.is_empty() {
                tex_content.push_str(&format!("\\textit{{{}}} \\vspace{{0.05in}}\n", subtitle));
            } else {
                tex_content.push_str("\\vspace{0.05in}\n");
            }

            if !resolved_bullets.is_empty() {
                tex_content.push_str("\\begin{itemize}[leftmargin=*,noitemsep,topsep=0pt,parsep=0pt,partopsep=0pt]\n");
                for bullet in &resolved_bullets {
                    tex_content.push_str(&format!("\\item {}\n", latex_escape(bullet)));
                }
                tex_content.push_str("\\end{itemize}\n");
            }
            tex_content.push_str("\\vspace{0.15in}\n\n");
        }
    }

    tex_content.push_str("\\end{document}\n");
    tex_content
}

#[server(DownloadResume, "/api")]
pub async fn download_resume(profile_id: i32) -> Result<Vec<u8>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    if profile_id <= 0 {
        return Err(ServerFnError::ServerError("Invalid Profile ID".into()));
    }

    let profile_row = sqlx::query("SELECT id, name, full_name, objective, is_public, target_role, contact_email, contact_phone, contact_location, contact_link, category_visibility, category_order FROM entry_collections WHERE id = $1")
        .bind(profile_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| -> ServerFnError { ServerFnError::ServerError("Profile not found".into()) })?;

    let profile = ResumeProfile {
        id: profile_row.get("id"),
        name: profile_row.get("name"),
        full_name: profile_row.get("full_name"),
        objective: profile_row.get("objective"),
        is_public: profile_row.try_get("is_public").unwrap_or(false),
        target_role: profile_row.get("target_role"),
        contact_email: profile_row.get("contact_email"),
        contact_phone: profile_row.get("contact_phone"),
        contact_location: profile_row.get("contact_location"),
        contact_link: profile_row.get("contact_link"),
        category_visibility: profile_row.get("category_visibility"),
        category_order: profile_row.get("category_order"),
    };

    let entries_rows = sqlx::query("SELECT e.id, pe.profile_id, e.category, e.title, e.subtitle, e.date_range, e.bullets, pe.display_order, pe.is_visible, e.slug, CAST(e.published_at AS TEXT) as published_at, e.metadata, pe.overrides FROM tenant_entries e JOIN collection_entries pe ON e.id = pe.entry_id WHERE pe.profile_id = $1 ORDER BY pe.display_order ASC")
        .bind(profile_id).fetch_all(&state.pool).await?;

    let entries: Vec<ResumeEntry> = entries_rows
        .into_iter()
        .map(|row| {
            let bullets_val: serde_json::Value = row.get("bullets");
            let bullets: Vec<String> = serde_json::from_value(bullets_val).unwrap_or_default();
            ResumeEntry {
                id: row.get("id"),
                profile_id: row.get("profile_id"),
                category: row.get("category"),
                title: row.get("title"),
                subtitle: row.get("subtitle"),
                date_range: row.get("date_range"),
                bullets,
                display_order: row.get("display_order"),
                is_visible: row.get("is_visible"),
                slug: row.try_get("slug").unwrap_or(None),
                published_at: row.try_get("published_at").unwrap_or(None),
                metadata: row.try_get("metadata").unwrap_or(None),
                overrides: row.try_get("overrides").unwrap_or(None),
            }
        })
        .collect();

    let tex_content = generate_latex_string(&profile, &entries);

    let tex_path = format!("/tmp/resume_output_{}.tex", uuid::Uuid::new_v4());
    let pdf_path = tex_path.replace(".tex", ".pdf");

    if let Err(e) = std::fs::write(&tex_path, &tex_content) {
        return Err(ServerFnError::ServerError(
            format!("Failed to write TEX source: {}", e).into(),
        ));
    }

    let mut command = std::process::Command::new("tectonic");
    command.current_dir("/tmp");
    command.arg("-X").arg("compile").arg(&tex_path);

    let status = command.output().map_err(|e| -> ServerFnError {
        ServerFnError::ServerError(format!("Failed to execute tectonic: {}", e).into())
    })?;

    if !status.status.success() {
        let output2 = std::process::Command::new("pdflatex")
            .current_dir("/tmp")
            .arg("-interaction=nonstopmode")
            .arg(&tex_path)
            .output()
            .map_err(|e| -> ServerFnError {
                ServerFnError::ServerError(
                    format!("Failed to execute pdflatex compiler: {}", e).into(),
                )
            })?;

        if !output2.status.success() {
            return Err(ServerFnError::ServerError(
                format!("Latex Compilation Error").into(),
            ));
        }
    }

    let pdf_bytes = std::fs::read(&pdf_path).map_err(|e| -> ServerFnError {
        ServerFnError::ServerError(format!("Failed to read compiled PDF: {}", e).into())
    })?;

    let _ = std::fs::remove_file(&tex_path);
    let _ = std::fs::remove_file(&pdf_path);

    Ok(pdf_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_escape() {
        let input = "M&A $100% #1 _test_ {foo} ~bar^";
        let expected =
            "M\\&A \\$100\\% \\#1 \\_test\\_ \\{foo\\} \\textasciitilde{}bar\\textasciicircum{}";
        assert_eq!(latex_escape(input), expected);
    }

    #[test]
    fn test_generate_latex_string_formats_correctly() {
        let profile = ResumeProfile {
            id: 1,
            name: "Internal Name".into(),
            full_name: "John Doe".into(),
            objective: Some("Software Dev".into()),
            is_public: true,
            target_role: Some("Engineer".into()),
            contact_email: Some("john@test.com".into()),
            contact_phone: None,
            contact_location: None,
            contact_link: None,
            category_visibility: serde_json::json!({"work": true, "education": true}),
            category_order: serde_json::json!(["work"]),
        };
        let entries = vec![ResumeEntry {
            id: 1,
            profile_id: 1,
            category: ResumeCategory::Work,
            title: "Developer".into(),
            subtitle: Some("Tech Corp".into()),
            date_range: Some("2020-2022".into()),
            bullets: vec!["Did things & stuff".into()],
            display_order: 0,
            is_visible: true,
            metadata: None,
            overrides: None,
        }];

        let tex = generate_latex_string(&profile, &entries);

        assert!(tex.contains("Software Dev"));
        assert!(tex.contains("Tech Corp"));
        assert!(tex.contains("Did things \\& stuff"));
        assert!(tex.contains("\\section*{Experience}"));
    }

    #[test]
    fn test_generate_latex_string_filters_and_masks() {
        let profile = ResumeProfile {
            id: 1,
            name: "Internal Name".into(),
            full_name: "Jane Doe".into(),
            objective: Some("Hidden Test".into()),
            is_public: true,
            target_role: None,
            contact_email: None,
            contact_phone: None,
            contact_location: None,
            contact_link: None,
            category_visibility: serde_json::json!({"work": true, "education": false}),
            category_order: serde_json::json!(["work", "education"]),
        };
        let entries = vec![
            ResumeEntry {
                id: 1,
                profile_id: 1,
                category: ResumeCategory::Work,
                title: "Hidden Job".into(),
                subtitle: None,
                date_range: None,
                bullets: vec![],
                display_order: 0,
                is_visible: false, // Target should hide inherently
                metadata: None,
                overrides: None,
            },
            ResumeEntry {
                id: 2,
                profile_id: 1,
                category: ResumeCategory::Education,
                title: "Hidden Education".into(),
                subtitle: None,
                date_range: None,
                bullets: vec![],
                display_order: 1,
                is_visible: true, // Should hide because master JSON hides Education
                metadata: None,
                overrides: None,
            },
        ];

        let tex = generate_latex_string(&profile, &entries);

        assert!(!tex.contains("Hidden Job"));
        assert!(!tex.contains("Hidden Education"));
    }
}
