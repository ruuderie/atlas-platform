use leptos::prelude::*;
use crate::api::crm::get_leads;
use crate::api::models::LeadModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
};

const PER_PAGE: u64 = 25;

/// Leads tab — server-side search + pagination.
#[component]
pub fn LeadsTab() -> impl IntoView {
    let stage_filter  = RwSignal::new("all".to_string());
    let search_filter = RwSignal::new(String::new());
    // Debounced version that actually triggers the fetch
    let search_debounced = RwSignal::new(String::new());
    let page = RwSignal::new(1_u64);

    // Drawer state
    let selected    = RwSignal::new(None::<LeadModel>);
    let drawer_open = RwSignal::new(false);

    // 400ms debounce: update search_debounced after user stops typing
    Effect::new(move |_| {
        let val = search_filter.get();
        let page_reset = page;
        leptos::task::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(400)).await;
            search_debounced.set(val.clone());
            page_reset.set(1); // reset to page 1 on new search
        });
    });

    // Resource — re-fetches whenever search, stage, or page changes
    let leads_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let stage  = stage_filter.get();
        let pg     = page.get();
        async move {
            let s = if search.is_empty() { None } else { Some(search.as_str()) };
            let st = Some(stage.as_str());
            get_leads(s, pg, PER_PAGE, st).await.unwrap_or_default()
        }
    });

    // KPI strip uses the current page result — a full total count endpoint
    // would need a separate API call; for now we show the current page counts.
    let kpi_items = Signal::derive(move || {
        let leads = leads_res.get().unwrap_or_default();
        let total = leads.len();
        let new_count = leads.iter().filter(|l| l.lead_status.as_deref().unwrap_or("New") == "New").count();
        vec![
            KpiItem::new("This Page", &total.to_string()).sub("Platform-wide · G-31"),
            KpiItem::new("New", &new_count.to_string()).color("var(--cobalt)"),
            KpiItem::new("Qualified", &leads.iter().filter(|l| l.lead_status.as_deref().unwrap_or("") == "Qualified").count().to_string()),
            KpiItem::new("Converted", &leads.iter().filter(|l| l.is_converted).count().to_string()).color("var(--green)"),
        ]
    });

    let stage_pills = vec![
        PillOption::new("all",           "All"),
        PillOption::new("New",           "New"),
        PillOption::new("Contacted",     "Contacted"),
        PillOption::new("Qualified",     "Qualified"),
        PillOption::new("Proposal",      "Proposal"),
        PillOption::new("Converted",     "Converted"),
        PillOption::new("Disqualified",  "Disqualified"),
    ];

    view! {
        <KpiStrip items=kpi_items />

        <FilterBar
            pills=stage_pills
            active=stage_filter
            search=search_filter
            search_placeholder="Search leads…"
        />

        <div class="table-container">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center text-on-surface-variant">"Loading leads..."</div>
            }>
                <table>
                    <thead>
                        <tr>
                            <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                            <th class="sortable">"Lead"</th>
                            <th class="sortable">"Contact"</th>
                            <th class="sortable">"Source"</th>
                            <th class="sortable">"Stage"</th>
                            <th class="sortable">"Last Activity"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            leads_res.get().unwrap_or_default().into_iter()
                                .map(|l| {
                                    let ini         = initials(&l.name);
                                    let email       = l.email.clone().unwrap_or_else(|| "—".to_string());
                                    let phone       = l.phone.clone().unwrap_or_else(|| "—".to_string());
                                    let source      = l.source.clone().unwrap_or_else(|| "—".to_string());
                                    let status      = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
                                    let last_active = l.created_at.clone().unwrap_or_else(|| "—".to_string());
                                    let l_for_drawer = l.clone();
                                    let l_for_open   = l.clone();

                                    view! {
                                        <tr on:click=move |_| {
                                            selected.set(Some(l_for_drawer.clone()));
                                            drawer_open.set(true);
                                        }>
                                            <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                            <td>
                                                <RecordRow
                                                    initials=ini
                                                    name=l.name.clone()
                                                    sub=l.company.clone().unwrap_or_default()
                                                />
                                            </td>
                                            <td>
                                                <div class="contact-cell flex flex-col">
                                                    <span class="contact-email text-xs">{email}</span>
                                                    <span class="contact-phone text-[11px] text-muted">{phone}</span>
                                                </div>
                                            </td>
                                            <td><span class="tag" style="color:var(--text-muted);border-color:var(--border-default)">{source}</span></td>
                                            <td>
                                                <div class="stage-cell flex items-center gap-1.5">
                                                    <span class="stage-dot" style="display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--cobalt)"></span>
                                                    {status}
                                                </div>
                                            </td>
                                            <td class="muted">{last_active}</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                    e.stop_propagation();
                                                    selected.set(Some(l_for_open.clone()));
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

        // ── Pagination controls ───────────────────────────────────────────────
        <div class="flex items-center justify-between px-1 pt-2 text-xs text-on-surface-variant">
            <span>
                "Page " {move || page.get().to_string()}
                " · " {move || leads_res.get().unwrap_or_default().len().to_string()}
                " records"
            </span>
            <div class="flex gap-2">
                <button
                    class=move || {
                        if page.get() <= 1 { "btn btn-ghost btn-sm opacity-40 cursor-not-allowed".to_string() }
                        else { "btn btn-ghost btn-sm".to_string() }
                    }
                    on:click=move |_| { if page.get() > 1 { page.update(|p| *p -= 1); } }
                    disabled=move || page.get() <= 1
                >"← Prev"</button>
                <button
                    class=move || {
                        let count = leads_res.get().unwrap_or_default().len() as u64;
                        if count < PER_PAGE { "btn btn-ghost btn-sm opacity-40 cursor-not-allowed".to_string() }
                        else { "btn btn-ghost btn-sm".to_string() }
                    }
                    on:click=move |_| {
                        let count = leads_res.get().unwrap_or_default().len() as u64;
                        if count >= PER_PAGE { page.update(|p| *p += 1); }
                    }
                    disabled=move || (leads_res.get().unwrap_or_default().len() as u64) < PER_PAGE
                >"Next →"</button>
            </div>
        </div>

        // Drawer
        {move || selected.get().map(|l| {
            let title    = Signal::derive(move || l.name.clone());
            let subtitle = Signal::derive(move || l.email.clone().unwrap_or_default());
            let href     = Signal::derive(move || format!("/crm/lead/{}", l.id));
            let status   = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
            let converted = l.is_converted.to_string();

            view! {
                <RecordDrawer
                    open=drawer_open
                    title=title
                    subtitle=subtitle
                    detail_href=href
                >
                    <div class="detail-grid">
                        <span class="detail-section-label">"Lead Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Pipeline Stage"</div>
                            <div class="detail-value">{status}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Converted"</div>
                            <div class="detail-value">{converted}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}
    }
}
