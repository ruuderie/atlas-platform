use leptos::prelude::*;
use crate::api::analytics::{get_business_kpis, get_engagement};
use crate::api::admin::{get_tenant_stats, get_all_platform_apps};

#[component]
pub fn Dashboard() -> impl IntoView {
    let kpis_res = LocalResource::new(|| async move { get_business_kpis().await.unwrap_or_default() });
    let engagement_res = LocalResource::new(|| async move { get_engagement().await.unwrap_or_default() });
    let tenants_res = LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });
    let apps_res = LocalResource::new(|| async move { get_all_platform_apps().await.unwrap_or_default() });

    let mrr = Signal::derive(move || kpis_res.get().unwrap_or_default().mrr.value);
    let mrr_prev = Signal::derive(move || kpis_res.get().unwrap_or_default().mrr.previous_value);
    let active_subs = Signal::derive(move || kpis_res.get().unwrap_or_default().active_subscriptions.value);
    let _liquidity_index = Signal::derive(move || kpis_res.get().unwrap_or_default().network_liquidity_index.value);
    let total_users = Signal::derive(move || engagement_res.get().unwrap_or_default().total_users.value);
    let active_listings = Signal::derive(move || engagement_res.get().unwrap_or_default().active_listings.value);

    let mrr_str = move || {
        let val = mrr.get();
        if val <= 0.0 {
            "—".to_string()
        } else if val >= 1000.0 {
            format!("${:.1}k", val / 1000.0)
        } else {
            format!("${:.0}", val)
        }
    };

    let mrr_delta_str = move || {
        let cur = mrr.get();
        let prev = mrr_prev.get();
        if prev <= 0.0 || cur <= 0.0 {
            "—".to_string()
        } else {
            let pct = ((cur - prev) / prev) * 100.0;
            format!("{:+.1}% MoM", pct)
        }
    };

    let active_tenants_str = move || {
        let val = active_subs.get();
        if val <= 0.0 { "—".to_string() } else { format!("{:.0}", val) }
    };

    let tenant_count_str = move || {
        let n = tenants_res.get().unwrap_or_default().len();
        if n == 0 { "—".to_string() } else { n.to_string() }
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
            <div>
                <h1 class="page-title">"Command Center"</h1>
                <p class="page-subtitle">{move || format!("Platform-wide telemetry — {} tenants · Real-time", tenant_count_str())}</p>
            </div>
            <div class="page-header-actions">
                <button
                    class="btn btn-ghost btn-icon"
                    title="Refresh"
                    on:click=move |_| {
                        let _ = web_sys::window()
                            .and_then(|w| w.location().reload().ok());
                    }
                >
                    <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M2 8a6 6 0 0 1 6-6 6 6 0 0 1 4.2 1.8L14 6"/>
                        <path d="M14 2v4h-4"/>
                        <path d="M14 8a6 6 0 0 1-6 6 6 6 0 0 1-4.2-1.8L2 10"/>
                        <path d="M2 14v-4h4"/>
                    </svg>
                </button>
                <button
                    class="btn btn-ghost"
                    disabled
                    title="CSV export coming soon"
                    style="opacity:0.45;cursor:not-allowed"
                >"Export CSV"</button>
                <a href="/apps/new" class="btn btn-primary" style="text-decoration:none;display:inline-flex;align-items:center;gap:6px;">
                    <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                        <path d="M8 3a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H9v3a1 1 0 1 1-2 0V9H4a1 1 0 1 1 0-2h3V4a1 1 0 0 1 1-1z"/>
                    </svg>
                    "New Tenant"
                </a>
            </div>
        </div>


        // ── KPI Row ──
        <div class="kpi-row">
            <div class="kpi-card">
                <div class="kpi-label">"Active Tenants"</div>
                <div class="kpi-value">{active_tenants_str}</div>
                <div class="kpi-delta up">
                    <svg viewBox="0 0 10 10" width="10" height="10" fill="currentColor">
                        <path d="M5 2l4 6H1z"/>
                    </svg>
                    {move || { let n = tenants_res.get().unwrap_or_default().len(); if n > 0 { format!("{} tenants registered", n) } else { "—".to_string() } }}
                </div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">"MRR"</div>
                <div class="kpi-value mono">{mrr_str}</div>
                <div class="kpi-delta up">
                    <svg viewBox="0 0 10 10" width="10" height="10" fill="currentColor">
                        <path d="M5 2l4 6H1z"/>
                    </svg>
                    {mrr_delta_str}
                </div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">"Active Listings"</div>
                <div class="kpi-value mono">{move || { let v = active_listings.get(); if v > 0.0 { format!("{:.0}", v) } else { "—".to_string() } }}</div>
                <div class="kpi-delta up">"· Across all tenants"</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">"Platform Users"</div>
                <div class="kpi-value mono">{move || { let v = total_users.get(); if v > 0.0 { format!("{:.0}", v) } else { "—".to_string() } }}</div>
                <div class="kpi-delta up">"· Registered"
                </div>
            </div>
        </div>

        // ── Tenant Registry Table ──
        <div class="section">
            <div class="section-header">
                <div class="section-title">
                    <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                        <rect x="1" y="5" width="12" height="8" rx="0.5"/>
                        <path d="M4 5V3a3 3 0 0 1 6 0v2"/>
                    </svg>
                    "Tenant Registry"
                    <span class="section-count">{move || format!("{} tenants", tenants_res.get().unwrap_or_default().len())}</span>
                </div>
                <div class="flex-row">
                    <button class="btn btn-ghost" style="font-size:11px;padding:4px 10px;">"Filter"</button>
                    <a href="/apps" class="section-action" style="text-decoration:none">"View All →"</a>
                </div>
            </div>
            <Suspense fallback=move || view! { <div class="p-8 text-center muted">"Loading tenants..."</div> }>
            <table>
                <thead>
                    <tr>
                        <th>"Tenant / Domain"</th>
                        <th>"Plan"</th>
                        <th>"Profiles"</th>
                        <th>"Listings"</th>
                        <th class="center">"Status"</th>
                        <th class="right">"MRR"</th>
                        <th class="right">"Joined"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || tenants_res.get().unwrap_or_default().into_iter().map(|t| {
                        let status_color = match t.site_status.as_deref().unwrap_or("active") {
                            "active" => "var(--green)",
                            "suspended" => "var(--red)",
                            "provisioning" => "var(--amber)",
                            _ => "var(--text-muted)",
                        };
                        let mrr_str = t.mrr_cents
                            .map(|c| if c >= 100_000 { format!("${:.1}k", c as f64 / 100_000.0) } else { format!("${:.2}", c as f64 / 100.0) })
                            .unwrap_or_else(|| "—".to_string());
                        let plan = t.plan.clone().unwrap_or_else(|| "—".to_string());
                        let joined = t.joined_at.clone().unwrap_or_else(|| "—".to_string());
                        let joined_short = joined.get(..7).unwrap_or(&joined).to_string();
                        view! {
                            <tr>
                                <td>
                                    <div class="tenant-name">{t.name.clone()}</div>
                                    <div class="tenant-domain" style="font-size:10px;color:var(--text-muted)">{t.tenant_id.clone()}</div>
                                </td>
                                <td><span class="plan-badge">{plan}</span></td>
                                <td class="right mono">{t.profile_count.to_string()}</td>
                                <td class="right mono">{t.listing_count.to_string()}</td>
                                <td class="center"><span class="status-dot" style=format!("background:{}", status_color)></span></td>
                                <td class="right mono">{mrr_str}</td>
                                <td class="right muted">{joined_short}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
            </Suspense>
        </div>

        // ── Bottom Split: Products + Generics Coverage ──
        <div class="two-col">
            // Platform Products
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="7" cy="7" r="5"/>
                            <path d="M7 4v3l2 2"/>
                        </svg>
                        "Platform Apps"
                        <span class="section-count">{move || format!("{} instances", apps_res.get().unwrap_or_default().len())}</span>
                    </div>
                    <a href="/apps" class="section-action" style="text-decoration:none">"Manage →"</a>
                </div>

                <Suspense fallback=move || view! { <div class="p-4 muted">"Loading apps..."</div> }>
                {move || {
                    let apps = apps_res.get().unwrap_or_default();
                    if apps.is_empty() {
                        view! { <div class="p-4 muted">"No app instances provisioned yet."</div> }.into_any()
                    } else {
                        apps.into_iter().map(|app| {
                            let dot_color = match app.site_status.as_str() {
                                "active" => "var(--green)",
                                "suspended" => "var(--red)",
                                "provisioning" => "var(--cobalt)",
                                "beta" => "var(--amber)",
                                _ => "var(--text-muted)",
                            };
                            let status_label = app.site_status.clone();
                            view! {
                                <div class="product-row">
                                    <span class="product-mode-dot" style=format!("background:{}", dot_color)></span>
                                    <div class="product-info">
                                        <div class="product-name">{app.name.clone()}</div>
                                        <div class="product-domain">{app.domain.clone()}</div>
                                    </div>
                                    <div class="product-meta">
                                        <span style="font-size:10px;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;font-weight:600;text-transform:uppercase">{status_label}</span>
                                        <span style="font-size:10px;color:var(--text-muted)">{app.app_type.clone()}</span>
                                    </div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
                </Suspense>
            </div>

            // Generics Coverage Panel
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <rect x="1" y="1" width="12" height="12" rx="1"/>
                            <path d="M1 5h12M5 5v8"/>
                        </svg>
                        "Platform Generics — G01–G31"
                        <span class="section-count">"Coverage"</span>
                    </div>
                    <a href="/billing/scorecards" class="section-action" style="text-decoration:none">"View Spec →"</a>
                </div>
                <div style="padding:12px 16px;display:flex;flex-direction:column;gap:8px;">
                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 5px;background:var(--cobalt-dim)">"G27"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Scorecards"</span>
                        <span style="font-size:10px;color:var(--green);font-weight:600">"● Deployed + UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 5px;background:var(--cobalt-dim)">"G31"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Leads (atlas_lead)"</span>
                        <span style="font-size:10px;color:var(--green);font-weight:600">"● Full UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G08"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"AI Tasks"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G19"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Campaigns"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G21"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Events"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G23"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Reservations"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0;border-bottom:1px solid var(--border-subtle)">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G24"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Quotes"</span>
                        <span style="font-size:10px;color:var(--amber);font-weight:600">"⚠ No UI"</span>
                    </div>

                    <div style="display:flex;align-items:center;gap:8px;padding:6px 0">
                        <span style="font-size:9px;font-weight:600;color:var(--text-muted);border:1px solid var(--border-default);border-radius:3px;padding:1px 5px;">"G06"</span>
                        <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">"Verification Queue"</span>
                        <span style="font-size:10px;color:var(--red);font-weight:600">"✗ 3 pending"</span>
                    </div>
                </div>
            </div>
        </div>
        </div>
    }
}
