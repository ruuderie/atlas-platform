use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResumeProfile {
    pub id: uuid::Uuid,
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
    pub id: uuid::Uuid,
    pub category: ResumeCategory,
    pub title: String,
    pub subtitle: Option<String>,
    pub date_range: Option<String>,
    pub bullets: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResumeEntry {
    pub id: uuid::Uuid,
    pub profile_id: uuid::Uuid,
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
    pub entry_id: uuid::Uuid,
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
    
    let rows = sqlx::query("SELECT id, title, payload FROM app_content WHERE tenant_id = $1 AND collection_type = 'resume_profile' ORDER BY created_at ASC")
        .bind(tenant_id)
        .fetch_all(&state.pool)
        .await?;

    let profiles = rows
        .into_iter()
        .map(|row| {
            let payload: serde_json::Value = row.try_get("payload").unwrap_or(serde_json::json!({}));
            ResumeProfile {
                id: row.get("id"),
                name: payload.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                full_name: payload.get("full_name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                objective: payload.get("objective").and_then(|v| v.as_str()).map(|s| s.to_string()),
                is_public: payload.get("is_public").and_then(|v| v.as_bool()).unwrap_or(false),
                target_role: payload.get("target_role").and_then(|v| v.as_str()).map(|s| s.to_string()),
                contact_email: payload.get("contact_email").and_then(|v| v.as_str()).map(|s| s.to_string()),
                contact_phone: payload.get("contact_phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
                contact_location: payload.get("contact_location").and_then(|v| v.as_str()).map(|s| s.to_string()),
                contact_link: payload.get("contact_link").and_then(|v| v.as_str()).map(|s| s.to_string()),
                category_visibility: payload.get("category_visibility").cloned().unwrap_or(serde_json::json!({})),
                category_order: payload.get("category_order").cloned().unwrap_or(serde_json::json!([])),
            }
        })
        .collect();

    Ok(profiles)
}

#[server(GetResumeEntries, "/api")]
pub async fn get_tenant_entries(
    profile_id: Option<uuid::Uuid>,
) -> Result<Vec<ResumeEntry>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query("SELECT id, title, display_order, payload FROM app_content WHERE tenant_id = $1 AND collection_type = 'resume_entry' ORDER BY display_order ASC, created_at DESC")
        .bind(tenant_id)
        .fetch_all(&state.pool)
        .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let payload: serde_json::Value = row.try_get("payload").unwrap_or(serde_json::json!({}));
            
            let category_str = payload.get("category").and_then(|v| v.as_str()).unwrap_or("work");
            let category = match category_str {
                "work" => ResumeCategory::Work,
                "education" => ResumeCategory::Education,
                "certification" => ResumeCategory::Certification,
                "skill" => ResumeCategory::Skill,
                "project" => ResumeCategory::Project,
                "language" => ResumeCategory::Language,
                "volunteer" => ResumeCategory::Volunteer,
                "extracurricular" => ResumeCategory::Extracurricular,
                "hobby" => ResumeCategory::Hobby,
                _ => ResumeCategory::Work,
            };

            let bullets: Vec<String> = payload.get("bullets")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            ResumeEntry {
                id: row.get("id"),
                profile_id: profile_id.unwrap_or_else(uuid::Uuid::nil),
                category,
                title: row.get("title"),
                subtitle: payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string()),
                date_range: payload.get("date_range").and_then(|v| v.as_str()).map(|s| s.to_string()),
                bullets,
                display_order: row.try_get("display_order").unwrap_or(0),
                is_visible: payload.get("is_visible").and_then(|v| v.as_bool()).unwrap_or(true),
                slug: payload.get("slug").and_then(|v| v.as_str()).map(|s| s.to_string()),
                published_at: None,
                metadata: payload.get("metadata").cloned(),
                overrides: payload.get("overrides").cloned(),
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
    let rows = sqlx::query("SELECT id, title, payload FROM app_content WHERE collection_type = 'resume_entry' ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let payload: serde_json::Value = row.try_get("payload").unwrap_or(serde_json::json!({}));
            
            let category_str = payload.get("category").and_then(|v| v.as_str()).unwrap_or("work");
            let category = match category_str {
                "work" => ResumeCategory::Work,
                "education" => ResumeCategory::Education,
                "certification" => ResumeCategory::Certification,
                "skill" => ResumeCategory::Skill,
                "project" => ResumeCategory::Project,
                "language" => ResumeCategory::Language,
                "volunteer" => ResumeCategory::Volunteer,
                "extracurricular" => ResumeCategory::Extracurricular,
                "hobby" => ResumeCategory::Hobby,
                _ => ResumeCategory::Work,
            };

            let bullets: Vec<String> = payload.get("bullets")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            BaseResumeEntry {
                id: row.get("id"),
                category,
                title: row.get("title"),
                subtitle: payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string()),
                date_range: payload.get("date_range").and_then(|v| v.as_str()).map(|s| s.to_string()),
                bullets,
                metadata: payload.get("metadata").cloned(),
            }
        })
        .collect();

    Ok(items)
}


