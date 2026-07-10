use leptos::prelude::*;
use crate::api::crm::{get_accounts, create_account};
use crate::api::models::{AccountModel, AccountType, AccountStatus, CreateAccount};
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
    pagination::Pagination,
};

const PER_PAGE: u64 = 25;

/// Return a display-safe account name, handling system-internal sentinel accounts.
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

// Utility free-fns removed — use AccountType/AccountStatus enum methods directly:
//   account.account_type.badge_class()  → CSS classes
//   account.account_type.label()        → display text
//   account.account_type.is_individual()→ bool
//   account.status.color()              → CSS color string
//   account.status.label()              → display text

#[component]
pub fn AccountsPage() -> impl IntoView {
    let search_filter    = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let status_filter    = RwSignal::new("all".to_string());
    let type_filter      = RwSignal::new("all".to_string());
    let page             = RwSignal::new(1_u64);

    let selected    = RwSignal::new(None::<AccountModel>);
    let drawer_open = RwSignal::new(false);

    // ── Create modal ──────────────────────────────────────────────────────────
    let show_create = RwSignal::new(false);
    let new_name    = RwSignal::new(String::new());
    let create_busy = RwSignal::new(false);
    let toast       = use_context::<crate::app::GlobalToast>().expect("toast");

    // Debounce search input
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
        let status = status_filter.get();
        let atype  = type_filter.get();
        let pg     = page.get();
        async move {
            let s = if search.is_empty() { None } else { Some(search.as_str()) };
            let st = if status == "all" { None } else { Some(status.as_str()) };
            let at = if atype == "all" { None } else { Some(atype.as_str()) };
            get_accounts(s, pg, PER_PAGE, st, at).await.unwrap_or_default()
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

    // ── KPI strip — use enum comparisons, not string matches ─────────────────
    let kpi_items = Signal::derive(move || {
        let accounts = accounts_res.get().unwrap_or_default();
        let total    = accounts.len();
        let active   = accounts.iter().filter(|a| a.status   == AccountStatus::Active).count();
        let prospect = accounts.iter().filter(|a| a.status   == AccountStatus::Prospect).count();
        let orgs     = accounts.iter().filter(|a| a.account_type == AccountType::Organization).count();
        let inds     = accounts.iter().filter(|a| a.account_type == AccountType::Individual).count();
        vec![
            KpiItem::new("Total",         &total.to_string()).color("var(--cobalt)"),
            KpiItem::new("Active",        &active.to_string()).color("var(--green)"),
            KpiItem::new("Prospect",      &prospect.to_string()).color("var(--amber)"),
            KpiItem::new("Organizations", &orgs.to_string()),
            KpiItem::new("Individuals",   &inds.to_string()),
        ]
    });

    // ── Filter pills ──────────────────────────────────────────────────────────
    let status_pills = vec![
        PillOption::new("all",       "All"),
        PillOption::new("active",    "Active"),
        PillOption::new("prospect",  "Prospect"),
        PillOption::new("suspended", "Suspended"),
    ];
    let type_pills = vec![
        PillOption::new("all",          "All Types"),
        PillOption::new("organization", "Orgs"),
        PillOption::new("individual",   "Individuals"),
    ];

    let page_count = Signal::derive(move || accounts_res.get().unwrap_or_default().len());

    view! {
        <div class="entity-page">
            // ── Page Header ───────────────────────────────────────────────────
            <div class="page-header" style="display:flex;align-items:flex-start;justify-content:space-between;padding:16px 20px;flex-shrink:0;gap:12px;">
                <div>
                    <h1 class="page-title">"Accounts"</h1>
                    <p class="page-subtitle">"Platform-wide · B2B party registry"</p>
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

            // ── KPI Strip ─────────────────────────────────────────────────────
            <KpiStrip items=kpi_items />

            // ── Filter Bar ────────────────────────────────────────────────────
            // Status pills
            <div style="display:flex;align-items:center;gap:6px;padding:10px 20px;border-bottom:1px solid var(--border-default);flex-shrink:0;flex-wrap:wrap;">
                {status_pills.into_iter().map(|p| {
                    let key_cls   = p.value.clone();
                    let key_click = p.value.clone();
                    let label = p.label.clone();
                    view! {
                        <button
                            class=move || if status_filter.get() == key_cls { "pill active" } else { "pill" }
                            on:click=move |_| {
                                status_filter.set(key_click.clone());
                                type_filter.set("all".to_string());
                                page.set(1);
                            }
                        >{label}</button>
                    }
                }).collect::<Vec<_>>()}
                <div style="width:1px;height:20px;background:var(--border-default);margin:0 4px;"></div>
                {type_pills.into_iter().map(|p| {
                    let key_cls   = p.value.clone();
                    let key_click = p.value.clone();
                    let label = p.label.clone();
                    view! {
                        <button
                            class=move || if type_filter.get() == key_cls { "pill active" } else { "pill" }
                            on:click=move |_| {
                                type_filter.set(key_click.clone());
                                status_filter.set("all".to_string());
                                page.set(1);
                            }
                        >{label}</button>
                    }
                }).collect::<Vec<_>>()}
                <div style="margin-left:auto;">
                    <input
                        type="text"
                        class="filter-input"
                        placeholder="Search name, domain, industry…"
                        on:input=move |ev| search_filter.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // ── Table ─────────────────────────────────────────────────────────
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
                                    <div class="empty-state-body">"Try adjusting your search or filters."</div>
                                </div>
                            }.into_any();
                        }
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:28%" class="sortable">"Account"</th>
                                        <th style="width:8%">"Type"</th>
                                        <th style="width:13%" class="sortable">"Status"</th>
                                        <th style="width:16%" class="sortable">"Domain"</th>
                                        <th style="width:15%" class="sortable">"Industry"</th>
                                        <th style="width:13%" class="sortable">"Location"</th>
                                        <th style="width:70px"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                {rows.into_iter().map(|account| {
                                    let acc_clone = account.clone();
                                    let acc_click = account.clone();
                                    let display_name = sanitize_account_name(&account.name);
                                    let initials_str = initials(&display_name);
                                    // Use enum methods — no string comparisons
                                    let status_col   = account.status.color();
                                    let status_label = account.status.label().to_string();
                                    let type_label   = account.account_type.label();
                                    let type_class   = account.account_type.badge_class();
                                    let domain   = account.domain.as_deref().unwrap_or("—").to_string();
                                    let industry = account.industry.as_deref().unwrap_or("—").to_string();
                                    let location = match (&account.city, &account.state) {
                                        (Some(c), Some(s)) => format!("{}, {}", c, s),
                                        (Some(c), None)    => c.clone(),
                                        (None, Some(s))    => s.clone(),
                                        _                  => "—".to_string(),
                                    };
                                    let sub = if let Some(yr) = account.year_established {
                                        format!("est. {}", yr)
                                    } else {
                                        account.company_type.clone().unwrap_or_default()
                                    };
                                    let is_ind = account.account_type.is_individual();
                                    let detail_href = format!("/accounts/{}", account.id);
                                    view! {
                                        <tr
                                            style="cursor:pointer;"
                                            on:click=move |_| {
                                                selected.set(Some(acc_click.clone()));
                                                drawer_open.set(true);
                                            }
                                        >
                                            <td>
                                                <div style="display:flex;align-items:center;gap:10px;">
                                                    <div class=move || if is_ind { "crm-avatar crm-avatar-ind" } else { "crm-avatar" }>
                                                        {initials_str.clone()}
                                                    </div>
                                                    <div>
                                                        <div style="font-size:12.5px;font-weight:500;">{display_name.clone()}</div>
                                                        <div style="font-size:11px;color:var(--text-muted);margin-top:1px;">{sub.clone()}</div>
                                                    </div>
                                                </div>
                                            </td>
                                            <td><span class={type_class}>{type_label}</span></td>
                                            <td>
                                                <div style="display:flex;align-items:center;gap:6px;">
                                                    <span style=move || format!("width:6px;height:6px;border-radius:50%;background:{};flex-shrink:0;", status_col)></span>
                                                    <span style="font-size:12px;color:var(--text-secondary);">{status_label.clone()}</span>
                                                </div>
                                            </td>
                                            <td class=if domain == "—" { "muted mono" } else { "mono" }>{domain.clone()}</td>
                                            <td class="muted">{industry.clone()}</td>
                                            <td class="muted">{location.clone()}</td>
                                            <td>
                                                <a
                                                    href=detail_href
                                                    class="btn btn-ghost btn-sm"
                                                    style="text-decoration:none;"
                                                    on:click=move |ev| ev.stop_propagation()
                                                >"Open"</a>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        }.into_any()
                    }}
                </Suspense>
            </div>

            // ── Pagination ────────────────────────────────────────────────────
            <Pagination
                page=page
                per_page=PER_PAGE
                count=page_count
            />

            // ── Detail Drawer ─────────────────────────────────────────────────
            {move || {
                let open = drawer_open.get();
                let account = selected.get();
                if !open { return view! { <div></div> }.into_any(); }
                let Some(acc) = account else { return view! { <div></div> }.into_any(); };

                let display_name = sanitize_account_name(&acc.name);
                let initials_str = initials(&display_name);
                // Use enum methods in the drawer too
                let is_ind        = acc.account_type.is_individual();
                let status_col    = acc.status.color();
                let type_label    = acc.account_type.label();
                let type_class    = acc.account_type.badge_class();
                let status_label  = acc.status.label().to_string();

                view! {
                    <div>
                        // Backdrop
                        <div
                            style="position:fixed;inset:0;background:rgba(0,0,0,0.4);backdrop-filter:blur(2px);z-index:100;"
                            on:click=move |_| drawer_open.set(false)
                        ></div>

                        // Drawer panel
                        <div class="record-drawer open">
                            // Header
                            <div class="panel-header">
                                <div class="panel-header-top">
                                    <div class="panel-identity">
                                        <div class=move || if is_ind { "crm-avatar crm-avatar-ind" } else { "crm-avatar" } style="width:40px;height:40px;font-size:14px;margin-bottom:8px;">
                                            {initials_str}
                                        </div>
                                        <div class="panel-title-text">{display_name.clone()}</div>
                                        <div class="panel-subtitle-text">
                                            <div style="display:flex;align-items:center;gap:6px;margin-top:2px;">
                                                <span class={type_class}>{type_label}</span>
                                                <span style=move || format!("width:5px;height:5px;border-radius:50%;background:{};", status_col)></span>
                                                <span style="font-size:11px;color:var(--text-muted);">{status_label}</span>
                                            </div>
                                        </div>
                                    </div>
                                    <button class="panel-close" on:click=move |_| drawer_open.set(false)>
                                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/></svg>
                                    </button>
                                </div>
                                <div class="panel-actions">
                                    <a href={format!("/accounts/{}", acc.id)}
                                        class="btn btn-ghost btn-sm"
                                        style="text-decoration:none;"
                                    >"View Full Record →"</a>
                                </div>
                            </div>

                            // Body — firmographic fields from atlas_accounts
                            <div class="panel-content" style="overflow-y:auto;padding:0 20px 20px;">

                                // ── Identity ──────────────────────────────────
                                <div class="detail-section-label" style="margin-top:16px;">"Identity"</div>
                                {acc.dba_name.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">"DBA Name"</div>
                                        <div class="detail-value">{v.clone()}</div>
                                    </div>
                                })}
                                {acc.year_established.map(|y| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">"Year Est."</div>
                                        <div class="detail-value">{y.to_string()}</div>
                                    </div>
                                })}

                                // ── Online ────────────────────────────────────
                                {if acc.website.is_some() || acc.domain.is_some() { Some(view! {
                                    <div class="detail-section-label" style="margin-top:14px;">"Online"</div>
                                    {acc.website.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Website"</div>
                                            <div class="detail-value mono" style="color:var(--cobalt)">{v.clone()}</div>
                                        </div>
                                    })}
                                    {acc.domain.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Domain"</div>
                                            <div class="detail-value mono">{v.clone()}</div>
                                        </div>
                                    })}
                                })} else { None }}

                                // ── Company Contact ───────────────────────────
                                {if acc.company_phone.is_some() || acc.company_email.is_some() { Some(view! {
                                    <div class="detail-section-label" style="margin-top:14px;">"Company Contact"</div>
                                    {acc.company_phone.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Phone"</div>
                                            <div class="detail-value mono">{v.clone()}</div>
                                        </div>
                                    })}
                                    {acc.company_email.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Email"</div>
                                            <div class="detail-value mono">{v.clone()}</div>
                                        </div>
                                    })}
                                })} else { None }}

                                // ── Firmographics ─────────────────────────────
                                {if acc.industry.is_some() || acc.company_type.is_some() || acc.num_employees.is_some() || acc.annual_revenue.is_some() { Some(view! {
                                    <div class="detail-section-label" style="margin-top:14px;">"Firmographics"</div>
                                    {acc.industry.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Industry"</div>
                                            <div class="detail-value">{v.clone()}</div>
                                        </div>
                                    })}
                                    {acc.company_type.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Company Type"</div>
                                            <div class="detail-value">{v.clone()}</div>
                                        </div>
                                    })}
                                    {acc.num_employees.map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Employees"</div>
                                            <div class="detail-value">{v.to_string()}</div>
                                        </div>
                                    })}
                                    {acc.annual_revenue.map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Annual Revenue"</div>
                                            <div class="detail-value mono">{format!("${:.0}", v)}</div>
                                        </div>
                                    })}
                                })} else { None }}

                                // ── Address ───────────────────────────────────
                                {if acc.city.is_some() || acc.street_address.is_some() { Some(view! {
                                    <div class="detail-section-label" style="margin-top:14px;">"Address"</div>
                                    {acc.street_address.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Street"</div>
                                            <div class="detail-value">{v.clone()}</div>
                                        </div>
                                    })}
                                    {(acc.city.is_some() || acc.state.is_some()).then(|| {
                                        let loc = match (&acc.city, &acc.state) {
                                            (Some(c), Some(s)) => format!("{}, {}", c, s),
                                            (Some(c), None)    => c.clone(),
                                            (None, Some(s))    => s.clone(),
                                            _                  => String::new(),
                                        };
                                        view! {
                                            <div class="detail-field">
                                                <div class="detail-label">"City / State"</div>
                                                <div class="detail-value">{loc}</div>
                                            </div>
                                        }
                                    })}
                                    {acc.country.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Country"</div>
                                            <div class="detail-value">{v.clone()}</div>
                                        </div>
                                    })}
                                })} else { None }}

                                // ── Data Source ───────────────────────────────
                                {acc.data_source.as_ref().map(|v| view! {
                                    <div>
                                        <div class="detail-section-label" style="margin-top:14px;">"Data Source"</div>
                                        <div class="detail-field">
                                            <div class="detail-label">"Source"</div>
                                            <div class="detail-value"><span class="tag tag-org">{v.clone()}</span></div>
                                        </div>
                                    </div>
                                })}

                                // ── Timestamps ────────────────────────────────
                                {acc.created_at.as_ref().map(|v| view! {
                                    <div>
                                        <div class="detail-section-label" style="margin-top:14px;">"Record"</div>
                                        <div class="detail-field">
                                            <div class="detail-label">"Added"</div>
                                            <div class="detail-value mono">{v[..10].to_string()}</div>
                                        </div>
                                    </div>
                                })}
                            </div>
                        </div>
                    </div>
                }.into_any()
            }}

            // ── Create Modal ──────────────────────────────────────────────────
            {move || show_create.get().then(|| view! {
                <div style="position:fixed;inset:0;background:rgba(0,0,0,0.5);z-index:200;display:flex;align-items:center;justify-content:center;"
                    on:click=move |_| show_create.set(false)>
                    <div style="background:var(--bg-surface);border:1px solid var(--border-default);border-radius:8px;padding:24px;width:360px;display:flex;flex-direction:column;gap:16px;"
                        on:click=|ev| ev.stop_propagation()>
                        <div style="font-size:15px;font-weight:600;">"New Account"</div>
                        <div class="n-form-row">
                            <label class="n-form-label">"Account Name"</label>
                            <input class="n-form-input"
                                placeholder="e.g. Nexus Property Group"
                                prop:value=move || new_name.get()
                                on:input=move |ev| new_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div style="display:flex;justify-content:flex-end;gap:8px;">
                            <button class="btn btn-ghost" on:click=move |_| show_create.set(false)>"Cancel"</button>
                            <button class="btn btn-primary"
                                disabled=create_busy
                                on:click=handle_create
                            >{move || if create_busy.get() { "Creating…" } else { "Create" }}</button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}
