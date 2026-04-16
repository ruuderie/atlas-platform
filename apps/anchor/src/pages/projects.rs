use crate::components::content_feed::{ContentFeed, ContentNode, LayoutMode};
use leptos::*;

#[server(GetProjects, "/api")]
pub async fn get_projects() -> Result<Vec<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query("SELECT id, title, date_range, bullets, metadata FROM tenant_entries WHERE category = 'project' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY id DESC")
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    let projs = rows
        .into_iter()
        .map(|row| {
            let meta: Option<serde_json::Value> = row.try_get("metadata").unwrap_or(None);
            let slug = meta
                .as_ref()
                .and_then(|m| m.get("slug"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let impact = meta
                .as_ref()
                .and_then(|m| m.get("impact"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = meta
                .as_ref()
                .and_then(|m| m.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("COMPLETED")
                .to_string();

            let tags: Vec<String> = meta
                .as_ref()
                .and_then(|m| m.get("tags"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| {
                    meta.as_ref()
                        .and_then(|m| m.get("tags"))
                        .and_then(|v| v.as_str())
                        .map(|s| {
                            s.split(',')
                                .map(|x| x.trim().to_string())
                                .filter(|x| !x.is_empty())
                                .collect()
                        })
                        .unwrap_or_default()
                });

            let bullets_val: serde_json::Value =
                row.try_get("bullets").unwrap_or(serde_json::json!([]));
            let bullets: Vec<String> = serde_json::from_value(bullets_val).unwrap_or_default();
            let date_range_opt: Option<String> = row.try_get("date_range").unwrap_or(None);
            let id: i32 = row.get("id");

            ContentNode {
                id: id.to_string(),
                category: "project".to_string(),
                title: row.get("title"),
                subtitle: if impact.is_empty() {
                    None
                } else {
                    Some(impact)
                },
                date_label: date_range_opt,
                status: Some(status),
                tags,
                bullets,
                markdown: None,
                link_url: if slug.is_empty() {
                    None
                } else {
                    Some(format!("https://github.com/ruuderie/{}", slug))
                },
                is_highlight: false,
            }
        })
        .collect();

    Ok(projs)
}

#[component]
pub fn Projects() -> impl IntoView {
    let projs_resource = create_resource(
        || (),
        |_| async move { get_projects().await.unwrap_or_else(|_| vec![]) },
    );

    view! {
        <main class="pt-32 pb-24 px-4 md:px-[8.5rem] bg-surface min-h-screen">
            <crate::components::dynamic_header::DynamicPageHeader route_path="/projects".to_string() badge_color="primary".to_string() />

            <Suspense fallback=move || view! { <div class="text-on-surface-variant font-bold jetbrains uppercase">"Indexing project graphs..."</div> }>
                {move || {
                    let projects = projs_resource.get().unwrap_or_default();
                    view! { <ContentFeed nodes=projects layout=LayoutMode::Grid /> }
                }}
            </Suspense>
        </main>
    }
}
