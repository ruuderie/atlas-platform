/// Tenant Detail — `/tenants/:tenant_id`
///
/// This is the canonical per-tenant view. It is keyed by the `:tenant_id`
/// **path parameter**, so the displayed tenant is always determined by the URL —
/// never by the global `active_network` dropdown context signal.
///
/// Previously, `/apps` served this purpose but read from `active_network`,
/// meaning "click tenant → navigate to /apps?tenant=X" silently displayed
/// whatever tenant was last selected in the nav dropdown. That bug is fixed here.
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::models::{PlatformAppModel, TenantStatModel};
use crate::api::networks::get_networks;
use crate::api::admin::{get_tenant_stats, impersonate_user};

// ── helpers (duplicated from apps/index.rs to keep files independent) ─────────

fn fmt_mrr(cents: i64) -> String {
    if cents == 0 { "$0".to_string() }
    else if cents % 100 == 0 { format!("${}", cents / 100) }
    else { format!("${:.2}", cents as f64 / 100.0) }
}

fn app_type_label(slug: &str) -> (&'static str, &'static str) {
    match slug {
        "property_management" | "folio" => ("🏠", "Folio PM"),
        "anchor"                        => ("⚓", "Anchor CMS"),
        "network_instance" | "network"  => ("🔗", "Network"),
        "str"                           => ("🏖️", "Atlas STR"),
        _                               => ("📦", "App"),
    }
}

