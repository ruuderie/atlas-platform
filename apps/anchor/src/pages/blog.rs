use leptos::*;

use crate::components::content_feed::{ContentFeed, ContentNode, LayoutMode};

#[server(GetPosts, "/api")]
pub async fn get_posts() -> Result<Vec<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    // content_format defaults to 'markdown' via column DEFAULT — all existing posts unaffected
    let rows = sqlx::query(
        "SELECT id, title, to_char(created_at, 'YYYY.MM.DD') as created_at, payload \
         FROM app_content WHERE collection_type = 'blog_post' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY created_at DESC"
    )
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    let posts = rows
        .into_iter()
        .map(|row| {
            let id: uuid::Uuid = row.get("id");
            let payload: serde_json::Value = row.get("payload");
            
            let slug = payload.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tags: Vec<String> = payload.get("tags").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let content_format = payload.get("content_format").and_then(|v| v.as_str()).unwrap_or("markdown").to_string();

            ContentNode {
                id: id.to_string(),
                category: "blog_post".to_string(),
                title: row.get("title"),
                subtitle: Some(slug),
                date_label: row.try_get("created_at").unwrap_or(None),
                status: None,
                tags,
                bullets: vec![],
                markdown: Some(content),
                link_url: None,
                is_highlight: false,
                content_format,
            }
        })
        .collect();

    Ok(posts)
}


#[server(AddPost, "/api")]
pub async fn add_post(
    slug: String,
    title: String,
    content: String,
    tags: Vec<String>,
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
        "slug": slug,
        "content": content,
        "tags": tags,
        "content_format": "markdown"
    });

    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload) VALUES ($1, 'blog_post', $2, $3)")
        .bind(tenant.0)
        .bind(title)
        .bind(payload)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(UpdatePost, "/api")]
pub async fn update_post(
    id: String,
    slug: String,
    title: String,
    content: String,
    tags: Vec<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    
    let uuid_id = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    
    let payload = serde_json::json!({
        "slug": slug,
        "content": content,
        "tags": tags,
        "content_format": "markdown"
    });

    sqlx::query(
        "UPDATE app_content SET title = $1, payload = $2 WHERE id = $3 AND tenant_id IS NOT DISTINCT FROM $4 AND collection_type = 'blog_post'",
    )
    .bind(title)
    .bind(payload)
    .bind(uuid_id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[server(DeletePost, "/api")]
pub async fn delete_post(id: String) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    
    let uuid_id = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2 AND collection_type = 'blog_post'")
        .bind(uuid_id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[component]
pub fn Blog() -> impl IntoView {
    let posts_resource = create_resource(
        || (),
        |_| async move { get_posts().await.unwrap_or_else(|_| vec![]) },
    );

    view! {
        <main class="pt-32 pb-24 px-4 md:px-[8.5rem] bg-surface-container-low min-h-screen">
            <crate::components::dynamic_header::DynamicPageHeader route_path="/blog".to_string() badge_color="primary".to_string() />

            <Suspense fallback=move || view! { <div class="text-on-surface-variant font-bold jetbrains uppercase">"Fetching remote Markdown streams..."</div> }>
                {move || {
                    let posts = posts_resource.get().unwrap_or_default();
                    view! { <ContentFeed nodes=posts layout=LayoutMode::List /> }
                }}
            </Suspense>
        </main>
    }
}
