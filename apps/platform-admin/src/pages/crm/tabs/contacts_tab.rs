use leptos::prelude::*;
use crate::api::crm::get_contacts;
use crate::api::models::ContactModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
    pagination::Pagination,
};

const PER_PAGE: u64 = 25;

fn fmt_date(s: &str) -> String {
    s.chars().take(10).collect()
}

#[component]
pub fn ContactsTab() -> impl IntoView {
    let search_filter    = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page             = RwSignal::new(1_u64);

    let selected    = RwSignal::new(None::<ContactModel>);
    let drawer_open = RwSignal::new(false);

    // 400ms debounce
    Effect::new(move |_| {
        let val = search_filter.get();
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(400)).await;
            search_debounced.set(val);
            page.set(1);
        });
    });

    let contacts_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let pg     = page.get();
        async move {
            let s = if search.is_empty() { None } else { Some(search.as_str()) };
            get_contacts(s, pg, PER_PAGE, None).await.unwrap_or_default()
        }
    });

    let kpi_items = Signal::derive(move || {
        let contacts = contacts_res.get().unwrap_or_default();
        let total    = contacts.len();
        let has_email = contacts.iter().filter(|c| c.email.is_some()).count();
        let has_phone = contacts.iter().filter(|c| c.phone.is_some()).count();
        vec![
            KpiItem::new("This Page", &total.to_string()).sub("Platform-wide"),
            KpiItem::new("Has Email", &has_email.to_string()).color("var(--cobalt)"),
            KpiItem::new("Has Phone", &has_phone.to_string()),
            KpiItem::new("Converted Leads", "—").color("var(--green)"),
        ]
    });

    // Contacts has no server-side status filter — keep pill group minimal
    let filter_pills = vec![
        PillOption::new("all", "All Contacts"),
    ];
    let active_filter = RwSignal::new("all".to_string());

    let page_count = Signal::derive(move || contacts_res.get().unwrap_or_default().len());

    view! {
        <KpiStrip items=kpi_items />

        <FilterBar
            pills=filter_pills
            active=active_filter
            search=search_filter
            search_placeholder="Search name, email, phone…"
        />

        <div class="table-container">
            <Suspense fallback=move || view! {
                <div style="padding:32px;text-align:center;color:var(--text-muted)">
                    "Loading contacts..."
                </div>
            }>
                {move || {
                    let rows = contacts_res.get().unwrap_or_default();
                    if rows.is_empty() {
                        return view! {
                            <div style="padding:40px;text-align:center;color:var(--text-muted);font-size:13px;">
                                "No contacts found."
                            </div>
                        }.into_any();
                    }
                    view! {
                        <table>
                            <thead>
                                <tr>
                                    <th style="width:32px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                    <th style="width:32%" class="sortable">"Contact"</th>
                                    <th style="width:28%" class="sortable">"Email"</th>
                                    <th style="width:16%" class="sortable">"Phone"</th>
                                    <th style="width:12%" class="sortable">"Added"</th>
                                    <th style="width:70px"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {rows.into_iter().map(|c| {
                                    let display     = c.display_name().to_string();
                                    let ini         = initials(&display);
                                    let email       = c.email.clone().unwrap_or_else(|| "—".to_string());
                                    let phone       = c.phone.clone().unwrap_or_else(|| "—".to_string());
                                    let job_title   = c.title.clone()
                                        .or_else(|| c.department.clone())
                                        .unwrap_or_default();
                                    let created     = c.created_at.as_deref()
                                        .map(fmt_date)
                                        .unwrap_or_default();
                                    let c_click     = c.clone();
                                    let c_open      = c.clone();

                                    view! {
                                        <tr style="cursor:pointer" on:click=move |_| {
                                            selected.set(Some(c_click.clone()));
                                            drawer_open.set(true);
                                        }>
                                            <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                            <td>
                                                <RecordRow
                                                    initials=ini
                                                    name=display
                                                    sub=job_title
                                                    bg="var(--violet-dim)"
                                                    color="var(--violet)"
                                                />
                                            </td>
                                            <td class="mono">{email}</td>
                                            <td class="muted">{phone}</td>
                                            <td class="muted">{created}</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                    e.stop_propagation();
                                                    selected.set(Some(c_open.clone()));
                                                    drawer_open.set(true);
                                                }>"Open"</button>
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

        // Detail Drawer
        {move || selected.get().map(|c| {
            let display_name = c.display_name().to_string();
            let email_c      = c.email.clone().unwrap_or_default();
            let id_c         = c.id.clone();
            let title_sig    = Signal::derive(move || display_name.clone());
            let subtitle_sig = Signal::derive(move || email_c.clone());
            let href         = Signal::derive(move || format!("/contacts/{}", id_c));
            let phone    = c.phone.clone().unwrap_or_else(|| "—".to_string());
            let whatsapp = c.whatsapp.clone().unwrap_or_else(|| "—".to_string());
            let telegram = c.telegram.clone().unwrap_or_else(|| "—".to_string());
            let email    = c.email.clone().unwrap_or_else(|| "—".to_string());
            let created  = c.created_at.as_deref().map(fmt_date).unwrap_or_default();

            view! {
                <RecordDrawer open=drawer_open title=title_sig subtitle=subtitle_sig detail_href=href>
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
    }
}
