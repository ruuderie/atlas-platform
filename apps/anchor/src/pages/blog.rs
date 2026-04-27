use leptos::*;
use leptos_router::*;

use crate::components::content_feed::{ContentFeed, ContentNode, LayoutMode};

#[server(GetPosts, "/api")]
pub async fn get_posts() -> Result<Vec<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
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
            let link_url = format!("/blog/{}", slug);

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
                link_url: Some(link_url),
                is_highlight: false,
                content_format,
            }
        })
        .collect();

    Ok(posts)
}

#[server(GetPostBySlug, "/api")]
pub async fn get_post_by_slug(slug: String) -> Result<Option<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let row = sqlx::query(
        "SELECT id, title, to_char(created_at, 'YYYY.MM.DD') as created_at, payload \
         FROM app_content WHERE collection_type = 'blog_post' \
         AND tenant_id IS NOT DISTINCT FROM $1 AND payload->>'slug' = $2 LIMIT 1"
    )
        .bind(tenant.0)
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?;

    Ok(row.map(|row| {
        let id: uuid::Uuid = row.get("id");
        let payload: serde_json::Value = row.get("payload");
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
    }))
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

    let uuid_id = match uuid::Uuid::parse_str(&id) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

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

    let uuid_id = match uuid::Uuid::parse_str(&id) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2 AND collection_type = 'blog_post'")
        .bind(uuid_id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

// ─── Blog list page ───────────────────────────────────────────────────────────

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

// ─── Blog post detail page ────────────────────────────────────────────────────

#[component]
pub fn BlogPost() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned().unwrap_or_default());

    let post_resource = create_resource(slug, |s| async move {
        get_post_by_slug(s).await.unwrap_or(None)
    });

    view! {
        <main class="pt-28 pb-24 bg-surface-container-low min-h-screen">
            // KaTeX CSS — only loaded on this page
            <link rel="stylesheet"
                href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css"
                crossorigin="anonymous" />

            <Suspense fallback=move || view! {
                <div class="max-w-3xl mx-auto px-6 pt-12">
                    <div class="text-on-surface-variant font-bold jetbrains uppercase animate-pulse">"Loading paper..."</div>
                </div>
            }>
                {move || {
                    match post_resource.get() {
                        None => view! { <div /> }.into_view(),
                        Some(None) => view! {
                            <div class="max-w-3xl mx-auto px-6 pt-12 text-on-surface-variant">
                                "Post not found."
                                <a href="/blog" class="ml-4 text-secondary underline">"← Back to blog"</a>
                            </div>
                        }.into_view(),
                        Some(Some(post)) => {
                            let content_format = post.content_format.clone();
                            let html = render_post_html(&post);
                            let title = post.title.clone();
                            let date = post.date_label.clone().unwrap_or_default();
                            let tags = post.tags.clone();
                            let needs_katex = content_format == "mdlatex" || content_format == "latex";

                            view! {
                                <article class="max-w-3xl mx-auto px-6 md:px-0">
                                    // Breadcrumb
                                    <nav class="mb-10">
                                        <a href="/blog"
                                           class="inline-flex items-center gap-2 text-on-surface-variant hover:text-secondary transition-colors jetbrains text-[0.65rem] uppercase tracking-widest">
                                            <span class="material-symbols-outlined text-sm">"arrow_back"</span>
                                            "Technical Writing"
                                        </a>
                                    </nav>

                                    // Paper header
                                    <header class="mb-12 pb-10 border-b border-outline-variant/30">
                                        <h1 class="text-3xl md:text-4xl font-extrabold text-primary leading-tight mb-6">
                                            {title}
                                        </h1>
                                        <div class="flex flex-wrap items-center gap-4">
                                            <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-widest">
                                                {date}
                                            </span>
                                            {tags.into_iter().map(|tag| view! {
                                                <span class="bg-secondary/10 text-secondary px-2 py-0.5 jetbrains text-[0.6rem] uppercase font-bold tracking-wider">
                                                    {tag}
                                                </span>
                                            }).collect_view()}
                                        </div>
                                    </header>

                                    // Paper body — academic prose styling
                                    <div
                                        class="prose prose-invert max-w-none
                                               prose-headings:font-bold prose-headings:text-primary prose-headings:mt-10 prose-headings:mb-4
                                               prose-h2:text-2xl prose-h3:text-xl prose-h4:text-lg
                                               prose-p:text-on-surface-variant prose-p:leading-relaxed prose-p:text-[0.95rem] prose-p:my-4
                                               prose-strong:text-on-surface prose-strong:font-bold
                                               prose-ul:text-on-surface-variant prose-ul:my-4 prose-ul:space-y-1
                                               prose-li:text-[0.95rem] prose-li:leading-relaxed
                                               prose-code:text-secondary prose-code:bg-surface-container-highest prose-code:px-1 prose-code:rounded prose-code:text-sm
                                               prose-pre:bg-surface-container-highest prose-pre:p-4 prose-pre:rounded-none prose-pre:text-sm
                                               prose-blockquote:border-l-2 prose-blockquote:border-secondary prose-blockquote:text-on-surface-variant
                                               [&_.katex]:text-on-surface [&_.katex-display]:my-8 [&_.katex-display]:overflow-x-auto"
                                        inner_html=html
                                    >
                                    </div>

                                    // KaTeX auto-render script (only when needed)
                                    {if needs_katex {
                                        view! {
                                            <script
                                                src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"
                                                crossorigin="anonymous"
                                                defer=true
                                            />
                                            <script>
                                                "document.addEventListener('DOMContentLoaded', function() { \
                                                    if (typeof katex !== 'undefined') { \
                                                        document.querySelectorAll('.math-inline').forEach(function(el) { \
                                                            katex.render(el.textContent, el, {displayMode: false, throwOnError: false}); \
                                                        }); \
                                                        document.querySelectorAll('.math-display').forEach(function(el) { \
                                                            katex.render(el.textContent, el, {displayMode: true, throwOnError: false}); \
                                                        }); \
                                                    } \
                                                });"
                                            </script>
                                        }.into_view()
                                    } else {
                                        view! { <span /> }.into_view()
                                    }}
                                </article>
                            }.into_view()
                        }
                    }
                }}
            </Suspense>
        </main>
    }
}

fn render_post_html(post: &ContentNode) -> String {
    if let Some(md) = &post.markdown {
        match post.content_format.as_str() {
            "latex" => {
                format!("<div class='katex-content'><pre class='katex-source'>{}</pre></div>",
                    html_escape::encode_text(md))
            }
            // mdlatex and markdown both go through pulldown_cmark;
            _ => {
                let mut options = pulldown_cmark::Options::empty();
                options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
                options.insert(pulldown_cmark::Options::ENABLE_MATH);
                let parser = pulldown_cmark::Parser::new_ext(md, options);
                let mut html = String::new();
                pulldown_cmark::html::push_html(&mut html, parser);
                html
            }
        }
    } else {
        String::new()
    }
}
