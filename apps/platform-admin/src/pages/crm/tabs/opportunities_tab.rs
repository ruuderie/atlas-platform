use leptos::prelude::*;
use crate::api::crm::get_deals;
use crate::api::models::DealModel;
use crate::components::milestone_modal::MilestoneModal;
use crate::pages::crm::components::{
    filter_bar::{FilterBar, PillOption},
    kpi_strip::{KpiStrip, KpiItem},
    record_drawer::RecordDrawer,
    record_row::{RecordRow, initials},
};

#[component]
pub fn OpportunitiesTab() -> impl IntoView {
    let stage_filter  = RwSignal::new("all".to_string());
    let search_filter = RwSignal::new(String::new());

    let selected    = RwSignal::new(None::<DealModel>);
    let drawer_open = RwSignal::new(false);
    let (show_milestone, set_show_milestone) = signal(false);

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let deals_res = LocalResource::new(move || async move {
        get_deals().await.unwrap_or_default()
    });

    let kpi_items = Signal::derive(move || {
        let deals = deals_res.get().unwrap_or_default();
        let open: Vec<_> = deals.iter()
            .filter(|d| d.stage != "Closed Won" && d.stage != "Closed Lost")
            .collect();
        let pipeline: f32 = open.iter().map(|d| d.amount).sum();
        let avg = if deals.is_empty() { 0f32 } else { deals.iter().map(|d| d.amount).sum::<f32>() / deals.len() as f32 };

        vec![
            KpiItem::new("Open", &open.len().to_string())
                .sub("↑ 3 this month"),
            KpiItem::new("Pipeline", &format!("${:.2}M", pipeline / 1_000_000.0))
                .color("var(--cobalt)"),
            KpiItem::new("Weighted (60%)", &format!("${:.2}M", (pipeline * 0.6) / 1_000_000.0))
                .color("var(--green)"),
            KpiItem::new("Avg Deal", &format!("${:.0}k", avg / 1000.0)),
        ]
    });

    let stage_pills = vec![
        PillOption::new("all", "All"),
        PillOption::new("Qualification", "Qualify"),
        PillOption::new("Proposal", "Proposal"),
        PillOption::new("Negotiation", "Negotiate"),
        PillOption::new("Closed Won", "Won"),
        PillOption::new("Closed Lost", "Lost"),
    ];

    view! {
        <KpiStrip items=kpi_items />

        <FilterBar
            pills=stage_pills
            active=stage_filter
            search=search_filter
            search_placeholder="Search opportunities…"
        />

        <div class="table-container">
            <Suspense fallback=move || view! {
                <div class="p-8 text-center text-on-surface-variant">"Loading opportunities..."</div>
            }>
                <table>
                    <thead>
                        <tr>
                            <th style="width:24px"><input type="checkbox" style="accent-color:var(--cobalt)"/></th>
                            <th class="sortable">"Opportunity"</th>
                            <th class="sortable">"Stage"</th>
                            <th class="sortable">"Value"</th>
                            <th class="sortable">"Est. Close"</th>
                            <th class="sortable">"Owner"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let search = search_filter.get().to_lowercase();
                            let stage  = stage_filter.get();
                            deals_res.get().unwrap_or_default().into_iter()
                                .filter(|d| {
                                    let matches_search = search.is_empty()
                                        || d.name.to_lowercase().contains(&search);
                                    let matches_stage = stage == "all" || d.stage == stage;
                                    matches_search && matches_stage
                                })
                                .map(|d| {
                                    let ini    = initials(&d.name);
                                    let amount = format!("${:.2}", d.amount);
                                    let is_won = d.stage == "Closed Won";
                                    let stage_class = match d.stage.as_str() {
                                        "Proposal"     => "tag-proposal",
                                        "Negotiation"  => "tag-negotiation",
                                        "Closed Won"   => "tag-won",
                                        "Closed Lost"  => "tag-lost",
                                        _              => "tag-qualify",
                                    };
                                    let d_for_drawer = d.clone();
                                    let d_for_open   = d.clone();

                                    view! {
                                        <tr on:click=move |_| {
                                            if d_for_drawer.stage.contains("Negotiation") || d_for_drawer.stage.contains("Won") {
                                                set_show_milestone.set(true);
                                            }
                                            selected.set(Some(d_for_drawer.clone()));
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
                                            <td><span class=format!("tag {}", stage_class)>{d.stage.clone()}</span></td>
                                            <td class="mono font-semibold" style=if is_won { "color:var(--green);" } else { "" }>{amount}</td>
                                            <td class="muted">"—"</td>
                                            <td>"—"</td>
                                            <td>
                                                <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                    e.stop_propagation();
                                                    selected.set(Some(d_for_open.clone()));
                                                    drawer_open.set(true);
                                                }>"Log"</button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()
                        }}
                    </tbody>
                </table>
            </Suspense>
        </div>

        {move || selected.get().map(|d| {
            let title    = Signal::derive(move || d.name.clone());
            let subtitle = Signal::derive(move || "—".to_string());
            let href     = Signal::derive(move || format!("/crm/deal/{}", d.id));
            let amount   = format!("${:.2}", d.amount);
            let stage    = d.stage.clone();
            let toast    = toast.clone();

            let extra = view! {
                <button class="btn btn-primary btn-sm" on:click=move |_| {
                    toast.message.set(Some("Deal closed won.".to_string()));
                    drawer_open.set(false);
                }>"Close Won"</button>
            }.into_any();

            view! {
                <RecordDrawer
                    open=drawer_open
                    title=title
                    subtitle=subtitle
                    detail_href=href
                    extra_actions=Some(extra)
                >
                    <div class="detail-grid">
                        <span class="detail-section-label">"Deal Details"</span>
                        <div class="detail-field">
                            <div class="detail-label">"Value"</div>
                            <div class="detail-value mono">{amount}</div>
                        </div>
                        <div class="detail-field">
                            <div class="detail-label">"Stage"</div>
                            <div class="detail-value">{stage}</div>
                        </div>
                    </div>
                </RecordDrawer>
            }
        })}

        <MilestoneModal
            open=show_milestone
            on_close=Callback::new(move |_| set_show_milestone.set(false))
            on_activate=Callback::new(move |_| {
                leptos::logging::log!("Upsell Event: Proposal Auto-Gen Activated");
                set_show_milestone.set(false);
            })
            title="Deal is heating up!".to_string()
            description="This deal is nearing the finish line. Do you want to automatically generate a tailored proposal?".to_string()
            feature_name="Atlas Proposal Auto-Gen".to_string()
            price_text="$49 / month".to_string()
        />
    }
}
