use leptos::prelude::*;
use crate::api::analytics::{get_business_kpis, get_engagement};
use crate::api::admin::{get_tenant_stats, get_all_platform_apps};
use crate::api::verification::get_verification_requests;

// ── Helper ────────────────────────────────────────────────────────────────────
fn app_type_emoji(slug: &str) -> &'static str {
    match slug {
        "anchor"                        => "⚓",
        "property_management" | "folio" => "🏠",
        "network_instance" | "network"  => "🔗",
        "str"                           => "🏖️",
        _                               => "📦",
    }
}

fn app_type_label(slug: &str) -> &'static str {
    match slug {
        "anchor"                        => "Anchor CMS",
        "property_management" | "folio" => "Folio PM",
        "network_instance" | "network"  => "Network",
        "str"                           => "Atlas STR",
        _                               => "App",
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let kpis_res    = LocalResource::new(|| async move { get_business_kpis().await.unwrap_or_default() });
    let engagement_res = LocalResource::new(|| async move { get_engagement().await.unwrap_or_default() });
    let tenants_res = LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });
    let apps_res    = LocalResource::new(|| async move { get_all_platform_apps().await.unwrap_or_default() });
    let verification_res = LocalResource::new(|| async move {
        get_verification_requests(None, None).await.unwrap_or_default()
    });

    // ── Derived KPI signals ───────────────────────────────────────────────────
    let mrr         = Signal::derive(move || kpis_res.get().unwrap_or_default().mrr.value);
    let mrr_prev    = Signal::derive(move || kpis_res.get().unwrap_or_default().mrr.previous_value);
    let active_subs = Signal::derive(move || kpis_res.get().unwrap_or_default().active_subscriptions.value);
    let total_users = Signal::derive(move || engagement_res.get().unwrap_or_default().total_users.value);
    let active_listings = Signal::derive(move || engagement_res.get().unwrap_or_default().active_listings.value);

    let mrr_str = move || {
        let val = mrr.get();
        if val <= 0.0        { "—".to_string() }
        else if val >= 1000.0 { format!("${:.1}k", val / 1000.0) }
        else                  { format!("${:.0}", val) }
    };

    let mrr_delta_str = move || {
        let cur = mrr.get(); let prev = mrr_prev.get();
        if prev <= 0.0 || cur <= 0.0 { "—".to_string() }
        else { format!("{:+.1}% MoM", ((cur - prev) / prev) * 100.0) }
    };

    let active_tenants_str = move || {
        let v = active_subs.get();
        if v <= 0.0 { "—".to_string() } else { format!("{:.0}", v) }
    };

    let tenant_count_str = move || {
        let n = tenants_res.get().unwrap_or_default().len();
        if n == 0 { "—".to_string() } else { n.to_string() }
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──────────────────────────────────────────────────
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Command Center"</h1>
                    <p class="page-subtitle">{move || format!("Platform-wide telemetry — {} tenants · Real-time", tenant_count_str())}</p>
                </div>
                <div class="page-header-actions">
                    <button
                        class="btn btn-ghost btn-icon"
                        title="Refresh"
                        on:click=move |_| { let _ = web_sys::window().and_then(|w| w.location().reload().ok()); }
                    >
                        <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M2 8a6 6 0 0 1 6-6 6 6 0 0 1 4.2 1.8L14 6"/>
                            <path d="M14 2v4h-4"/>
                            <path d="M14 8a6 6 0 0 1-6 6 6 6 0 0 1-4.2-1.8L2 10"/>
                            <path d="M2 14v-4h4"/>
                        </svg>
                    </button>
                    <a href="/apps/new" class="btn btn-primary" style="text-decoration:none;display:inline-flex;align-items:center;gap:6px;">
                        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                            <path d="M8 3a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H9v3a1 1 0 1 1-2 0V9H4a1 1 0 1 1 0-2h3V4a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Tenant"
                    </a>
                </div>
            </div>

            // ── KPI Row ───────────────────────────────────────────────────────
            <div class="kpi-row">
                <div class="kpi-card">
                    <div class="kpi-label">"Active Tenants"</div>
                    <div class="kpi-value">{active_tenants_str}</div>
                    <div class="kpi-delta up">
                        {move || { let n = tenants_res.get().unwrap_or_default().len(); if n > 0 { format!("{} registered", n) } else { "—".to_string() } }}
                    </div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Platform MRR"</div>
                    <div class="kpi-value mono">{mrr_str}</div>
                    <div class="kpi-delta up">{mrr_delta_str}</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"App Instances"</div>
                    <div class="kpi-value mono">
                        {move || {
                            let n = apps_res.get().unwrap_or_default().len();
                            if n == 0 { "—".to_string() } else { n.to_string() }
                        }}
                    </div>
                    <div class="kpi-delta up">
                        {move || {
                            let apps = apps_res.get().unwrap_or_default();
                            let live = apps.iter().filter(|a| a.site_status.to_lowercase() == "active").count();
                            let total = apps.len();
                            if total == 0 { "—".to_string() } else { format!("{} live · {} suspended", live, total - live) }
                        }}
                    </div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Active Listings"</div>
                    <div class="kpi-value mono">{move || { let v = active_listings.get(); if v > 0.0 { format!("{:.0}", v) } else { "—".to_string() } }}</div>
                    <div class="kpi-delta up">"Across all tenants"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Platform Users"</div>
                    <div class="kpi-value mono">{move || { let v = total_users.get(); if v > 0.0 { format!("{:.0}", v) } else { "—".to_string() } }}</div>
                    <div class="kpi-delta up">"Registered"</div>
                </div>
            </div>

            // ── Platform Hierarchy: Client → Tenant → App Instances ──────────
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="7" cy="2.5" r="1.5"/>
                            <line x1="7" y1="4" x2="7" y2="6"/>
                            <circle cx="3" cy="8" r="1.5"/>
                            <circle cx="11" cy="8" r="1.5"/>
                            <line x1="7" y1="6" x2="3" y2="6.5"/>
                            <line x1="7" y1="6" x2="11" y2="6.5"/>
                            <circle cx="1.5" cy="12" r="1"/>
                            <circle cx="4.5" cy="12" r="1"/>
                            <circle cx="9.5" cy="12" r="1"/>
                            <circle cx="12.5" cy="12" r="1"/>
                            <line x1="3" y1="9.5" x2="1.5" y2="11"/>
                            <line x1="3" y1="9.5" x2="4.5" y2="11"/>
                            <line x1="11" y1="9.5" x2="9.5" y2="11"/>
                            <line x1="11" y1="9.5" x2="12.5" y2="11"/>
                        </svg>
                        "Platform Hierarchy"
                        <span class="section-count">
                            {move || format!("Tenant → App Instances · {} tenants", tenants_res.get().unwrap_or_default().len())}
                        </span>
                    </div>
                    <a href="/tenants" class="section-action" style="text-decoration:none">"View All Tenants →"</a>
                </div>
                <Suspense fallback=move || view! {
                    <div class="p-6 flex flex-col gap-2">
                        {(0..3u8).map(|_| view! {
                            <div class="animate-pulse flex items-center gap-3 py-2">
                                <div class="w-7 h-7 rounded-full bg-surface-container-highest/30 shrink-0"></div>
                                <div class="flex-1 space-y-1.5">
                                    <div class="h-3 bg-surface-container-highest/30 rounded w-32"></div>
                                    <div class="h-2.5 bg-surface-container-highest/20 rounded w-48"></div>
                                </div>
                            </div>
                        }).collect_view()}
                    </div>
                }>
                {move || {
                    let tenants = tenants_res.get().unwrap_or_default();
                    let apps    = apps_res.get().unwrap_or_default();

                    if tenants.is_empty() {
                        return view! {
                            <div class="flex flex-col items-center justify-center py-12 gap-3">
                                <span class="material-symbols-outlined text-3xl text-on-surface-variant/20">"account_tree"</span>
                                <p class="text-xs text-on-surface-variant/50">"No tenants provisioned yet. Create one to see the hierarchy."</p>
                                <a href="/tenants" class="px-3 py-1.5 text-[10px] font-semibold bg-surface-container border border-outline-variant/30 hover:border-primary/40 rounded-lg transition-all text-on-surface-variant" style="text-decoration:none">
                                    "Manage Tenants"
                                </a>
                            </div>
                        }.into_any();
                    }

                    let display_tenants = if tenants.len() > 5 { &tenants[..5] } else { &tenants[..] };
                    let overflow = if tenants.len() > 5 { Some(tenants.len() - 5) } else { None };

                    view! {
                        <div class="divide-y divide-outline-variant/10">
                            {display_tenants.iter().map(|t| {
                                // Avatar initials
                                let initials = t.name.chars().next().unwrap_or('T').to_uppercase().to_string();
                                // Status color
                                let (dot_cls, dot_title) = match t.site_status.as_deref().unwrap_or("active") {
                                    "active"       => ("bg-emerald-400", "Active"),
                                    "suspended"    => ("bg-red-400",     "Suspended"),
                                    "provisioning" => ("bg-amber-400",   "Provisioning"),
                                    _              => ("bg-on-surface-variant/30", "Unknown"),
                                };
                                // Instances for this tenant
                                let tenant_apps: Vec<&crate::api::models::PlatformAppSummary> = apps.iter()
                                    .filter(|a| a.tenant_id == t.tenant_id)
                                    .collect();
                                let tenant_id = t.tenant_id.clone();
                                let tenant_name = t.name.clone();
                                let plan = t.plan.clone().unwrap_or_else(|| "—".to_string());

                                view! {
                                    <div class="flex items-center gap-3 px-4 py-3 hover:bg-surface-bright/5 transition-colors">
                                        // Avatar
                                        <div class="w-7 h-7 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center text-[10px] font-bold text-primary shrink-0">
                                            {initials}
                                        </div>

                                        // Tenant info
                                        <div class="min-w-[140px]">
                                            <a
                                                href=format!("/tenants/{}", tenant_id)
                                                class="text-xs font-semibold text-on-surface hover:text-primary transition-colors"
                                                style="text-decoration:none"
                                            >
                                                {tenant_name}
                                            </a>
                                            <div class="flex items-center gap-1.5 mt-0.5">
                                                <span class=format!("w-1.5 h-1.5 rounded-full shrink-0 {}", dot_cls)
                                                      title=dot_title></span>
                                                <span class="text-[10px] text-on-surface-variant/50">{plan}</span>
                                            </div>
                                        </div>

                                        // Arrow connector
                                        <span class="text-on-surface-variant/20 text-xs shrink-0">"›"</span>

                                        // App instance pills
                                        <div class="flex flex-wrap gap-1.5 flex-1">
                                            {if tenant_apps.is_empty() {
                                                view! {
                                                    <span class="text-[10px] text-on-surface-variant/30 italic">"No instances provisioned"</span>
                                                }.into_any()
                                            } else {
                                                tenant_apps.iter().map(|app| {
                                                    let type_label = match app.app_type.as_str() {
                                                        "folio"            => "Folio",
                                                        "anchor"           => "Anchor",
                                                        "network_instance" | "network" => "Network",
                                                        other              => other,
                                                    };
                                                    let (pill_bg, pill_text) = match app.site_status.to_lowercase().as_str() {
                                                        "active"       => ("bg-emerald-500/10 border-emerald-500/20", "text-emerald-400"),
                                                        "suspended"    => ("bg-red-500/10 border-red-500/20",         "text-red-400"),
                                                        "provisioning" => ("bg-amber-500/10 border-amber-500/20",     "text-amber-400"),
                                                        _              => ("bg-surface-container border-outline-variant/20", "text-on-surface-variant/60"),
                                                    };
                                                    let display_domain = app.custom_domain.clone()
                                                        .or_else(|| if app.domain.is_empty() { None } else { Some(app.domain.clone()) })
                                                        .unwrap_or_else(|| "—".to_string());
                                                    let instance_href = format!("/apps/{}/instance", app.instance_id);
                                                    view! {
                                                        <a
                                                            href=instance_href
                                                            class=format!("inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-semibold border {} {} transition-colors hover:opacity-80", pill_bg, pill_text)
                                                            style="text-decoration:none"
                                                            title=display_domain
                                                        >
                                                            {type_label}
                                                        </a>
                                                    }
                                                }).collect_view().into_any()
                                            }}
                                        </div>

                                        // Quick link
                                        <a
                                            href=format!("/tenants/{}", t.tenant_id.clone())
                                            class="shrink-0 text-on-surface-variant/30 hover:text-primary transition-colors"
                                            style="text-decoration:none"
                                            title="View tenant"
                                        >
                                            <svg viewBox="0 0 14 14" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
                                                <path d="M5 3h6v6M3 11l8-8"/>
                                            </svg>
                                        </a>
                                    </div>
                                }
                            }).collect_view()}

                            {overflow.map(|n| view! {
                                <div class="px-4 py-2.5 text-center">
                                    <a href="/tenants" class="text-[10px] font-semibold text-primary hover:underline" style="text-decoration:none">
                                        {format!("+ {} more tenants →", n)}
                                    </a>
                                </div>
                            })}
                        </div>
                    }.into_any()
                }}
                </Suspense>
            </div>

            // ── App Instance Health by Type ───────────────────────────────────
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <rect x="1" y="3" width="12" height="9" rx="1"/>
                            <path d="M4 3V2a3 3 0 0 1 6 0v1"/>
                        </svg>
                        "App Instance Fleet"
                        <span class="section-count">
                            {move || format!("{} instances", apps_res.get().unwrap_or_default().len())}
                        </span>
                    </div>
                    <a href="/tenants" class="section-action" style="text-decoration:none">"View All Tenants →"</a>
                </div>
                <Suspense fallback=move || view! { <div class="p-4 muted">"Loading fleet..."</div> }>
                {move || {
                    let apps = apps_res.get().unwrap_or_default();
                    if apps.is_empty() {
                        return view! { <div class="p-4 muted">"No app instances provisioned yet."</div> }.into_any();
                    }
                    // Group by app_type
                    let mut type_map: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
                    for app in &apps {
                        let entry = type_map.entry(app.app_type.clone()).or_insert((0, 0));
                        if app.site_status.to_lowercase() == "active" {
                            entry.0 += 1;
                        } else {
                            entry.1 += 1;
                        }
                    }
                    let mut type_list: Vec<(String, usize, usize)> = type_map
                        .into_iter()
                        .map(|(k, (live, other))| (k, live, other))
                        .collect();
                    type_list.sort_by(|a, b| b.1.cmp(&a.1)); // most live first

                    view! {
                        <div style="overflow-x:auto;">
                            <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px;padding:16px 0 8px;min-width:600px;">
                                {type_list.into_iter().map(|(slug, live, suspended)| {
                                    let emoji = app_type_emoji(&slug);
                                    let label = app_type_label(&slug);
                                    let all = live + suspended;
                                    let health_pct = if all > 0 { (live * 100) / all } else { 0 };
                                    let bar_color = if health_pct == 100 { "var(--green)" }
                                        else if health_pct >= 50 { "var(--amber)" }
                                        else { "var(--red)" };
                                    // Link to tenant list filtered by this app type
                                    let href = format!("/tenants?type={}", slug);
                                    view! {
                                        <a
                                            href=href
                                            style="display:block;text-decoration:none;background:var(--surface-container,rgba(255,255,255,0.04));border:1px solid var(--border,rgba(255,255,255,0.07));border-radius:10px;padding:16px 18px;cursor:pointer;transition:border-color 0.15s,background 0.15s;"
                                        >
                                            <div style="display:flex;align-items:center;gap:8px;margin-bottom:10px;">
                                                <span style="font-size:22px">{emoji}</span>
                                                <span style="font-size:13px;font-weight:600;color:var(--text-primary)">{label}</span>
                                            </div>
                                            <div style="display:flex;gap:16px;font-size:11px;color:var(--text-muted);margin-bottom:10px;">
                                                <span><strong style="color:var(--green);font-size:16px;font-family:monospace">{live.to_string()}</strong>" live"</span>
                                                {if suspended > 0 {
                                                    view! { <span><strong style="color:var(--red)">{suspended.to_string()}</strong>" suspended"</span> }.into_any()
                                                } else {
                                                    view! { <></> }.into_any()
                                                }}
                                            </div>
                                            // Health bar
                                            <div style="height:4px;background:var(--border,rgba(255,255,255,0.07));border-radius:2px;overflow:hidden;">
                                                <div style=format!("height:100%;width:{}%;background:{};border-radius:2px;transition:width 0.4s;", health_pct, bar_color)></div>
                                            </div>
                                            <div style="margin-top:6px;font-size:10px;color:var(--text-muted);font-family:monospace">{format!("{}% healthy · click to view", health_pct)}</div>
                                        </a>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }.into_any()
                }}
                </Suspense>
            </div>

            // ── Tenant Registry Table + Onboarding Funnel ────────────────────
            <div class="two-col">
                // Tenant Registry
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
                        <a href="/tenants" class="section-action" style="text-decoration:none">"View All →"</a>
                    </div>
                    <Suspense fallback=move || view! { <div class="p-8 text-center muted">"Loading tenants..."</div> }>
                    <table>
                        <thead>
                            <tr>
                                <th>"Tenant"</th>
                                <th>"Plan"</th>
                                <th class="right">"MRR"</th>
                                <th class="center">"Setup"</th>
                                <th class="center">"Health"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || tenants_res.get().unwrap_or_default().into_iter().map(|t| {
                                let status = t.site_status.clone().unwrap_or_else(|| "active".to_string());
                                let health_color = match status.to_lowercase().as_str() {
                                    "active"       => "var(--green)",
                                    "suspended"    => "var(--red)",
                                    "provisioning" => "var(--amber)",
                                    _              => "var(--text-muted)",
                                };
                                let mrr_str = t.mrr_cents
                                    .map(|c| if c >= 100_000 { format!("${:.1}k", c as f64 / 100_000.0) } else { format!("${:.2}", c as f64 / 100.0) })
                                    .unwrap_or_else(|| "$0".to_string());
                                let plan = t.plan.clone().unwrap_or_else(|| "—".to_string());

                                // Setup score: live + MRR + profiles + listings
                                let mut score = 0u8;
                                if status.to_lowercase() == "active" { score += 1; }
                                if t.mrr_cents.map(|c| c > 0).unwrap_or(false) { score += 1; }
                                if t.profile_count > 0 { score += 1; }
                                if t.listing_count > 0 { score += 1; }
                                let score_color = if score == 4 { "var(--green)" }
                                    else if score >= 2 { "var(--amber)" }
                                    else { "var(--red)" };

                                // Always route to /tenants/:tenant_id — path-param driven, never dropdown-driven.
                                // Previously fell back to /apps?tenant=X which was silently ignored.
                                let href = format!("/tenants/{}", t.tenant_id);
                                view! {
                                    <tr style="cursor:pointer;" on:click={
                                        let href = href.clone();
                                        move |_| { let _ = web_sys::window().and_then(|w| w.location().assign(&href).ok()); }
                                    }>
                                        <td>
                                            <div class="tenant-name">{t.name.clone()}</div>
                                            <div class="tenant-domain" style="font-size:10px;color:var(--text-muted);font-family:monospace">
                                                {t.joined_at.as_ref().and_then(|d| d.get(..7)).unwrap_or("—").to_string()}
                                            </div>
                                        </td>
                                        <td><span class="plan-badge">{plan}</span></td>
                                        <td class="right" style="font-family:monospace;font-size:12px">{mrr_str}</td>
                                        <td class="center">
                                            <span style=format!("font-size:11px;font-weight:700;color:{}", score_color)>
                                                {format!("{}/4", score)}
                                            </span>
                                        </td>
                                        <td class="center">
                                            <span class="status-dot" style=format!("background:{}", health_color)></span>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                    </Suspense>
                </div>

                // Onboarding Funnel + Verification Queue
                <div style="display:flex;flex-direction:column;gap:14px;">
                    // Onboarding Funnel
                    <div class="section">
                        <div class="section-header">
                            <div class="section-title">
                                <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <path d="M2 2l10 0M4 6l6 0M6 10l2 0"/>
                                </svg>
                                "Tenant Onboarding Funnel"
                            </div>
                            <a href="/tenants" class="section-action" style="text-decoration:none">"Manage →"</a>
                        </div>
                        <Suspense fallback=move || view! { <div class="p-4 muted">"Loading..."</div> }>
                        {move || {
                            let tenants = tenants_res.get().unwrap_or_default();
                            let total = tenants.len();
                            if total == 0 {
                                return view! { <div class="p-4 muted">"No tenants provisioned."</div> }.into_any();
                            }
                            // Compute setup score per tenant, bucket into stages
                            let mut live_count      = 0usize;
                            let mut partial_count   = 0usize;
                            let mut inactive_count  = 0usize;
                            for t in &tenants {
                                let status = t.site_status.as_deref().unwrap_or("active");
                                let mut score = 0u8;
                                if status.to_lowercase() == "active"          { score += 1; }
                                if t.mrr_cents.map(|c| c > 0).unwrap_or(false) { score += 1; }
                                if t.profile_count > 0                         { score += 1; }
                                if t.listing_count > 0                         { score += 1; }
                                match score {
                                    4    => live_count    += 1,
                                    2..=3 => partial_count += 1,
                                    _    => inactive_count += 1,
                                }
                            }
                            let live_pct = (live_count * 100) / total.max(1);
                            let partial_pct = (partial_count * 100) / total.max(1);

                            view! {
                                <div style="padding:14px 0 4px;display:flex;flex-direction:column;gap:10px;">
                                    // Fully Live
                                    <div>
                                        <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:4px;">
                                            <span style="color:var(--text-primary);font-weight:600">"Fully Live (4/4)"</span>
                                            <span style="color:var(--green);font-family:monospace">{format!("{} tenants", live_count)}</span>
                                        </div>
                                        <div style="height:6px;background:var(--border,rgba(255,255,255,0.07));border-radius:3px;overflow:hidden;">
                                            <div style=format!("height:100%;width:{}%;background:var(--green);border-radius:3px;", live_pct)></div>
                                        </div>
                                    </div>
                                    // Partial
                                    <div>
                                        <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:4px;">
                                            <span style="color:var(--text-primary);font-weight:600">"Partially Setup (2–3/4)"</span>
                                            <span style="color:var(--amber);font-family:monospace">{format!("{} tenants", partial_count)}</span>
                                        </div>
                                        <div style="height:6px;background:var(--border,rgba(255,255,255,0.07));border-radius:3px;overflow:hidden;">
                                            <div style=format!("height:100%;width:{}%;background:var(--amber);border-radius:3px;", partial_pct)></div>
                                        </div>
                                    </div>
                                    // Inactive
                                    <div>
                                        <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:4px;">
                                            <span style="color:var(--text-primary);font-weight:600">"Need Activation (0–1/4)"</span>
                                            <span style="color:var(--red);font-family:monospace">{format!("{} tenants", inactive_count)}</span>
                                        </div>
                                        <div style="height:6px;background:var(--border,rgba(255,255,255,0.07));border-radius:3px;overflow:hidden;">
                                            <div style=format!("height:100%;width:{}%;background:var(--red);border-radius:3px;",
                                                (inactive_count * 100) / total.max(1))></div>
                                        </div>
                                    </div>
                                    {if inactive_count > 0 {
                                        view! {
                                            <div style="margin-top:8px;padding:8px 12px;background:rgba(239,68,68,0.07);border:1px solid rgba(239,68,68,0.2);border-radius:8px;font-size:11px;color:var(--red);">
                                                <strong>{inactive_count.to_string()}</strong>" tenant(s) need activation attention. "
                                                <a href="/apps" style="color:var(--red);text-decoration:underline;">"Review →"</a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }}
                        </Suspense>
                    </div>

                    // Verification Queue
                    <div class="section">
                        <div class="section-header">
                            <div class="section-title">
                                <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <path d="M8 2l5 2v4c0 3-2 5.5-5 6.5C5 13.5 3 11 3 8V4l5-2z"/>
                                </svg>
                                "Verification Queue"
                                <span class="section-count">
                                    {move || {
                                        let pending = verification_res.get().unwrap_or_default()
                                            .iter().filter(|r| r.status == "pending" || r.status == "review").count();
                                        if pending == 0 { "All Clear".to_string() } else { format!("{} pending", pending) }
                                    }}
                                </span>
                            </div>
                            <a href="/verification" class="section-action" style="text-decoration:none">"Review All →"</a>
                        </div>
                        <Suspense fallback=move || view! { <div class="p-4 muted">"Loading queue…"</div> }>
                        {move || {
                            let items = verification_res.get().unwrap_or_default();
                            let pending: Vec<_> = items.into_iter()
                                .filter(|r| r.status == "pending" || r.status == "review")
                                .take(5).collect();
                            if pending.is_empty() {
                                view! {
                                    <div style="padding:20px 16px;display:flex;align-items:center;gap:10px;">
                                        <span style="font-size:20px">"✅"</span>
                                        <div>
                                            <div style="font-size:13px;font-weight:600;color:var(--text-primary)">"Queue is clear"</div>
                                            <div style="font-size:11px;color:var(--text-muted)">"No pending verification requests."</div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div style="padding:8px 0;display:flex;flex-direction:column;">
                                        {pending.into_iter().map(|item| {
                                            let status_color = if item.status == "review" { "var(--amber)" } else { "var(--cobalt)" };
                                            let submitted = item.created_at.get(..10).map(|d| d.to_string()).unwrap_or_else(|| "—".to_string());
                                            view! {
                                                <a href="/verification" style="display:flex;align-items:center;gap:8px;padding:8px 16px;border-bottom:1px solid var(--border-subtle);text-decoration:none;transition:background 0.1s;">
                                                    <span style=format!("width:6px;height:6px;border-radius:50%;background:{};flex-shrink:0", status_color)></span>
                                                    <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">{item.entity_name.clone()}</span>
                                                    <span style="font-size:10px;color:var(--text-muted);font-family:monospace">{item.req_type.clone()}</span>
                                                    <span style="font-size:10px;color:var(--text-muted)">{submitted}</span>
                                                </a>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}
