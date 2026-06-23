use leptos::prelude::*;
use crate::api::crm::get_leads;
use crate::api::models::LeadModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
    pagination::Pagination,
};

const PER_PAGE: u64 = 25;

fn stage_tag_class(status: &str) -> &'static str {
    match status {
        "New"           => "tag",
        "Contacted"     => "tag tag-contacted",
        "Qualified"     => "tag tag-proposal",
        "Proposal"      => "tag tag-proposal",
        "Converted"     => "tag tag-won",
        "Disqualified"  => "tag tag-disqualified",
        _               => "tag",
    }
}

fn fmt_date(s: &str) -> String {
    s.chars().take(10).collect()
}

/// Leads tab — server-side search + pagination.
#[component]
pub fn LeadsTab() -> impl IntoView {
    let stage_filter     = RwSignal::new("all".to_string());
    let search_filter    = RwSignal::new(String::new());
    let search_debounced = RwSignal::new(String::new());
    let page             = RwSignal::new(1_u64);

    let selected    = RwSignal::new(None::<LeadModel>);
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

    let leads_res = LocalResource::new(move || {
        let search = search_debounced.get();
        let stage  = stage_filter.get();
        let pg     = page.get();
        async move {
            let s  = if search.is_empty() { None } else { Some(search.as_str()) };
            let st = Some(stage.as_str());
            get_leads(s, pg, PER_PAGE, st, None).await.unwrap_or_default()
        }
    });

    let kpi_items = Signal::derive(move || {
        let leads = leads_res.get().unwrap_or_default();
        let total = leads.len();
        let new_count       = leads.iter().filter(|l| l.lead_status.as_deref().unwrap_or("New") == "New").count();
        let qualified_count = leads.iter().filter(|l| l.lead_status.as_deref().unwrap_or("") == "Qualified").count();
        let converted_count = leads.iter().filter(|l| l.is_converted).count();
        vec![
            KpiItem::new("This Page", &total.to_string()).sub("Platform-wide"),
            KpiItem::new("New",       &new_count.to_string()).color("var(--cobalt)"),
            KpiItem::new("Qualified", &qualified_count.to_string()),
            KpiItem::new("Converted", &converted_count.to_string()).color("var(--green)"),
        ]
    });

    let stage_pills = vec![
        PillOption::new("all",          "All"),
        PillOption::new("New",          "New"),
        PillOption::new("Contacted",    "Contacted"),
        PillOption::new("Qualified",    "Qualified"),
        PillOption::new("Proposal",     "Proposal"),
        PillOption::new("Converted",    "Converted"),
        PillOption::new("Disqualified", "Disqualified"),
    ];

    let page_count = Signal::derive(move || leads_res.get().unwrap_or_default().len());

    view! {
        <KpiStrip items=kpi_items />

        <FilterBar
            pills=stage_pills
            active=stage_filter
            search=search_filter
            search_placeholder="Search name, email, company…"
        />

        <div class="table-container">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center text-muted" style="padding:32px;text-align:center;color:var(--text-muted)">
                    "Loading leads..."
                </div>
            }>
                {move || {
                    let rows = leads_res.get().unwrap_or_default();
                    if rows.is_empty() {
                        return view! {
                            <div style="padding:40px;text-align:center;color:var(--text-muted);font-size:13px;">
                                "No leads found."
                            </div>
                        }.into_any();
                    }
                    view! {
                        <table>
                            <thead>
                                <tr>
                                    <th style="width:32px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                    <th style="width:30%" class="sortable">"Lead"</th>
                                    <th style="width:25%" class="sortable">"Email / Phone"</th>
                                    <th style="width:12%" class="sortable">"Source"</th>
                                    <th style="width:12%" class="sortable">"Stage"</th>
                                    <th style="width:10%" class="sortable">"Created"</th>
                                    <th style="width:70px"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {rows.into_iter().map(|l| {
                                    let ini         = initials(&l.name);
                                    let email       = l.email.clone().unwrap_or_else(|| "—".to_string());
                                    let phone       = l.phone.clone().unwrap_or_else(|| "—".to_string());
                                    let source      = l.source.clone().unwrap_or_else(|| "—".to_string());
                                    let status      = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
                                    let created     = l.created_at.as_deref().map(fmt_date).unwrap_or_else(|| "—".to_string());
                                    let tag_class   = stage_tag_class(&status).to_string();
                                    let l_click     = l.clone();
                                    let l_open      = l.clone();

                                    view! {
                                        <tr style="cursor:pointer" on:click=move |_| {
                                            selected.set(Some(l_click.clone()));
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
                                                <div style="display:flex;flex-direction:column;gap:2px;">
                                                    <span style="font-size:12px;color:var(--text-secondary)">{email}</span>
                                                    <span style="font-size:11px;color:var(--text-muted)">{phone}</span>
                                                </div>
                                            </td>
                                            <td>
                                                <span class="tag" style="color:var(--text-muted);border-color:var(--border-default)">{source}</span>
                                            </td>
                                            <td>
                                                <span class=tag_class>{status}</span>
                                            </td>
                                            <td class="muted">{created}</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                    e.stop_propagation();
                                                    selected.set(Some(l_open.clone()));
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
        {move || selected.get().map(|l| {
            let name_c    = l.name.clone();
            let company_c = l.company.clone().unwrap_or_default();
            let email_c   = l.email.clone().unwrap_or_else(|| company_c.clone());
            let id_c      = l.id.clone();
            let title    = Signal::derive(move || name_c.clone());
            let subtitle = Signal::derive(move || email_c.clone());
            let href     = Signal::derive(move || format!("/crm/lead/{}", id_c));
            let status   = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
            let source   = l.source.clone().unwrap_or_else(|| "—".to_string());
            let company  = if company_c.is_empty() { "—".to_string() } else { company_c };
            let title_str = l.title.clone().unwrap_or_else(|| "—".to_string());
            let phone    = l.phone.clone().unwrap_or_else(|| "—".to_string());
            let created  = l.created_at.as_deref().map(fmt_date).unwrap_or_else(|| "—".to_string());
            let tag_class = stage_tag_class(&status).to_string();

            view! {
                <RecordDrawer open=drawer_open title=title subtitle=subtitle detail_href=href>
                    <div class="detail-grid">
                        <span class="detail-section-label">"Lead Info"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Stage"</div>
                            <div class="detail-value"><span class=tag_class>{status}</span></div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Source"</div>
                            <div class="detail-value">{source}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Company"</div>
                            <div class="detail-value">{company}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Job Title"</div>
                            <div class="detail-value">{title_str}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Phone"</div>
                            <div class="detail-value">{phone}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Created"</div>
                            <div class="detail-value mono">{created}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}
    }
}
