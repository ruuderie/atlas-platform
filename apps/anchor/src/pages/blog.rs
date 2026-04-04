use leptos::*;

use crate::components::content_feed::{ContentFeed, ContentNode, LayoutMode};

#[server(GetPosts, "/api")]
pub async fn get_posts() -> Result<Vec<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query("SELECT id, slug, title, content, to_char(created_at, 'YYYY.MM.DD') as created_at, tags FROM blog_posts WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY created_at DESC")
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    let posts = rows
        .into_iter()
        .map(|row| {
            let id: i32 = row.get("id");
            let slug: String = row.get("slug");

            ContentNode {
                id: id.to_string(),
                category: "blog_post".to_string(),
                title: row.get("title"),
                subtitle: Some(slug),
                date_label: row.try_get("created_at").unwrap_or(None),
                status: None,
                tags: row.get::<Vec<String>, _>("tags"),
                bullets: vec![],
                markdown: Some(row.get("content")),
                link_url: None,
                is_highlight: false,
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
    sqlx::query("INSERT INTO blog_posts (tenant_id, slug, title, content, tags) VALUES ($1, $2, $3, $4, $5)")
        .bind(tenant.0)
        .bind(slug)
        .bind(title)
        .bind(content)
        .bind(tags)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(UpdatePost, "/api")]
pub async fn update_post(
    id: i32,
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
    sqlx::query(
        "UPDATE blog_posts SET slug = $1, title = $2, content = $3, tags = $4 WHERE id = $5 AND tenant_id IS NOT DISTINCT FROM $6",
    )
    .bind(slug)
    .bind(title)
    .bind(content)
    .bind(tags)
    .bind(id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[server(DeletePost, "/api")]
pub async fn delete_post(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM blog_posts WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(id)
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
