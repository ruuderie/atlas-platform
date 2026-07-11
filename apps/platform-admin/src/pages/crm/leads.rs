use crate::api::crm::{create_lead, get_leads};
use crate::api::models::{CreateLead, LeadModel};
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiItem, KpiStrip},
    pagination::Pagination,
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
};
use leptos::prelude::*;

const PER_PAGE: u64 = 25;

fn lead_stage_tag(status: &str) -> &'static str {
    match status {
        "New" => "tag",
        "Contacted" => "tag tag-contacted",
        "Qualified" => "tag tag-proposal",
        "Proposal" => "tag tag-proposal",
        "Converted" => "tag tag-won",
        "Disqualified" => "tag tag-disqualified",
        _ => "tag",
    }
}

/// Returns an inline-style color for the stage dot/circle.
fn stage_dot_color(status: &str) -> &'static str {
    match status {
        "New" => "var(--cobalt)",
        "Contacted" => "var(--amber)",
        "Qualified" => "var(--cobalt)",
        "Proposal" => "var(--violet)",
        "Converted" => "var(--green)",
        "Disqualified" => "var(--red)",
        _ => "var(--text-muted)",
    }
}

/// Returns border+color style string for a source tag badge.
fn source_tag_style(source: &str) -> &'static str {
    match source.to_lowercase().as_str() {
        s if s.contains("fmcsa") => {
            "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)"
        }
        s if s.contains("biz") => {
            "color:var(--green);border-color:var(--green);background:var(--green-dim)"
        }
        s if s.contains("dot") => {
            "color:var(--amber);border-color:var(--amber);background:var(--amber-dim)"
        }
        s if s.contains("campaign") => {
            "color:var(--violet);border-color:var(--violet);background:var(--violet-dim)"
        }
        s if s.contains("referral") => {
            "color:var(--green);border-color:var(--green);background:var(--green-dim)"
        }
        _ => "color:var(--text-muted);border-color:var(--border-default)",
    }
}

fn fmt_date(s: &str) -> String {
    s.chars().take(10).collect()
}

