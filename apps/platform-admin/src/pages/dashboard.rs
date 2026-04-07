use leptos::prelude::*;
use crate::api::analytics::{get_business_kpis, get_engagement, get_trends, BusinessKpisResponse, EngagementResponse};
use crate::pages::shared::svg_charts::SvgLineChart;

#[component]
pub fn Dashboard() -> impl IntoView {
    let kpis_res = LocalResource::new(|| async move { get_business_kpis().await.unwrap_or_default() });
    let engagement_res = LocalResource::new(|| async move { get_engagement().await.unwrap_or_default() });
    
    // Fetch 30 day trends for MRR
    let mrr_trends_res = LocalResource::new(|| async move { 
        get_trends("mrr", 30).await.map(|p| p.into_iter().map(|t| t.value).collect::<Vec<f32>>()).unwrap_or_default() 
    });

    let mrr = Signal::derive(move || kpis_res.get().unwrap_or_default().mrr.value);
    let active_subs = Signal::derive(move || kpis_res.get().unwrap_or_default().active_subscriptions.value);
    let liquidity_index = Signal::derive(move || kpis_res.get().unwrap_or_default().network_liquidity_index.value);
    
    let total_users = Signal::derive(move || engagement_res.get().unwrap_or_default().total_users.value);
    let active_listings = Signal::derive(move || engagement_res.get().unwrap_or_default().active_listings.value);

    let mrr_trend_data = Signal::derive(move || mrr_trends_res.get().unwrap_or_default());

    view! {
        <div class="space-y-8">
            // ── Header ──
            <header>
                <h1 class="text-3xl font-extrabold tracking-tight text-on-surface mb-2">"Executive Platform Analytics"</h1>
                <p class="text-on-surface-variant font-medium">"Monitoring core business health and network telemetry."</p>
            </header>

            // ── Business KPI Grid ──
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                // MRR
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Monthly Recurring Revenue"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"payments"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("${:.2}", mrr.get())}</div>
                    <div class="flex items-center gap-2">
                        <span class="text-tertiary text-xs font-bold">"Platform Growth"</span>
                    </div>
                </div>

                // Active Subscriptions
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Active Subscriptions"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"card_membership"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("{}", active_subs.get())}</div>
                </div>

                // Liquidity Index
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Network Liquidity"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"water_drop"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("{:.1}%", liquidity_index.get())}</div>
                </div>
            </div>

            // ── Trend Graphs ──
            <div class="bg-surface-container rounded-xl p-6 border border-outline-variant/5">
                <div class="flex justify-between items-center mb-6">
                    <h3 class="text-lg font-bold text-on-surface flex items-center gap-2">
                        <span class="material-symbols-outlined text-primary">"trending_up"</span>
                        "MRR 30-Day Trend"
                    </h3>
                </div>
                <div class="w-full h-48 bg-surface-container-lowest/50 rounded-lg p-4 border border-outline-variant/10">
                    <Suspense fallback=move || view! { <div class="w-full h-full animate-pulse bg-outline-variant/20 rounded"></div> }>
                        <SvgLineChart data=mrr_trend_data width=800.0 height=160.0 color_class="stroke-primary".to_string() />
                    </Suspense>
                </div>
            </div>

            // ── Engagement Grid ──
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                // Total Users
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Total Global Users"</span>
                        <span class="material-symbols-outlined text-tertiary text-xl">"group"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("{}", total_users.get())}</div>
                </div>

                // Active Listings
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Active Global Listings"</span>
                        <span class="material-symbols-outlined text-tertiary text-xl">"list_alt"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("{}", active_listings.get())}</div>
                </div>
            </div>
        </div>
    }
}