#[server(GetEntryProfileMappings, "/api")]
pub async fn get_entry_profile_mappings(entry_id: uuid::Uuid) -> Result<Vec<uuid::Uuid>, ServerFnError> {
    Ok(vec![])
}

#[server(GetProfileEntryMappings, "/api")]
pub async fn get_profile_entry_mappings(
    profile_id: uuid::Uuid,
) -> Result<Vec<ProfileEntryMapping>, ServerFnError> {
    Ok(vec![])
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
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let payload = serde_json::json!({
        "name": name,
        "full_name": full_name,
        "objective": objective,
        "is_public": is_public,
        "target_role": target_role,
        "contact_email": contact_email,
        "contact_phone": contact_phone,
        "contact_location": contact_location,
        "contact_link": contact_link,
        "category_visibility": category_visibility,
        "category_order": category_order
    });

    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload) VALUES ($1, 'resume_profile', $2, $3)")
        .bind(tenant_id).bind(&name).bind(&payload)
        .execute(&state.pool).await?;

    Ok(())
}

#[server(UpdateResumeProfile, "/api")]
pub async fn update_resume_profile(
    id: uuid::Uuid,
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
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let payload = serde_json::json!({
        "name": name,
        "full_name": full_name,
        "objective": objective,
        "is_public": is_public,
        "target_role": target_role,
        "contact_email": contact_email,
        "contact_phone": contact_phone,
        "contact_location": contact_location,
        "contact_link": contact_link,
        "category_visibility": category_visibility,
        "category_order": category_order
    });

    sqlx::query("UPDATE app_content SET title = $1, payload = $2 WHERE id = $3 AND collection_type = 'resume_profile'")
        .bind(&name).bind(&payload).bind(id)
        .execute(&state.pool).await?;

    Ok(())
}

#[server(DeleteResumeProfile, "/api")]
pub async fn delete_resume_profile(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1")
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
    active_profiles: Vec<uuid::Uuid>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let payload = serde_json::json!({
        "category": category.to_string(),
        "subtitle": subtitle,
        "date_range": date_range,
        "bullets": bullets,
        "metadata": metadata
    });

    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload) VALUES ($1, 'resume_entry', $2, $3)")
        .bind(tenant_id).bind(&title).bind(&payload)
        .execute(&state.pool).await?;

    Ok(())
}

#[server(UpdateBaseEntry, "/api")]
pub async fn update_base_entry(
    id: uuid::Uuid,
    category: ResumeCategory,
    title: String,
    subtitle: Option<String>,
    date_range: Option<String>,
    bullets: Vec<String>,
    metadata: Option<serde_json::Value>,
    active_profiles: Vec<uuid::Uuid>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let payload = serde_json::json!({
        "category": category.to_string(),
        "subtitle": subtitle,
        "date_range": date_range,
        "bullets": bullets,
        "metadata": metadata
    });

    sqlx::query("UPDATE app_content SET title = $1, payload = $2 WHERE id = $3 AND collection_type = 'resume_entry'")
        .bind(&title).bind(&payload).bind(id)
        .execute(&state.pool).await?;

    Ok(())
}

#[server(DeleteBaseEntry, "/api")]
pub async fn delete_base_entry(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(SetProfileEntry, "/api")]
pub async fn set_profile_entry(
    profile_id: uuid::Uuid,
    entry_id: uuid::Uuid,
    active: bool,
) -> Result<(), ServerFnError> {
    Ok(())
}