// ── component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantDetail() -> impl IntoView {
    // ── Route param — THIS is what determines which tenant is shown ──────────
    let params = use_params_map();
    let tenant_id_param = move || params.with(|p| p.get("tenant_id").unwrap_or_default());

    let active_tab = RwSignal::new("apps".to_string());
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Data resources ────────────────────────────────────────────────────────
    let networks = LocalResource::new(move || async move {
        get_networks().await.unwrap_or_default()
    });

    let tenant_stats = LocalResource::new(move || async move {
        get_tenant_stats().await.unwrap_or_default()
    });

    view! {
        <Suspense fallback=move || view! {
            <div style="display:flex;align-items:center;justify-content:center;height:200px;gap:10px;color:var(--text-muted);">
                <div style="width:18px;height:18px;border:2px solid var(--primary);border-top-color:transparent;border-radius:50%;animation:spin 0.7s linear infinite;"></div>
                "Loading tenant…"
            </div>
        }>
        {move || {
            let tid = tenant_id_param();
            let all_apps: Vec<PlatformAppModel> = networks.get().unwrap_or_default();

            // Find this tenant's apps — keyed by URL path param, NOT by dropdown
            let apps: Vec<PlatformAppModel> = all_apps.into_iter()
                .filter(|a| a.tenant_id == tid)
                .collect();

            // Tenant name from the first app (or from stats)
            let tenant_name_from_apps = apps.first().map(|a| a.name.clone()).unwrap_or_else(|| tid.clone());

            // Stat lookup — also keyed by URL param
            let stat: Option<TenantStatModel> = tenant_stats.get()
                .and_then(|stats: Vec<TenantStatModel>| {
                    stats.into_iter().find(|s| s.tenant_id == tid)
                });

            let tenant_name = stat.as_ref()
                .map(|s| s.name.clone())
                .unwrap_or(tenant_name_from_apps);

            // Not found
            if stat.is_none() && apps.is_empty() {
                return view! {
                    <div class="main-area" style="display:flex;flex-direction:column;align-items:center;justify-content:center;padding:80px 24px;gap:16px;text-align:center;">
                        <div style="font-size:48px;">"🔍"</div>
                        <h2 style="font-size:18px;font-weight:700;color:var(--text-primary);margin:0;">"Tenant not found"</h2>
                        <p style="color:var(--text-muted);font-size:13px;max-width:400px;margin:0;">
                            "No tenant with ID "{tid.clone()}" exists on this platform."
                        </p>
                        <a href="/tenants" class="btn btn-ghost" style="text-decoration:none;margin-top:8px;">
                            "← Back to Tenants"
                        </a>
                    </div>
                }.into_any();
            }

            let apps_val = StoredValue::new(apps.clone());
            let total_apps = apps_val.with_value(|a| a.len());
            let live_count = apps_val.with_value(|a| {
                a.iter().filter(|x| x.site_status.to_lowercase() == "active").count()
            });
            let beta_count = total_apps - live_count;

            // ── Derived from stat ─────────────────────────────────────────────
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

            // Setup score
            let setup_score: u8 = {
                let mut s = 0u8;
                if live_count > 0 { s += 1; }
                if mrr_display.is_some() { s += 1; }
                if profile_count.map(|n| n > 0).unwrap_or(false) { s += 1; }
                if listing_count.map(|n| n > 0).unwrap_or(false) { s += 1; }
                s
            };

            // App type chips
            let app_type_chips: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                apps_val.with_value(|a| {
                    a.iter()
                        .map(|x| app_type_label(&x.app_type).1.to_string())
                        .filter(|l| seen.insert(l.clone()))
                        .collect()
                })
            };

            // Impersonate
            let tid_for_impersonate = tid.clone();
            let t_impersonate = toast.clone();
            let handle_impersonate = move |_| {
                if let Ok(uid) = uuid::Uuid::parse_str(&tid_for_impersonate) {
                    let t = t_impersonate.clone();
                    leptos::task::spawn_local(async move {
                        match impersonate_user(uid).await {
                            Ok(_) => t.show_toast("Impersonating", "Session switched to tenant context.", "success"),
                            Err(e) => t.show_toast("Error", &format!("Impersonation failed: {}", e), "error"),
                        }
                    });
                } else {
                    toast.show_toast("Error", "Invalid tenant ID.", "error");
                }
            };

            // Edit → anchor instance config, or fallback to /apps/:tid
            let anchor_instance_id = stat.as_ref()
                .and_then(|s| s.anchor_instance_id.clone());
            let edit_href = anchor_instance_id
                .as_ref()
                .map(|id| format!("/apps/{}/instance", id))
                .unwrap_or_else(|| format!("/tenants/{}", tid));

            let tid_clone = tid.clone();

            view! {
                <div class="main-area" style="padding:0;gap:0;">

                    // ── Tenant Hero ─────────────────────────────────────────
                    <div class="tenant-hero">
                        <div>
                            <div class="breadcrumb">
                                <a href="/">"Platform"</a>" › "
                                <a href="/tenants">"Tenants"</a>" › "
                                {tenant_name.clone()}
                            </div>
                            <div class="t-identity" style="display:flex;align-items:center;gap:14px;">
                                <div class="t-avatar" style="width:40px;height:40px;border-radius:10px;background:var(--cobalt-dim,rgba(59,130,246,0.15));color:var(--cobalt,#3b82f6);font-size:16px;font-weight:800;display:flex;align-items:center;justify-content:center;flex-shrink:0;">
                                    {tenant_name.chars().next().unwrap_or('?').to_string()}
                                </div>
                                <div>
                                    <div class="t-name">
                                        {tenant_name.clone()}
                                        {app_type_chips.into_iter().map(|label| view! {
                                            <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt)">{label}</span>
                                        }).collect_view()}
                                        {if tenant_status.to_lowercase() == "active" {
                                            view! { <span class="tag" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span> }.into_any()
                                        } else {
                                            view! { <span class="tag" style="color:var(--amber);border-color:var(--amber)">{tenant_status.clone()}</span> }.into_any()
                                        }}
                                        {if !plan_label.is_empty() && plan_label != "—" {
                                            view! { <span class="tag" style="color:var(--primary);border-color:var(--primary)">{plan_label.clone()}</span> }.into_any()
                                        } else {
                                            view! { <></> }.into_any()
                                        }}
                                    </div>
                                    <div class="t-domain">
                                        {format!("tenant_id: {} · {} app instances", tid, total_apps)}
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="hero-actions">
                            <button class="btn btn-ghost" on:click=handle_impersonate>"Impersonate"</button>
                            <a href="/apps/new" class="btn btn-ghost" style="font-weight:500;text-decoration:none;">"+ New App Instance"</a>
                            <a href=edit_href class="btn btn-primary" style="text-decoration:none;">"Edit Tenant"</a>
                        </div>
                    </div>

                    // ── KPI Strip ───────────────────────────────────────────
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

                    // ── Tab Bar ─────────────────────────────────────────────
                    <div class="tab-bar">
                        <button class=move || format!("tab {}", if active_tab.get() == "apps" { "active" } else { "" }) on:click=move |_| active_tab.set("apps".to_string())>"App Instances"</button>
                        <button class=move || format!("tab {}", if active_tab.get() == "crm" { "active" } else { "" }) on:click=move |_| active_tab.set("crm".to_string())>"CRM — All Apps"</button>
                        <button class=move || format!("tab {}", if active_tab.get() == "billing" { "active" } else { "" }) on:click=move |_| active_tab.set("billing".to_string())>"Billing"</button>
                        <button class=move || format!("tab {}", if active_tab.get() == "config" { "active" } else { "" }) on:click=move |_| active_tab.set("config".to_string())>"Configuration"</button>
                        <button class=move || format!("tab {}", if active_tab.get() == "audit" { "active" } else { "" }) on:click=move |_| active_tab.set("audit".to_string())>"Audit Log"</button>
                    </div>

                    // ── Tab Content ─────────────────────────────────────────
                    <div class="content" style="padding:20px 24px;">
                        {move || match active_tab.get().as_str() {

                            // ── App Instances Tab ─────────────────────────────
                            "apps" => view! {
                                <div style="display:flex;flex-direction:column;gap:16px;">
                                    <div style="display:flex;align-items:center;justify-content:space-between;">
                                        <div style="font-size:11px;color:var(--text-muted);">
                                            {format!("{} instance{} · {} live",
                                                total_apps,
                                                if total_apps == 1 { "" } else { "s" },
                                                live_count
                                            )}
                                        </div>
                                        <a href="/apps/new" class="btn btn-ghost btn-sm" style="text-decoration:none;font-size:11px;">
                                            "+ Provision Instance"
                                        </a>
                                    </div>

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
                                                    "anchor"                        => ("rgba(99,102,241,0.12)",  "#818cf8", "⚓", "Anchor CMS"),
                                                    "property_management" | "folio" => ("rgba(59,130,246,0.12)",  "#60a5fa", "🏠", "Folio PM"),
                                                    "network_instance" | "network"  => ("rgba(16,185,129,0.12)", "#34d399", "🔗", "Network"),
                                                    "str"                           => ("rgba(245,158,11,0.12)", "#fbbf24", "🏖️", "STR"),
                                                    _                               => ("rgba(107,114,128,0.12)", "#9ca3af", "📦", "App"),
                                                };

                                                let instance_url = format!("/apps/{}/instance", app.instance_id);

                                                view! {
                                                    <div class="instance-row"
                                                        on:click={
                                                            let url = instance_url.clone();
                                                            move |_| { let _ = web_sys::window().and_then(|w| w.location().assign(&url).ok()); }
                                                        }
                                                    >
                                                        <div style=format!("width:40px;height:40px;border-radius:8px;background:{};display:flex;align-items:center;justify-content:center;font-size:18px;flex-shrink:0;", type_bg)>
                                                            {type_emoji}
                                                        </div>
                                                        <div style="flex:1;min-width:0;">
                                                            <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;">
                                                                <span style=format!("font-size:13px;font-weight:700;color:{}", type_color)>{type_label}</span>
                                                                <span style=format!("font-size:9.5px;font-weight:700;letter-spacing:0.05em;padding:2px 7px;border-radius:4px;background:{};color:{};", status_bg, status_color)>
                                                                    {status_label}
                                                                </span>
                                                            </div>
                                                            <div style="font-size:11.5px;color:var(--text-muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                                                                {app.domain.clone()}
                                                            </div>
                                                            <div style="font-size:9.5px;font-family:monospace;color:var(--text-muted);opacity:0.6;margin-top:2px;">
                                                                {app.instance_id.clone()}
                                                            </div>
                                                        </div>
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

                                    {if total_apps == 0 {
                                        view! {
                                            <div style="padding:40px 24px;text-align:center;color:var(--text-muted);font-size:13px;background:var(--bg-surface);border:1px dashed var(--border-default);border-radius:8px;">
                                                <div style="font-size:28px;margin-bottom:12px;">"📦"</div>
                                                <div style="font-weight:600;color:var(--text-primary);margin-bottom:6px;">"No instances provisioned"</div>
                                                <div style="font-size:11px;margin-bottom:16px;">"Provision an Anchor, Folio, or Network instance to deploy a product for this tenant."</div>
                                                <a href="/apps/new" class="btn btn-primary" style="text-decoration:none;">"Provision First Instance"</a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }}
                                </div>
                            }.into_any(),

                            // ── Billing Tab ───────────────────────────────────
                            "billing" => view! {
                                <div style="display:flex;flex-direction:column;gap:14px;">
                                    <div class="card">
                                        <div class="card-hdr">
                                            <span class="card-title">"Subscription & Billing"</span>
                                            <a href="/billing" class="btn btn-ghost btn-sm" style="text-decoration:none;">"→ Full Billing Dashboard"</a>
                                        </div>
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
                                    </div>
                                </div>
                            }.into_any(),

                            // ── Configuration Tab ─────────────────────────────
                            "config" => view! {
                                <div style="display:flex;flex-direction:column;gap:14px;">
                                    <div class="card">
                                        <div class="card-hdr">
                                            <span class="card-title">"Platform Tools"</span>
                                        </div>
                                        <div class="stat-row">
                                            <span class="s-label">"Feature Flags"</span>
                                            <a href=format!("/flags?tenant_id={}", tid_clone.clone()) class="s-value" style="color:var(--cobalt);text-decoration:none;font-size:12px;">"Manage flags →"</a>
                                        </div>
                                        <div class="stat-row">
                                            <span class="s-label">"Audit Log"</span>
                                            <a href=format!("/logs?tenant_id={}", tid_clone.clone()) class="s-value" style="color:var(--cobalt);text-decoration:none;font-size:12px;">"View audit log →"</a>
                                        </div>
                                    </div>
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
                                                                    <a href=url style="color:var(--cobalt);font-size:12px;text-decoration:none;white-space:nowrap;">"Configure →"</a>
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

                            // ── Audit Tab ─────────────────────────────────────
                            "audit" => view! {
                                <div style="display:flex;flex-direction:column;gap:14px;">
                                    <div class="card">
                                        <div class="card-hdr">
                                            <span class="card-title">"Security Audit Ledger"</span>
                                            <a href=format!("/logs?tenant_id={}", tid_clone.clone()) class="btn btn-ghost btn-sm" style="text-decoration:none;">"→ Open Full Log"</a>
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
                                            <span class="s-value" style="font-family:monospace;font-size:10px;color:var(--text-muted)">{tid_clone.clone()}</span>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),

                            // ── CRM Tab ───────────────────────────────────────
                            _ => view! {
                                <div style="padding:32px 0;text-align:center;color:var(--text-muted);font-size:13px;">
                                    "Coming soon."
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </div>
            }.into_any()
        }}
        </Suspense>
    }
}
