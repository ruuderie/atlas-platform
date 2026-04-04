use leptos::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PageHeaderData {
    pub route_path: String,
    pub badge_text: Option<String>,
    pub title: String,
    pub subtitle: Option<String>,
}

impl Default for PageHeaderData {
    fn default() -> Self {
        Self {
            route_path: "".to_string(),
            badge_text: None,
            title: "".to_string(),
            subtitle: None,
        }
    }
}

#[server(GetPageHeader, "/api")]
pub async fn get_page_header(route_path: String) -> Result<PageHeaderData, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    match sqlx::query(
        "SELECT route_path, badge_text, title, subtitle FROM page_headers WHERE route_path = $1",
    )
    .bind(&route_path)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(row)) => Ok(PageHeaderData {
            route_path: row.get("route_path"),
            badge_text: row.get("badge_text"),
            title: row.get("title"),
            subtitle: row.get("subtitle"),
        }),
        Ok(None) | Err(_) => Ok(PageHeaderData {
            route_path: route_path.clone(),
            badge_text: None,
            title: if route_path.is_empty() || route_path == "/" {
                "HOME".to_string()
            } else {
                route_path.replace("/", "").to_uppercase()
            },
            subtitle: None,
        }),
    }
}

#[server(GetAllPageHeaders, "/api")]
pub async fn get_all_page_headers() -> Result<Vec<PageHeaderData>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = match sqlx::query(
        "SELECT route_path, badge_text, title, subtitle FROM page_headers ORDER BY route_path ASC",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return Ok(vec![]),
    };

    let mut headers = Vec::new();
    for row in rows {
        headers.push(PageHeaderData {
            route_path: row.get("route_path"),
            badge_text: row.get("badge_text"),
            title: row.get("title"),
            subtitle: row.get("subtitle"),
        });
    }

    Ok(headers)
}

#[server(UpdatePageHeader, "/api")]
pub async fn update_page_header(
    route_path: String,
    badge_text: Option<String>,
    title: String,
    subtitle: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query("INSERT INTO page_headers (route_path, badge_text, title, subtitle) VALUES ($1, $2, $3, $4) ON CONFLICT (route_path) DO UPDATE SET badge_text = EXCLUDED.badge_text, title = EXCLUDED.title, subtitle = EXCLUDED.subtitle")
        .bind(route_path)
        .bind(badge_text)
        .bind(title)
        .bind(subtitle)
        .execute(&state.pool).await?;

    Ok(())
}

#[component]
pub fn DynamicPageHeader(
    route_path: String,
    #[prop(default = "secondary".to_string())] badge_color: String,
) -> impl IntoView {
    let header_resource = create_resource(move || route_path.clone(), |r| get_page_header(r));

    let badge_classes = match badge_color.as_str() {
        "primary" => "text-on-primary bg-primary",
        _ => "text-secondary border border-secondary/30 bg-secondary/5",
    };

    view! {
        <Suspense fallback=move || view! { <header class="mb-16 md:mb-24 flex flex-col items-start"><div class="h-6 w-32 bg-surface-container-high animate-pulse mb-8"></div><div class="h-12 w-96 bg-surface-container-high animate-pulse mb-6"></div><div class="h-6 w-full max-w-2xl bg-surface-container-high animate-pulse"></div></header> }>
            {move || {
                let header = header_resource.get().unwrap_or(Ok(PageHeaderData {
                    route_path: String::new(),
                    badge_text: None,
                    title: "LOADING...".to_string(),
                    subtitle: None
                })).unwrap_or_default();

                view! {
                    <header class="mb-16 md:mb-24 flex flex-col items-start">
                        {if let Some(bt) = header.badge_text {
                            if !bt.is_empty() {
                                Some(view! {
                                    <div class=format!("px-3 py-1 jetbrains text-[0.625rem] font-bold tracking-widest uppercase mb-8 {}", badge_classes)>
                                        {bt}
                                    </div>
                                }.into_view())
                            } else { None }
                        } else { None }}
                        <h1 class="text-4xl md:text-6xl font-extrabold tracking-[-0.02em] text-primary mb-6 uppercase">
                            {header.title}
                        </h1>
                        {if let Some(st) = header.subtitle {
                            if !st.is_empty() {
                                Some(view! {
                                    <p class="text-lg md:text-xl text-on-surface-variant font-medium tracking-tight max-w-3xl leading-relaxed">
                                        {st}
                                    </p>
                                }.into_view())
                            } else { None }
                        } else { None }}
                    </header>
                }
            }}
        </Suspense>
    }
}