#[server(UpdateProfileEntryVisibility, "/api")]
pub async fn update_profile_entry_visibility(
    profile_id: uuid::Uuid,
    entry_id: uuid::Uuid,
    display_order: i32,
    is_visible: bool,
) -> Result<(), ServerFnError> {
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

    // ─────────────────────────────────────────────────────────────────────────
    // Kami design system: build contact line in stone-colored metadata row
    // ─────────────────────────────────────────────────────────────────────────

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
    // Stone-colored metadata separator (textbullet is more legible than textbar in small sizes)
    let contact_str = contact_parts.join(" \\textbullet{} ");

    // ─────────────────────────────────────────────────────────────────────────
    // Kami preamble
    // Color tokens  --brand #1B365D · --near-black #141413 · --olive #504e49
    //               --stone #6b6a64 · --border-soft #e5e3d8 · --parchment #f5f4ed
    //               --tagbg #EEF2F7 (ink-blue @ 0.08 solid equivalent)
    // Packages auto-fetched by tectonic from TeX Live 2023 on first compile.
    // ─────────────────────────────────────────────────────────────────────────
    let mut tex_content = format!(
        r#"
\documentclass[9pt,a4paper]{{article}}
\usepackage[utf8]{{inputenc}}
\usepackage[T1]{{fontenc}}
\usepackage{{charter}}
\usepackage{{xcolor}}
\usepackage[left=13mm,top=11mm,right=13mm,bottom=11mm]{{geometry}}
\usepackage{{enumitem}}
\usepackage{{titlesec}}
\usepackage{{mdframed}}
\usepackage{{parskip}}
\usepackage{{microtype}}

% ── Kami color tokens ────────────────────────────────────────────────────────
\definecolor{{brand}}{{HTML}}{{1B365D}}
\definecolor{{nearblack}}{{HTML}}{{141413}}
\definecolor{{olive}}{{HTML}}{{504e49}}
\definecolor{{stone}}{{HTML}}{{6b6a64}}
\definecolor{{bordersoft}}{{HTML}}{{e5e3d8}}
\definecolor{{parchment}}{{HTML}}{{f5f4ed}}
\definecolor{{tagbg}}{{HTML}}{{EEF2F7}}

\pagecolor{{parchment}}
\color{{nearblack}}

% ── Section title: 2.5pt brand left bar + soft bottom rule ──────────────────
\titleformat{{\section}}
  {{\normalfont\large\bfseries\color{{nearblack}}}}{{}}{{0em}}
  {{\leavevmode\llap{{\textcolor{{brand}}{{\rule[-0.3ex]{{2.5pt}}{{1.1em}}\hspace{{6pt}}}}}}}}
  [{{\vspace{{-4pt}}\textcolor{{bordersoft}}{{\hrule height 0.4pt}}\vspace{{2pt}}}}]
\titlespacing*{{\section}}{{0pt}}{{14pt}}{{6pt}}

% ── En-dash bullet list (Kami editorial style) ───────────────────────────────
\setlist[itemize,1]{{
  label={{\textcolor{{brand}}{{\textendash}}}},
  leftmargin=*,nosep,topsep=2pt,parsep=0pt,itemsep=1.5pt
}}

\begin{{document}}
\pagestyle{{empty}}

% ── Header: name (26pt nearblack bold) · role (olive) · contact (stone small)
\begin{{center}}
  {{\fontsize{{26pt}}{{30pt}}\selectfont\textbf{{\textcolor{{nearblack}}{{{}}}}}}} \\[4pt]
  {{\normalsize\textcolor{{olive}}{{{}}}}} \\[3pt]
  {{\small\textcolor{{stone}}{{{}}}}}
\end{{center}}
\vspace{{8pt}}
"#,
        latex_escape(&profile.full_name),
        latex_escape(&target_role_str),
        contact_str
    );

    // ── Objective: mdframed brand left-bar quote box (Kami quote component) ──
    if !objective_str.is_empty() {
        tex_content.push_str(&format!(
            r#"\begin{{mdframed}}[linewidth=2pt,linecolor=brand,topline=false,bottomline=false,rightline=false,innerleftmargin=10pt,innerrightmargin=0pt,innertopmargin=4pt,innerbottommargin=4pt,backgroundcolor=parchment]
  \textcolor{{olive}}{{\small {}}}
\end{{mdframed}}
\vspace{{6pt}}
"#,
            latex_escape(&objective_str)
        ));
    }

    // ── Category ordering / visibility ────────────────────────────────────────
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

        // ── Skill section: Kami tag component — inline colorbox per skill ──────
        if cat_enum == ResumeCategory::Skill {
            tex_content.push_str("\\noindent\n");
            for (i, entry) in cat_entries.iter().enumerate() {
                if i > 0 {
                    tex_content.push_str("\\;");
                }
                tex_content.push_str(&format!(
                    "\\colorbox{{tagbg}}{{\\small\\textcolor{{brand}}{{\\strut {}}}}}",
                    latex_escape(&entry.title)
                ));
            }
            tex_content.push_str("\n\\vspace{6pt}\n\n");
            continue;
        }

        // ── All other categories: title · date · subtitle · bullet list ────────
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

            // Title (nearblack bold) right-flush date (stone small)
            tex_content.push_str(&format!(
                "\\noindent\\textbf{{\\textcolor{{nearblack}}{{{}}}}} \\hfill \\textcolor{{stone}}{{\\small {}}}\\\\\n",
                title, date
            ));

            // Subtitle in olive, small weight
            if !subtitle.is_empty() {
                tex_content.push_str(&format!(
                    "\\textcolor{{olive}}{{\\small {}}}\\vspace{{3pt}}\n",
                    subtitle
                ));
            } else {
                tex_content.push_str("\\vspace{3pt}\n");
            }

            // Bullets rendered as en-dash list (setlist config above)
            if !resolved_bullets.is_empty() {
                tex_content.push_str("\\begin{itemize}\n");
                for bullet in &resolved_bullets {
                    tex_content.push_str(&format!("\\item {}\n", latex_escape(bullet)));
                }
                tex_content.push_str("\\end{itemize}\n");
            }
            tex_content.push_str("\\vspace{8pt}\n\n");
        }
    }

    tex_content.push_str("\\end{document}\n");
    tex_content
}



