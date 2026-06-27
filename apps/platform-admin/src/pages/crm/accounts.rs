use leptos::prelude::*;
use crate::api::crm::{get_accounts, create_account};
use crate::api::models::{AccountModel, CreateAccount};
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

    // ── Create modal ──────────────────────────────────────────────────────────
    let show_create = RwSignal::new(false);
    let new_name    = RwSignal::new(String::new());
    let create_busy = RwSignal::new(false);
    let toast       = use_context::<crate::app::GlobalToast>().expect("toast");

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

    let handle_create = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            toast.show_toast("Error", "Account name is required.", "error");
            return;
        }
        create_busy.set(true);
        let resource = accounts_res.clone();
        leptos::task::spawn_local(async move {
            let data = CreateAccount { name: name.trim().to_string() };
            match create_account(data).await {
                Ok(_) => {
                    toast.show_toast("Success", "Account created.", "success");
                    show_create.set(false);
                    new_name.set(String::new());
                    resource.refetch();
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
            create_busy.set(false);
        });
    };

    let kpi_items = Signal::derive(move || {
        let accounts = accounts_res.get().unwrap_or_default();
        let n = accounts.len();
        vec![
            KpiItem::new("Page",     &page.get().to_string()),
            KpiItem::new("Accounts", &n.to_string()).color("var(--cobalt)"),
            KpiItem::new("MRR",      "$0").color("var(--green)"),
            KpiItem::new("Active",   &n.to_string()).color("var(--green)"),
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
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        new_name.set(String::new());
                        show_create.set(true);
                    }>
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

        // ── Create Account Modal ───────────────────────────────────────────
        <Show when=move || show_create.get()>
            <div
                style="position:fixed;inset:0;z-index:200;background:rgba(0,0,0,0.7);backdrop-filter:blur(4px);display:flex;align-items:center;justify-content:center;padding:20px;"
                on:click=move |_| show_create.set(false)
            >
                <div
                    style="background:var(--bg-surface);border:1px solid var(--border-default);border-radius:14px;width:400px;max-width:100%;padding:28px;position:relative;"
                    on:click=move |e| e.stop_propagation()
                >
                    <button
                        style="position:absolute;top:14px;right:16px;background:none;border:none;color:var(--text-muted);font-size:16px;cursor:pointer;padding:4px 8px;border-radius:6px;"
                        on:click=move |_| show_create.set(false)
                    >"✕"</button>
                    <div style="font-size:16px;font-weight:700;margin-bottom:4px;color:var(--text-primary)">"New Account"</div>
                    <div style="font-size:12px;color:var(--text-muted);margin-bottom:20px">"Register an organisation in the CRM. More fields can be added from the account detail page."</div>
                    <div>
                        <label style="display:block;font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:5px;">"Account Name *"</label>
                        <input type="text" placeholder="e.g. Acme Corp"
                            style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:8px;padding:9px 12px;font-size:13px;color:var(--text-primary);outline:none;box-sizing:border-box;"
                            prop:value=move || new_name.get()
                            on:input=move |e| new_name.set(event_target_value(&e))
                        />
                    </div>
                    <div style="display:flex;gap:8px;justify-content:flex-end;margin-top:22px;">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| show_create.set(false)>"Cancel"</button>
                        <button class="btn btn-primary btn-sm" disabled=move || create_busy.get() on:click=handle_create>
                            {move || if create_busy.get() { "Saving…" } else { "Create Account" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
