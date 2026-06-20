use leptos::prelude::*;
use crate::api::crm::get_contacts;
use crate::api::models::ContactModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
};

#[component]
pub fn ContactsTab() -> impl IntoView {
    let ver_filter    = RwSignal::new("all".to_string());
    let search_filter = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page = RwSignal::new(1_u64);
    const PER_PAGE: u64 = 25;

    let selected    = RwSignal::new(None::<ContactModel>);
    let drawer_open = RwSignal::new(false);

    Effect::new(move |_| {
        let val = search_filter.get();
        let page_reset = page;
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(400)).await;
            search_debounced.set(val);
            page_reset.set(1);
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

    let kpi_items = Signal::derive(move || {
        let contacts = contacts_res.get().unwrap_or_default();
        let total = contacts.len();
        vec![
            KpiItem::new("Total Contacts", &total.to_string())
                .sub("Platform-wide"),
            KpiItem::new("Verified Profiles", &total.to_string())
                .color("var(--green)"),
            KpiItem::new("Pending G-06", "—")
                .color("var(--amber)"),
            KpiItem::new("Flagged", "—")
                .color("var(--red)"),
        ]
    });

    let ver_pills = vec![
        PillOption::new("all", "All"),
        PillOption::new("Verified", "Verified"),
        PillOption::new("Pending", "Pending"),
        PillOption::new("Flagged", "Flagged"),
    ];

    view! {
        <KpiStrip items=kpi_items />

        <FilterBar
            pills=ver_pills
            active=ver_filter
            search=search_filter
            search_placeholder="Search contacts…"
        />

        <div class="table-container">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center text-on-surface-variant">"Loading contacts..."</div>
            }>
                <table>
                    <thead>
                        <tr>
                            <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                            <th class="sortable">"Contact"</th>
                            <th class="sortable">"Account"</th>
                            <th class="sortable">"Email"</th>
                            <th class="sortable">"Phone"</th>
                            <th class="sortable">"Verification (G-06)"</th>
                            <th class="sortable">"Last Active"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            contacts_res.get().unwrap_or_default().into_iter()
                                .map(|c| {
                                    let ini   = initials(&c.name);
                                    let email = c.email.clone().unwrap_or_else(|| "—".to_string());
                                    let phone = c.phone.clone().unwrap_or_else(|| "—".to_string());
                                    let job_title = c.properties.as_ref()
                                        .and_then(|p| p.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()))
                                        .unwrap_or_default();
                                    let last_active = c.updated_at.clone();
                                    let c_for_drawer = c.clone();
                                    let c_for_open   = c.clone();

                                    view! {
                                        <tr on:click=move |_| {
                                            selected.set(Some(c_for_drawer.clone()));
                                            drawer_open.set(true);
                                        }>
                                            <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                            <td>
                                                <RecordRow
                                                    initials=ini
                                                    name=c.name.clone()
                                                    sub=job_title
                                                />
                                            </td>
                                            <td>"—"</td>
                                            <td class="mono">{email}</td>
                                            <td class="muted">{phone}</td>
                                            <td><span class="tag tag-verified">"Active"</span></td>
                                            <td class="muted">{last_active}</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                    e.stop_propagation();
                                                    selected.set(Some(c_for_open.clone()));
                                                    drawer_open.set(true);
                                                }>"Open"</button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()
                        }}
                    </tbody>
                </table>
            </Suspense>
        </div>

        // ── Pagination controls ───────────────────────────────────────────────────
        <div class="flex items-center justify-between px-1 pt-2 text-xs text-on-surface-variant">
            <span>
                "Page " {move || page.get().to_string()}
                " · " {move || contacts_res.get().unwrap_or_default().len().to_string()}
                " records"
            </span>
            <div class="flex gap-2">
                <button
                    class=move || { if page.get() <= 1 { "btn btn-ghost btn-sm opacity-40 cursor-not-allowed".to_string() } else { "btn btn-ghost btn-sm".to_string() } }
                    on:click=move |_| { if page.get() > 1 { page.update(|p| *p -= 1); } }
                    disabled=move || page.get() <= 1
                >"← Prev"</button>
                <button
                    class=move || { let c = contacts_res.get().unwrap_or_default().len() as u64; if c < PER_PAGE { "btn btn-ghost btn-sm opacity-40 cursor-not-allowed".to_string() } else { "btn btn-ghost btn-sm".to_string() } }
                    on:click=move |_| { let c = contacts_res.get().unwrap_or_default().len() as u64; if c >= PER_PAGE { page.update(|p| *p += 1); } }
                    disabled=move || (contacts_res.get().unwrap_or_default().len() as u64) < PER_PAGE
                >"Next →"</button>
            </div>
        </div>

        {move || selected.get().map(|c| {
            let name_for_title  = c.name.clone();
            let sub_for_title   = c.email.clone().unwrap_or_default();
            let id_for_href     = c.id.clone();
            let phone = c.phone.clone().unwrap_or_else(|| "—".to_string());
            let title    = Signal::derive(move || name_for_title.clone());
            let subtitle = Signal::derive(move || sub_for_title.clone());
            let href     = Signal::derive(move || format!("/crm/contact/{}", id_for_href));

            view! {
                <RecordDrawer
                    open=drawer_open
                    title=title
                    subtitle=subtitle
                    detail_href=href
                >
                    <div class="detail-grid">
                        <span class="detail-section-label">"Contact Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Phone"</div>
                            <div class="detail-value">{phone}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}
    }
}