#[server(DownloadResume, "/api")]
pub async fn download_resume(profile_id: uuid::Uuid) -> Result<Vec<u8>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    if profile_id.is_nil() {
        return Err(ServerFnError::ServerError("Invalid Profile ID".into()));
    }

    let profile_row = sqlx::query("SELECT id, title, payload, tenant_id FROM app_content WHERE id = $1 AND collection_type = 'resume_profile'")
        .bind(profile_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| -> ServerFnError { ServerFnError::ServerError("Profile not found".into()) })?;

    let payload: serde_json::Value = profile_row.try_get("payload").unwrap_or(serde_json::json!({}));
    let tenant_id: uuid::Uuid = profile_row.get("tenant_id");

    let profile = ResumeProfile {
        id: profile_row.get("id"),
        name: payload.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        full_name: payload.get("full_name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        objective: payload.get("objective").and_then(|v| v.as_str()).map(|s| s.to_string()),
        is_public: payload.get("is_public").and_then(|v| v.as_bool()).unwrap_or(false),
        target_role: payload.get("target_role").and_then(|v| v.as_str()).map(|s| s.to_string()),
        contact_email: payload.get("contact_email").and_then(|v| v.as_str()).map(|s| s.to_string()),
        contact_phone: payload.get("contact_phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
        contact_location: payload.get("contact_location").and_then(|v| v.as_str()).map(|s| s.to_string()),
        contact_link: payload.get("contact_link").and_then(|v| v.as_str()).map(|s| s.to_string()),
        category_visibility: payload.get("category_visibility").cloned().unwrap_or(serde_json::json!({})),
        category_order: payload.get("category_order").cloned().unwrap_or(serde_json::json!([])),
    };

    let entries_rows = sqlx::query("SELECT id, title, display_order, payload FROM app_content WHERE tenant_id = $1 AND collection_type = 'resume_entry' ORDER BY display_order ASC")
        .bind(tenant_id).fetch_all(&state.pool).await?;

    let entries: Vec<ResumeEntry> = entries_rows
        .into_iter()
        .map(|row| {
            let payload: serde_json::Value = row.try_get("payload").unwrap_or(serde_json::json!({}));
            
            let category_str = payload.get("category").and_then(|v| v.as_str()).unwrap_or("work");
            let category = match category_str {
                "work" => ResumeCategory::Work,
                "education" => ResumeCategory::Education,
                "certification" => ResumeCategory::Certification,
                "skill" => ResumeCategory::Skill,
                "project" => ResumeCategory::Project,
                "language" => ResumeCategory::Language,
                "volunteer" => ResumeCategory::Volunteer,
                "extracurricular" => ResumeCategory::Extracurricular,
                "hobby" => ResumeCategory::Hobby,
                _ => ResumeCategory::Work,
            };

            let bullets: Vec<String> = payload.get("bullets")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            ResumeEntry {
                id: row.get("id"),
                profile_id,
                category,
                title: row.get("title"),
                subtitle: payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string()),
                date_range: payload.get("date_range").and_then(|v| v.as_str()).map(|s| s.to_string()),
                bullets,
                display_order: row.try_get("display_order").unwrap_or(0),
                is_visible: payload.get("is_visible").and_then(|v| v.as_bool()).unwrap_or(true),
                slug: payload.get("slug").and_then(|v| v.as_str()).map(|s| s.to_string()),
                published_at: None,
                metadata: payload.get("metadata").cloned(),
                overrides: payload.get("overrides").cloned(),
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

#[server(GetSingleTenantEntry, "/api")]
pub async fn get_single_tenant_entry(
    slug: String,
) -> Result<Option<ResumeEntry>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let row = sqlx::query("SELECT id, title, display_order, payload FROM app_content WHERE tenant_id = $1 AND collection_type = 'resume_entry' AND payload->>'slug' = $2")
        .bind(tenant_id)
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?;

    if let Some(r) = row {
        let payload: serde_json::Value = r.try_get("payload").unwrap_or(serde_json::json!({}));
        
        let category_str = payload.get("category").and_then(|v| v.as_str()).unwrap_or("work");
        let category = match category_str {
            "work" => ResumeCategory::Work,
            "education" => ResumeCategory::Education,
            "certification" => ResumeCategory::Certification,
            "skill" => ResumeCategory::Skill,
            "project" => ResumeCategory::Project,
            "language" => ResumeCategory::Language,
            "volunteer" => ResumeCategory::Volunteer,
            "extracurricular" => ResumeCategory::Extracurricular,
            "hobby" => ResumeCategory::Hobby,
            _ => ResumeCategory::Work,
        };

        let bullets: Vec<String> = payload.get("bullets")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(Some(ResumeEntry {
            id: r.get("id"),
            profile_id: uuid::Uuid::nil(),
            category,
            title: r.get("title"),
            subtitle: payload.get("subtitle").and_then(|v| v.as_str()).map(|s| s.to_string()),
            date_range: payload.get("date_range").and_then(|v| v.as_str()).map(|s| s.to_string()),
            bullets,
            display_order: r.try_get("display_order").unwrap_or(0),
            is_visible: payload.get("is_visible").and_then(|v| v.as_bool()).unwrap_or(true),
            slug: payload.get("slug").and_then(|v| v.as_str()).map(|s| s.to_string()),
            published_at: None,
            metadata: payload.get("metadata").cloned(),
            overrides: payload.get("overrides").cloned(),
        }))
    } else {
        Ok(None)
    }
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
            published_at: None,
            slug: None,
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
                published_at: None,
                slug: None,
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
                published_at: None,
                slug: None,
            },
        ];

        let tex = generate_latex_string(&profile, &entries);

        assert!(!tex.contains("Hidden Job"));
        assert!(!tex.contains("Hidden Education"));
    }

    #[test]
    fn test_generate_latex_kami_tokens() {
        let profile = ResumeProfile {
            id: 1,
            name: "Kami Profile".into(),
            full_name: "Kami User".into(),
            objective: Some("Design test".into()),
            is_public: true,
            target_role: Some("Designer".into()),
            contact_email: None,
            contact_phone: None,
            contact_location: None,
            contact_link: None,
            category_visibility: serde_json::json!({"skill": true}),
            category_order: serde_json::json!(["skill"]),
        };
        let entries = vec![ResumeEntry {
            id: 1,
            profile_id: 1,
            category: ResumeCategory::Skill,
            title: "Rust".into(),
            subtitle: None,
            date_range: None,
            bullets: vec![],
            display_order: 0,
            is_visible: true,
            metadata: None,
            overrides: None,
            published_at: None,
            slug: None,
        }];

        let tex = generate_latex_string(&profile, &entries);

        // Verify Kami design tokens are present
        assert!(tex.contains("1B365D")); // --brand
        assert!(tex.contains("f5f4ed")); // --parchment
        assert!(tex.contains("EEF2F7")); // --tagbg
        assert!(tex.contains("\\pagecolor{parchment}"));
        assert!(tex.contains("\\textcolor{brand}{\\textendash}"));
        assert!(tex.contains("mdframed")); // Quote box
        assert!(tex.contains("\\colorbox{tagbg}")); // Skill tag
    }
}