/// Render a 5-step stage progress stepper for the lead drawer.
fn stage_stepper(current: String) -> impl IntoView {
    let stages = ["New", "Contacted", "Qualified", "Proposal", "Converted"];
    let current_idx = stages
        .iter()
        .position(|&s| s == current.as_str())
        .unwrap_or(0);
    view! {
        <div style="display:flex;align-items:center;padding:14px 20px 12px;border-bottom:1px solid var(--border-default);flex-shrink:0;gap:0;">
            {stages.iter().enumerate().map(|(i, &stage)| {
                let done   = i < current_idx;
                let active = i == current_idx;
                let circle_style = if done {
                    "width:18px;height:18px;border-radius:50%;background:var(--cobalt);border:1.5px solid var(--cobalt);display:flex;align-items:center;justify-content:center;position:relative;z-index:1;flex-shrink:0;".to_string()
                } else if active {
                    "width:18px;height:18px;border-radius:50%;background:var(--bg-base);border:2px solid var(--cobalt);box-shadow:0 0 0 3px rgba(10,132,255,0.15);display:flex;align-items:center;justify-content:center;position:relative;z-index:1;flex-shrink:0;".to_string()
                } else {
                    "width:18px;height:18px;border-radius:50%;background:var(--bg-base);border:1.5px solid var(--border-default);display:flex;align-items:center;justify-content:center;position:relative;z-index:1;flex-shrink:0;".to_string()
                };
                let label_color = if active { "var(--cobalt)" } else if done { "var(--text-secondary)" } else { "var(--text-muted)" };
                let label_weight = if active { "600" } else { "500" };
                let inner = if done {
                    view! { <span style="display:block;width:5px;height:8px;border:1.5px solid #fff;border-left:none;border-top:none;transform:rotate(45deg) translate(-1px,-1px);"></span> }.into_any()
                } else if active {
                    view! { <span style="display:block;width:6px;height:6px;background:var(--cobalt);border-radius:50%;"></span> }.into_any()
                } else {
                    view! { <></> }.into_any()
                };
                // connector line before node (except first)
                let connector = if i > 0 {
                    let line_color = if done || active { "var(--cobalt)" } else { "var(--border-default)" };
                    view! {
                        <div style=format!("flex:1;height:1px;background:{};margin-top:-1px;", line_color)></div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                };
                view! {
                    {connector}
                    <div style="display:flex;flex-direction:column;align-items:center;gap:4px;">
                        <div style=circle_style>{inner}</div>
                        <span style=format!("font-size:9px;font-weight:{};color:{};text-transform:uppercase;letter-spacing:0.06em;text-align:center;", label_weight, label_color)>
                            {stage}
                        </span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn LeadsPage() -> impl IntoView {
    let stage_filter = RwSignal::new("all".to_string());
    let search_filter = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page = RwSignal::new(1_u64);

    let selected = RwSignal::new(None::<LeadModel>);
    let drawer_open = RwSignal::new(false);

    // ── Create modal ──────────────────────────────────────────────────────────
    let show_create = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let new_email = RwSignal::new(String::new());
    let create_busy = RwSignal::new(false);
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    // 350 ms debounce
    Effect::new(move |_| {
        let val = search_filter.get();
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(350)).await;
            search_debounced.set(val);
            page.set(1);
        });
    });

    let leads_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let stage = stage_filter.get();
        let pg = page.get();
        async move {
            let s = if search.is_empty() {
                None
            } else {
                Some(search.as_str())
            };
            let st = Some(stage.as_str());
            get_leads(s, pg, PER_PAGE, st, None)
                .await
                .unwrap_or_default()
        }
    });

    let handle_create = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            toast.show_toast("Error", "Name is required.", "error");
            return;
        }
        let email = new_email.get();
        let email_opt = if email.trim().is_empty() {
            None
        } else {
            Some(email.trim().to_string())
        };
        create_busy.set(true);
        let resource = leads_res.clone();
        leptos::task::spawn_local(async move {
            let data = CreateLead {
                name: name.trim().to_string(),
                email: email_opt,
            };
            match create_lead(data).await {
                Ok(_) => {
                    toast.show_toast("Success", "Lead created.", "success");
                    show_create.set(false);
                    new_name.set(String::new());
                    new_email.set(String::new());
                    resource.refetch();
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
            create_busy.set(false);
        });
    };

    let kpi_items = Signal::derive(move || {
        let leads = leads_res.get().unwrap_or_default();
        let n = leads.len();
        let new_c = leads
            .iter()
            .filter(|l| l.lead_status.as_deref().unwrap_or("New") == "New")
            .count();
        let qual = leads
            .iter()
            .filter(|l| l.lead_status.as_deref() == Some("Qualified"))
            .count();
        let conv = leads.iter().filter(|l| l.is_converted).count();
        vec![
            KpiItem::new("Showing", &n.to_string()).sub("this page"),
            KpiItem::new("New", &new_c.to_string()).color("var(--cobalt)"),
            KpiItem::new("Qualified", &qual.to_string()),
            KpiItem::new("Converted", &conv.to_string()).color("var(--green)"),
        ]
    });

    let stage_pills = vec![
        PillOption::new("all", "All"),
        PillOption::new("New", "New"),
        PillOption::new("Contacted", "Contacted"),
        PillOption::new("Qualified", "Qualified"),
        PillOption::new("Proposal", "Proposal"),
        PillOption::new("Converted", "Converted"),
        PillOption::new("Disqualified", "Disqualified"),
    ];

    let page_count = Signal::derive(move || leads_res.get().unwrap_or_default().len());

    view! {
        <div class="entity-page">
            <div class="page-header" style="display:flex;align-items:flex-start;justify-content:space-between;padding:16px 20px;flex-shrink:0;gap:12px;">
                <div>
                    <h1 class="page-title">"Leads"</h1>
                    <p class="page-subtitle">"Platform-wide · All tenants"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm">"Export CSV"</button>
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        new_name.set(String::new());
                        new_email.set(String::new());
                        show_create.set(true);
                    }>
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Lead"
                    </button>
                </div>
            </div>

            <KpiStrip items=kpi_items />

            <FilterBar
                pills=stage_pills
                active=stage_filter
                search=search_filter
                search_placeholder="Search name, email, company…"
            />

            // ── Top pagination bar (compact) ──────────────────────────────────
            <div style="display:flex;align-items:center;justify-content:space-between;padding:4px 20px;border-bottom:1px solid var(--border-default);flex-shrink:0;font-size:11px;color:var(--text-muted);background:var(--bg-surface);">
                <span>
                    "Page " {move || page.get().to_string()}
                    " · " {move || page_count.get().to_string()}
                    " records"
                </span>
                <div style="display:flex;gap:6px;">
                    <button
                        class="btn btn-ghost btn-sm"
                        disabled=move || page.get() <= 1
                        on:click=move |_| { if page.get() > 1 { page.update(|p| *p -= 1); } }
                    >"← Prev"</button>
                    <button
                        class="btn btn-ghost btn-sm"
                        disabled=move || (page_count.get() as u64) < PER_PAGE
                        on:click=move |_| { if (page_count.get() as u64) >= PER_PAGE { page.update(|p| *p += 1); } }
                    >"Next →"</button>
                </div>
            </div>

            <div class="table-container">
                <Suspense fallback=move || view! {
                    <div style="padding:32px;text-align:center;color:var(--text-muted)">
                        "Loading leads…"
                    </div>
                }>
                    {move || {
                        let rows = leads_res.get().unwrap_or_default();
                        if rows.is_empty() {
                            return view! {
                                <div class="empty-state">
                                    <div class="empty-state-icon">"◎"</div>
                                    <div class="empty-state-title">"No leads found"</div>
                                    <div class="empty-state-body">"Try adjusting your filters or search query."</div>
                                </div>
                            }.into_any();
                        }
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:28%" class="sortable">"Lead"</th>
                                        <th style="width:22%" class="sortable">"Email / Phone"</th>
                                        <th style="width:12%" class="sortable">"Source"</th>
                                        <th style="width:14%" class="sortable">"Stage"</th>
                                        <th style="width:12%" class="sortable">"Last Activity"</th>
                                        <th style="width:70px"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|l| {
                                        let ini        = initials(&l.name);
                                        let email      = l.email.clone().unwrap_or_else(|| "—".to_string());
                                        let phone      = l.phone.clone().unwrap_or_else(|| "—".to_string());
                                        let source     = l.source.clone().unwrap_or_else(|| "—".to_string());
                                        let status     = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
                                        // Use updated_at as proxy for last activity
                                        let last_act   = l.updated_at.as_deref().map(fmt_date)
                                            .or_else(|| l.created_at.as_deref().map(fmt_date))
                                            .unwrap_or_else(|| "—".to_string());
                                        let dot_color  = stage_dot_color(&status);
                                        let src_style  = source_tag_style(&source);
                                        let detail_href = format!("/leads/{}", l.id);
                                        let l_click    = l.clone();

                                        view! {
                                            <tr style="cursor:pointer" on:click=move |_| {
                                                selected.set(Some(l_click.clone()));
                                                drawer_open.set(true);
                                            }>
                                                <td>
                                                    <RecordRow
                                                        initials=ini
                                                        name=l.name.clone()
                                                        sub=l.company.clone().unwrap_or_default()
                                                    />
                                                </td>
                                                <td>
                                                    <div style="display:flex;flex-direction:column;gap:2px;">
                                                        <span style="font-size:12px;color:var(--text-secondary)">{email}</span>
                                                        <span style="font-size:11px;color:var(--text-muted)">{phone}</span>
                                                    </div>
                                                </td>
                                                <td>
                                                    // Colored source tag
                                                    <span class="tag" style=src_style>{source}</span>
                                                </td>
                                                <td>
                                                    // Stage cell: colored dot + label
                                                    <div style="display:flex;align-items:center;gap:6px;">
                                                        <span style=format!("width:6px;height:6px;border-radius:50%;flex-shrink:0;background:{}", dot_color)></span>
                                                        <span style="font-size:12px;color:var(--text-secondary)">{status}</span>
                                                    </div>
                                                </td>
                                                <td class="muted">{last_act}</td>
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

            // Bottom pagination — sticky at bottom of entity-page
            <div style="position:sticky;bottom:0;z-index:2;">
                <Pagination page=page per_page=PER_PAGE count=page_count />
            </div>
        </div>

        // Drawer rendered outside the scroll container — does not re-mount on data refresh
        {move || selected.get().map(|l| {
            let name_c    = l.name.clone();
            let company_c = l.company.clone().unwrap_or_default();
            let email_c   = l.email.clone().unwrap_or_else(|| company_c.clone());
            let id_c      = l.id.clone();
            let title    = Signal::derive(move || name_c.clone());
            let subtitle = Signal::derive(move || email_c.clone());
            let href     = Signal::derive(move || format!("/leads/{}", id_c));
            let status   = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
            let source   = l.source.clone().unwrap_or_else(|| "—".to_string());
            let company  = if company_c.is_empty() { "—".to_string() } else { company_c };
            let title_str = l.title.clone().unwrap_or_else(|| "—".to_string());
            let phone    = l.phone.clone().unwrap_or_else(|| "—".to_string());
            let whatsapp = l.whatsapp.clone().unwrap_or_else(|| "—".to_string());
            let telegram = l.telegram.clone().unwrap_or_else(|| "—".to_string());
            let created  = l.created_at.as_deref().map(fmt_date).unwrap_or_else(|| "—".to_string());
            let src_style = source_tag_style(&source);
            let status_for_stepper = status.clone();

            view! {
                <RecordDrawer open=drawer_open title=title subtitle=subtitle detail_href=href>
                    // Stage progress stepper
                    {stage_stepper(status_for_stepper)}

                    <div class="detail-grid">
                        <span class="detail-section-label">"Lead Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Stage"</div>
                            <div class="detail-value">
                                <div style="display:flex;align-items:center;gap:6px;">
                                    <span style=format!("width:6px;height:6px;border-radius:50%;flex-shrink:0;background:{}", stage_dot_color(&status))></span>
                                    <span style="font-size:12px;">{status}</span>
                                </div>
                            </div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Source"</div>
                            <div class="detail-value">
                                <span class="tag" style=src_style>{source}</span>
                            </div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Company"</div>
                            <div class="detail-value">{company}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Job Title"</div>
                            <div class="detail-value">{title_str}</div>
                        </div>
                        <span class="detail-section-label">"Contact Channels"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Phone"</div>
                            <div class="detail-value mono">{phone}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"WhatsApp"</div>
                            <div class="detail-value mono">{whatsapp}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Telegram"</div>
                            <div class="detail-value mono">{telegram}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Added"
                            </div>
                            <div class="detail-value mono">{created}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}

        // ── Create Lead Modal ───────────────────────────────────────────────
        <Show when=move || show_create.get()>
            <div
                style="position:fixed;inset:0;z-index:200;background:rgba(0,0,0,0.7);backdrop-filter:blur(4px);display:flex;align-items:center;justify-content:center;padding:20px;"
                on:click=move |_| show_create.set(false)
            >
                <div
                    style="background:var(--bg-surface);border:1px solid var(--border-default);border-radius:14px;width:420px;max-width:100%;padding:28px;position:relative;"
                    on:click=move |e| e.stop_propagation()
                >
                    <button
                        style="position:absolute;top:14px;right:16px;background:none;border:none;color:var(--text-muted);font-size:16px;cursor:pointer;padding:4px 8px;border-radius:6px;"
                        on:click=move |_| show_create.set(false)
                    >"✕"</button>

                    <div style="font-size:16px;font-weight:700;margin-bottom:4px;color:var(--text-primary)">"New Lead"</div>
                    <div style="font-size:12px;color:var(--text-muted);margin-bottom:20px">"Create a new lead in the CRM. Fields marked * are required."</div>

                    <div style="display:flex;flex-direction:column;gap:14px;">
                        <div>
                            <label style="display:block;font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:5px;">"Full Name *"</label>
                            <input
                                type="text"
                                placeholder="e.g. Jane Doe"
                                style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:8px;padding:9px 12px;font-size:13px;color:var(--text-primary);outline:none;box-sizing:border-box;"
                                prop:value=move || new_name.get()
                                on:input=move |e| new_name.set(event_target_value(&e))
                            />
                        </div>
                        <div>
                            <label style="display:block;font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:5px;">"Email"</label>
                            <input
                                type="email"
                                placeholder="jane@example.com"
                                style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:8px;padding:9px 12px;font-size:13px;color:var(--text-primary);outline:none;box-sizing:border-box;"
                                prop:value=move || new_email.get()
                                on:input=move |e| new_email.set(event_target_value(&e))
                            />
                        </div>
                    </div>

                    <div style="display:flex;gap:8px;justify-content:flex-end;margin-top:22px;">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| show_create.set(false)>"Cancel"</button>
                        <button
                            class="btn btn-primary btn-sm"
                            disabled=move || create_busy.get()
                            on:click=handle_create
                        >
                            {move || if create_busy.get() { "Saving…" } else { "Create Lead" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
