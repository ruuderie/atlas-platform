use crate::components::content_feed::{ContentFeed, ContentNode, LayoutMode};
use leptos::*;

#[server(GetCertifications, "/api")]
pub async fn get_certifications() -> Result<Vec<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query("SELECT id, title, subtitle, date_range, metadata FROM tenant_entries WHERE category = 'certification' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY id DESC")
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    let certs = rows
        .into_iter()
        .map(|row| {
            let meta: Option<serde_json::Value> = row.try_get("metadata").unwrap_or(None);
            let is_training = meta
                .as_ref()
                .and_then(|m| m.get("is_training"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let date_range_opt: Option<String> = row.try_get("date_range").unwrap_or(None);
            let id: i32 = row.get("id");

            ContentNode {
                id: id.to_string(),
                category: "certification".to_string(),
                title: row.get("title"),
                subtitle: row.try_get("subtitle").unwrap_or(None),
                date_label: date_range_opt,
                status: None,
                tags: vec![],
                bullets: vec![],
                markdown: None,
                link_url: meta
                    .as_ref()
                    .and_then(|m| m.get("url"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                is_highlight: is_training,
            }
        })
        .collect();

    Ok(certs)
}

#[component]
pub fn Certifications() -> impl IntoView {
    let certs_resource = create_resource(
        || (),
        |_| async move { get_certifications().await.unwrap_or_else(|_| vec![]) },
    );

    view! {
        <main class="pt-32 pb-24 px-4 md:px-[8.5rem] bg-surface min-h-screen">
            <crate::components::dynamic_header::DynamicPageHeader route_path="/certifications".to_string() badge_color="primary".to_string() />

            <Suspense fallback=move || view! { <div class="text-on-surface-variant font-bold jetbrains uppercase">"Verifying cryptographic tokens..."</div> }>
                {move || {
                    let certs = certs_resource.get().unwrap_or_default();
                    let salesforce_certs: Vec<_> = certs.iter().filter(|c| !c.is_highlight).cloned().collect();
                    let training: Vec<_> = certs.iter().filter(|c| c.is_highlight).cloned().collect();

                    view! {
                        <div class="space-y-24">
                            {(!salesforce_certs.is_empty()).then(|| view! {
                                <section>
                                    <h2 class="text-2xl font-extrabold text-secondary mb-8 pl-4 border-l-4 border-secondary uppercase tracking-widest">"Salesforce Architect Credentials"</h2>
                                    <ContentFeed nodes=salesforce_certs layout=LayoutMode::Carousel />
                                </section>
                            })}

                            <section>
                                {(!training.is_empty()).then(|| view! {
                                    <>
                                        <h2 class="text-2xl font-extrabold text-primary mb-8 pl-4 border-l-4 border-primary uppercase tracking-widest">"Executive Training / AI"</h2>
                                        <ContentFeed nodes=training layout=LayoutMode::Carousel />
                                    </>
                                })}

                                <div class="max-w-6xl mt-16 bg-surface-container-low p-8 border-l-4 border-outline">
                                    <h3 class="text-xl font-bold text-on-surface mb-2 uppercase">"Publications"</h3>
                                    <p class="text-sm font-bold text-on-surface-variant mb-6">"Experience ADITL Podcast | A Day In The Life of a Salesforce Technical Architect"</p>
                                    <a href="https://youtube.com/watch?v=wQ2uO4Xw2Ww" target="_blank" class="inline-flex items-center gap-2 px-4 py-2 bg-on-surface text-surface text-xs font-bold uppercase jetbrains hover:bg-secondary cursor-pointer transition-colors shadow-none text-center">
                                        <span class="material-symbols-outlined text-sm">"play_circle"</span>
                                        "Play Podcast"
                                    </a>
                                </div>
                            </section>
                        </div>
                    }
                }}
            </Suspense>
        </main>
    }
}
