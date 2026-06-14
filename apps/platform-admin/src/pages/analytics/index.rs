use leptos::prelude::*;
use crate::api::analytics::{get_business_kpis, get_billing_summary};

#[component]
pub fn Analytics() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Tab switching state
    let active_tab = RwSignal::new("p-overview".to_string());
    
    // Modals
    let show_campaign_modal = RwSignal::new(false);
    
    // Dropdown filters
    let selected_range = RwSignal::new("June 2026".to_string());
    let selected_tenant = RwSignal::new("All Tenants".to_string());
    
    // Campaign form states
    let campaign_name = RwSignal::new(String::new());
    let campaign_type = RwSignal::new("email".to_string());
    let campaign_goal = RwSignal::new("lead_capture".to_string());
    let campaign_budget = RwSignal::new("500000".to_string());
    
    // Resources fetching real backend metrics
    let business_kpis = LocalResource::new(move || async move { get_business_kpis().await });
    let billing_summary = LocalResource::new(move || async move { get_billing_summary().await });

    view! {
        <div class="space-y-6">
            // ── Page Header ──
            <div class="flex justify-between items-center border-b border-outline-variant/20 pb-4">
                <div>
                    <h1 class="text-3xl font-extrabold tracking-tight text-on-surface">"Analytics"</h1>
                    <p class="text-xs text-on-surface-variant mt-1 font-mono">
                        "platform_metrics_daily · atlas_attribution_touchpoints · atlas_campaigns · atlas_scorecard_time_series · request_log"
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <select 
                        class="bg-surface-container-high border border-outline-variant/30 text-on-surface text-xs rounded-lg p-2 outline-none cursor-pointer focus:border-primary"
                        on:change=move |ev| selected_range.set(event_target_value(&ev))
                        prop:value=selected_range
                    >
                        <option value="Last 30 days">"Last 30 days"</option>
                        <option value="Last 7 days">"Last 7 days"</option>
                        <option value="June 2026">"June 2026"</option>
                        <option value="Q2 2026">"Q2 2026"</option>
                        <option value="YTD 2026">"YTD 2026"</option>
                    </select>
                    <select 
                        class="bg-surface-container-high border border-outline-variant/30 text-on-surface text-xs rounded-lg p-2 outline-none cursor-pointer focus:border-primary"
                        on:change=move |ev| selected_tenant.set(event_target_value(&ev))
                        prop:value=selected_tenant
                    >
                        <option value="All Tenants">"All Tenants"</option>
                        <option value="Nexus PM Group">"Nexus PM Group"</option>
                        <option value="Biscayne STR Co.">"Biscayne STR Co."</option>
                        <option value="Harbor Media">"Harbor Media"</option>
                    </select>
                    <button 
                        class="btn-ghost text-xs px-3.5 py-2 border border-outline-variant/30 rounded-lg hover:bg-surface-bright/20 transition-all font-semibold"
                        on:click=move |_| toast.show_toast("Export Queue", "Analytics CSV export triggered.", "success")
                    >
                        "↓ Export CSV"
                    </button>
                    <button 
                        class="btn-primary-gradient text-xs px-3.5 py-2 rounded-lg font-semibold text-on-primary-container shadow hover:opacity-90 active:scale-95 transition-all"
                        on:click=move |_| show_campaign_modal.set(true)
                    >
                        "+ New Campaign"
                    </button>
                </div>
            </div>

            // ── KPI Strip ──
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 xl:grid-cols-9 gap-4 bg-surface-container-low border border-outline-variant/15 p-4 rounded-xl shadow-inner overflow-x-auto shrink-0 select-none">
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Total GMV"</span>
                    <span class="text-lg font-black text-emerald-400">"$2.14M"</span>
                    <span class="text-[9.5px] text-emerald-400">"↑ 18% vs May"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Platform MRR"</span>
                    <span class="text-lg font-black text-primary">
                        {move || match business_kpis.get() {
                            Some(Ok(kpis)) => format!("${:.0}k", kpis.mrr.value / 1000.0),
                            _ => "$84k".to_string()
                        }}
                    </span>
                    <span class="text-[9.5px] text-emerald-400">"↑ 12% vs May"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"New Leads"</span>
                    <span class="text-lg font-black font-mono">"47"</span>
                    <span class="text-[9.5px] text-emerald-400">"↑ 31% vs May"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Converted"</span>
                    <span class="text-lg font-black font-mono">"8"</span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"→ 17% rate"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Open Opps"</span>
                    <span class="text-lg font-black font-mono text-amber-400">"12"</span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"$14.2M pipeline"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Reservations"</span>
                    <span class="text-lg font-black font-mono">"618"</span>
                    <span class="text-[9.5px] text-emerald-400">"↑ 9% vs May"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Avg G-27 Score"</span>
                    <span class="text-lg font-black text-[#88cc00] font-mono">"7.4"</span>
                    <span class="text-[9.5px] text-emerald-400">"↑ 0.3 vs May"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"API Req (24h)"</span>
                    <span class="text-lg font-black font-mono">"2.1M"</span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"→ p99: 84ms"</span>
                </div>
                <div class="flex flex-col gap-1 p-2">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Error Rate"</span>
                    <span class="text-lg font-black font-mono text-error">"0.4%"</span>
                    <span class="text-[9.5px] text-error">"↑ from 0.2%"</span>
                </div>
            </div>

            // ── Tab Navigation Bar ──
            <div class="flex border-b border-outline-variant/20 overflow-x-auto shrink-0 select-none">
                {
                    let tab_btn = move |id: &str, label: &str| {
                        let id = id.to_string();
                        let label = label.to_string();
                        let id_class = id.clone();
                        let id_click = id.clone();
                        view! {
                            <button 
                                class=move || if active_tab.get() == id_class { "px-4 py-2.5 text-sm font-semibold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent" } else { "px-4 py-2.5 text-sm text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent" }
                                on:click=move |_| active_tab.set(id_click.clone())
                            >
                                {label.clone()}
                            </button>
                        }
                    };
                    view! {
                        {tab_btn("p-overview", "Overview")}
                        {tab_btn("p-revenue", "Revenue & GMV")}
                        {tab_btn("p-crm", "CRM Funnel")}
                        {tab_btn("p-attribution", "Attribution · UTM")}
                        {tab_btn("p-campaigns", "Campaigns")}
                        {tab_btn("p-scorecards", "G-27 Trends")}
                        {tab_btn("p-platform", "Platform Metrics · Raw")}
                        {tab_btn("p-api", "API & Request Log")}
                    }
                }
            </div>

            // ── TAB CONTENT: Overview ──
            <Show when=move || active_tab.get() == "p-overview">
                <div class="grid grid-cols-1 xl:grid-cols-10 gap-6">
                    <div class="xl:col-span-6 space-y-6">
                        // Simulated GMV Bar Chart
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Gross Merchandise Value · June 2026"
                                    <span class="text-[10px] font-normal text-on-surface-variant/60 block mt-0.5">"platform_metrics_daily · metric_key=gmv_cents"</span>
                                </h3>
                                <div class="flex gap-1.5">
                                    <button class="text-[10px] font-bold px-2 py-1 bg-surface-bright text-primary border border-outline-variant rounded" on:click=move |_| toast.show_toast("Chart Action", "Showing daily view", "info")>"Day"</button>
                                    <button class="text-[10px] font-bold px-2 py-1 text-on-surface-variant border border-outline-variant/20 rounded hover:bg-surface-bright/20" on:click=move |_| toast.show_toast("Chart Action", "Showing weekly view", "info")>"Week"</button>
                                    <button class="text-[10px] font-bold px-2 py-1 text-on-surface-variant border border-outline-variant/20 rounded hover:bg-surface-bright/20" on:click=move |_| toast.show_toast("Chart Action", "Showing monthly view", "info")>"Month"</button>
                                </div>
                            </div>
                            <div class="p-6">
                                // SVG Bar chart simulation
                                <div class="h-32 flex items-end gap-1.5 relative border-b border-outline-variant/20 pb-1">
                                    <div class="absolute top-2 left-2 text-[10px] text-on-surface-variant/40 italic">"GMV / day (USD)"</div>
                                    <div class="h-[55%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 1 · $91k"></div>
                                    <div class="h-[60%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 2 · $99k"></div>
                                    <div class="h-[48%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 3 · $79k"></div>
                                    <div class="h-[70%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 4 · $115k"></div>
                                    <div class="h-[80%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 5 · $132k"></div>
                                    <div class="h-[90%] bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 6 · $148k"></div>
                                    <div class="h-[65%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 7 · $107k"></div>
                                    <div class="h-[75%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 8 · $123k"></div>
                                    <div class="h-[82%] bg-primary/70 hover:bg-primary rounded-t w-full transition-all cursor-pointer" title="Jun 9 · $135k"></div>
                                    <div class="h-[60%] bg-primary/50 hover:bg-primary rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[50%] bg-primary/50 hover:bg-primary rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[72%] bg-primary/50 hover:bg-primary rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[85%] bg-primary/50 hover:bg-primary rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[91%] bg-primary/50 hover:bg-primary rounded-t w-full transition-all cursor-pointer"></div>
                                </div>
                            </div>
                        </div>

                        // Simulated Lead Volume Chart
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Lead Volume · Daily"
                                    <span class="text-[10px] font-normal text-on-surface-variant/60 block mt-0.5">"platform_metrics_daily · metric_key=lead_created"</span>
                                </h3>
                            </div>
                            <div class="p-6">
                                <div class="h-20 flex items-end gap-2 relative border-b border-outline-variant/20 pb-1">
                                    <div class="absolute top-2 left-2 text-[10px] text-on-surface-variant/40 italic">"Leads created / day"</div>
                                    <div class="h-[40%] bg-amber-500/70 hover:bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[60%] bg-amber-500/70 hover:bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[80%] bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[50%] bg-amber-500/70 hover:bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[70%] bg-amber-500/70 hover:bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[90%] bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[55%] bg-amber-500/70 hover:bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                    <div class="h-[65%] bg-amber-500/70 hover:bg-amber-400 rounded-t w-full transition-all cursor-pointer"></div>
                                </div>
                            </div>
                        </div>

                        // Conversion Funnel
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "CRM Conversion Funnel · G-31→G-15"
                                </h3>
                                <button class="text-xs text-primary hover:underline font-bold" on:click=move |_| active_tab.set("p-crm".to_string())>"Full Funnel →"</button>
                            </div>
                            <div class="p-5 space-y-4">
                                {
                                    let funnel_row = |stage: &str, count: &str, pct: &str, fill: &str, color_class: &str| {
                                        let stage = stage.to_string();
                                        let count = count.to_string();
                                        let pct = pct.to_string();
                                        let fill = fill.to_string();
                                        let color_class = color_class.to_string();
                                        view! {
                                            <div class="flex items-center gap-4 text-xs">
                                                <span class="w-32 text-on-surface-variant font-medium">{stage}</span>
                                                <div class="flex-1 bg-surface-container h-6 rounded-lg overflow-hidden relative border border-outline-variant/10">
                                                    <div class=format!("h-full rounded-r transition-all {}", color_class) style=format!("width: {}", fill)></div>
                                                    <span class="absolute right-3 top-1/2 -translate-y-1/2 font-bold font-mono text-on-surface">{count}</span>
                                                </div>
                                                <span class="w-12 text-right text-on-surface-variant font-mono">{pct}</span>
                                            </div>
                                        }
                                    };
                                    view! {
                                        {funnel_row("Leads Imported", "47", "100%", "100%", "bg-primary")}
                                        {funnel_row("Contacted", "40", "85%", "85%", "bg-primary/80")}
                                        {funnel_row("Qualifying", "28", "60%", "60%", "bg-primary/70")}
                                        {funnel_row("Qualified", "17", "36%", "36%", "bg-amber-400/80")}
                                        {funnel_row("Converted", "8", "17%", "17%", "bg-emerald-400")}
                                    }
                                }
                            </div>
                        </div>
                    </div>

                    // Right column: revenue breakdown, anomalies, campaigns
                    <div class="xl:col-span-4 space-y-6">
                        // Revenue by app
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Revenue by App · June"
                                </h3>
                            </div>
                            <div class="p-5 space-y-3">
                                {
                                    let rev_row = |name: &str, amt: &str, pct: &str, fill: &str, color_class: &str, dot_color: &str| {
                                        let name = name.to_string();
                                        let amt = amt.to_string();
                                        let pct = pct.to_string();
                                        let fill = fill.to_string();
                                        let color_class = color_class.to_string();
                                        let dot_color = dot_color.to_string();
                                        view! {
                                            <div class="flex items-center gap-3 text-xs">
                                                <span class=format!("w-2 h-2 rounded-full shrink-0 {}", dot_color)></span>
                                                <span class="w-24 text-on-surface-variant font-medium">{name}</span>
                                                <div class="flex-1 bg-surface-container h-2 rounded-full overflow-hidden">
                                                    <div class=format!("h-full rounded-full {}", color_class) style=format!("width: {}", fill)></div>
                                                </div>
                                                <span class=format!("w-16 text-right font-bold {}", color_class)>{amt}</span>
                                                <span class="w-10 text-right text-on-surface-variant/60 font-mono">{pct}</span>
                                            </div>
                                        }
                                    };
                                    view! {
                                        {rev_row("Folio PM", "$1.20M", "56%", "56%", "text-primary", "bg-primary")}
                                        {rev_row("Folio STR", "$480k", "22%", "40%", "text-emerald-400", "bg-emerald-400")}
                                        {rev_row("Anchor", "$220k", "10%", "18%", "text-purple-400", "bg-purple-400")}
                                        {rev_row("Network", "$155k", "7%", "13%", "text-amber-400", "bg-amber-400")}
                                        {rev_row("Other", "$85k", "4%", "8%", "text-cyan-400", "bg-cyan-400")}
                                    }
                                }
                            </div>
                        </div>

                        // Attribution
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Attribution by Channel · G-20"
                                </h3>
                                <button class="text-xs text-primary hover:underline font-bold" on:click=move |_| active_tab.set("p-attribution".to_string())>"Full Report →"</button>
                            </div>
                            <div class="p-5 space-y-3">
                                {
                                    let attr_row = |name: &str, amt: &str, pct: &str, fill: &str, color_class: &str, dot_color: &str| {
                                        let name = name.to_string();
                                        let amt = amt.to_string();
                                        let pct = pct.to_string();
                                        let fill = fill.to_string();
                                        let color_class = color_class.to_string();
                                        let dot_color = dot_color.to_string();
                                        view! {
                                            <div class="flex items-center gap-3 text-xs">
                                                <span class=format!("w-2 h-2 rounded-full shrink-0 {}", dot_color)></span>
                                                <span class="w-28 text-on-surface-variant font-medium">{name}</span>
                                                <div class="flex-1 bg-surface-container h-1.5 rounded-full overflow-hidden">
                                                    <div class=format!("h-full rounded-full {}", color_class) style=format!("width: {}", fill)></div>
                                                </div>
                                                <span class=format!("w-14 text-right font-bold {}", color_class)>{amt}</span>
                                                <span class="w-10 text-right text-on-surface-variant/60 font-mono">{pct}</span>
                                            </div>
                                        }
                                    };
                                    view! {
                                        {attr_row("Direct", "$680k", "32%", "100%", "text-orange-400", "bg-orange-400")}
                                        {attr_row("Organic Search", "$512k", "24%", "75%", "text-primary", "bg-primary")}
                                        {attr_row("Email Campaign", "$278k", "13%", "40%", "text-purple-400", "bg-purple-400")}
                                        {attr_row("FMCSA Import", "$210k", "10%", "30%", "text-emerald-400", "bg-emerald-400")}
                                        {attr_row("Referral", "$156k", "7%", "22%", "text-amber-400", "bg-amber-400")}
                                        {attr_row("Paid Ads", "$112k", "5%", "17%", "text-cyan-400", "bg-cyan-400")}
                                    }
                                }
                            </div>
                        </div>

                        // Scorecard anomalies
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-error">
                                    "⚠ G-27 Anomalies"
                                    <span class="text-[10px] text-on-surface-variant/50 font-normal block mt-0.5">"z-score > 2.0 · atlas_scorecard_time_series"</span>
                                </h3>
                            </div>
                            <div class="divide-y divide-outline-variant/10">
                                <div class="p-4 flex gap-3 text-xs">
                                    <span class="px-2 py-0.5 rounded bg-error-container/20 border border-error/30 text-error text-[10px] font-bold uppercase tracking-wider shrink-0 mt-0.5">"Spike"</span>
                                    <div class="flex-1 min-w-0">
                                        <div class="font-semibold truncate">"Bathroom Cleanliness · Biscayne STR"</div>
                                        <div class="text-[10px] text-on-surface-variant/70 mt-0.5">"Dimension: cleanliness · Session: reservation"</div>
                                    </div>
                                    <span class="font-bold text-error shrink-0">"z = -2.8"</span>
                                </div>
                                <div class="p-4 flex gap-3 text-xs">
                                    <span class="px-2 py-0.5 rounded bg-amber-500/10 border border-amber-500/20 text-amber-400 text-[10px] font-bold uppercase tracking-wider shrink-0 mt-0.5">"Drop"</span>
                                    <div class="flex-1 min-w-0">
                                        <div class="font-semibold truncate">"Response Time · Harbor Media"</div>
                                        <div class="text-[10px] text-on-surface-variant/70 mt-0.5">"Dimension: response_time · 3 consecutive drops"</div>
                                    </div>
                                    <span class="font-bold text-amber-400 shrink-0">"z = -2.1"</span>
                                </div>
                                <div class="p-4 flex gap-3 text-xs">
                                    <span class="px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-[10px] font-bold uppercase tracking-wider shrink-0 mt-0.5">"Surge"</span>
                                    <div class="flex-1 min-w-0">
                                        <div class="font-semibold truncate">"Vendor On-Time Rate · Nexus PM"</div>
                                        <div class="text-[10px] text-on-surface-variant/70 mt-0.5">"Dimension: on_time_delivery · Unusually high period"</div>
                                    </div>
                                    <span class="font-bold text-emerald-400 shrink-0">"z = +2.4"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Revenue & GMV ──
            <Show when=move || active_tab.get() == "p-revenue">
                <div class="space-y-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Revenue Breakdown · platform_metrics_daily · metric_source = ledger"
                            </h3>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-3 divide-y md:divide-y-0 md:divide-x divide-outline-variant/10">
                            <div class="p-5 text-center">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Total GMV"</span>
                                <h4 class="text-2xl font-black text-primary font-mono mt-1">"$2,140,000"</h4>
                                <span class="text-[10px] text-emerald-400">"↑ 18% vs May"</span>
                            </div>
                            <div class="p-5 text-center">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Platform Commission"</span>
                                <h4 class="text-2xl font-black text-emerald-400 font-mono mt-1">"$171,200"</h4>
                                <span class="text-[10px] text-emerald-400">"↑ 14% vs May"</span>
                            </div>
                            <div class="p-5 text-center">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Platform MRR (SaaS)"</span>
                                <h4 class="text-2xl font-black text-primary font-mono mt-1">"$84,000"</h4>
                                <span class="text-[10px] text-emerald-400">"↑ 12% vs May"</span>
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        // LifeCycle
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Subscription Lifecycle & Collection Funnel"
                                </h3>
                            </div>
                            <Suspense fallback=move || view! { <div class="p-6 text-on-surface-variant">"Loading lifecycle statistics..."</div> }>
                                {move || billing_summary.get().map(|res| match res {
                                    Ok(data) => view! {
                                        <div class="divide-y divide-outline-variant/10 text-xs">
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Active Subscriptions"</span>
                                                <span class="font-bold text-primary font-mono">{data.active_subscriptions}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"In Trial Period"</span>
                                                <span class="font-bold text-emerald-400 font-mono">{data.in_trial}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"In Grace Period"</span>
                                                <span class="font-bold text-amber-400 font-mono">{data.in_grace_period}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Suspended (Past Due)"</span>
                                                <span class="font-bold text-error font-mono">{data.suspended}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Canceled (Jun)"</span>
                                                <span class="text-on-surface-variant/60 font-mono">{data.canceled}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Monthly Gross Churn Rate"</span>
                                                <span class="text-error font-bold">{format!("{:.1}%", data.gross_churn_rate)}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Collection Success Rate"</span>
                                                <span class="text-emerald-400 font-bold">{format!("{:.1}%", data.collection_success_rate)}</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Failed Invoices (ACH/Card)"</span>
                                                <span class="text-error font-bold">{format!("{} failures (${:.0} value)", data.failed_invoices_count, data.failed_invoices_value)}</span>
                                            </div>
                                        </div>
                                    }.into_any(),
                                    Err(_) => view! { <div class="p-6 text-error">"Failed to load analytics data"</div> }.into_any()
                                })}
                            </Suspense>
                        </div>

                        // Exemption Override Table
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Active Billing Exemption Overrides (Lost Rev Audit)"
                                </h3>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="w-full text-left border-collapse text-xs">
                                    <thead>
                                        <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/10 bg-surface-container-high/20">
                                            <th class="py-2.5 px-4 font-semibold">"Tenant"</th>
                                            <th class="py-2.5 px-4 font-semibold">"App Instance"</th>
                                            <th class="py-2.5 px-4 font-semibold text-center">"Lost Revenue"</th>
                                            <th class="py-2.5 px-4 font-semibold">"Reason"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/5">
                                        <Suspense fallback=move || view! { <tr><td colspan="4" class="p-4 text-center text-on-surface-variant">"Loading exemptions registry..."</td></tr> }>
                                            {move || billing_summary.get().map(|res| match res {
                                                Ok(data) => view! {
                                                    <For 
                                                        each=move || data.exemptions.clone()
                                                        key=|ex| format!("{}_{}", ex.tenant_name, ex.app_slug)
                                                        children=move |ex| view! {
                                                            <tr class="hover:bg-surface-bright/5 transition-colors">
                                                                <td class="py-3 px-4 font-bold">{ex.tenant_name}</td>
                                                                <td class="py-3 px-4 text-on-surface-variant/70">{ex.app_slug}</td>
                                                                <td class="py-3 px-4 font-bold text-amber-400 font-mono text-center">{ex.lost_revenue}</td>
                                                                <td class="py-3 px-4 text-on-surface-variant/70">{ex.reason}</td>
                                                            </tr>
                                                        }
                                                    />
                                                }.into_any(),
                                                _ => view! { <tr></tr> }.into_any()
                                            })}
                                        </Suspense>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: CRM Funnel ──
            <Show when=move || active_tab.get() == "p-crm">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Full CRM Funnel · G-31 → G-15"
                            </h3>
                        </div>
                        <div class="p-5 space-y-4">
                            {
                                let detail_funnel_row = |stage: &str, count: &str, pct: &str, fill: &str, step_pct: &str, color_class: &str| {
                                    let stage = stage.to_string();
                                    let count = count.to_string();
                                    let pct = pct.to_string();
                                    let fill = fill.to_string();
                                    let step_pct = step_pct.to_string();
                                    let color_class = color_class.to_string();
                                    view! {
                                        <div class="flex items-center gap-4 text-xs">
                                            <span class="w-32 text-on-surface-variant font-medium">{stage}</span>
                                            <div class="flex-1 bg-surface-container h-6 rounded-lg overflow-hidden relative border border-outline-variant/10">
                                                <div class=format!("h-full rounded-r transition-all {}", color_class) style=format!("width: {}", fill)></div>
                                                <span class="absolute right-3 top-1/2 -translate-y-1/2 font-bold font-mono text-on-surface">{count}</span>
                                            </div>
                                            <span class="w-12 text-right text-on-surface-variant font-mono">{pct}</span>
                                            <span class="w-20 text-right font-medium text-emerald-400">{step_pct}</span>
                                        </div>
                                    }
                                };
                                view! {
                                    {detail_funnel_row("Leads Imported (total)", "47", "100%", "100%", "", "bg-primary")}
                                    {detail_funnel_row("Contacted", "40", "85%", "85%", "85% step", "bg-primary/85")}
                                    {detail_funnel_row("Qualifying", "28", "60%", "60%", "70% step", "bg-primary/70")}
                                    {detail_funnel_row("Qualified", "17", "36%", "36%", "61% step", "bg-amber-400/90")}
                                    {detail_funnel_row("Opportunity Created", "12", "26%", "26%", "71% step", "bg-amber-400/70")}
                                    {detail_funnel_row("Proposal Sent", "9", "19%", "19%", "75% step", "bg-amber-400/60")}
                                    {detail_funnel_row("Closed Won", "8", "17%", "17%", "89% step", "bg-emerald-400")}
                                    {detail_funnel_row("Disqualified", "7", "14%", "14%", "", "bg-error/70")}
                                }
                            }
                        </div>
                    </div>

                    <div class="space-y-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Pipeline Summary"
                                </h3>
                            </div>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Open Opportunities"</span>
                                    <span class="font-bold text-primary">"12"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Total Pipeline Value"</span>
                                    <span class="font-bold font-mono text-primary">"$14,200,000"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Weighted Pipeline"</span>
                                    <span class="font-bold font-mono">"$9,230,000"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Avg Deal Size"</span>
                                    <span class="font-bold font-mono">"$1,183,333"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Avg Probability"</span>
                                    <span class="font-bold font-mono text-amber-400">"65%"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Avg Days in Pipeline"</span>
                                    <span class="font-bold text-on-surface-variant/80">"34 days"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant">"Sales Cycle (won)"</span>
                                    <span class="font-bold text-on-surface-variant/80">"22 days avg"</span>
                                </div>
                            </div>
                        </div>

                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Lead Source Performance"
                                </h3>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="w-full text-left border-collapse text-xs">
                                    <thead>
                                        <tr class="border-b border-outline-variant/15 text-[10px] text-on-surface-variant/70 bg-surface-container-high/10">
                                            <th class="py-2.5 px-4 font-semibold">"Source"</th>
                                            <th class="py-2.5 px-4 font-semibold text-center">"Leads"</th>
                                            <th class="py-2.5 px-4 font-semibold text-center">"Converted"</th>
                                            <th class="py-2.5 px-4 font-semibold text-center">"Conv %"</th>
                                            <th class="py-2.5 px-4 font-semibold text-center">"Avg Score"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/5">
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="py-2.5 px-4 font-bold">"FMCSA Import"</td>
                                            <td class="py-2.5 px-4 text-center font-mono">"28"</td>
                                            <td class="py-2.5 px-4 text-center font-mono text-emerald-400">"6"</td>
                                            <td class="py-2.5 px-4 text-center text-emerald-400 font-semibold">"21%"</td>
                                            <td class="py-2.5 px-4 text-center text-[#88cc00] font-bold font-mono">"7.8"</td>
                                        </tr>
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="py-2.5 px-4 font-bold">"Organic"</td>
                                            <td class="py-2.5 px-4 text-center font-mono">"11"</td>
                                            <td class="py-2.5 px-4 text-center font-mono text-emerald-400">"1"</td>
                                            <td class="py-2.5 px-4 text-center text-amber-400 font-semibold">"9%"</td>
                                            <td class="py-2.5 px-4 text-center text-amber-400 font-bold font-mono">"5.9"</td>
                                        </tr>
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="py-2.5 px-4 font-bold">"Referral"</td>
                                            <td class="py-2.5 px-4 text-center font-mono">"5"</td>
                                            <td class="py-2.5 px-4 text-center font-mono text-emerald-400">"1"</td>
                                            <td class="py-2.5 px-4 text-center text-emerald-400 font-semibold">"20%"</td>
                                            <td class="py-2.5 px-4 text-center text-[#88cc00] font-bold font-mono">"8.1"</td>
                                        </tr>
                                        <tr class="hover:bg-surface-bright/5 transition-colors">
                                            <td class="py-2.5 px-4 font-bold">"Event (G-21)"</td>
                                            <td class="py-2.5 px-4 text-center font-mono">"3"</td>
                                            <td class="py-2.5 px-4 text-center font-mono">"0"</td>
                                            <td class="py-2.5 px-4 text-center text-on-surface-variant/40 font-semibold">"0%"</td>
                                            <td class="py-2.5 px-4 text-center text-amber-400 font-bold font-mono">"6.2"</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Attribution ──
            <Show when=move || active_tab.get() == "p-attribution">
                <div class="space-y-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Attribution Touchpoints · atlas_attribution_touchpoints · G-20"
                            </h3>
                            <div class="flex items-center gap-2">
                                <select class="bg-surface-container border border-outline-variant/40 rounded p-1 text-[11px] text-on-surface outline-none cursor-pointer focus:border-primary">
                                    <option>"Last Touch"</option>
                                    <option selected=true>"Linear"</option>
                                    <option>"First Touch"</option>
                                    <option>"Time Decay"</option>
                                </select>
                                <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| toast.show_toast("Attribution Export", "Generating touchpoints report", "success")>"Export"</button>
                            </div>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse text-xs">
                                <thead>
                                    <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/15">
                                        <th class="py-3 px-4 font-semibold">"Channel"</th>
                                        <th class="py-3 px-4 font-semibold">"UTM Source"</th>
                                        <th class="py-3 px-4 font-semibold">"UTM Medium"</th>
                                        <th class="py-3 px-4 font-semibold">"UTM Campaign"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Touchpoints"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Conversions"</th>
                                        <th class="py-3 px-4 font-semibold text-right">"Attributed Rev."</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Conv. Rate"</th>
                                        <th class="py-3 px-4 font-semibold">"Model"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/5 font-mono text-[11px]">
                                    <tr class="hover:bg-surface-bright/5 transition-colors font-sans text-xs">
                                        <td class="py-3 px-4 font-bold text-on-surface">"direct"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/40">"—"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/40">"—"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/40">"—"</td>
                                        <td class="py-3 px-4 text-center font-mono">"4,812"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-mono">"3"</td>
                                        <td class="py-3 px-4 text-right text-primary font-bold font-mono">"$680,000"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-bold">"18%"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/50">"linear"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors font-sans text-xs">
                                        <td class="py-3 px-4 font-bold text-on-surface">"organic_search"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"google"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"organic"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/40">"—"</td>
                                        <td class="py-3 px-4 text-center font-mono">"3,240"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-mono">"4"</td>
                                        <td class="py-3 px-4 text-right text-primary font-bold font-mono">"$512,000"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-bold">"22%"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/50">"linear"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors font-sans text-xs">
                                        <td class="py-3 px-4 font-bold text-on-surface">"email"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"instantly"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"email"</td>
                                        <td class="py-3 px-4 font-semibold text-primary">"fmcsa_outreach_jun"</td>
                                        <td class="py-3 px-4 text-center font-mono">"1,880"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-mono">"2"</td>
                                        <td class="py-3 px-4 text-right text-primary font-bold font-mono">"$278,000"</td>
                                        <td class="py-3 px-4 text-center text-amber-400 font-bold">"11%"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/50">"linear"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors font-sans text-xs">
                                        <td class="py-3 px-4 font-bold text-on-surface">"import"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"fmcsa"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"import"</td>
                                        <td class="py-3 px-4 font-semibold text-primary">"fmcsa_mc_batch_1"</td>
                                        <td class="py-3 px-4 text-center font-mono">"920"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-mono">"6"</td>
                                        <td class="py-3 px-4 text-right text-primary font-bold font-mono">"$210,000"</td>
                                        <td class="py-3 px-4 text-center text-emerald-400 font-bold">"21%"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/50">"linear"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Top Referrer URLs"
                                </h3>
                            </div>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium">"fmcsa.dot.gov"</span>
                                    <span class="font-bold text-primary font-mono">"28 leads · 6 conv."</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium">"google.com (organic)"</span>
                                    <span class="font-bold font-mono">"4,100 sessions"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium">"instantly.ai (email)"</span>
                                    <span class="font-bold font-mono">"1,880 clicks"</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium">"linkedin.com"</span>
                                    <span class="font-bold font-mono">"340 sessions"</span>
                                </div>
                            </div>
                        </div>

                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Top Landing Pages"
                                </h3>
                            </div>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium font-mono">"/fleet-management"</span>
                                    <span class="font-bold text-primary font-mono">"1,240 sessions · 4 conv."</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium font-mono">"/property-management"</span>
                                    <span class="font-bold font-mono">"820 sessions · 2 conv."</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium font-mono">"/str-compliance"</span>
                                    <span class="font-bold font-mono">"560 sessions · 1 conv."</span>
                                </div>
                                <div class="flex justify-between items-center px-5 py-3">
                                    <span class="text-on-surface-variant font-medium font-mono">"/pricing"</span>
                                    <span class="font-bold font-mono">"440 sessions · 0 conv."</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Campaigns ──
            <Show when=move || active_tab.get() == "p-campaigns">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                    <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                            "Active Campaigns Registry"
                        </h3>
                        <button class="btn-primary-gradient px-3 py-1 rounded text-xs font-semibold text-on-primary-container" on:click=move |_| show_campaign_modal.set(true)>"+ New Campaign"</button>
                    </div>
                    <div class="overflow-x-auto">
                        <table class="w-full text-left border-collapse text-xs">
                            <thead>
                                <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/15">
                                    <th class="py-3 px-4 font-semibold">"Name"</th>
                                    <th class="py-3 px-4 font-semibold">"Type"</th>
                                    <th class="py-3 px-4 font-semibold">"Status"</th>
                                    <th class="py-3 px-4 font-semibold">"Goal"</th>
                                    <th class="py-3 px-4 font-semibold text-right">"Budget"</th>
                                    <th class="py-3 px-4 font-semibold text-right">"Spent"</th>
                                    <th class="py-3 px-4 font-semibold text-center">"Conversions"</th>
                                    <th class="py-3 px-4 font-semibold text-center">"Conv. Rate"</th>
                                    <th class="py-3 px-4 font-semibold text-right">"Attr. Rev."</th>
                                    <th class="py-3 px-4 font-semibold text-center">"Window"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/5 font-sans">
                                <tr class="hover:bg-surface-bright/5 transition-colors">
                                    <td class="py-3 px-4 font-bold text-primary hover:underline cursor-pointer">"FMCSA Outreach Jun"</td>
                                    <td class="py-3 px-4"><span class="px-2 py-0.5 rounded bg-primary/10 border border-primary/20 text-primary text-[9px] font-bold uppercase">"email"</span></td>
                                    <td class="py-3 px-4"><span class="text-emerald-400 font-semibold">"active"</span></td>
                                    <td class="py-3 px-4 text-on-surface-variant/70">"lead_capture"</td>
                                    <td class="py-3 px-4 text-right font-mono">$4,800</td>
                                    <td class="py-3 px-4 text-right font-mono text-amber-400">$3,120</td>
                                    <td class="py-3 px-4 text-center font-mono text-emerald-400">"6"</td>
                                    <td class="py-3 px-4 text-center text-emerald-400 font-bold">"21%"</td>
                                    <td class="py-3 px-4 text-right text-primary font-bold font-mono">"$210k"</td>
                                    <td class="py-3 px-4 text-center text-on-surface-variant/50 font-mono">"30d"</td>
                                </tr>
                                <tr class="hover:bg-surface-bright/5 transition-colors">
                                    <td class="py-3 px-4 font-bold text-primary hover:underline cursor-pointer">"PM Expansion Q2 Paid"</td>
                                    <td class="py-3 px-4"><span class="px-2 py-0.5 rounded bg-purple-500/10 border border-purple-500/20 text-purple-400 text-[9px] font-bold uppercase">"paid"</span></td>
                                    <td class="py-3 px-4"><span class="text-emerald-400 font-semibold">"active"</span></td>
                                    <td class="py-3 px-4 text-on-surface-variant/70">"lead_capture"</td>
                                    <td class="py-3 px-4 text-right font-mono">$10,000</td>
                                    <td class="py-3 px-4 text-right font-mono text-error">$9,840</td>
                                    <td class="py-3 px-4 text-center font-mono text-error">"0"</td>
                                    <td class="py-3 px-4 text-center text-error font-bold">"0%"</td>
                                    <td class="py-3 px-4 text-right text-on-surface-variant/40 font-mono">$0</td>
                                    <td class="py-3 px-4 text-center text-on-surface-variant/50 font-mono">"14d"</td>
                                </tr>
                                <tr class="hover:bg-surface-bright/5 transition-colors">
                                    <td class="py-3 px-4 font-bold text-primary hover:underline cursor-pointer">"Miami PM Summit"</td>
                                    <td class="py-3 px-4"><span class="px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-[9px] font-bold uppercase">"event"</span></td>
                                    <td class="py-3 px-4"><span class="text-on-surface-variant/50">"ended"</span></td>
                                    <td class="py-3 px-4 text-on-surface-variant/70">"registration"</td>
                                    <td class="py-3 px-4 text-right font-mono">$2,000</td>
                                    <td class="py-3 px-4 text-right font-mono">$2,000</td>
                                    <td class="py-3 px-4 text-center font-mono text-amber-400">"0"</td>
                                    <td class="py-3 px-4 text-center text-amber-400 font-bold">"0%"</td>
                                    <td class="py-3 px-4 text-right text-on-surface-variant/40 font-mono">$0</td>
                                    <td class="py-3 px-4 text-center text-on-surface-variant/50 font-mono">"7d"</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Scorecard trends ──
            <Show when=move || active_tab.get() == "p-scorecards">
                <div class="space-y-6">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "G-27 Scorecard Time Series · atlas_scorecard_time_series"
                                <span class="text-[10px] text-on-surface-variant/60 font-normal block mt-0.5">"Hourly refresh · Anomaly threshold: |z| > 2.0 · Trend threshold: Δ ±0.3"</span>
                            </h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse text-xs">
                                <thead>
                                    <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/15">
                                        <th class="py-3 px-4 font-semibold">"Scorecard"</th>
                                        <th class="py-3 px-4 font-semibold">"Dimension"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Apr"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"May"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Jun"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Δ MoM"</th>
                                        <th class="py-3 px-4 font-semibold">"Trend"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Z-Score"</th>
                                        <th class="py-3 px-4 font-semibold">"Anomaly"</th>
                                        <th class="py-3 px-4 font-semibold text-center">"Sessions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/5">
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-4 font-bold">"Logística Meridional"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"Legal / Blockers"</td>
                                        <td class="py-3 px-4 text-center font-mono text-on-surface-variant/50">"—"</td>
                                        <td class="py-3 px-4 text-center font-mono">"6.2"</td>
                                        <td class="py-3 px-4 text-center font-mono text-[#88cc00] font-bold">"7.0"</td>
                                        <td class="py-3 px-4 text-center font-bold text-emerald-400">"+0.8"</td>
                                        <td class="py-3 px-4 text-emerald-400 font-semibold">"↑ improving"</td>
                                        <td class="py-3 px-4 text-center font-mono">+1.1</td>
                                        <td class="py-3 px-4 text-on-surface-variant/40">"—"</td>
                                        <td class="py-3 px-4 text-center text-on-surface-variant/50 font-mono">"3"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-3 px-4 font-bold">"Biscayne STR · Unit 12"</td>
                                        <td class="py-3 px-4 text-on-surface-variant/70">"Cleanliness"</td>
                                        <td class="py-3 px-4 text-center font-mono">"8.4"</td>
                                        <td class="py-3 px-4 text-center font-mono">"8.1"</td>
                                        <td class="py-3 px-4 text-center font-mono text-error font-bold">"6.2"</td>
                                        <td class="py-3 px-4 text-center font-bold text-error">"-1.9"</td>
                                        <td class="py-3 px-4 text-error font-semibold">"↓ declining"</td>
                                        <td class="py-3 px-4 text-center font-mono text-error">-2.8</td>
                                        <td class="py-3 px-4"><span class="px-2 py-0.5 rounded bg-error-container/20 border border-error/30 text-error text-[10px] font-bold uppercase tracking-wider">"Spike (drop)"</span></td>
                                        <td class="py-3 px-4 text-center text-on-surface-variant/50 font-mono">"7"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Raw Metrics ──
            <Show when=move || active_tab.get() == "p-platform">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                    <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                            "platform_metrics_daily · Raw Metric Viewer"
                        </h3>
                    </div>
                    <div class="divide-y divide-outline-variant/10 text-xs">
                        {
                            let raw_row = |source: &str, date: &str, key: &str, val: &str, fill: &str, bar_color: &str| {
                                let source = source.to_string();
                                let date = date.to_string();
                                let key = key.to_string();
                                let val = val.to_string();
                                let fill = fill.to_string();
                                let bar_color = bar_color.to_string();
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-4 items-center p-4 gap-4 hover:bg-surface-bright/5 transition-colors">
                                        <div class="font-bold text-[10px] uppercase tracking-wider text-on-surface-variant/70">{source}</div>
                                        <div class="text-on-surface-variant/50 font-mono">{date}</div>
                                        <div class="flex items-center gap-4 md:col-span-2">
                                            <div class="flex-1 bg-surface-container h-1 rounded-full overflow-hidden">
                                                <div class=format!("h-full rounded-full {}", bar_color) style=format!("width: {}", fill)></div>
                                            </div>
                                            <div class="w-48 truncate font-mono text-on-surface-variant">{key}</div>
                                            <div class="w-24 text-right font-bold font-mono">{val}</div>
                                        </div>
                                    </div>
                                }
                            };
                            view! {
                                {raw_row("ledger", "Jun 10", "gmv_cents", "$214,000", "100%", "bg-primary")}
                                {raw_row("ledger", "Jun 10", "platform_commission_cents", "$17,120", "14%", "bg-emerald-400")}
                                {raw_row("crm", "Jun 10", "leads_created", "3", "40%", "bg-amber-400")}
                                {raw_row("api", "Jun 10", "api_requests_total", "88,420", "85%", "bg-primary")}
                                {raw_row("api", "Jun 10", "api_error_rate_pct", "0.38%", "4%", "bg-error")}
                            }
                        }
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: API Logs ──
            <Show when=move || active_tab.get() == "p-api">
                <div class="space-y-6">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h4 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Requests (24h)"</h4>
                            </div>
                            <div class="p-4 grid grid-cols-2 gap-4 text-center">
                                <div class="p-3 bg-surface-container rounded-lg">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/70 block">"Total"</span>
                                    <span class="text-lg font-bold font-mono">"2.1M"</span>
                                </div>
                                <div class="p-3 bg-surface-container rounded-lg">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/70 block">"Error Rate"</span>
                                    <span class="text-lg font-bold font-mono text-error">"0.4%"</span>
                                </div>
                            </div>
                        </div>
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h4 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Latency"</h4>
                            </div>
                            <div class="p-4 grid grid-cols-3 gap-2 text-center text-xs">
                                <div class="p-2 bg-surface-container rounded">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/50 block">"p50"</span>
                                    <span class="font-bold font-mono">"22ms"</span>
                                </div>
                                <div class="p-2 bg-surface-container rounded">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/50 block">"p95"</span>
                                    <span class="font-bold font-mono">"48ms"</span>
                                </div>
                                <div class="p-2 bg-surface-container rounded">
                                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/50 block">"p99"</span>
                                    <span class="font-bold font-mono text-amber-400">"84ms"</span>
                                </div>
                            </div>
                        </div>
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h4 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Request Types"</h4>
                            </div>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center px-4 py-2">
                                    <span class="text-on-surface-variant">"API"</span>
                                    <span class="font-mono font-bold">"1,840,000"</span>
                                </div>
                                <div class="flex justify-between items-center px-4 py-2">
                                    <span class="text-on-surface-variant">"HTML Page"</span>
                                    <span class="font-mono font-bold">"220,000"</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Errors table
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-error">
                                "Recent Errors · request_log · status_code ≥ 400"
                            </h3>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse text-xs">
                                <thead>
                                    <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/15">
                                        <th class="py-3 px-4 font-semibold text-center">"Status"</th>
                                        <th class="py-3 px-4 font-semibold">"Method"</th>
                                        <th class="py-3 px-4 font-semibold">"Path"</th>
                                        <th class="py-3 px-4 font-semibold">"User / IP"</th>
                                        <th class="py-3 px-4 font-semibold">"Failure Reason"</th>
                                        <th class="py-3 px-4 font-semibold text-right">"Time"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/5 font-mono text-[11px]">
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-2.5 px-4 text-center font-bold text-error">"500"</td>
                                        <td class="py-2.5 px-4 font-bold text-on-surface">"POST"</td>
                                        <td class="py-2.5 px-4 text-on-surface-variant">"/api/v1/ledger/entries"</td>
                                        <td class="py-2.5 px-4 text-on-surface-variant/70">"10.0.1.4"</td>
                                        <td class="py-2.5 px-4 text-error">"DB timeout: ledger_entries write"</td>
                                        <td class="py-2.5 px-4 text-right text-on-surface-variant/50">"2m ago"</td>
                                    </tr>
                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                        <td class="py-2.5 px-4 text-center font-bold text-amber-400">"404"</td>
                                        <td class="py-2.5 px-4 font-bold text-on-surface">"GET"</td>
                                        <td class="py-2.5 px-4 text-on-surface-variant">"/api/v1/tenants/t_xxx/assets/missing"</td>
                                        <td class="py-2.5 px-4 text-on-surface-variant/70">"usr_abc"</td>
                                        <td class="py-2.5 px-4 text-amber-400">"Asset not found"</td>
                                        <td class="py-2.5 px-4 text-right text-on-surface-variant/50">"8m ago"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </Show>

            // ── NEW CAMPAIGN DIALOG MODAL ──
            <Show when=move || show_campaign_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-lg p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_campaign_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"New Marketing Campaign"</h3>
                        <p class="text-on-surface-variant text-xs mb-6">"Provision a new marketing target inside the campaigns scheduler database tracker."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Campaign Name"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary" 
                                        placeholder="e.g. FMCSA Outreach Jul"
                                        on:input=move |ev| campaign_name.set(event_target_value(&ev))
                                        prop:value=campaign_name
                                    />
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Campaign Type"</label>
                                    <select 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary"
                                        on:change=move |ev| campaign_type.set(event_target_value(&ev))
                                        prop:value=campaign_type
                                    >
                                        <option value="email">"email"</option>
                                        <option value="paid">"paid"</option>
                                        <option value="event">"event"</option>
                                        <option value="referral">"referral"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Campaign Goal"</label>
                                    <select 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none cursor-pointer focus:border-primary"
                                        on:change=move |ev| campaign_goal.set(event_target_value(&ev))
                                        prop:value=campaign_goal
                                    >
                                        <option value="lead_capture">"lead_capture"</option>
                                        <option value="booking">"booking"</option>
                                        <option value="registration">"registration"</option>
                                    </select>
                                </div>
                                <div class="space-y-1.5">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Budget (cents)"</label>
                                    <input 
                                        type="number" 
                                        class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary" 
                                        placeholder="500000"
                                        on:input=move |ev| campaign_budget.set(event_target_value(&ev))
                                        prop:value=campaign_budget
                                    />
                                </div>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_campaign_modal.set(false)>"Cancel"</button>
                            <button 
                                class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold text-on-primary-container"
                                on:click=move |_| {
                                    show_campaign_modal.set(false);
                                    toast.show_toast("Success", &format!("Campaign '{}' provisioned successfully.", campaign_name.get()), "success");
                                    campaign_name.set(String::new());
                                }
                            >
                                "Create Campaign"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
