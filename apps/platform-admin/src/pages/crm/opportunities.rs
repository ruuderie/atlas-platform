use leptos::prelude::*;
use crate::api::crm::get_deals;
use crate::api::models::DealModel;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
    pagination::Pagination,
};

const PER_PAGE: u64 = 25;

fn deal_stage_tag(stage: &str) -> &'static str {
    match stage {
        "Qualification" => "tag tag-qualify",
        "Proposal"      => "tag tag-proposal",
        "Negotiation"   => "tag tag-negotiation",
        "Closed Won"    => "tag tag-won",
        "Closed Lost"   => "tag tag-lost",
        _               => "tag",
    }
}

#[component]
pub fn OpportunitiesPage() -> impl IntoView {
    let stage_filter  = RwSignal::new("all".to_string());
    let search_filter = RwSignal::new(String::new());
    let page          = RwSignal::new(1_u64);

    let selected    = RwSignal::new(None::<DealModel>);
    let drawer_open = RwSignal::new(false);

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Deals API loads all at once; filter + paginate client-side
    let deals_res = LocalResource::new(move || async move {
        get_deals().await.unwrap_or_default()
    });

    let kpi_items = Signal::derive(move || {
        let deals   = deals_res.get().unwrap_or_default();
        let open: Vec<_> = deals.iter()
            .filter(|d| d.stage != "Closed Won" && d.stage != "Closed Lost")
            .collect();
        let pipeline: f32 = open.iter().map(|d| d.amount).sum();
        let won: f32 = deals.iter()
            .filter(|d| d.stage == "Closed Won")
            .map(|d| d.amount).sum();
        let avg = if deals.is_empty() { 0f32 } else {
            deals.iter().map(|d| d.amount).sum::<f32>() / deals.len() as f32
        };
        vec![
            KpiItem::new("Open",       &open.len().to_string()).color("var(--cobalt)"),
            KpiItem::new("Pipeline",   &format!("${:.1}k", pipeline / 1000.0)).color("var(--cobalt)"),
            KpiItem::new("Closed Won", &format!("${:.1}k", won / 1000.0)).color("var(--green)"),
            KpiItem::new("Avg Deal",   &format!("${:.0}", avg)),
        ]
    });

    let stage_pills = vec![
        PillOption::new("all",           "All"),
        PillOption::new("Qualification", "Qualify"),
        PillOption::new("Proposal",      "Proposal"),
        PillOption::new("Negotiation",   "Negotiate"),
        PillOption::new("Closed Won",    "Won"),
        PillOption::new("Closed Lost",   "Lost"),
    ];

    let filtered = Signal::derive(move || {
        let search = search_filter.get().to_lowercase();
        let stage  = stage_filter.get();
        let pg     = page.get() as usize;
        let offset = (pg - 1) * PER_PAGE as usize;
        deals_res.get().unwrap_or_default()
            .into_iter()
            .filter(|d| {
                let ms = search.is_empty() || d.name.to_lowercase().contains(&search);
                let mg = stage == "all" || d.stage == stage;
                ms && mg
            })
            .skip(offset)
            .take(PER_PAGE as usize)
            .collect::<Vec<_>>()
    });

    let page_count = Signal::derive(move || filtered.get().len());

    view! {
        <div class="entity-page">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Pipeline"</h1>
                    <p class="page-subtitle">"Platform-wide · All tenants"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button class="btn btn-ghost btn-sm">"Export CSV"</button>
                    <button class="btn btn-primary btn-sm">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="currentColor" style="margin-right:4px;">
                            <path d="M7 2a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H8v3a1 1 0 1 1-2 0V8H3a1 1 0 1 1 0-2h3V3a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Deal"
                    </button>
                </div>
            </div>

            <KpiStrip items=kpi_items />

            <FilterBar
                pills=stage_pills
                active=stage_filter
                search=search_filter
                search_placeholder="Search opportunities…"
            />

            <div class="table-container">
                <Suspense fallback=move || view! {
                    <div style="padding:32px;text-align:center;color:var(--text-muted)">"Loading pipeline…"</div>
                }>
                    {move || {
                        let rows = filtered.get();
                        if rows.is_empty() {
                            return view! {
                                <div class="empty-state">
                                    <div class="empty-state-icon">"◎"</div>
                                    <div class="empty-state-title">"No deals found"</div>
                                    <div class="empty-state-body">"Try adjusting your stage filter or search."</div>
                                </div>
                            }.into_any();
                        }
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th style="width:32px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                                        <th style="width:35%" class="sortable">"Opportunity"</th>
                                        <th style="width:18%" class="sortable">"Stage"</th>
                                        <th style="width:13%" class="sortable right">"Value"</th>
                                        <th style="width:20%" class="sortable">"Status"</th>
                                        <th style="width:70px"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|d| {
                                        let ini        = initials(&d.name);
                                        let amount     = format!("${:.2}", d.amount);
                                        let is_won     = d.stage == "Closed Won";
                                        let tag_class  = deal_stage_tag(&d.stage).to_string();
                                        let status_str = d.status.clone();
                                        let detail_href = format!("/pipeline/{}", d.id);
                                        let d_click    = d.clone();

                                        view! {
                                            <tr style="cursor:pointer" on:click=move |_| {
                                                selected.set(Some(d_click.clone()));
                                                drawer_open.set(true);
                                            }>
                                                <td><input type="checkbox" on:click=move |e| e.stop_propagation() style="accent-color:var(--cobalt)"/></td>
                                                <td>
                                                    <RecordRow
                                                        initials=ini
                                                        name=d.name.clone()
                                                        bg="var(--amber-dim)"
                                                        color="var(--amber)"
                                                    />
                                                </td>
                                                <td><span class=tag_class>{d.stage.clone()}</span></td>
                                                <td class=move || if is_won { "mono right green" } else { "mono right" }>{amount}</td>
                                                <td class="muted">{status_str}</td>
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

        {move || selected.get().map(|d| {
            let name_c  = d.name.clone();
            let stage_c = d.stage.clone();
            let id_c    = d.id.clone();
            let title    = Signal::derive(move || name_c.clone());
            let subtitle = Signal::derive(move || stage_c.clone());
            let href     = Signal::derive(move || format!("/pipeline/{}", id_c));
            let amount   = format!("${:.2}", d.amount);
            let stage    = d.stage.clone();
            let status   = d.status.clone();
            let tag_class = deal_stage_tag(&stage).to_string();
            let toast_c  = toast.clone();

            let mark_won = view! {
                <button class="btn btn-primary btn-sm" on:click=move |_| {
                    toast_c.message.set(Some("Marked as Closed Won.".to_string()));
                    drawer_open.set(false);
                }>"Mark Won"</button>
            }.into_any();

            view! {
                <RecordDrawer
                    open=drawer_open
                    title=title
                    subtitle=subtitle
                    detail_href=href
                    extra_actions=Some(mark_won)
                >
                    <div class="detail-grid">
                        <span class="detail-section-label">"Deal Details"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Stage"</div>
                            <div class="detail-value"><span class=tag_class>{stage}</span></div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Value"</div>
                            <div class="detail-value mono" style="color:var(--green);font-size:16px;font-weight:700">{amount}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Status"</div>
                            <div class="detail-value">{status}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}
    }
}
