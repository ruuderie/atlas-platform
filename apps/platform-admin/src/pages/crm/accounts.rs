use leptos::prelude::*;
use crate::api::crm::get_accounts;
use crate::api::models::AccountModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
    pagination::Pagination,
};

const PER_PAGE: u64 = 25;

fn sanitize_account_name(name: &str) -> String {
    match name {
        "__platform__" => "Platform (System)".to_string(),
        n if n.starts_with("__") && n.ends_with("__") => {
            n.trim_matches('_').replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None    => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        n if n == "—" || n.is_empty() => "Unnamed Account".to_string(),
        n => n.to_string(),
    }
}

#[component]
pub fn AccountsPage() -> impl IntoView {
    let search_filter    = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page             = RwSignal::new(1_u64);

    let selected    = RwSignal::new(None::<AccountModel>);
    let drawer_open = RwSignal::new(false);

    Effect::new(move |_| {
        let val = search_filter.get();
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(350)).await;
            search_debounced.set(val);
            page.set(1);
        });
    });

    let accounts_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let pg     = page.get();
        async move {
            let s = if search.is_empty() { None } else { Some(search.as_str()) };
            get_accounts(s, pg, PER_PAGE).await.unwrap_or_default()
        }
    });

    let kpi_items = Signal::derive(move || {
        let accounts = accounts_res.get().unwrap_or_default();
        let n = accounts.len();
        vec![
            KpiItem::new("Showing",   &n.to_string()).sub("this page"),
            KpiItem::new("Accounts",  &n.to_string()).color("var(--cobalt)"),
            KpiItem::new("MRR",       "—").color("var(--green)"),
            KpiItem::new("Suspended", "—"),
        ]
    });

    let filter_pills = vec![PillOption::new("all", "All Accounts")];
    let active_filter = RwSignal::new("all".to_string());
    let page_count    = Signal::derive(move || accounts_res.get().unwrap_or_default().len());

    view! {
        <div class="entity-page">
            <div class="page-header" style="display:flex;align-items:flex-start;justify-content:space-between;padding:16px 20px;flex-shrink:0;gap:12px;">
                <div>
                    <h1 class="page-title">"Accounts"</h1>
                    <p class="page-subtitle">"Platform-wide · All tenants"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm">"Export CSV"</button>
                    <button class="btn btn-primary btn-sm">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Account"
                    </button>
                </div>
            </div>

            <KpiStrip items=kpi_items />

            <FilterBar
                pills=filter_pills
                active=active_filter
                search=search_filter
                search_placeholder="Search accounts…"
            />

            <div class="table-container">
                <Suspense fallback=move || view! {
                    <div style="padding:32px;text-align:center;color:var(--text-muted)">"Loading accounts…"</div>
                }>
                    {move || {
                        let rows = accounts_res.get().unwrap_or_default();
                        if rows.is_empty() {
                            return view! {
                                <div class="empty-state">
                                    <div class="empty-state-icon">"◎"</div>
                                    <div class="empty-state-title">"No accounts found"</div>
                                    <div class="empty-state-body">"Try adjusting your search query."</div>
                                </div>
                            }.into_any();
                        }
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:32px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                        <th style="width:60%" class="sortable">"Account"</th>
                                        <th style="width:30%" class="sortable">"ID"</th>
                                        <th style="width:70px"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|a| {
                                        let display_name = sanitize_account_name(&a.name);
                                        let ini          = initials(&display_name);
                                        let id_short     = a.id.chars().take(8).collect::<String>() + "…";
                                        let detail_href  = format!("/accounts/{}", a.id);
                                        let a_click      = a.clone();

                                        view! {
                                            <tr style="cursor:pointer" on:click=move |_| {
                                                selected.set(Some(a_click.clone()));
                                                drawer_open.set(true);
                                            }>
                                                <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                                <td>
                                                    <RecordRow
                                                        initials=ini
                                                        name=display_name
                                                        bg="var(--amber-dim)"
                                                        color="var(--amber)"
                                                    />
                                                </td>
                                                <td class="muted mono">{id_short}</td>
                                                <td>
                                                    <a
                                                        href=detail_href
                                                        class="btn btn-ghost btn-sm"
                                                        style="text-decoration:none;"
                                                        on:click=move |e| e.stop_propagation()
                                                    >"Open"</a>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any()
                    }}
                </Suspense>
            </div>

            <Pagination page=page per_page=PER_PAGE count=page_count />
        </div>

        {move || selected.get().map(|a| {
            let display_name = sanitize_account_name(&a.name);
            let name_c  = display_name.clone();
            let id_c    = a.id.clone();
            let title    = Signal::derive(move || name_c.clone());
            let subtitle = Signal::derive(|| "Account".to_string());
            let href     = Signal::derive(move || format!("/accounts/{}", id_c.clone()));
            let full_id  = a.id.clone();

            view! {
                <RecordDrawer open=drawer_open title=title subtitle=subtitle detail_href=href>
                    <div class="detail-grid">
                        <span class="detail-section-label">"Account Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Name"</div>
                            <div class="detail-value">{display_name}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Account ID"</div>
                            <div class="detail-value mono" style="font-size:11px;word-break:break-all">{full_id}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}
    }
}
