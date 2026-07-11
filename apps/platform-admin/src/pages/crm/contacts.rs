use crate::api::crm::{create_contact, get_contacts};
use crate::api::models::{ContactModel, CreateContact};
use crate::pages::crm::components::{
    kpi_strip::{KpiItem, KpiStrip},
    pagination::Pagination,
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
};
use leptos::prelude::*;

const PER_PAGE: u64 = 25;

fn fmt_date(s: &str) -> String {
    s.chars().take(10).collect()
}

#[component]
pub fn ContactsPage() -> impl IntoView {
    let search_filter = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page = RwSignal::new(1_u64);
    let primary_filter = RwSignal::new(false);

    let selected = RwSignal::new(None::<ContactModel>);
    let drawer_open = RwSignal::new(false);

    // ── Create modal signals ──────────────────────────────────────────────────
    let show_create = RwSignal::new(false);
    let new_first = RwSignal::new(String::new());
    let new_last = RwSignal::new(String::new());
    let new_email = RwSignal::new(String::new());
    let new_phone = RwSignal::new(String::new());
    let new_title = RwSignal::new(String::new());
    let create_busy = RwSignal::new(false);
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    // Debounce search
    Effect::new(move |_| {
        let val = search_filter.get();
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(350)).await;
            search_debounced.set(val);
            page.set(1);
        });
    });

    let contacts_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let pg = page.get();
        let role = if primary_filter.get() {
            Some("primary")
        } else {
            None
        };
        async move {
            let s = if search.is_empty() {
                None
            } else {
                Some(search.as_str())
            };
            get_contacts(s, pg, PER_PAGE, role)
                .await
                .unwrap_or_default()
        }
    });

    let handle_create = move |_| {
        let first = new_first.get();
        let last = new_last.get();
        if first.trim().is_empty() && last.trim().is_empty() {
            toast.show_toast("Error", "At least first or last name is required.", "error");
            return;
        }
        create_busy.set(true);
        let resource = contacts_res.clone();
        leptos::task::spawn_local(async move {
            let data = CreateContact {
                first_name: if first.trim().is_empty() {
                    None
                } else {
                    Some(first.trim().to_string())
                },
                last_name: if last.trim().is_empty() {
                    None
                } else {
                    Some(last.trim().to_string())
                },
                full_name: None,
                email: if new_email.get().trim().is_empty() {
                    None
                } else {
                    Some(new_email.get().trim().to_string())
                },
                phone: if new_phone.get().trim().is_empty() {
                    None
                } else {
                    Some(new_phone.get().trim().to_string())
                },
                title: if new_title.get().trim().is_empty() {
                    None
                } else {
                    Some(new_title.get().trim().to_string())
                },
                department: None,
                whatsapp: None,
                telegram: None,
                linkedin_url: None,
                account_id: None,
            };
            match create_contact(data).await {
                Ok(_) => {
                    toast.show_toast("Success", "Contact created.", "success");
                    show_create.set(false);
                    new_first.set(String::new());
                    new_last.set(String::new());
                    new_email.set(String::new());
                    new_phone.set(String::new());
                    new_title.set(String::new());
                    resource.refetch();
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
            create_busy.set(false);
        });
    };

    // ── KPI strip ─────────────────────────────────────────────────────────────
    let kpi_items = Signal::derive(move || {
        let contacts = contacts_res.get().unwrap_or_default();
        let total = contacts.len();
        let verified_email = contacts.iter().filter(|c| c.email_verified).count();
        let has_whatsapp = contacts.iter().filter(|c| c.whatsapp.is_some()).count();
        let primary_count = contacts.iter().filter(|c| c.is_primary).count();
        vec![
            KpiItem::new("Total", &total.to_string()),
            KpiItem::new("Email Verified", &verified_email.to_string()).color("var(--green)"),
            KpiItem::new("WhatsApp", &has_whatsapp.to_string()).color("var(--cobalt)"),
            KpiItem::new("Primary", &primary_count.to_string()).color("var(--amber)"),
        ]
    });

    let page_count = Signal::derive(move || contacts_res.get().unwrap_or_default().len());

    view! {
        <div class="entity-page">
            // ── Header ────────────────────────────────────────────────────────
            <div class="page-header" style="display:flex;align-items:flex-start;justify-content:space-between;padding:16px 20px;flex-shrink:0;gap:12px;">
                <div>
                    <h1 class="page-title">"Contacts"</h1>
                    <p class="page-subtitle">"Platform-wide · Individual people linked to Accounts"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm">"Export CSV"</button>
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        new_first.set(String::new());
                        new_last.set(String::new());
                        new_email.set(String::new());
                        new_phone.set(String::new());
                        new_title.set(String::new());
                        show_create.set(true);
                    }>
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Contact"
                    </button>
                </div>
            </div>

            // ── KPI Strip ─────────────────────────────────────────────────────
            <KpiStrip items=kpi_items />

            // ── Filter Bar ────────────────────────────────────────────────────
            <div style="display:flex;align-items:center;gap:6px;padding:10px 20px;border-bottom:1px solid var(--border-default);flex-shrink:0;flex-wrap:wrap;">
                <button
                    class=move || if !primary_filter.get() { "pill active" } else { "pill" }
                    on:click=move |_| { primary_filter.set(false); page.set(1); }
                >"All Contacts"</button>
                <button
                    class=move || if primary_filter.get() { "pill active" } else { "pill" }
                    on:click=move |_| { primary_filter.set(true); page.set(1); }
                >"Primary Only"</button>
                <div style="margin-left:auto;">
                    <input
                        type="text"
                        class="filter-input"
                        placeholder="Search name, email, phone, title…"
                        on:input=move |ev| search_filter.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // ── Table ─────────────────────────────────────────────────────────
            <div class="table-container">
                <Suspense fallback=move || view! {
                    <div style="padding:32px;text-align:center;color:var(--text-muted)">"Loading contacts…"</div>
                }>
                    {move || {
                        let rows = contacts_res.get().unwrap_or_default();
                        if rows.is_empty() {
                            return view! {
                                <div class="empty-state">
                                    <div class="empty-state-icon">"◎"</div>
                                    <div class="empty-state-title">"No contacts found"</div>
                                    <div class="empty-state-body">"Try adjusting your search or filter."</div>
                                </div>
                            }.into_any();
                        }
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:28%" class="sortable">"Contact"</th>
                                        <th style="width:10%">"Primary"</th>
                                        <th style="width:24%" class="sortable">"Email"</th>
                                        <th style="width:14%">"Channels"</th>
                                        <th style="width:14%" class="sortable">"Title / Dept"</th>
                                        <th style="width:70px"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                {rows.into_iter().map(|c| {
                                    let display  = c.display_name().to_string();
                                    let ini      = initials(&display);
                                    let sub      = c.title.clone().unwrap_or_else(|| {
                                        c.department.clone().unwrap_or_default()
                                    });
                                    let email    = c.email.clone().unwrap_or_else(|| "—".to_string());
                                    let email_ok = c.email_verified;

                                    // WhatsApp / Telegram channel dots
                                    let has_wa   = c.whatsapp.is_some();
                                    let has_tg   = c.telegram.is_some();
                                    let has_li   = c.linkedin_url.is_some();

                                    let is_primary = c.is_primary;
                                    let c_click  = c.clone();

                                    view! {
                                        <tr style="cursor:pointer;" on:click=move |_| {
                                            selected.set(Some(c_click.clone()));
                                            drawer_open.set(true);
                                        }>
                                            <td>
                                                <RecordRow
                                                    initials=ini
                                                    name=display
                                                    sub=sub
                                                    bg="var(--violet-dim)"
                                                    color="var(--violet)"
                                                />
                                            </td>
                                            <td>
                                                {if is_primary {
                                                    view! { <span class="tag" style="color:var(--amber);border-color:var(--amber);background:var(--amber-dim);">"Primary"</span> }.into_any()
                                                } else {
                                                    view! { <span class="muted" style="font-size:11px;">"—"</span> }.into_any()
                                                }}
                                            </td>
                                            <td>
                                                <div style="display:flex;align-items:center;gap:5px;">
                                                    <span class="mono" style="font-size:11.5px;">{email.clone()}</span>
                                                    {if email_ok && email != "—" {
                                                        view! { <span title="Email verified" style="color:var(--green);font-size:9px;">{"✓"}</span> }.into_any()
                                                    } else { view! { <span></span> }.into_any() }}
                                                </div>
                                            </td>
                                            <td>
                                                <div style="display:flex;align-items:center;gap:5px;">
                                                    {if has_wa { view! { <span title="WhatsApp" style="font-size:10px;background:rgba(37,211,102,0.12);color:#25D366;border:1px solid #25D366;border-radius:3px;padding:1px 5px;">{"WA"}</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                                    {if has_tg { view! { <span title="Telegram" style="font-size:10px;background:rgba(0,136,204,0.12);color:#0088CC;border:1px solid #0088CC;border-radius:3px;padding:1px 5px;">{"TG"}</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                                    {if has_li { view! { <span title="LinkedIn" style="font-size:10px;background:rgba(10,102,194,0.12);color:#0A66C2;border:1px solid #0A66C2;border-radius:3px;padding:1px 5px;">{"LI"}</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                                </div>
                                            </td>
                                            <td class="muted" style="font-size:11.5px;">{
                                                match (&c.title, &c.department) {
                                                    (Some(t), Some(d)) => format!("{} · {}", t, d),
                                                    (Some(t), None)    => t.clone(),
                                                    (None, Some(d))    => d.clone(),
                                                    _                  => "—".to_string(),
                                                }
                                            }</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm"
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        selected.set(Some(c.clone()));
                                                        drawer_open.set(true);
                                                    }
                                                >"Open"</button>
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
            <Pagination page=page per_page=PER_PAGE count=page_count />

            // ── Detail Drawer ─────────────────────────────────────────────────
            {move || {
                if !drawer_open.get() { return view! { <div></div> }.into_any(); }
                let Some(c) = selected.get() else { return view! { <div></div> }.into_any(); };

                let display = c.display_name().to_string();
                let ini     = initials(&display);
                let subtitle = {
                    let mut parts = Vec::new();
                    if let Some(ref t) = c.title      { parts.push(t.clone()); }
                    if let Some(ref d) = c.department { parts.push(d.clone()); }
                    parts.join(" · ")
                };

                view! {
                    <div>
                        <div
                            style="position:fixed;inset:0;background:rgba(0,0,0,0.4);backdrop-filter:blur(2px);z-index:100;"
                            on:click=move |_| drawer_open.set(false)
                        ></div>
                        <div class="record-drawer open">
                            <div class="panel-header">
                                <div class="panel-header-top">
                                    <div class="panel-identity">
                                        <div class="crm-avatar crm-avatar-ind" style="width:40px;height:40px;font-size:14px;margin-bottom:8px;">{ini}</div>
                                        <div class="panel-title-text">{display}</div>
                                        <div class="panel-subtitle-text" style="color:var(--text-muted);font-size:11.5px;">{subtitle}</div>
                                    </div>
                                    <button class="panel-close" on:click=move |_| drawer_open.set(false)>
                                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/></svg>
                                    </button>
                                </div>
                            </div>
                            <div class="panel-content" style="overflow-y:auto;padding:0 20px 20px;">
                                // ── Professional ──────────────────────────────
                                {if c.title.is_some() || c.department.is_some() || c.is_primary { Some(view! {
                                    <div class="detail-section-label" style="margin-top:16px;">"Professional"</div>
                                    {c.title.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Title"</div>
                                            <div class="detail-value">{v.clone()}</div>
                                        </div>
                                    })}
                                    {c.department.as_ref().map(|v| view! {
                                        <div class="detail-field">
                                            <div class="detail-label">"Department"</div>
                                            <div class="detail-value">{v.clone()}</div>
                                        </div>
                                    })}
                                    <div class="detail-field">
                                        <div class="detail-label">"Primary Contact"</div>
                                        <div class="detail-value">{if c.is_primary { "Yes" } else { "No" }}</div>
                                    </div>
                                })} else { None }}

                                // ── Contact Channels ──────────────────────────
                                <div class="detail-section-label" style="margin-top:14px;">"Contact Channels"</div>
                                {c.email.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">
                                            "Email"
                                            {if c.email_verified { view! { <span style="color:var(--green);font-size:9px;margin-left:4px;">{"✓"}</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                        </div>
                                        <div class="detail-value mono">{v.clone()}</div>
                                    </div>
                                })}
                                {c.phone.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">
                                            "Phone"
                                            {if c.phone_verified { view! { <span style="color:var(--green);font-size:9px;margin-left:4px;">{"✓"}</span> }.into_any() } else { view! { <span></span> }.into_any() }}
                                        </div>
                                        <div class="detail-value mono">{v.clone()}</div>
                                    </div>
                                })}
                                {c.whatsapp.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">"WhatsApp"</div>
                                        <div class="detail-value mono">{v.clone()}</div>
                                    </div>
                                })}
                                {c.telegram.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">"Telegram"</div>
                                        <div class="detail-value">{v.clone()}</div>
                                    </div>
                                })}
                                {c.linkedin_url.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">"LinkedIn"</div>
                                        <div class="detail-value mono" style="color:var(--cobalt);">{v.clone()}</div>
                                    </div>
                                })}
                                {c.twitter.as_ref().map(|v| view! {
                                    <div class="detail-field">
                                        <div class="detail-label">"Twitter / X"</div>
                                        <div class="detail-value">{v.clone()}</div>
                                    </div>
                                })}

                                // ── Data Source ───────────────────────────────
                                {c.data_source.as_ref().map(|v| view! {
                                    <div>
                                        <div class="detail-section-label" style="margin-top:14px;">"Data Source"</div>
                                        <div class="detail-field">
                                            <div class="detail-label">"Source"</div>
                                            <div class="detail-value"><span class="tag tag-org">{v.clone()}</span></div>
                                        </div>
                                    </div>
                                })}

                                // ── Timestamps ────────────────────────────────
                                {c.created_at.as_ref().map(|v| view! {
                                    <div>
                                        <div class="detail-section-label" style="margin-top:14px;">"Record"</div>
                                        <div class="detail-field">
                                            <div class="detail-label">"Added"</div>
                                            <div class="detail-value mono">{fmt_date(v)}</div>
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
                <div
                    style="position:fixed;inset:0;z-index:200;background:rgba(0,0,0,0.7);backdrop-filter:blur(4px);display:flex;align-items:center;justify-content:center;padding:20px;"
                    on:click=move |_| show_create.set(false)
                >
                    <div
                        style="background:var(--bg-surface);border:1px solid var(--border-default);border-radius:14px;width:440px;max-width:100%;padding:28px;position:relative;"
                        on:click=|ev| ev.stop_propagation()
                    >
                        <button
                            style="position:absolute;top:14px;right:16px;background:none;border:none;color:var(--text-muted);font-size:16px;cursor:pointer;"
                            on:click=move |_| show_create.set(false)
                        >"✕"</button>
                        <div style="font-size:16px;font-weight:700;margin-bottom:4px;">"New Contact"</div>
                        <div style="font-size:12px;color:var(--text-muted);margin-bottom:20px;">"First or last name is required."</div>
                        <div style="display:flex;flex-direction:column;gap:12px;">
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                                <div class="n-form-row">
                                    <label class="n-form-label">"First Name *"</label>
                                    <input class="n-form-input" placeholder="João"
                                        prop:value=move || new_first.get()
                                        on:input=move |e| new_first.set(event_target_value(&e))/>
                                </div>
                                <div class="n-form-row">
                                    <label class="n-form-label">"Last Name"</label>
                                    <input class="n-form-input" placeholder="Silva"
                                        prop:value=move || new_last.get()
                                        on:input=move |e| new_last.set(event_target_value(&e))/>
                                </div>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Title"</label>
                                <input class="n-form-input" placeholder="e.g. VP Operations"
                                    prop:value=move || new_title.get()
                                    on:input=move |e| new_title.set(event_target_value(&e))/>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Email"</label>
                                <input class="n-form-input" type="email" placeholder="email@example.com"
                                    prop:value=move || new_email.get()
                                    on:input=move |e| new_email.set(event_target_value(&e))/>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Phone"</label>
                                <input class="n-form-input" type="tel" placeholder="+1 555 000 0000"
                                    prop:value=move || new_phone.get()
                                    on:input=move |e| new_phone.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div style="display:flex;gap:8px;justify-content:flex-end;margin-top:22px;">
                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_create.set(false)>"Cancel"</button>
                            <button class="btn btn-primary btn-sm"
                                disabled=create_busy
                                on:click=handle_create
                            >{move || if create_busy.get() { "Saving…" } else { "Create Contact" }}</button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}
