use leptos::prelude::*;
use crate::api::crm::get_accounts;
use crate::api::models::AccountModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
};

#[component]
pub fn AccountsTab() -> impl IntoView {
    let type_filter   = RwSignal::new("all".to_string());
    let search_filter = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page = RwSignal::new(1_u64);
    const PER_PAGE: u64 = 25;

    let selected    = RwSignal::new(None::<AccountModel>);
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
        let total = accounts.len();
        // Use account_type field once available; until then show total only
        vec![
            KpiItem::new("Total Accounts", &total.to_string())
                .sub("Platform-wide"),
            KpiItem::new("Active", &total.to_string())
                .color("var(--green)"),
            KpiItem::new("Suspended", "0"),
            KpiItem::new("Contribution MRR", "—")
                .color("var(--cobalt)"),
        ]
    });

    let type_pills = vec![
        PillOption::new("all", "All"),
        PillOption::new("Organization", "Orgs"),
        PillOption::new("Individual", "Individuals"),
        PillOption::new("Active", "Active"),
        PillOption::new("Suspended", "Suspended"),
    ];

    view! {
        <KpiStrip items=kpi_items />

        <FilterBar
            pills=type_pills
            active=type_filter
            search=search_filter
            search_placeholder="Search accounts…"
        />

        <div class="table-container">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center text-on-surface-variant">"Loading accounts..."</div>
            }>
                <table>
                    <thead>
                        <tr>
                            <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                            <th class="sortable">"Account"</th>
                            <th class="sortable">"Type"</th>
                            <th class="sortable">"Status"</th>
                            <th class="sortable">"MRR"</th>
                            <th class="sortable">"Created"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            accounts_res.get().unwrap_or_default().into_iter()
                                .map(|a| {
                                    let ini = initials(&a.name);
                                    let a_for_drawer = a.clone();
                                    let a_for_open   = a.clone();

                                    view! {
                                        <tr on:click=move |_| {
                                            selected.set(Some(a_for_drawer.clone()));
                                            drawer_open.set(true);
                                        }>
                                            <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                            <td><RecordRow initials=ini name=a.name.clone() /></td>
                                            <td>"—"</td>
                                            <td><span class="tag tag-verified">"Active"</span></td>
                                            <td class="mono font-semibold">"—"</td>
                                            <td class="muted">"—"</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                    e.stop_propagation();
                                                    selected.set(Some(a_for_open.clone()));
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
                " · " {move || accounts_res.get().unwrap_or_default().len().to_string()}
                " records"
            </span>
            <div class="flex gap-2">
                <button
                    class=move || { if page.get() <= 1 { "btn btn-ghost btn-sm opacity-40 cursor-not-allowed".to_string() } else { "btn btn-ghost btn-sm".to_string() } }
                    on:click=move |_| { if page.get() > 1 { page.update(|p| *p -= 1); } }
                    disabled=move || page.get() <= 1
                >"← Prev"</button>
                <button
                    class=move || { let c = accounts_res.get().unwrap_or_default().len() as u64; if c < PER_PAGE { "btn btn-ghost btn-sm opacity-40 cursor-not-allowed".to_string() } else { "btn btn-ghost btn-sm".to_string() } }
                    on:click=move |_| { let c = accounts_res.get().unwrap_or_default().len() as u64; if c >= PER_PAGE { page.update(|p| *p += 1); } }
                    disabled=move || (accounts_res.get().unwrap_or_default().len() as u64) < PER_PAGE
                >"Next →"</button>
            </div>
        </div>

        {move || selected.get().map(|a| {
            let name_for_title  = a.name.clone();
            let name_for_detail = a.name.clone();
            let id_for_href     = a.id.clone();
            let title    = Signal::derive(move || name_for_title.clone());
            let subtitle = Signal::derive(move || "—".to_string());
            let href     = Signal::derive(move || format!("/crm/account/{}", id_for_href));

            view! {
                <RecordDrawer
                    open=drawer_open
                    title=title
                    subtitle=subtitle
                    detail_href=href
                >
                    <div class="detail-grid">
                        <span class="detail-section-label">"Account Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Account Name"</div>
                            <div class="detail-value">{name_for_detail}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}
    }
}
