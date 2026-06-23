use leptos::prelude::*;
use crate::api::crm::{get_deals, update_deal, create_deal};
use crate::api::models::{DealModel, CreateDeal};
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

    // New Deal modal state
    let show_create = RwSignal::new(false);
    let new_name    = RwSignal::new(String::new());
    let new_amount  = RwSignal::new(String::new());
    let new_stage   = RwSignal::new("Qualification".to_string());
    let is_creating = RwSignal::new(false);

    // Deals API — declared before handle_create_deal so the closure can call refetch()
    let fetch_trigger = RwSignal::new(0_u32);
    let deals_res = LocalResource::new(move || {
        fetch_trigger.get(); // tracked — incremented after create to force reload
        async move { get_deals().await.unwrap_or_default() }
    });

    let handle_create_deal = move |_| {
        let name   = new_name.get().trim().to_string();
        let amount_str = new_amount.get().trim().to_string();
        let stage  = new_stage.get();
        if name.is_empty() {
            toast.show_toast("Validation", "Deal name is required.", "error");
            return;
        }
        let amount: f32 = amount_str.parse().unwrap_or(0.0);
        if is_creating.get() { return; }
        is_creating.set(true);
        let t = toast.clone();
        leptos::task::spawn_local(async move {
            let payload = CreateDeal {
                customer_id: String::new(), // platform-level deal — no customer scope required
                name,
                amount,
                status: "open".to_string(),
                stage,
            };
            match create_deal(payload).await {
                Ok(_) => {
                    t.show_toast("Deal Created", "New deal added to pipeline.", "success");
                    show_create.set(false);
                    new_name.set(String::new());
                    new_amount.set(String::new());
                    new_stage.set("Qualification".to_string());
                    fetch_trigger.update(|n| *n += 1);
                }
                Err(e) => t.show_toast("Error", &format!("Failed to create deal: {e}"), "error"),
            }
            is_creating.set(false);
        });
    };

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
            <div class="page-header" style="display:flex;align-items:flex-start;justify-content:space-between;padding:16px 20px;flex-shrink:0;gap:12px;">
                <div>
                    <h1 class="page-title">"Pipeline"</h1>
                    <p class="page-subtitle">"Platform-wide · All tenants"</p>
                </div>
                <div style="display:flex;gap:8px;">
                    <button
                        class="btn btn-ghost btn-sm opacity-40 cursor-not-allowed"
                        title="CSV export endpoint pending"
                        disabled
                    >"Export CSV"</button>
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| show_create.set(true)
                    >
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
            let deal_id_for_won = d.id.clone();

            let mark_won = view! {
                <button class="btn btn-primary btn-sm" on:click=move |_| {
                    let id = deal_id_for_won.clone();
                    let t  = toast_c.clone();
                    leptos::task::spawn_local(async move {
                        match update_deal(&id, "Closed Won", "Closed Won").await {
                            Ok(_)  => { t.show_toast("Pipeline", "Deal marked as Closed Won.", "success"); }
                            Err(e) => { t.show_toast("Error", &format!("Failed to update deal: {}", e), "error"); }
                        }
                        drawer_open.set(false);
                    });
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

        // ── New Deal Modal ────────────────────────────────────────────────────
        <Show when=move || show_create.get()>
            <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl">
                    <button class="absolute top-4 right-4 text-slate-400 hover:text-white"
                        on:click=move |_| show_create.set(false)>"✕"</button>
                    <h3 class="text-xl font-semibold mb-4 text-foreground">"New Deal"</h3>
                    <div class="space-y-4 mb-6">
                        <div class="grid gap-1">
                            <label class="text-sm font-medium text-on-surface-variant">"Deal Name *"</label>
                            <input
                                class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full"
                                type="text"
                                placeholder="e.g. Acme Corp Renewal"
                                prop:value=move || new_name.get()
                                on:input=move |e| new_name.set(event_target_value(&e))
                            />
                        </div>
                        <div class="grid gap-1">
                            <label class="text-sm font-medium text-on-surface-variant">"Value (USD)"</label>
                            <input
                                class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full"
                                type="number"
                                placeholder="e.g. 12500"
                                prop:value=move || new_amount.get()
                                on:input=move |e| new_amount.set(event_target_value(&e))
                            />
                        </div>
                        <div class="grid gap-1">
                            <label class="text-sm font-medium text-on-surface-variant">"Stage"</label>
                            <select
                                class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full"
                                on:change=move |e| new_stage.set(event_target_value(&e))
                            >
                                <option value="Qualification" selected=true>"Qualification"</option>
                                <option value="Proposal">"Proposal"</option>
                                <option value="Negotiation">"Negotiation"</option>
                                <option value="Closed Won">"Closed Won"</option>
                                <option value="Closed Lost">"Closed Lost"</option>
                            </select>
                        </div>
                    </div>
                    <div class="flex justify-end gap-3">
                        <button class="btn btn-ghost" on:click=move |_| show_create.set(false)>"Cancel"</button>
                        <button
                            class="btn btn-primary"
                            on:click=handle_create_deal
                            disabled=move || is_creating.get()
                        >
                            {move || if is_creating.get() { "Saving…" } else { "Save Deal" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
