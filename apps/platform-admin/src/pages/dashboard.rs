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

            // ── Infrastructure Overview ──
            <div class="bg-surface-container rounded-xl border border-outline-variant/10 overflow-hidden">
                <div class="flex justify-between items-center px-6 py-4 border-b border-outline-variant/10">
                    <h3 class="text-base font-bold text-on-surface flex items-center gap-2">
                        <span class="material-symbols-outlined text-primary text-xl">"dns"</span>
                        "Platform Infrastructure"
                    </h3>
                    <span class="text-[10px] uppercase font-bold tracking-wider text-on-surface-variant px-2 py-1 rounded bg-surface-container-highest border border-outline-variant/20">"Ops Reference"</span>
                </div>

                <div class="p-6 grid grid-cols-1 lg:grid-cols-3 gap-6">

                    // Services
                    <div class="space-y-3">
                        <h4 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-3">"Core Services"</h4>
                        {[
                            ("backend", "8000", "API (Axum + SeaORM)", "api"),
                            ("anchor-app", "80", "Tenant Frontend (Leptos SSR)", "language"),
                            ("platform-admin", "80", "Admin Panel (Leptos CSR)", "admin_panel_settings"),
                            ("network-instance", "80", "Network Frontend", "hub"),
                            ("ingress-sidecar", "8085", "K8s Ingress Provisioner", "route"),
                        ].into_iter().map(|(name, port, desc, icon)| view! {
                            <div class="flex items-center gap-3 p-3 rounded-lg bg-surface-container-highest border border-outline-variant/10">
                                <span class="material-symbols-outlined text-primary text-base">{icon}</span>
                                <div class="min-w-0">
                                    <div class="flex items-center gap-2">
                                        <span class="text-xs font-bold text-on-surface font-mono">{name}</span>
                                        <span class="text-[10px] text-on-surface-variant bg-surface px-1.5 py-0.5 rounded border border-outline-variant/20 font-mono">{":" }{port}</span>
                                    </div>
                                    <div class="text-[10px] text-on-surface-variant truncate">{desc}</div>
                                </div>
                            </div>
                        }).collect_view()}
                    </div>

                    // TLS / DNS Requirements
                    <div class="space-y-3">
                        <h4 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-3">"TLS Certificate Requirements"</h4>
                        <div class="p-4 rounded-lg bg-warning/10 border border-warning/30 space-y-2">
                            <div class="flex items-center gap-2">
                                <span class="material-symbols-outlined text-warning text-base">"warning"</span>
                                <span class="text-xs font-bold text-warning">"DNS-01 Issuer Required"</span>
                            </div>
                            <p class="text-[11px] text-on-surface-variant leading-relaxed">
                                "Use "<span class="font-mono font-bold text-on-surface">"letsencrypt-cloudflare"</span>" on all Ingress resources. Cloudflare's \"Always Use HTTPS\" redirect breaks HTTP-01 challenges."
                            </p>
                        </div>
                        <div class="p-4 rounded-lg bg-surface-container-highest border border-outline-variant/10 space-y-2">
                            <div class="text-xs font-bold text-on-surface mb-2">"Cloudflare API Token Scopes"</div>
                            {[
                                ("Zone → DNS → Edit", "Create ACME TXT records"),
                                ("Zone → Zone → Read", "Resolve zone IDs"),
                            ].into_iter().map(|(scope, reason)| view! {
                                <div class="flex items-start gap-2">
                                    <span class="material-symbols-outlined text-success text-sm mt-0.5">"check_circle"</span>
                                    <div>
                                        <div class="text-[11px] font-mono font-bold text-on-surface">{scope}</div>
                                        <div class="text-[10px] text-on-surface-variant">{reason}</div>
                                    </div>
                                </div>
                            }).collect_view()}
                            <div class="pt-2 mt-2 border-t border-outline-variant/20">
                                <div class="text-[10px] text-on-surface-variant">"Secret: "<span class="font-mono">"cloudflare-api-token-secret"</span>" in "<span class="font-mono">"cert-manager"</span>" namespace"</div>
                            </div>
                        </div>
                    </div>

                    // Environment Matrix
                    <div class="space-y-3">
                        <h4 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-3">"Environments"</h4>
                        {[
                            ("dev", "atlas-dev", "dev.buildwithruud.com", "success"),
                            ("uat", "atlas-uat", "uat.buildwithruud.com", "success"),
                            ("main", "atlas-prod", "buildwithruud.com", "warning"),
                        ].into_iter().map(|(branch, ns, domain, status)| {
                            let (dot_class, label) = if status == "success" {
                                ("bg-success", "Active")
                            } else {
                                ("bg-warning", "Pending")
                            };
                            view! {
                                <div class="p-3 rounded-lg bg-surface-container-highest border border-outline-variant/10">
                                    <div class="flex items-center justify-between mb-1.5">
                                        <span class="font-mono text-xs font-bold text-on-surface">{branch}</span>
                                        <div class="flex items-center gap-1.5">
                                            <div class=format!("w-1.5 h-1.5 rounded-full {}", dot_class)></div>
                                            <span class="text-[10px] text-on-surface-variant">{label}</span>
                                        </div>
                                    </div>
                                    <div class="text-[10px] font-mono text-on-surface-variant">{ns}</div>
                                    <div class="text-[10px] text-on-surface-variant/60 truncate">{domain}</div>
                                </div>
                            }
                        }).collect_view()}
                        <div class="p-3 rounded-lg bg-surface-container-highest border border-outline-variant/10 mt-1">
                            <div class="text-[10px] font-bold text-on-surface mb-1">"Ingress Sidecar"</div>
                            <div class="text-[10px] text-on-surface-variant leading-relaxed">
                                "Standalone Deployment (not a pod sidecar). Provisions K8s Ingress + TLS for new tenant domains via "<span class="font-mono">"POST /api/ingress/provision"</span>"."
                            </div>
                        </div>
                    </div>

                </div>
            </div>

        </div>
    }
}
