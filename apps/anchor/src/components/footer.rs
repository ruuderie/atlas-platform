use leptos::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FooterItemRecord {
    pub id: i32,
    pub label: String,
    pub href: Option<String>,
    pub display_order: i32,
    pub is_visible: bool,
}

#[server(GetFooterItems, "/api")]
pub async fn get_footer_items() -> Result<Vec<FooterItemRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows = sqlx::query(
        "SELECT * FROM footer_items WHERE is_visible = true ORDER BY display_order ASC",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut items = Vec::new();
    for row in rows {
        items.push(FooterItemRecord {
            id: row.get("id"),
            label: row.get("label"),
            href: row.get("href"),
            display_order: row.get("display_order"),
            is_visible: row.get("is_visible"),
        });
    }
    Ok(items)
}

#[server(GetAllFooterItems, "/api")]
pub async fn get_all_footer_items() -> Result<Vec<FooterItemRecord>, ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows = sqlx::query("SELECT * FROM footer_items ORDER BY display_order ASC")
        .fetch_all(&state.pool)
        .await?;
    let mut items = Vec::new();
    for row in rows {
        items.push(FooterItemRecord {
            id: row.get("id"),
            label: row.get("label"),
            href: row.get("href"),
            display_order: row.get("display_order"),
            is_visible: row.get("is_visible"),
        });
    }
    Ok(items)
}

#[server(AddFooterItem, "/api")]
pub async fn add_footer_item(
    label: String,
    href: Option<String>,
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
    sqlx::query(
        "INSERT INTO footer_items (label, href, display_order, is_visible) VALUES ($1, $2, $3, $4)",
    )
    .bind(label)
    .bind(href)
    .bind(display_order)
    .bind(is_visible)
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[server(UpdateFooterItem, "/api")]
pub async fn update_footer_item(
    id: i32,
    label: String,
    href: Option<String>,
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
    sqlx::query("UPDATE footer_items SET label = $1, href = $2, display_order = $3, is_visible = $4 WHERE id = $5")
        .bind(label).bind(href).bind(display_order).bind(is_visible).bind(id)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteFooterItem, "/api")]
pub async fn delete_footer_item(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    sqlx::query("DELETE FROM footer_items WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[component]
pub fn Footer() -> impl IntoView {
    let footer_resource = create_resource(|| (), |_| get_footer_items());
    let settings_resource = create_resource(|| (), |_| crate::pages::landing::get_site_settings());

    view! {
        <footer class="w-full border-t border-outline-variant/30 py-8 px-6 lg:px-[8.5rem] bg-surface-container-low mt-auto flex flex-col lg:flex-row flex-wrap justify-between items-center text-xs jetbrains text-outline gap-8 lg:gap-6">
            <div class="flex flex-col lg:flex-row items-center space-y-2 lg:space-y-0 lg:space-x-4 text-center">
                <span class="break-words">"© 2026 RUUD SALYM ERIE. ALL RIGHTS RESERVED."</span>
                <span class="hidden lg:inline text-on-surface-variant">"|"</span>
                <span class="text-surface-variant font-bold text-outline break-words">"OPLYST INTERNATIONAL, LLC."</span>
            </div>

            <div class="flex flex-wrap justify-center items-center gap-6">
                <Suspense fallback=move || view! { <div class="w-24 h-4 bg-outline-variant/20 animate-pulse rounded"></div> }>
                    {move || {
                        let items = footer_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();
                        items.into_iter().map(|item| {
                            view! {
                                <a href=item.href.clone().unwrap_or_else(|| "#".to_string()) class="text-slate-500 dark:text-slate-400 font-medium hover:text-primary transition-colors tracking-widest uppercase text-[0.65rem] text-center whitespace-normal break-words">
                                    {item.label.clone()}
                                </a>
                            }
                        }).collect_view()
                    }}
                </Suspense>
            </div>

            <div class="flex flex-wrap items-center justify-center space-x-6">
                <Suspense fallback=move || view! { <div></div> }>
                    {move || {
                        let s = settings_resource.get().unwrap_or(Ok(crate::pages::landing::SiteSettings::default())).unwrap_or_default();
                        view! {
                            <div class="flex items-center space-x-4">
                                {if !s.github_url.is_empty() {
                                    Some(view! {
                                        <a href=&s.github_url target="_blank" rel="noopener noreferrer" class="text-slate-400 hover:text-primary transition-colors">
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                <path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" />
                                            </svg>
                                        </a>
                                    })
                                } else { None }}

                                {if !s.x_url.is_empty() {
                                    Some(view! {
                                        <a href=&s.x_url target="_blank" rel="noopener noreferrer" class="text-slate-400 hover:text-primary transition-colors">
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
                                            </svg>
                                        </a>
                                    })
                                } else { None }}

                                {if !s.linkedin_url.is_empty() {
                                    Some(view! {
                                        <a href=&s.linkedin_url target="_blank" rel="noopener noreferrer" class="text-slate-400 hover:text-primary transition-colors">
                                            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                                <path fill-rule="evenodd" d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z" clip-rule="evenodd" />
                                            </svg>
                                        </a>
                                    })
                                } else { None }}
                            </div>
                        }
                    }}
                </Suspense>
            </div>

            <div class="flex flex-col sm:flex-row items-center justify-center space-y-2 sm:space-y-0 sm:space-x-3 w-full lg:w-auto mt-4 lg:mt-0">
                <span class="text-[0.65rem] tracking-widest uppercase text-on-surface-variant break-words text-center">"Engineered natively in"</span>
                <a href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer" class="flex items-center opacity-70 hover:opacity-100 transition-opacity p-2 bg-surface-container hover:bg-surface-container-high rounded-sm">
                    <img src="https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg" alt="Rust Logo" class="h-5 w-5 dark:invert" />
                    <span class="ml-2 font-bold text-on-surface tracking-widest">"RUST"</span>
                </a>
            </div>
        </footer>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_footer_rendering() {
        // Mock footer mount without DB
        // anchor uses Leptos 0.6 standard architecture
    }
}
