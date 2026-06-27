use leptos::prelude::*;
use crate::api::crm::{get_contacts, create_contact};
use crate::api::models::{ContactModel, CreateContact};
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
    pagination::Pagination,
};

const PER_PAGE: u64 = 25;

fn fmt_date(s: &str) -> String { s.chars().take(10).collect() }

#[component]
pub fn ContactsPage() -> impl IntoView {
    let search_filter    = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page             = RwSignal::new(1_u64);

    let selected    = RwSignal::new(None::<ContactModel>);
    let drawer_open = RwSignal::new(false);

    // ── Create modal ──────────────────────────────────────────────────────────
    let show_create = RwSignal::new(false);
    let new_name    = RwSignal::new(String::new());
    let new_email   = RwSignal::new(String::new());
    let new_phone   = RwSignal::new(String::new());
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

    let contacts_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let pg     = page.get();
        async move {
            let s = if search.is_empty() { None } else { Some(search.as_str()) };
            get_contacts(s, pg, PER_PAGE).await.unwrap_or_default()
        }
    });

    let handle_create = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            toast.show_toast("Error", "Name is required.", "error");
            return;
        }
        let email = new_email.get();
        let phone = new_phone.get();
        create_busy.set(true);
        let resource = contacts_res.clone();
        leptos::task::spawn_local(async move {
            let data = CreateContact {
                name: name.trim().to_string(),
                email: if email.trim().is_empty() { None } else { Some(email.trim().to_string()) },
                phone: if phone.trim().is_empty() { None } else { Some(phone.trim().to_string()) },
                whatsapp: None, telegram: None, twitter: None,
                instagram: None, facebook: None, properties: None,
            };
            match create_contact(data).await {
                Ok(_) => {
                    toast.show_toast("Success", "Contact created.", "success");
                    show_create.set(false);
                    new_name.set(String::new());
                    new_email.set(String::new());
                    new_phone.set(String::new());
                    resource.refetch();
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
            create_busy.set(false);
        });
    };

    let kpi_items = Signal::derive(move || {
        let contacts   = contacts_res.get().unwrap_or_default();
        let n          = contacts.len();
        let has_email  = contacts.iter().filter(|c| c.email.is_some()).count();
        let missing    = contacts.iter().filter(|c| c.email.is_none() && c.phone.is_none()).count();
        vec![
            KpiItem::new("Page",         &page.get().to_string()),
            KpiItem::new("Has Email",    &has_email.to_string()).color("var(--cobalt)"),
            KpiItem::new("Missing Info", &missing.to_string()).color(if missing > 0 { "var(--amber)" } else { "var(--green)" }),
            KpiItem::new("Total",        &n.to_string()),
        ]
    });

    let filter_pills = vec![PillOption::new("all", "All Contacts")];
    let active_filter = RwSignal::new("all".to_string());
    let page_count    = Signal::derive(move || contacts_res.get().unwrap_or_default().len());

    view! {
        <div class="entity-page">
            <div class="page-header" style="display:flex;align-items:flex-start;justify-content:space-between;padding:16px 20px;flex-shrink:0;gap:12px;">
                <div>
                    <h1 class="page-title">"Contacts"</h1>
                    <p class="page-subtitle">"Platform-wide · All tenants"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm">"Export CSV"</button>
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        new_name.set(String::new());
                        new_email.set(String::new());
                        new_phone.set(String::new());
                        show_create.set(true);
                    }>
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Contact"
                    </button>
                </div>
            </div>

            <KpiStrip items=kpi_items />

            <FilterBar
                pills=filter_pills
                active=active_filter
                search=search_filter
                search_placeholder="Search name, email, phone…"
            />

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
                                    <div class="empty-state-body">"Try adjusting your search query."</div>
                                </div>
                            }.into_any();
                        }
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:32%" class="sortable">"Contact"</th>
                                        <th style="width:28%" class="sortable">"Email"</th>
                                        <th style="width:16%" class="sortable">"Phone"</th>
                                        <th style="width:12%" class="sortable">"Added"</th>
                                        <th style="width:70px"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|c| {
                                        let ini       = initials(&c.name);
                                        let email     = c.email.clone().unwrap_or_else(|| "—".to_string());
                                        let phone     = c.phone.clone().unwrap_or_else(|| "—".to_string());
                                        let job_title = c.properties.as_ref()
                                            .and_then(|p| p.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
                                            .unwrap_or_default();
                                        let created   = fmt_date(&c.created_at);
                                        let detail_href = format!("/contacts/{}", c.id);
                                        let c_click   = c.clone();

                                        view! {
                                            <tr style="cursor:pointer" on:click=move |_| {
                                                selected.set(Some(c_click.clone()));
                                                drawer_open.set(true);
                                            }>
                                                <td>
                                                    <RecordRow
                                                        initials=ini
                                                        name=c.name.clone()
                                                        sub=job_title
                                                        bg="var(--violet-dim)"
                                                        color="var(--violet)"
                                                    />
                                                </td>
                                                <td class="mono">{email}</td>
                                                <td class="muted">{phone}</td>
                                                <td class="muted">{created}</td>
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

        {move || selected.get().map(|c| {
            let name_c  = c.name.clone();
            let email_c = c.email.clone().unwrap_or_default();
            let id_c    = c.id.clone();
            let title    = Signal::derive(move || name_c.clone());
            let subtitle = Signal::derive(move || email_c.clone());
            let href     = Signal::derive(move || format!("/contacts/{}", id_c));
            let phone    = c.phone.clone().unwrap_or_else(|| "—".to_string());
            let whatsapp = c.whatsapp.clone().unwrap_or_else(|| "—".to_string());
            let telegram = c.telegram.clone().unwrap_or_else(|| "—".to_string());
            let email    = c.email.clone().unwrap_or_else(|| "—".to_string());
            let created  = fmt_date(&c.created_at);

            view! {
                <RecordDrawer open=drawer_open title=title subtitle=subtitle detail_href=href>
                    <div class="detail-grid">
                        <span class="detail-section-label">"Contact Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Email"</div>
                            <div class="detail-value mono">{email}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Phone"</div>
                            <div class="detail-value">{phone}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"WhatsApp"</div>
                            <div class="detail-value">{whatsapp}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Telegram"</div>
                            <div class="detail-value">{telegram}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Added"</div>
                            <div class="detail-value mono">{created}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}

        // ── Create Contact Modal ───────────────────────────────────────────
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
                    <div style="font-size:16px;font-weight:700;margin-bottom:4px;color:var(--text-primary)">"New Contact"</div>
                    <div style="font-size:12px;color:var(--text-muted);margin-bottom:20px">"Add a person to your CRM contact list. Name is required."</div>
                    <div style="display:flex;flex-direction:column;gap:14px;">
                        <div>
                            <label style="display:block;font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:5px;">"Full Name *"</label>
                            <input type="text" placeholder="e.g. João Silva"
                                style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:8px;padding:9px 12px;font-size:13px;color:var(--text-primary);outline:none;box-sizing:border-box;"
                                prop:value=move || new_name.get()
                                on:input=move |e| new_name.set(event_target_value(&e))
                            />
                        </div>
                        <div>
                            <label style="display:block;font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:5px;">"Email"</label>
                            <input type="email" placeholder="email@example.com"
                                style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:8px;padding:9px 12px;font-size:13px;color:var(--text-primary);outline:none;box-sizing:border-box;"
                                prop:value=move || new_email.get()
                                on:input=move |e| new_email.set(event_target_value(&e))
                            />
                        </div>
                        <div>
                            <label style="display:block;font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:5px;">"Phone"</label>
                            <input type="tel" placeholder="+1 555 000 0000"
                                style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:8px;padding:9px 12px;font-size:13px;color:var(--text-primary);outline:none;box-sizing:border-box;"
                                prop:value=move || new_phone.get()
                                on:input=move |e| new_phone.set(event_target_value(&e))
                            />
                        </div>
                    </div>
                    <div style="display:flex;gap:8px;justify-content:flex-end;margin-top:22px;">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| show_create.set(false)>"Cancel"</button>
                        <button class="btn btn-primary btn-sm" disabled=move || create_busy.get() on:click=handle_create>
                            {move || if create_busy.get() { "Saving…" } else { "Create Contact" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
