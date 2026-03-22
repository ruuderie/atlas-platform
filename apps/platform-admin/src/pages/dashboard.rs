use leptos::prelude::*;

use crate::api::directories::get_directories;
use crate::api::crm::{get_users, get_deals};
use crate::app::GlobalToast;

#[component]
pub fn Dashboard() -> impl IntoView {
    let users_res = LocalResource::new(|| async move { get_users().await.unwrap_or_default() });
    let dirs_res = LocalResource::new(|| async move { get_directories().await.unwrap_or_default() });
    let deals_res = LocalResource::new(|| async move { get_deals().await.unwrap_or_default() });

    let active_dirs = Signal::derive(move || dirs_res.get().unwrap_or_default().len());
    let total_users = Signal::derive(move || users_res.get().unwrap_or_default().len());
    let deals_pipeline = Signal::derive(move || {
        let sum: f32 = deals_res.get().unwrap_or_default().iter().map(|d| d.amount).sum();
        if sum >= 1_000_000.0 {
            format!("${:.1}M", sum / 1_000_000.0)
        } else if sum >= 1_000.0 {
            format!("${:.0}K", sum / 1_000.0)
        } else {
            format!("${:.2}", sum)
        }
    });

    view! {
        <div class="space-y-8">
            // ── System Alert Banner ──
            <div class="flex items-center justify-between p-4 bg-error-container/20 border-l-4 border-error rounded-r-xl glass-panel">
                <div class="flex items-center gap-3">
                    <span class="material-symbols-outlined text-error" style="font-variation-settings: 'FILL' 1;">"warning"</span>
                    <div>
                        <h4 class="text-sm font-bold text-on-surface">"Scheduled System Maintenance"</h4>
                        <p class="text-xs text-on-surface-variant">"Infrastructure upgrade scheduled for Sunday, 02:00 AM UTC. Expect minimal downtime."</p>
                    </div>
                </div>
                <button class="text-on-surface-variant hover:text-on-surface transition-colors">
                    <span class="material-symbols-outlined text-sm">"close"</span>
                </button>
            </div>

            // ── Header ──
            <header>
                <h1 class="text-3xl font-extrabold tracking-tight text-on-surface mb-2">"Platform Overview"</h1>
                <p class="text-on-surface-variant font-medium">"Monitoring real-time telemetry across the intelligence mesh."</p>
            </header>

            // ── KPI Grid ──
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                // Active Directories
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Active Directories"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"lan"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("{}", active_dirs.get())}</div>
                    <div class="flex items-center gap-2">
                        <span class="text-tertiary text-xs font-bold">"Dynamic"</span>
                        <div class="flex-1 h-1 bg-surface-container rounded-full overflow-hidden">
                            <div class="h-full bg-tertiary w-2/3"></div>
                        </div>
                    </div>
                </div>

                // Total Users
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Total Users"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"group"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || format!("{}", total_users.get())}</div>
                    <div class="flex items-center gap-2">
                        <span class="text-tertiary text-xs font-bold">"API"</span>
                        <div class="flex-1 h-1 bg-surface-container rounded-full overflow-hidden">
                            <div class="h-full bg-primary w-[45%]"></div>
                        </div>
                    </div>
                </div>

                // Active Listings
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Active Listings"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"list_alt"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">"—"</div>
                    <div class="flex items-center gap-2">
                        <span class="text-on-surface-variant text-xs font-bold">"Pending"</span>
                        <div class="flex-1 h-1 bg-surface-container rounded-full overflow-hidden">
                            <div class="h-full bg-outline-variant w-[15%]"></div>
                        </div>
                    </div>
                </div>

                // Deals Pipeline
                <div class="bg-surface-container-high rounded-xl p-5 border border-outline-variant/10 group hover:border-primary/30 transition-all duration-300">
                    <div class="flex justify-between items-start mb-4">
                        <span class="text-[10px] font-bold uppercase tracking-[0.1em] text-on-surface-variant">"Deals Pipeline"</span>
                        <span class="material-symbols-outlined text-primary text-xl">"payments"</span>
                    </div>
                    <div class="text-3xl font-bold tracking-tight mb-1">{move || deals_pipeline.get()}</div>
                    <div class="flex items-center gap-2">
                        <span class="text-tertiary text-xs font-bold">"Stable"</span>
                        <div class="flex-1 h-1 bg-surface-container rounded-full overflow-hidden">
                            <div class="h-full bg-tertiary w-1/2"></div>
                        </div>
                    </div>
                </div>
            </div>

            // ── Bottom Grid: Activity + Quick Actions ──
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                // Activity Feed
                <div class="lg:col-span-2 bg-surface-container rounded-xl p-6 border border-outline-variant/5">
                    <div class="flex justify-between items-center mb-6">
                        <h3 class="text-lg font-bold text-on-surface flex items-center gap-2">
                            <span class="material-symbols-outlined text-primary">"history"</span>
                            "Recent Activity"
                        </h3>
                        <button class="text-[10px] font-bold uppercase text-primary tracking-widest hover:underline">"Export Audit Log"</button>
                    </div>
                    <div class="space-y-6 relative before:absolute before:left-[11px] before:top-2 before:bottom-2 before:w-[1px] before:bg-outline-variant/20">
                        // Item 1
                        <div class="relative pl-8 flex justify-between items-start group">
                            <div class="absolute left-0 top-1.5 w-6 h-6 rounded-full bg-surface-container-high border-2 border-primary-dim flex items-center justify-center z-10 group-hover:scale-110 transition-transform">
                                <span class="material-symbols-outlined text-[14px] text-primary">"check_circle"</span>
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-on-surface">"Deal Closed by " <span class="text-primary">"Marcus Holloway"</span></p>
                                <p class="text-xs text-on-surface-variant">"Site B • Enterprise Cloud Tier"</p>
                            </div>
                            <span class="text-[10px] font-medium text-on-surface-variant">"2m ago"</span>
                        </div>
                        // Item 2
                        <div class="relative pl-8 flex justify-between items-start group">
                            <div class="absolute left-0 top-1.5 w-6 h-6 rounded-full bg-surface-container-high border-2 border-outline-variant/40 flex items-center justify-center z-10 group-hover:scale-110 transition-transform">
                                <span class="material-symbols-outlined text-[14px] text-on-surface-variant">"dns"</span>
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-on-surface">"Site Y Deployed"</p>
                                <p class="text-xs text-on-surface-variant">"Frankfurt Region • Secondary Node"</p>
                            </div>
                            <span class="text-[10px] font-medium text-on-surface-variant">"14m ago"</span>
                        </div>
                        // Item 3
                        <div class="relative pl-8 flex justify-between items-start group">
                            <div class="absolute left-0 top-1.5 w-6 h-6 rounded-full bg-surface-container-high border-2 border-tertiary/40 flex items-center justify-center z-10 group-hover:scale-110 transition-transform">
                                <span class="material-symbols-outlined text-[14px] text-tertiary">"person_add"</span>
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-on-surface">"New Lead converted"</p>
                                <p class="text-xs text-on-surface-variant">"Organic Search • \"Data Fabric Solutions\""</p>
                            </div>
                            <span class="text-[10px] font-medium text-on-surface-variant">"1h ago"</span>
                        </div>
                        // Item 4
                        <div class="relative pl-8 flex justify-between items-start group">
                            <div class="absolute left-0 top-1.5 w-6 h-6 rounded-full bg-surface-container-high border-2 border-error/40 flex items-center justify-center z-10 group-hover:scale-110 transition-transform">
                                <span class="material-symbols-outlined text-[14px] text-error">"security_update_warning"</span>
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-on-surface">"Auth Attempt Rejected"</p>
                                <p class="text-xs text-on-surface-variant">"IP: 192.168.1.104 • 3 failed attempts"</p>
                            </div>
                            <span class="text-[10px] font-medium text-on-surface-variant">"2h ago"</span>
                        </div>
                    </div>
                </div>

                // Right Column
                <div class="space-y-6">
                    // Quick Actions
                    <div class="bg-surface-container-high rounded-xl p-6 border border-outline-variant/10">
                        <h3 class="text-xs font-bold uppercase tracking-widest text-on-surface-variant mb-6">"Quick Actions"</h3>
                        <div class="space-y-3">
                            <a href="/sites/new" class="w-full flex items-center justify-between p-4 bg-surface-container rounded-lg border border-outline-variant/10 hover:border-primary/50 hover:bg-surface-bright/20 transition-all group">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-primary">"add_circle"</span>
                                    <span class="font-bold text-sm tracking-tight">"New Site"</span>
                                </div>
                                <span class="material-symbols-outlined text-on-surface-variant group-hover:translate-x-1 transition-transform">"chevron_right"</span>
                            </a>
                            <a href="/cms?tab=editor" class="w-full flex items-center justify-between p-4 bg-surface-container rounded-lg border border-outline-variant/10 hover:border-primary/50 hover:bg-surface-bright/20 transition-all group">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-primary">"edit_note"</span>
                                    <span class="font-bold text-sm tracking-tight">"Write Article"</span>
                                </div>
                                <span class="material-symbols-outlined text-on-surface-variant group-hover:translate-x-1 transition-transform">"chevron_right"</span>
                            </a>
                            <a href="/crm/new" class="w-full flex items-center justify-between p-4 bg-surface-container rounded-lg border border-outline-variant/10 hover:border-primary/50 hover:bg-surface-bright/20 transition-all group">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-primary">"leaderboard"</span>
                                    <span class="font-bold text-sm tracking-tight">"New Lead"</span>
                                </div>
                                <span class="material-symbols-outlined text-on-surface-variant group-hover:translate-x-1 transition-transform">"chevron_right"</span>
                            </a>
                        </div>
                    </div>

                    // System Health
                    <div class="bg-surface-container-lowest rounded-xl p-6 border border-outline-variant/10 overflow-hidden relative">
                        <div class="absolute top-0 right-0 w-32 h-32 bg-primary/10 blur-[60px] rounded-full -mr-16 -mt-16"></div>
                        <div class="relative z-10">
                            <h3 class="text-xs font-bold uppercase tracking-widest text-on-surface-variant mb-4">"System Integrity"</h3>
                            <div class="flex items-end gap-2 mb-2">
                                <span class="text-4xl font-extrabold text-primary tracking-tighter">"99.9%"</span>
                                <span class="text-xs font-bold text-tertiary pb-1">"Uptime"</span>
                            </div>
                            <p class="text-[10px] text-on-surface-variant leading-relaxed">"Network status is nominal across all edge clusters. Last ping 40ms."</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
