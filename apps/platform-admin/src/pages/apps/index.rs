use leptos::prelude::*;
use crate::api::models::{PlatformAppModel, TenantStatModel};
use crate::api::networks::get_networks;
use crate::api::admin::{get_tenant_stats, impersonate_user};
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format cents → "$X" or "$X.XX" string for display.
fn fmt_mrr(cents: i64) -> String {
    if cents == 0 {
        "$0".to_string()
    } else if cents % 100 == 0 {
        format!("${}", cents / 100)
    } else {
        format!("${:.2}", cents as f64 / 100.0)
    }
}

/// Maps a canonical `app_slug` / `app_type` to (emoji, display label).
fn app_type_label(slug: &str) -> (&'static str, &'static str) {
    match slug {
        "property_management" => ("🏠", "Folio PM"),
        "anchor"              => ("⚓", "Anchor CMS"),
        "network_instance"    => ("🔗", "Network"),
        "str"                 => ("🏖️", "Atlas STR"),
        _                     => ("📦", "App"),
    }
}

#[component]
pub fn Apps() -> impl IntoView {
    let active_tab = RwSignal::new("apps".to_string());
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");

    // ── Data resources ───────────────────────────────────────────────────────
    let networks = LocalResource::new(move || async move {
        get_networks().await.unwrap_or_default()
    });

    // Fetch all tenant stats in one call — resolved against the rendered tenant below.
    let tenant_stats = LocalResource::new(move || async move {
        get_tenant_stats().await.unwrap_or_default()
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="flex items-center justify-center h-full text-on-surface-variant gap-3">
                <div class="animate-spin h-5 w-5 border-2 border-primary border-t-transparent rounded-full"></div>
                "Loading tenant registry..."
            </div>
        }>
            {move || networks.get().map(|dirs: Vec<PlatformAppModel>| {
                let grouped_map = crate::utils::group_apps_by_tenant(dirs);
                let grouped_vec: Vec<(String, String, Vec<PlatformAppModel>)> = grouped_map
                    .into_iter()
                    .map(|(tid, (name, apps))| (tid, name, apps))
                    .collect();

                // ── Empty state ──────────────────────────────────────────────
                if grouped_vec.is_empty() {
                    return view! {
                        <div class="main-area flex flex-col items-center justify-center py-24 gap-6">
                            <div class="text-5xl">"🏗"</div>
                            <h2 class="text-2xl font-bold text-on-surface">"No tenants provisioned yet"</h2>
                            <p class="text-on-surface-variant text-sm max-w-md text-center">
                                "Your platform has no tenants. Provision your first tenant to get started — it creates the app instance, domain, CMS, and admin user in one step."
                            </p>
                            <a href="/apps/create">
                                <button class="btn btn-primary px-6 py-2 rounded-lg font-semibold">
                                    "Provision First Tenant"
                                </button>
                            </a>
                        </div>
                    }.into_any();
                }

                // ── Resolve which tenant to render ───────────────────────────
                // Prefer the active_network dropdown selection; fall back to first.
                let selected_tenant_id = active_network.get().map(|id| id.to_string());
                let (tenant_id, tenant_name, apps) = selected_tenant_id
                    .as_ref()
                    .and_then(|tid| grouped_vec.iter().find(|(id, _, _)| id == tid).cloned())
                    .or_else(|| grouped_vec.first().cloned())
                    .unwrap_or_else(|| (String::new(), String::new(), Vec::new()));

                // ── Stat lookup — keyed to the RENDERED tenant_id ────────────
                // Bug fix: previously keyed off `active_network` which could be None.
                // Now we key off `tenant_id` derived above — always matches something.
                let stat: Option<TenantStatModel> = tenant_stats.get()
                    .and_then(|stats: Vec<TenantStatModel>| {
                        stats.into_iter().find(|s| s.tenant_id == tenant_id)
                    });

                let apps_val = StoredValue::new(apps);
                let total_apps = apps_val.with_value(|a| a.len());
                let live_count = apps_val.with_value(|a| {
                    a.iter().filter(|x| x.site_status.to_lowercase() == "active").count()
                });
                let beta_count = total_apps - live_count;

                // ── Derived KPI values ────────────────────────────────────────
                let plan_label = stat.as_ref()
                    .and_then(|s| s.plan.clone())
                    .unwrap_or_else(|| "—".to_string());
                let mrr_display = stat.as_ref()
                    .and_then(|s| s.mrr_cents)
                    .map(fmt_mrr);
                let profile_count = stat.as_ref().map(|s| s.profile_count);
                let listing_count = stat.as_ref().map(|s| s.listing_count);
                let tenant_status = stat.as_ref()
                    .and_then(|s| s.site_status.clone())
                    .unwrap_or_else(|| "active".to_string());
                let joined_at = stat.as_ref()
                    .and_then(|s| s.joined_at.as_ref())
                    .and_then(|d| d.get(..10))
                    .map(|d| d.to_string());

                // Setup completion score: simple 4-point check
                // (has active instance, has MRR, has profiles, has listings)
                let setup_score: u8 = {
                    let mut s = 0u8;
                    if live_count > 0 { s += 1; }
                    if mrr_display.is_some() { s += 1; }
                    if profile_count.map(|n| n > 0).unwrap_or(false) { s += 1; }
                    if listing_count.map(|n| n > 0).unwrap_or(false) { s += 1; }
                    s
                };

                // ── Impersonate ───────────────────────────────────────────────
                let impersonate_tenant_uuid: Option<Uuid> = active_network.get();
                let t_impersonate = toast.clone();
                let handle_impersonate = move |_| {
                    if let Some(uid) = impersonate_tenant_uuid {
                        let t = t_impersonate.clone();
                        leptos::task::spawn_local(async move {
                            match impersonate_user(uid).await {
                                Ok(_) => t.show_toast("Impersonating", "Session switched to tenant context.", "success"),
                                Err(e) => t.show_toast("Error", &format!("Impersonation failed: {}", e), "error"),
                            }
                        });
                    } else {
                        toast.show_toast("Error", "No tenant selected — select a network first.", "error");
                    }
                };

                // The correct link to this tenant's primary anchor instance.
                // This is what "Edit Tenant" should navigate to.
                let anchor_instance_id = stat.as_ref()
                    .and_then(|s| s.anchor_instance_id.clone());
                let edit_href = anchor_instance_id
                    .as_ref()
                    .map(|id| format!("/apps/{}/instance", id))
                    .unwrap_or_else(|| format!("/apps/{}", tenant_id));

                // App type badge chips from actual instances
                let app_type_chips: Vec<String> = {
                    let mut seen = std::collections::HashSet::new();
                    apps_val.with_value(|a| {
                        a.iter()
                            .map(|x| app_type_label(&x.app_type).1.to_string())
                            .filter(|l| seen.insert(l.clone()))
                            .collect()
                    })
                };

                let tenant_id_clone = tenant_id.clone();

                view! {
                    <div class="main-area" style="padding: 0; gap: 0;">

                        // ── Tenant Hero ──────────────────────────────────────
                        <div class="tenant-hero">
                            <div>
                                <div class="breadcrumb">
                                    <a href="/">"Platform"</a>" › "
                                    <a href="/apps">"Tenants"</a>" › "
                                    {tenant_name.clone()}
                                </div>
                                <div class="t-identity" style="display:flex;align-items:center;gap:14px;">
                                    <div class="t-avatar" style="width:40px;height:40px;border-radius:10px;background:var(--cobalt-dim,rgba(59,130,246,0.15));color:var(--cobalt,#3b82f6);font-size:16px;font-weight:800;display:flex;align-items:center;justify-content:center;flex-shrink:0;">
                                        {tenant_name.chars().next().unwrap_or('N').to_string()}
                                    </div>
                                    <div>
                                        <div class="t-name">
                                            {tenant_name.clone()}
                                            // App type chips — derived from actual instances, not hardcoded
                                            {app_type_chips.into_iter().map(|label| view! {
                                                <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt)">{label}</span>
                                            }).collect_view()}
                                            // Status badge
                                            {if tenant_status.to_lowercase() == "active" {
                                                view! { <span class="tag" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span> }.into_any()
                                            } else {
                                                view! { <span class="tag" style="color:var(--amber);border-color:var(--amber)">{tenant_status.clone()}</span> }.into_any()
                                            }}
                                            // Plan badge — from real stat.plan
                                            {if !plan_label.is_empty() && plan_label != "—" {
                                                view! { <span class="tag" style="color:var(--primary);border-color:var(--primary)">{plan_label.clone()}</span> }.into_any()
                                            } else {
                                                view! { <></> }.into_any()
                                            }}
                                        </div>
                                        <div class="t-domain">
                                            {format!("tenant_id: {} · {} app instances", tenant_id, total_apps)}
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="hero-actions">
                                <button class="btn btn-ghost" on:click=handle_impersonate>"Impersonate"</button>
                                <a href="/apps/new" class="btn btn-ghost" style="font-weight:500;text-decoration:none">"+ New App Instance"</a>
                                // Edit Tenant → correct anchor instance page, not tenant UUID
                                <a
                                    href=edit_href
                                    class="btn btn-primary"
                                    style="text-decoration:none"
                                >"Edit Tenant"</a>
                            </div>
                        </div>

                        // ── KPI Strip — all from real stat data ───────────────
                        <div class="kpi-strip">
                            <div class="kpi">
                                <div class="kpi-label">"App Instances"</div>
                                <div class="kpi-value">{total_apps}</div>
                                <div class="kpi-sub">{format!("{} live · {} beta", live_count, beta_count)}</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Total MRR"</div>
                                {match mrr_display.clone() {
                                    Some(v) => view! {
                                        <div class="kpi-value mono">{v}</div>
                                        <div class="kpi-sub">"Monthly recurring"</div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="kpi-value mono">"$0"</div>
                                        <div class="kpi-sub">"No active subscription"</div>
                                    }.into_any(),
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Profiles"</div>
                                {match profile_count {
                                    Some(n) => view! {
                                        <div class="kpi-value mono">{n.to_string()}</div>
                                        <div class="kpi-sub">"Active users"</div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="kpi-value mono">"0"</div>
                                        <div class="kpi-sub">"No profiles yet"</div>
                                    }.into_any(),
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Listings"</div>
                                {match listing_count {
                                    Some(n) => view! {
                                        <div class="kpi-value mono">{n.to_string()}</div>
                                        <div class="kpi-sub">"Active listings"</div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="kpi-value mono">"0"</div>
                                        <div class="kpi-sub">"No listings yet"</div>
                                    }.into_any(),
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Plan"</div>
                                {if plan_label != "—" && !plan_label.is_empty() {
                                    view! {
                                        <div class="kpi-value" style="color:var(--cobalt)">{plan_label.clone()}</div>
                                        <div class="kpi-sub">"Subscription tier"</div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="kpi-value" style="color:var(--text-muted);font-size:12px">"No subscription"</div>
                                        <div class="kpi-sub">"Upgrade in Billing"</div>
                                    }.into_any()
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Setup"</div>
                                <div class="kpi-value" style=move || {
                                    if setup_score == 4 { "color:var(--green)" }
                                    else if setup_score >= 2 { "color:var(--amber)" }
                                    else { "color:var(--red)" }
                                }>
                                    {format!("{}/4", setup_score)}
                                </div>
                                <div class="kpi-sub">"Instance · MRR · Users · Listings"</div>
                            </div>
                        </div>

                        // ── Tab Bar ───────────────────────────────────────────
                        <div class="tab-bar">
                            <button class=move || format!("tab {}", if active_tab.get() == "apps" { "active" } else { "" }) on:click=move |_| active_tab.set("apps".to_string())>"App Instances"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "crm" { "active" } else { "" }) on:click=move |_| active_tab.set("crm".to_string())>"CRM — All Apps"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "billing" { "active" } else { "" }) on:click=move |_| active_tab.set("billing".to_string())>"Billing"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "config" { "active" } else { "" }) on:click=move |_| active_tab.set("config".to_string())>"Configuration"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "audit" { "active" } else { "" }) on:click=move |_| active_tab.set("audit".to_string())>"Audit Log"</button>
                        </div>

                        // ── Tab Content ───────────────────────────────────────
                        <div class="content" style="padding: 20px 24px;">
                            {move || match active_tab.get().as_str() {

                                // ── APP INSTANCES TAB ────────────────────────
                                "apps" => view! {
                                    <div style="display:flex;flex-direction:column;gap:16px;">
                                        // Header strip
                                        <div style="display:flex;align-items:center;justify-content:space-between;">
                                            <div style="font-size:11px;color:var(--text-muted);">
                                                {format!("{} instance{} · {} live",
                                                    total_apps,
                                                    if total_apps == 1 { "" } else { "s" },
                                                    live_count
                                                )}
                                            </div>
                                            <a href="/apps/new"
                                               class="btn btn-ghost btn-sm"
                                               style="text-decoration:none;font-size:11px;"
                                            >
                                                "+ Provision Instance"
                                            </a>
                                        </div>

                                        // Instance list
                                        <div style="display:flex;flex-direction:column;gap:10px;">
                                            <For
                                                each=move || apps_val.with_value(|a| a.clone())
                                                key=|app| app.instance_id.clone()
                                                children=move |app| {
                                                    let status = app.site_status.to_lowercase();
                                                    let is_live = status == "active";
                                                    let is_suspended = status == "suspended";

                                                    let (status_bg, status_color, status_label) = if is_live {
                                                        ("rgba(34,197,94,0.1)", "#22c55e", "LIVE")
                                                    } else if is_suspended {
                                                        ("rgba(239,68,68,0.1)", "#ef4444", "SUSPENDED")
                                                    } else {
                                                        ("rgba(245,158,11,0.1)", "#f59e0b", "PROVISIONING")
                                                    };

                                                    let (type_bg, type_color, type_emoji, type_label) = match app.app_type.to_lowercase().as_str() {
                                                        "anchor"              => ("rgba(99,102,241,0.12)",  "#818cf8", "⚓", "Anchor CMS"),
                                                        "property_management" => ("rgba(59,130,246,0.12)",  "#60a5fa", "🏠", "Folio PM"),
                                                        "network_instance"    => ("rgba(16,185,129,0.12)", "#34d399", "🔗", "Network"),
                                                        "str"                 => ("rgba(245,158,11,0.12)", "#fbbf24", "🏖️", "STR"),
                                                        _                     => ("rgba(107,114,128,0.12)", "#9ca3af", "📦", "App"),
                                                    };

                                                    let instance_url = format!("/apps/{}/instance", app.instance_id);

                                                    view! {
                                                        // Each instance as a clean horizontal row card
                                                        <div class="instance-row"
                                                            on:click={
                                                                let url = instance_url.clone();
                                                                move |_| { let _ = web_sys::window().and_then(|w| w.location().assign(&url).ok()); }
                                                            }
                                                        >
                                                            // App type icon
                                                            <div style=format!("\
                                                                width:40px;\
                                                                height:40px;\
                                                                border-radius:8px;\
                                                                background:{};\
                                                                display:flex;\
                                                                align-items:center;\
                                                                justify-content:center;\
                                                                font-size:18px;\
                                                                flex-shrink:0;\
                                                            ", type_bg)>
                                                                {type_emoji}
                                                            </div>

                                                            // Main content block
                                                            <div style="flex:1;min-width:0;">
                                                                <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
                                                                    // App type label
                                                                    <span style=format!("font-size:13px;font-weight:700;color:{}", type_color)>
                                                                        {type_label}
                                                                    </span>
                                                                    // Status pill
                                                                    <span style=format!("\
                                                                        font-size:9.5px;\
                                                                        font-weight:700;\
                                                                        letter-spacing:0.05em;\
                                                                        padding:2px 7px;\
                                                                        border-radius:4px;\
                                                                        background:{};\
                                                                        color:{};\
                                                                    ", status_bg, status_color)>
                                                                        {status_label}
                                                                    </span>
                                                                </div>
                                                                // Domain + instance ID
                                                                <div style="font-size:11.5px;color:var(--text-muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                                                                    {app.domain.clone()}
                                                                </div>
                                                                <div style="font-size:9.5px;font-family:monospace;color:var(--text-muted);opacity:0.6;margin-top:2px;">
                                                                    {app.instance_id.clone()}
                                                                </div>
                                                            </div>

                                                            // Right side: CTA
                                                            <a
                                                                href=instance_url.clone()
                                                                class="btn btn-ghost btn-sm"
                                                                style="text-decoration:none;white-space:nowrap;flex-shrink:0;"
                                                                on:click=move |e| e.stop_propagation()
                                                            >
                                                                "Open Instance →"
                                                            </a>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>

                                        // Empty state
                                        {if total_apps == 0 {
                                            view! {
                                                <div style="\
                                                    padding:40px 24px;\
                                                    text-align:center;\
                                                    color:var(--text-muted);\
                                                    font-size:13px;\
                                                    background:var(--bg-surface);\
                                                    border:1px dashed var(--border-default);\
                                                    border-radius:8px;\
                                                ">
                                                    <div style="font-size:28px;margin-bottom:12px;">"📦"</div>
                                                    <div style="font-weight:600;color:var(--text-primary);margin-bottom:6px;">"No instances provisioned"</div>
                                                    <div style="font-size:11px;margin-bottom:16px;">
                                                        "Provision an Anchor, Folio, or Network instance to deploy a product for this tenant."
                                                    </div>
                                                    <a href="/apps/new" class="btn btn-primary" style="text-decoration:none;">
                                                        "Provision First Instance"
                                                    </a>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <></> }.into_any()
                                        }}
                                    </div>
                                }.into_any(),


                                // ── CRM — ALL APPS TAB ────────────────────────
                                "crm" => {
                                    let tid = tenant_id_clone.clone();
                                    view! {
                                        <div>
                                            // Header row with tenant-scoped counts inline
                                            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px;">
                                                <div>
                                                    <p style="color:var(--text-muted);font-size:12px;margin:0 0 4px;">
                                                        "Cross-app CRM view — accounts, contacts and opportunities across all app instances for this tenant."
                                                    </p>
                                                    <div style="display:flex;gap:16px;font-size:11px;color:var(--text-muted);">
                                                        <span>
                                                            <strong style="color:var(--text-primary)">
                                                                {profile_count.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())}
                                                            </strong>
                                                            " users"
                                                        </span>
                                                        <span>
                                                            <strong style="color:var(--text-primary)">
                                                                {listing_count.map(|n| n.to_string()).unwrap_or_else(|| "0".to_string())}
                                                            </strong>
                                                            " listings"
                                                        </span>
                                                        {joined_at.as_ref().map(|d| view! {
                                                            <span>"Joined "{d.clone()}</span>
                                                        })}
                                                    </div>
                                                </div>
                                                <div style="display:flex;gap:8px;">
                                                    // Links pre-scoped to this tenant where supported
                                                    <a href="/accounts" style="color:var(--text-link);font-size:12px;text-decoration:none">
                                                        "→ Accounts"
                                                    </a>
                                                    <span style="color:var(--text-muted)">"-"</span>
                                                    <a href="/contacts" style="color:var(--text-link);font-size:12px;text-decoration:none">
                                                        "Contacts"
                                                    </a>
                                                    <span style="color:var(--text-muted)">"-"</span>
                                                    <a href="/pipeline" style="color:var(--text-link);font-size:12px;text-decoration:none">
                                                        "Pipeline"
                                                    </a>
                                                </div>
                                            </div>

                                            <div class="two-col">
                                                // ── Profiles / Users card ────
                                                <div class="card">
                                                    <div class="card-hdr">
                                                        <span class="card-title">"Users & Profiles"</span>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/accounts", Default::default());
                                                        }>"View Accounts →"</button>
                                                    </div>
                                                    {match profile_count {
                                                        Some(n) if n > 0 => view! {
                                                            <div style="padding:16px 0;">
                                                                <div style="font-size:32px;font-weight:800;color:var(--text-primary);line-height:1">{n.to_string()}</div>
                                                                <div style="font-size:12px;color:var(--text-muted);margin-top:4px">"Registered profiles across all instances"</div>
                                                                <a href="/accounts" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">
                                                                    "→ View All Accounts"
                                                                </a>
                                                            </div>
                                                        }.into_any(),
                                                        _ => view! {
                                                            <div class="empty-state" style="padding:32px 20px;">
                                                                <div class="empty-state-icon" style="font-size:28px">"👥"</div>
                                                                <div class="empty-state-title">"No profiles yet"</div>
                                                                <div class="empty-state-body">"Users who register through this tenant's apps will appear here."</div>
                                                                <a href="/accounts" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">
                                                                    "→ Go to Accounts"
                                                                </a>
                                                            </div>
                                                        }.into_any(),
                                                    }}
                                                </div>

                                                // ── Listings card ────────────
                                                <div class="card">
                                                    <div class="card-hdr">
                                                        <span class="card-title">"Listings & Inventory"</span>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/network", Default::default());
                                                        }>"View Network →"</button>
                                                    </div>
                                                    {match listing_count {
                                                        Some(n) if n > 0 => view! {
                                                            <div style="padding:16px 0;">
                                                                <div style="font-size:32px;font-weight:800;color:var(--text-primary);line-height:1">{n.to_string()}</div>
                                                                <div style="font-size:12px;color:var(--text-muted);margin-top:4px">"Active listings across all network instances"</div>
                                                                <a href="/network" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">
                                                                    "→ View Network"
                                                                </a>
                                                            </div>
                                                        }.into_any(),
                                                        _ => view! {
                                                            <div class="empty-state" style="padding:32px 20px;">
                                                                <div class="empty-state-icon" style="font-size:28px">"📋"</div>
                                                                <div class="empty-state-title">"No listings yet"</div>
                                                                <div class="empty-state-body">"Listings published through this tenant's network instances will appear here."</div>
                                                                <a href="/network" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">
                                                                    "→ Go to Network"
                                                                </a>
                                                            </div>
                                                        }.into_any(),
                                                    }}
                                                </div>
                                            </div>

                                            // ── Pipeline / Opportunities ─────
                                            <div class="card" style="margin-top:14px;">
                                                <div class="card-hdr">
                                                    <span class="card-title">"Open Opportunities"</span>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/pipeline", Default::default());
                                                    }>"View Pipeline →"</button>
                                                </div>
                                                <div class="empty-state" style="padding:24px 20px;">
                                                    <div class="empty-state-icon" style="font-size:24px">"📊"</div>
                                                    <div class="empty-state-title">"No open deals"</div>
                                                    <div class="empty-state-body">"Pipeline opportunities for this tenant will appear here once CRM records are linked."</div>
                                                    <a href="/pipeline" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">
                                                        "→ Go to Pipeline"
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                },

                                // ── BILLING TAB ───────────────────────────────
                                "billing" => view! {
                                    <div style="display:flex;flex-direction:column;gap:14px;">
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"Subscription & Billing"</span>
                                                <a href="/billing" class="btn btn-ghost btn-sm" style="text-decoration:none;">
                                                    "→ Full Billing Dashboard"
                                                </a>
                                            </div>
                                            // Plan + MRR from real stat data
                                            <div class="stat-row">
                                                <span class="s-label">"Plan"</span>
                                                {if plan_label != "—" && !plan_label.is_empty() {
                                                    view! { <span class="s-value cobalt">{plan_label.clone()}</span> }.into_any()
                                                } else {
                                                    view! { <span class="s-value" style="color:var(--text-muted)">"No subscription"</span> }.into_any()
                                                }}
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"Monthly MRR"</span>
                                                {match mrr_display.clone() {
                                                    Some(v) => view! { <span class="s-value mono">{v}</span> }.into_any(),
                                                    None    => view! { <span class="s-value" style="color:var(--text-muted)">"$0 — no active subscription"</span> }.into_any(),
                                                }}
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"Status"</span>
                                                {if tenant_status.to_lowercase() == "active" {
                                                    view! { <span class="s-value" style="color:var(--green)">"● Active"</span> }.into_any()
                                                } else {
                                                    view! { <span class="s-value" style="color:var(--amber)">{tenant_status.clone()}</span> }.into_any()
                                                }}
                                            </div>
                                            {joined_at.as_ref().map(|d| view! {
                                                <div class="stat-row">
                                                    <span class="s-label">"Customer Since"</span>
                                                    <span class="s-value">{d.clone()}</span>
                                                </div>
                                            })}
                                            <div class="stat-row">
                                                <span class="s-label">"App Instances"</span>
                                                <span class="s-value">{total_apps.to_string()}" instances"</span>
                                            </div>
                                            <div style="margin-top:12px;padding-top:12px;border-top:1px solid var(--border,rgba(255,255,255,0.06));">
                                                <a href="/billing" style="color:var(--cobalt);font-size:12px;text-decoration:none;">
                                                    "View invoices, subscription history and payment methods →"
                                                </a>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),

                                // ── CONFIGURATION TAB ─────────────────────────
                                "config" => view! {
                                    <div style="display:flex;flex-direction:column;gap:14px;">
                                        // ── Subscription Plan ──────────────────
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"Subscription"
                                                </span>
                                                <a href="/billing" class="btn btn-ghost btn-sm" style="text-decoration:none;">
                                                    "View Billing →"
                                                </a>
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"Current Plan"</span>
                                                {if !plan_label.is_empty() && plan_label != "—" {
                                                    view! {
                                                        <span class="s-value" style="color:var(--cobalt);font-weight:700;">
                                                            {plan_label.clone()}
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="s-value" style="color:var(--text-muted);">
                                                            "No active subscription"
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"App Instances"</span>
                                                <span class="s-value">{total_apps.to_string()}" provisioned"</span>
                                            </div>
                                        </div>

                                        // ── Platform Tools ─────────────────────
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"Platform Tools"</span>
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"Feature Flags"</span>
                                                <a
                                                    href=format!("/flags?tenant_id={}", tenant_id_clone.clone())
                                                    class="s-value"
                                                    style="color:var(--cobalt);text-decoration:none;font-size:12px;"
                                                >
                                                    "Manage flags →"
                                                </a>
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"Audit Log"</span>
                                                <a
                                                    href=format!("/logs?tenant_id={}", tenant_id_clone.clone())
                                                    class="s-value"
                                                    style="color:var(--cobalt);text-decoration:none;font-size:12px;"
                                                >
                                                    "View audit log →"
                                                </a>
                                            </div>
                                            <div class="stat-row">
                                                <span class="s-label">"Verification"
                                                </span>
                                                <a
                                                    href="/verification"
                                                    class="s-value"
                                                    style="color:var(--cobalt);text-decoration:none;font-size:12px;"
                                                >
                                                    "Review queue →"
                                                </a>
                                            </div>
                                        </div>

                                        // ── Per-App Instance Config ─────────────
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"App Instance Configuration"</span>
                                            </div>
                                            {if total_apps > 0 {
                                                view! {
                                                    <div style="display:flex;flex-direction:column;">
                                                        <For
                                                            each=move || apps_val.with_value(|a| a.clone())
                                                            key=|app| app.instance_id.clone()
                                                            children=move |app| {
                                                                let (emoji, _label) = app_type_label(&app.app_type);
                                                                let url = format!("/apps/{}/instance", app.instance_id);
                                                                let status = app.site_status.to_lowercase();
                                                                let (status_color, status_text) = if status == "active" {
                                                                    ("#22c55e", "Live")
                                                                } else if status == "suspended" {
                                                                    ("#ef4444", "Suspended")
                                                                } else {
                                                                    ("#f59e0b", "Provisioning")
                                                                };
                                                                view! {
                                                                    <div class="stat-row">
                                                                        <span style="display:flex;align-items:center;gap:8px;font-size:12px;color:var(--text-primary);">
                                                                            <span>{emoji}</span>
                                                                            <span>{app.app_type.clone()}</span>
                                                                            <span style=format!("font-size:9px;font-weight:700;color:{};padding:1px 6px;background:{}22;border-radius:3px;", status_color, status_color)>
                                                                                {status_text}
                                                                            </span>
                                                                        </span>
                                                                        <a
                                                                            href=url
                                                                            style="color:var(--cobalt);font-size:12px;text-decoration:none;white-space:nowrap;"
                                                                        >
                                                                            "Configure →"
                                                                        </a>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div style="font-size:12px;color:var(--text-muted);padding:12px 14px;">
                                                        "No app instances provisioned. "
                                                        <a href="/apps/new" style="color:var(--cobalt);text-decoration:none;">"Provision one →"</a>
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }.into_any(),


                                // ── AUDIT LOG TAB ─────────────────────────────
                                "audit" => {
                                    let tid_audit = tenant_id_clone.clone();
                                    view! {
                                        <div style="display:flex;flex-direction:column;gap:14px;">
                                            <div class="card">
                                                <div class="card-hdr">
                                                    <span class="card-title">"Security Audit Ledger"</span>
                                                    // Pre-filtered link to the actual audit log
                                                    <a
                                                        href=format!("/logs?tenant_id={}", tid_audit)
                                                        class="btn btn-ghost btn-sm"
                                                        style="text-decoration:none;"
                                                    >
                                                        "→ Open Full Log"
                                                    </a>
                                                </div>
                                                <div style="font-size:12px;color:var(--text-muted);padding:8px 0 4px;">
                                                    "Immutable record of all state changes across this tenant's app instances. Audit entries span authentication, flag rollouts, billing events, and provisioning operations."
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Coverage"</span>
                                                    <span class="s-value">"All "{total_apps.to_string()}" app instances"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Retention"</span>
                                                    <span class="s-value">"365 days"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Tenant Scope"</span>
                                                    <span class="s-value" style="font-family:monospace;font-size:10px;color:var(--text-muted)">
                                                        {tenant_id_clone.clone()}
                                                    </span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Export"</span>
                                                    <button
                                                        class="btn btn-ghost btn-sm opacity-40 cursor-not-allowed"
                                                        title="Audit log CSV export endpoint pending"
                                                        disabled
                                                        style="padding:0"
                                                    >"Download CSV"</button>
                                                </div>
                                                <div style="margin-top:12px;padding-top:12px;border-top:1px solid var(--border,rgba(255,255,255,0.06));">
                                                    <a
                                                        href=format!("/logs?tenant_id={}", tenant_id_clone.clone())
                                                        style="color:var(--cobalt);font-size:12px;text-decoration:none;"
                                                    >
                                                        "View all audit events for this tenant →"
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                },

                                _ => view! {}.into_any()
                            }}
                        </div>
                    </div>
                }.into_any()
            })}
        </Suspense>
    }
}
