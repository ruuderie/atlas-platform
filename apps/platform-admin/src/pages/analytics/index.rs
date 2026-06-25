use leptos::prelude::*;
use crate::api::analytics::{get_business_kpis, get_billing_summary};
use crate::api::admin::{get_tenant_stats, create_campaign, CreateCampaignInput};

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
    let tenant_list = LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    view! {
        <div class="space-y-6">
            // ── Page Header ──
            <div class="flex justify-between items-center border-b border-outline-variant/20 pb-4">
                <div>
                    <h1 class="text-3xl font-extrabold tracking-tight text-on-surface">"Analytics"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">
                        "Platform-wide metrics and attribution analysis"
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
                        {move || tenant_list.get().unwrap_or_default().into_iter().map(|t| {
                            let n = t.name.clone();
                            view! { <option value=n.clone()>{n.clone()}</option> }
                        }).collect_view()}
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
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 xl:grid-cols-7 gap-4 bg-surface-container-low border border-outline-variant/15 p-4 rounded-xl shadow-inner overflow-x-auto shrink-0 select-none">
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Platform MRR"</span>
                    <span class="text-lg font-black text-primary">
                        {move || match business_kpis.get() {
                            Some(Ok(kpis)) => format!("${:.0}k", kpis.mrr.value / 1000.0),
                            Some(Err(_)) => "Err".to_string(),
                            None => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">
                        {move || match business_kpis.get() {
                            Some(Ok(kpis)) if kpis.mrr.previous_value > 0.0 => {
                                let pct = ((kpis.mrr.value - kpis.mrr.previous_value) / kpis.mrr.previous_value) * 100.0;
                                format!("{:+.0}% vs prev", pct)
                            },
                            _ => "vs prev period".to_string(),
                        }}
                    </span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Active Subscriptions"</span>
                    <span class="text-lg font-black font-mono">
                        {move || match billing_summary.get() {
                            Some(Ok(s)) => s.active_subscriptions.to_string(),
                            _ => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"Active plans"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"In Trial"</span>
                    <span class="text-lg font-black font-mono text-emerald-400">
                        {move || match billing_summary.get() {
                            Some(Ok(s)) => s.in_trial.to_string(),
                            _ => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"Trial period"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Failed Invoices"</span>
                    <span class="text-lg font-black font-mono text-amber-400">
                        {move || match billing_summary.get() {
                            Some(Ok(s)) => s.failed_invoices_count.to_string(),
                            _ => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"Outstanding"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Collection Rate"</span>
                    <span class="text-lg font-black font-mono text-emerald-400">
                        {move || match billing_summary.get() {
                            Some(Ok(s)) => format!("{:.0}%", s.collection_success_rate * 100.0),
                            _ => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"G-03"</span>
                </div>
                <div class="flex flex-col gap-1 p-2 border-r border-outline-variant/10">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Churn Rate"</span>
                    <span class="text-lg font-black font-mono">
                        {move || match billing_summary.get() {
                            Some(Ok(s)) => format!("{:.1}%", s.gross_churn_rate * 100.0),
                            _ => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"Monthly gross"</span>
                </div>
                <div class="flex flex-col gap-1 p-2">
                    <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Suspended"</span>
                    <span class="text-lg font-black font-mono text-error">
                        {move || match billing_summary.get() {
                            Some(Ok(s)) => s.suspended.to_string(),
                            _ => "—".to_string(),
                        }}
                    </span>
                    <span class="text-[9.5px] text-on-surface-variant/70">"Past due"</span>
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
                    {tab_btn("p-bi", "Platform BI · Revenue")}
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

            <Show when=move || active_tab.get() == "p-overview">
                <div class="grid grid-cols-1 xl:grid-cols-10 gap-6">
                    <div class="xl:col-span-6 space-y-6">
                        // GMV chart — pending real time-series endpoint
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Gross Merchandise Value · Trend"
                                </h3>
                            </div>
                            <div class="p-10 flex flex-col items-center justify-center gap-3 text-center">
                                <svg viewBox="0 0 40 40" width="36" height="36" fill="none" stroke="currentColor" stroke-width="1.5" style="color:var(--text-muted);opacity:.5">
                                    <path d="M4 30 L12 20 L20 24 L28 14 L36 10"/>
                                    <rect x="2" y="2" width="36" height="36" rx="3" stroke-dasharray="3 2"/>
                                </svg>
                                <span class="text-sm font-semibold text-on-surface-variant">"GMV time-series pending"</span>
                                <span class="text-[11px] text-on-surface-variant/60 max-w-xs">
                                    "Requires the platform_metrics_daily analytics endpoint. Connect the trends API to populate this chart."
                                </span>
                                <button class="text-[11px] text-primary hover:underline font-bold mt-1" on:click=move |_| active_tab.set("p-revenue".to_string())>"→ View Revenue Breakdown"</button>
                            </div>
                        </div>

                        // CRM Funnel — driven from real billing_summary data
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Subscription Lifecycle · G-03"
                                </h3>
                            </div>
                            <div class="p-5 space-y-3">
                                <Suspense fallback=move || view! { <div class="py-6 text-center text-on-surface-variant/50 text-xs">"Loading..."</div> }>
                                {move || billing_summary.get().map(|res| match res {
                                    Ok(data) => view! {
                                        <div class="space-y-3">
                                            {
                                                let total = (data.active_subscriptions + data.in_trial + data.in_grace_period + data.suspended + data.canceled).max(1);
                                                let rows = vec![
                                                    ("Active", data.active_subscriptions, "bg-emerald-400"),
                                                    ("In Trial", data.in_trial, "bg-primary"),
                                                    ("Grace Period", data.in_grace_period, "bg-amber-400"),
                                                    ("Suspended", data.suspended, "bg-error"),
                                                    ("Canceled", data.canceled, "bg-surface-container"),
                                                ];
                                                rows.into_iter().map(|(label, count, bar_class)| {
                                                    let pct = (count as f64 / total as f64) * 100.0;
                                                    let fill = format!("{}%", pct as u32);
                                                    view! {
                                                        <div class="flex items-center gap-4 text-xs">
                                                            <span class="w-28 text-on-surface-variant font-medium">{label}</span>
                                                            <div class="flex-1 bg-surface-container h-5 rounded-lg overflow-hidden relative border border-outline-variant/10">
                                                                <div class=format!("h-full rounded-r transition-all {}", bar_class) style=format!("width: {}", fill)></div>
                                                                <span class="absolute right-3 top-1/2 -translate-y-1/2 font-bold font-mono text-on-surface">{count.to_string()}</span>
                                                            </div>
                                                            <span class="w-10 text-right text-on-surface-variant font-mono">{format!("{:.0}%", pct)}</span>
                                                        </div>
                                                    }
                                                }).collect_view()
                                            }
                                        </div>
                                    }.into_any(),
                                    Err(_) => view! {
                                        <div class="py-4 text-center text-error text-xs">"Failed to load subscription data"</div>
                                    }.into_any()
                                })}
                                </Suspense>
                            </div>
                        </div>
                    </div>

                    // Right column
                    <div class="xl:col-span-4 space-y-6">
                        // Revenue breakdown — pending real product-level endpoint
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Revenue by Product"
                                </h3>
                            </div>
                            <div class="p-8 flex flex-col items-center gap-2 text-center">
                                <span class="text-sm font-semibold text-on-surface-variant">"Product-level revenue pending"</span>
                                <span class="text-[11px] text-on-surface-variant/60 max-w-xs">
                                    "Requires a per-product revenue endpoint from the billing ledger splits API."
                                </span>
                                <a href="/billing" class="text-[11px] text-primary hover:underline font-bold mt-1" style="text-decoration:none">"→ Open Billing Ledger"</a>
                            </div>
                        </div>

                        // Billing summary card
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Collection Health"
                                </h3>
                            </div>
                            <Suspense fallback=move || view! { <div class="p-4 text-center text-on-surface-variant/50 text-xs">"Loading..."</div> }>
                            {move || billing_summary.get().map(|res| match res {
                                Ok(data) => view! {
                                    <div class="divide-y divide-outline-variant/10 text-xs">
                                        <div class="flex justify-between items-center px-5 py-3">
                                            <span class="text-on-surface-variant">"Collection Success Rate"</span>
                                            <span class="text-emerald-400 font-bold">{format!("{:.1}%", data.collection_success_rate * 100.0)}</span>
                                        </div>
                                        <div class="flex justify-between items-center px-5 py-3">
                                            <span class="text-on-surface-variant">"Monthly Gross Churn"</span>
                                            <span class="text-error font-bold">{format!("{:.1}%", data.gross_churn_rate * 100.0)}</span>
                                        </div>
                                        <div class="flex justify-between items-center px-5 py-3">
                                            <span class="text-on-surface-variant">"Failed Invoices"</span>
                                            <span class="text-amber-400 font-bold font-mono">{format!("{} (${:.0}k)", data.failed_invoices_count, data.failed_invoices_value / 1000.0)}</span>
                                        </div>
                                        <div class="flex justify-between items-center px-5 py-3">
                                            <span class="text-on-surface-variant">"In Grace Period"</span>
                                            <span class="text-amber-400 font-bold font-mono">{data.in_grace_period.to_string()}</span>
                                        </div>
                                    </div>
                                }.into_any(),
                                Err(_) => view! { <div class="p-4 text-error text-xs text-center">"Failed to load"</div> }.into_any()
                            })}
                            </Suspense>
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
                        <Suspense fallback=move || view! { <div class="p-6 text-on-surface-variant text-xs">"Loading revenue data…"</div> }>
                            {move || business_kpis.get().map(|res| match res {
                                Ok(kpis) => view! {
                                    <div class="grid grid-cols-1 md:grid-cols-3 divide-y md:divide-y-0 md:divide-x divide-outline-variant/10">
                                        <div class="p-5 text-center">
                                            <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Platform MRR (SaaS)"</span>
                                            <h4 class="text-2xl font-black text-primary font-mono mt-1">
                                                {format!("${:.0}k", kpis.mrr.value / 1000.0)}
                                            </h4>
                                            <span class="text-[10px] text-emerald-400">
                                                {move || {
                                                    let pct = if kpis.mrr.previous_value > 0.0 {
                                                        ((kpis.mrr.value - kpis.mrr.previous_value) / kpis.mrr.previous_value) * 100.0
                                                    } else { 0.0 };
                                                    format!("{:+.0}% vs prev period", pct)
                                                }}
                                            </span>
                                        </div>
                                        <div class="p-5 text-center">
                                            <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Active Subscriptions"</span>
                                            <h4 class="text-2xl font-black text-emerald-400 font-mono mt-1">
                                                {format!("{:.0}", kpis.active_subscriptions.value)}
                                            </h4>
                                            <span class="text-[10px] text-on-surface-variant/60">"from billing_summary"
                                            </span>
                                        </div>
                                        <div class="p-5 text-center">
                                            <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70">"Total GMV / Commission"</span>
                                            <h4 class="text-2xl font-black text-on-surface-variant/40 font-mono mt-1">"—"</h4>
                                            <span class="text-[10px] text-on-surface-variant/50">"Pending platform_metrics_daily endpoint"</span>
                                        </div>
                                    </div>
                                }.into_any(),
                                Err(_) => view! {
                                    <div class="p-5 text-xs text-error">"Failed to load revenue metrics."</div>
                                }.into_any(),
                            })}
                        </Suspense>
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
                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                "Full CRM Funnel · G-31 → G-15"
                            </h3>
                            <span class="text-[9px] font-bold uppercase tracking-wider text-amber-400/80 bg-amber-400/10 border border-amber-400/20 px-2 py-0.5 rounded">"Static — Pending get_crm_pipeline()"</span>
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
                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                    "Pipeline Summary"
                                </h3>
                                <span class="text-[9px] font-bold uppercase tracking-wider text-amber-400/80 bg-amber-400/10 border border-amber-400/20 px-2 py-0.5 rounded">"Static — Pending get_crm_pipeline()"</span>
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
                                    <tr>
                                        <td colspan="10" class="py-8 px-4 text-center">
                                            <div class="inline-flex items-center gap-2 text-[10px] font-bold uppercase tracking-wider text-amber-400/80 bg-amber-400/10 border border-amber-400/20 px-3 py-1.5 rounded">
                                                "Static — Pending get_scorecard_analytics() endpoint"
                                            </div>
                                            <p class="text-[10px] text-on-surface-variant/50 mt-2">"Scorecard anomaly data will populate here once the platform_metrics_daily aggregate is wired."</p>
                                        </td>
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
                                    let name = campaign_name.get();
                                    let ctype = campaign_type.get();
                                    let goal = campaign_goal.get();
                                    let budget = campaign_budget.get().parse::<i64>().unwrap_or(0);
                                    if name.is_empty() { return; }
                                    let t_toast = toast.clone();
                                    show_campaign_modal.set(false);
                                    campaign_name.set(String::new());
                                    leptos::task::spawn_local(async move {
                                        match create_campaign(CreateCampaignInput {
                                            name: name.clone(),
                                            campaign_type: ctype,
                                            goal,
                                            budget_cents: budget,
                                        }).await {
                                            Ok(_) => t_toast.show_toast("Campaign Created", &format!("Campaign '{}' provisioned successfully.", name), "success"),
                                            Err(e) => t_toast.show_toast("Error", &format!("Failed to create campaign: {}", e), "error"),
                                        }
                                    });
                                }
                            >
                                "Create Campaign"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
            // ── TAB CONTENT: Platform BI ─────────────────────────────────────
            <Show when=move || active_tab.get() == "p-bi">
                <div class="space-y-6">
                    // BI Header
                    <div class="flex items-center gap-3 p-4 bg-primary/10 border border-primary/20 rounded-xl text-xs text-on-surface-variant leading-relaxed">
                        <span class="material-symbols-outlined text-primary text-base">"insights"</span>
                        <span>
                            <strong class="text-on-surface">"Platform Business Intelligence"</strong>
                            " — MRR breakdown by plan tier, subscription health cohort, and tenant ranking. "
                            "Data sourced from live tenant registry and billing summary."
                        </span>
                    </div>

                    // Subscription health funnel (from billing_summary)
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Subscription Lifecycle · Health Funnel"</h3>
                        </div>
                        <div class="p-5 space-y-3">
                            <Suspense fallback=move || view! { <div class="py-4 text-center text-xs text-on-surface-variant/50">"Loading..."</div> }>
                            {move || billing_summary.get().map(|res| match res {
                                Ok(data) => view! {
                                    <div class="space-y-3">
                                        {{
                                            let total = (data.active_subscriptions + data.in_trial + data.in_grace_period + data.suspended + data.canceled).max(1);
                                            let stages = vec![
                                                ("Active",       data.active_subscriptions, "#22c55e"),
                                                ("In Trial",     data.in_trial,             "#818cf8"),
                                                ("Grace Period", data.in_grace_period,      "#f59e0b"),
                                                ("Suspended",    data.suspended,            "#ef4444"),
                                                ("Canceled",     data.canceled,             "#6b7280"),
                                            ];
                                            stages.into_iter().map(|(label, count, color)| {
                                                let pct = (count as f64 / total as f64 * 100.0) as u32;
                                                view! {
                                                    <div style="display:flex;flex-direction:column;gap:4px;">
                                                        <div style="display:flex;align-items:center;justify-content:space-between;font-size:11px;">
                                                            <div style="display:flex;align-items:center;gap:7px;">
                                                                <span style=format!("display:inline-block;width:8px;height:8px;border-radius:50%;background:{};", color)></span>
                                                                <span>{label}</span>
                                                            </div>
                                                            <span style="font-family:monospace;color:var(--text-muted);">
                                                                {format!("{} ({pct}%)" , count)}
                                                            </span>
                                                        </div>
                                                        <div style="height:5px;background:rgba(255,255,255,0.06);border-radius:3px;">
                                                            <div style=format!("height:5px;border-radius:3px;width:{}%;background:{};", pct, color)></div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>
                                }.into_any(),
                                Err(_) => view! {
                                    <p class="text-xs text-on-surface-variant/50 py-4 text-center">"Billing summary unavailable."</p>
                                }.into_any(),
                            })}
                            </Suspense>
                        </div>
                    </div>

                    // MRR by tier + tenant cohort (from tenant_list)
                    <Suspense fallback=move || view! { <div class="p-6 text-center text-xs text-on-surface-variant/50">"Loading tenant data..."</div> }>
                    {move || tenant_list.get().map(|tenants| {
                        let total = tenants.len().max(1);

                        let enterprise: Vec<_> = tenants.iter().filter(|t| t.plan.as_deref().map(|p| p.to_lowercase().contains("enterprise")).unwrap_or(false)).collect();
                        let growth: Vec<_>     = tenants.iter().filter(|t| t.plan.as_deref().map(|p| p.to_lowercase().contains("growth")).unwrap_or(false)).collect();
                        let starter: Vec<_>    = tenants.iter().filter(|t| t.plan.as_deref().map(|p| {
                            let p = p.to_lowercase(); p.contains("starter") || p.contains("basic") || p.contains("free")
                        }).unwrap_or(false)).collect();

                        let e_mrr: i64 = enterprise.iter().filter_map(|t| t.mrr_cents).sum();
                        let g_mrr: i64 = growth.iter().filter_map(|t| t.mrr_cents).sum();
                        let s_mrr: i64 = starter.iter().filter_map(|t| t.mrr_cents).sum();
                        let total_mrr = (e_mrr + g_mrr + s_mrr).max(1);

                        let tier_rows = vec![
                            ("Enterprise", enterprise.len(), e_mrr, "#818cf8"),
                            ("Growth",     growth.len(),     g_mrr, "#34d399"),
                            ("Starter",    starter.len(),    s_mrr, "#fbbf24"),
                        ];

                        view! {
                            <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
                                // MRR by tier breakdown
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                    <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20">
                                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"MRR by Plan Tier"</h3>
                                        <span class="text-[10px] text-on-surface-variant/60">{format!("${}/mo total", total_mrr / 100)}</span>
                                    </div>
                                    <div class="p-5 space-y-4">
                                        {tier_rows.into_iter().map(|(name, count, mrr, color)| {
                                            let mrr_pct = (mrr as f64 / total_mrr as f64 * 100.0) as u32;
                                            let count_pct = (count as f64 / total as f64 * 100.0) as u32;
                                            view! {
                                                <div style="display:flex;flex-direction:column;gap:5px;">
                                                    <div style="display:flex;justify-content:space-between;font-size:12px;">
                                                        <div style="display:flex;align-items:center;gap:7px;">
                                                            <span style=format!("width:9px;height:9px;border-radius:2px;background:{};display:inline-block;", color)></span>
                                                            <span style="font-weight:600;">{name}</span>
                                                            <span style="color:var(--text-muted);font-size:10px;">{format!("{} tenants ({}%)", count, count_pct)}</span>
                                                        </div>
                                                        <span style="font-family:monospace;font-weight:700;">
                                                            {format!("${}/mo · {}%", mrr / 100, mrr_pct)}
                                                        </span>
                                                    </div>
                                                    <div style="height:6px;background:rgba(255,255,255,0.06);border-radius:3px;">
                                                        <div style=format!("height:6px;border-radius:3px;width:{}%;background:{};", mrr_pct, color)></div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                // Tenant cohort — full/partial/inactive
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                    <div class="px-5 py-3.5 border-b border-outline-variant/20">
                                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Activation Cohort"</h3>
                                    </div>
                                    <div class="p-5 space-y-4">
                                        {{
                                            let fully_live = tenants.iter().filter(|t| {
                                                t.site_status.as_deref().map(|s| s == "active").unwrap_or(false)
                                                && t.mrr_cents.unwrap_or(0) > 0
                                            }).count();
                                            let partial = tenants.iter().filter(|t| {
                                                t.site_status.as_deref().map(|s| s == "active").unwrap_or(false)
                                                && t.mrr_cents.unwrap_or(0) == 0
                                            }).count();
                                            let inactive = tenants.iter().filter(|t| {
                                                !t.site_status.as_deref().map(|s| s == "active").unwrap_or(false)
                                            }).count();

                                            let cohorts = vec![
                                                ("Fully Live + Paying", fully_live, "#22c55e"),
                                                ("Live · No Billing",   partial,    "#f59e0b"),
                                                ("Inactive / Suspended",inactive,   "#ef4444"),
                                            ];
                                            cohorts.into_iter().map(|(label, count, color)| {
                                                let pct = (count as f64 / total as f64 * 100.0) as u32;
                                                view! {
                                                    <div style="display:flex;flex-direction:column;gap:4px;">
                                                        <div style="display:flex;justify-content:space-between;font-size:11px;">
                                                            <div style="display:flex;align-items:center;gap:7px;">
                                                                <span style=format!("width:8px;height:8px;border-radius:50%;background:{};display:inline-block;", color)></span>
                                                                <span>{label}</span>
                                                            </div>
                                                            <span style="font-family:monospace;color:var(--text-muted);">{format!("{} ({pct}%)", count)}</span>
                                                        </div>
                                                        <div style="height:5px;background:rgba(255,255,255,0.06);border-radius:3px;">
                                                            <div style=format!("height:5px;border-radius:3px;width:{}%;background:{};", pct, color)></div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    })}
                    </Suspense>
                </div>
            </Show>

        </div>
    }
}
