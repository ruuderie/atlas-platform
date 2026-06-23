use leptos::prelude::*;
use crate::api::models::{PlatformAppModel, TenantStatModel};
use crate::api::networks::get_networks;
use crate::api::admin::{get_tenant_stats, impersonate_user};
use uuid::Uuid;

#[component]
pub fn Apps() -> impl IntoView {
    let (trigger_fetch, _set_trigger_fetch) = signal(0);
    let active_tab = RwSignal::new("apps".to_string());
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");

    let networks = LocalResource::new(
        move || {
            trigger_fetch.get();
            async move { get_networks().await.unwrap_or_default() }
        }
    );

    // Fetch tenant stats — used to populate KPI strip with real MRR/profile/listing counts.
    let tenant_stats = LocalResource::new(move || {
        trigger_fetch.get();
        async move { get_tenant_stats().await.unwrap_or_default() }
    });

    view! {
        <Suspense fallback=move || view! { <div class="text-on-surface-variant">"Loading tenant details..."</div> }>
            {move || networks.get().map(|dirs: Vec<PlatformAppModel>| {
                let grouped_map = crate::utils::group_apps_by_tenant(dirs);
                let grouped_vec: Vec<(String, String, Vec<PlatformAppModel>)> = grouped_map
                    .into_iter()
                    .map(|(tid, (name, apps))| (tid, name, apps))
                    .collect();

                let selected_tenant_id = active_network.get().map(|id| id.to_string());
                let selected = if let Some(ref tid) = selected_tenant_id {
                    grouped_vec.iter().find(|(id, _, _)| id == tid).cloned()
                } else {
                    None
                };

                // If no tenant selected and no tenants exist, show empty state
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

                // grouped_vec is guaranteed non-empty here (empty case returned early above)
                let (tenant_id, tenant_name, apps) = selected
                    .or_else(|| grouped_vec.first().cloned())
                    .unwrap_or_else(|| (String::new(), String::new(), Vec::new()));


                // Derive stats for this tenant from the tenant_stats resource.
                let tid_for_stats = tenant_id.clone();
                let stat: Option<TenantStatModel> = tenant_stats.get()
                    .and_then(|stats: Vec<TenantStatModel>| {
                        stats.into_iter().find(|s| s.tenant_id == tid_for_stats)
                    });

                // Tenant UUID for impersonate — only set if the active_network matches a real UUID.
                let impersonate_tenant_uuid: Option<Uuid> = active_network.get();

                let apps_val = StoredValue::new(apps);
                let total_apps = apps_val.with_value(|a| a.len());
                let live_count = apps_val.with_value(|a| a.iter().filter(|x| x.site_status.to_lowercase() == "active").count());
                let beta_count = total_apps - live_count;

                // KPI values derived from real API where available.
                let mrr_display = stat.as_ref()
                    .and_then(|s| s.mrr_cents)
                    .map(|c| format!("${:.0}", c as f64 / 100.0));
                let profile_count = stat.as_ref().map(|s| s.profile_count.to_string());
                let listing_count = stat.as_ref().map(|s| s.listing_count.to_string());

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

                view! {
                    <div class="main-area" style="padding: 0; gap: 0;">
                        // ── Tenant Hero ──
                        <div class="tenant-hero">
                            <div>
                                <div class="breadcrumb">
                                    <a href="/">"Platform"</a>" › "<a href="/apps">"Tenants"</a>" › "{tenant_name.clone()}
                                </div>
                                <div class="t-identity" style="display:flex;align-items:center;gap:14px;">
                                    <div class="t-avatar" style="width:40px;height:40px;border-radius:10px;background:var(--cobalt-dim,rgba(59,130,246,0.15));color:var(--cobalt,#3b82f6);font-size:16px;font-weight:800;display:flex;align-items:center;justify-content:center;flex-shrink:0;">{tenant_name.chars().next().unwrap_or('N').to_string()}</div>
                                    <div>
                                        <div class="t-name">
                                            {tenant_name.clone()}
                                            <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt)">"PM"</span>
                                            <span class="tag" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                            <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt)">"Enterprise"</span>
                                        </div>
                                        <div class="t-domain">{format!("tenant_id: {} · {} app instances", tenant_id, total_apps)}</div>
                                    </div>
                                </div>
                            </div>
                            <div class="hero-actions">
                                <button class="btn btn-ghost" on:click=handle_impersonate>"Impersonate"</button>
                                // → /apps/new = real tenant provisioning wizard
                                <a href="/apps/new" class="btn btn-ghost" style="font-weight:500;text-decoration:none">"+ New App Instance"</a>
                                // → /apps/:id = tenant detail & settings
                                <a
                                    href=move || format!("/apps/{}", tenant_id.clone())
                                    class="btn btn-primary"
                                    style="text-decoration:none"
                                >"Edit Tenant"</a>
                            </div>
                        </div>

                        // ── KPI Strip ──
                        <div class="kpi-strip">
                            <div class="kpi">
                                <div class="kpi-label">"App Instances"</div>
                                <div class="kpi-value">{total_apps}</div>
                                <div class="kpi-sub">{format!("{} live · {} beta", live_count, beta_count)}</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Total MRR"</div>
                                {move || match mrr_display.clone() {
                                    Some(v) => view! {
                                        <div class="kpi-value mono">{v}</div>
                                        <div class="kpi-sub">"From billing API"</div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="kpi-value" style="font-size:11px;color:var(--amber,#f59e0b);opacity:0.7">"Pending"</div>
                                        <div class="kpi-sub">"tenant_stats endpoint"</div>
                                    }.into_any(),
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Profiles"</div>
                                {move || match profile_count.clone() {
                                    Some(v) => view! {
                                        <div class="kpi-value mono">{v}</div>
                                        <div class="kpi-sub">"Active users"</div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="kpi-value" style="font-size:11px;color:var(--amber,#f59e0b);opacity:0.7">"Pending"</div>
                                        <div class="kpi-sub">"tenant_stats endpoint"</div>
                                    }.into_any(),
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Listings"</div>
                                {move || match listing_count.clone() {
                                    Some(v) => view! {
                                        <div class="kpi-value mono">{v}</div>
                                        <div class="kpi-sub">"Active listings"</div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="kpi-value" style="font-size:11px;color:var(--amber,#f59e0b);opacity:0.7">"Pending"</div>
                                        <div class="kpi-sub">"tenant_stats endpoint"</div>
                                    }.into_any(),
                                }}
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"CRM"</div>
                                <div class="kpi-value" style="font-size:11px;color:var(--amber,#f59e0b);opacity:0.7">"Pending"</div>
                                <div class="kpi-sub">"per-tenant CRM stats API"</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Health Score"</div>
                                <div class="kpi-value" style="font-size:11px;color:var(--amber,#f59e0b);opacity:0.7">"Pending"</div>
                                <div class="kpi-sub">"G27 scorecard aggregate"</div>
                            </div>
                        </div>

                        // ── Tab Bar ──
                        <div class="tab-bar">
                            <button class=move || format!("tab {}", if active_tab.get() == "apps" { "active" } else { "" }) on:click=move |_| active_tab.set("apps".to_string())>"App Instances"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "crm" { "active" } else { "" }) on:click=move |_| active_tab.set("crm".to_string())>"CRM — All Apps"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "billing" { "active" } else { "" }) on:click=move |_| active_tab.set("billing".to_string())>"Billing"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "config" { "active" } else { "" }) on:click=move |_| active_tab.set("config".to_string())>"Configuration"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "audit" { "active" } else { "" }) on:click=move |_| active_tab.set("audit".to_string())>"Audit Log"</button>
                        </div>

                        <div class="content" style="padding: 20px 24px;">
                            {move || match active_tab.get().as_str() {
                                "apps" => view! {
                                    <div>
                                        <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:14px;">
                                            <div class="section-label" style="margin-bottom:0">{format!("{} App Instances · {} Live · {} Beta", total_apps, live_count, beta_count)}</div>
                                            // → /apps/new = real provisioning wizard
                                            <a href="/apps/new" class="btn btn-ghost btn-sm" style="text-decoration:none">"+ Provision New Instance"</a>
                                        </div>

                                        <div class="apps-grid">
                                            <For
                                                each=move || apps_val.with_value(|a| a.clone())
                                                key=|app| app.instance_id.clone()
                                                children=move |app| {
                                                    let is_live = app.site_status.to_lowercase() == "active";
                                                    let app_type_icon = app.app_type.clone();
                                                    let app_manage_url = format!("/apps/{}", app.instance_id);

                                                    view! {
                                                        <div class="app-card" on:click={
                                                            let app_manage_url = app_manage_url.clone();
                                                            move |_| {
                                                                let navigate = leptos_router::hooks::use_navigate();
                                                                navigate(&app_manage_url, Default::default());
                                                            }
                                                        }>
                                                            <div class="app-card-hdr">
                                                                <div class="app-icon" style="background:var(--cobalt-dim);color:var(--cobalt)">{app_type_icon}</div>
                                                                {if is_live {
                                                                    view! {
                                                                        <div class="app-mode mode-live">
                                                                            <span class="live-dot" style="background:var(--green)"></span>
                                                                            "Live"
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <div class="app-mode mode-beta">"Beta"</div>
                                                                    }.into_any()
                                                                }}
                                                            </div>
                                                            <div class="app-name">{app.name.clone()}</div>
                                                            <div class="app-domain">{app.domain.clone()}</div>
                                                            {
                                                                let at = app.app_type.to_lowercase();
                                                                let (s1_label, s2_label, s3_label): (&str, &str, &str) = match at.as_str() {
                                                                    "anchor" => ("Pages", "Media", "Forms"),
                                                                    "str"    => ("Listings", "Bookings", "Revenue"),
                                                                    "net" | "network" => ("Listings", "Views", "Members"),
                                                                    "com"    => ("Units", "Tenants", "MRR"),
                                                                    _        => ("Leads", "Assets", "MRR"),
                                                                };
                                                                let features: Vec<(&str, bool)> = match at.as_str() {
                                                                    "anchor" => vec![
                                                                        ("Pages", true), ("Media", true), ("Forms", true),
                                                                        ("SEO", true), ("Analytics", true), ("Custom Fields", false),
                                                                    ],
                                                                    "str" => vec![
                                                                        ("Listings", true), ("Bookings", true), ("Payments", true),
                                                                        ("Pricing", true), ("Reviews", false), ("Analytics", true),
                                                                    ],
                                                                    "net" | "network" => vec![
                                                                        ("Directory", true), ("Search", true), ("Listings", true),
                                                                        ("Maps", true), ("Profiles", true), ("Messaging", false),
                                                                    ],
                                                                    "com" => vec![
                                                                        ("Listings", true), ("Leases", true), ("Payments", true),
                                                                        ("Tenants", true), ("Analytics", true), ("Maintenance", false),
                                                                    ],
                                                                    _ => vec![
                                                                        ("Leads", true), ("Listings", true), ("Payments", true),
                                                                        ("Analytics", true), ("Events", true), ("Custom Fields", false),
                                                                    ],
                                                                };
                                                                view! {
                                                                    <div class="app-stats" style="display:grid;grid-template-columns:repeat(3,1fr);gap:8px;margin-top:10px;">
                                                                        <div class="app-stat" style="display:flex;flex-direction:column;gap:2px;">
                                                                            <div class="app-stat-label" style="font-size:10px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">{s1_label}</div>
                                                                            <div class="app-stat-value" style="font-size:15px;font-weight:700;color:var(--text-muted);">"—"</div>
                                                                        </div>
                                                                        <div class="app-stat" style="display:flex;flex-direction:column;gap:2px;">
                                                                            <div class="app-stat-label" style="font-size:10px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">{s2_label}</div>
                                                                            <div class="app-stat-value" style="font-size:15px;font-weight:700;color:var(--text-muted);">"—"</div>
                                                                        </div>
                                                                        <div class="app-stat" style="display:flex;flex-direction:column;gap:2px;">
                                                                            <div class="app-stat-label" style="font-size:10px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">{s3_label}</div>
                                                                            <div class="app-stat-value" style="font-size:15px;font-weight:700;color:var(--text-muted);">"—"</div>
                                                                        </div>
                                                                    </div>
                                                                    <div class="app-modules" style="display:flex;flex-wrap:wrap;gap:5px;margin-top:10px;">
                                                                        {features.into_iter().map(|(label, active)| {
                                                                            view! {
                                                                                <span class=if active { "mod-chip on" } else { "mod-chip" }>{label}</span>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }
                                                            }
                                                            <div class="app-card-footer">
                                                                <span class="app-footer-meta">{format!("{} instance", app.app_type.to_uppercase())}</span>
                                                                <div class="app-footer-actions">
                                                                    // Config → detail/settings view for this instance
                                                                    {
                                                                        let url = StoredValue::new(app_manage_url.clone());
                                                                        view! {
                                                                            <a
                                                                                href=move || url.get_value()
                                                                                class="btn btn-ghost btn-sm"
                                                                                style="text-decoration:none"
                                                                                on:click=move |e| e.stop_propagation()
                                                                            >"Config"</a>
                                                                        }
                                                                    }
                                                                    <button class="btn btn-primary btn-sm" on:click={
                                                                        let app_manage_url = app_manage_url.clone();
                                                                        move |e| {
                                                                            e.stop_propagation();
                                                                            let navigate = leptos_router::hooks::use_navigate();
                                                                            navigate(&app_manage_url, Default::default());
                                                                        }
                                                                    }>"Open →"</button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>

                                        // Instance analytics pending per-instance stats API.
                                        <div style="margin-top:20px;padding:14px 16px;background:var(--surface-2,#1a1a2e);border:1px solid var(--border,rgba(255,255,255,0.08));border-radius:10px;color:var(--text-muted);font-size:12px;text-align:center;">
                                            "Instance analytics will appear here once real data is available from the platform stats API."
                                        </div>
                                    </div>
                                }.into_any(),
                                "crm" => view! {
                                    <div>
                                        <p style="color:var(--text-muted);font-size:12px;margin-bottom:16px">
                                            "Cross-app CRM view — accounts, contacts and opportunities across all app instances for this tenant. "
                                            <a href="/accounts" style="color:var(--text-link)">"→ Open Accounts"</a>
                                            " · "
                                            <a href="/contacts" style="color:var(--text-link)">"→ Contacts"</a>
                                            " · "
                                            <a href="/pipeline" style="color:var(--text-link)">"→ Pipeline"</a>
                                        </p>
                                        <div class="two-col">
                                            <div class="card">
                                                <div class="card-hdr">
                                                    <span class="card-title">"Recent Accounts"</span>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/accounts", Default::default());
                                                    }>"View All →"</button>
                                                </div>
                                                <div class="empty-state" style="padding:32px 20px;">
                                                    <div class="empty-state-icon" style="font-size:28px">"🏢"</div>
                                                    <div class="empty-state-title">"No accounts yet"</div>
                                                    <div class="empty-state-body">"Accounts created for this tenant will appear here."</div>
                                                    <a href="/accounts" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">"→ Go to Accounts"</a>
                                                </div>
                                            </div>
                                            <div class="card">
                                                <div class="card-hdr">
                                                    <span class="card-title">"Open Opportunities"</span>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/pipeline", Default::default());
                                                    }>"View All →"</button>
                                                </div>
                                                <div class="empty-state" style="padding:32px 20px;">
                                                    <div class="empty-state-icon" style="font-size:28px">"📊"</div>
                                                    <div class="empty-state-title">"No open deals"</div>
                                                    <div class="empty-state-body">"Pipeline opportunities for this tenant will appear here."</div>
                                                    <a href="/pipeline" class="btn btn-ghost btn-sm" style="margin-top:12px;text-decoration:none;">"→ Go to Pipeline"</a>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                "billing" => view! {
                                    <div>
                                        <p style="color:var(--text-muted);font-size:12px">
                                            "Consolidated billing across all app instances. "
                                            <a href="/billing" style="color:var(--text-link)">"View detailed billing →"</a>
                                        </p>
                                    </div>
                                }.into_any(),
                                "config" => view! {
                                    <div style="display:flex;flex-direction:column;gap:14px;">
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"Tenant Settings"</span>
                                                <a href="/developer" class="btn btn-ghost btn-sm" style="text-decoration:none;">"→ Full Config"</a>
                                            </div>
                                            <div class="stat-row"><span class="s-label">"Plan"</span><span class="s-value cobalt">"Enterprise"</span></div>
                                            <div class="stat-row"><span class="s-label">"Region"</span><span class="s-value">"us-east-1"</span></div>
                                            <div class="stat-row"><span class="s-label">"Data Residency"</span><span class="s-value">"US — AWS"</span></div>
                                            <div class="stat-row"><span class="s-label">"Feature Flags"</span>
                                                <a href="/flags" class="s-value" style="color:var(--cobalt);text-decoration:none;">"→ Manage flags"</a>
                                            </div>
                                            <div class="stat-row"><span class="s-label">"Audit Log"</span>
                                                <a href="/logs" class="s-value" style="color:var(--cobalt);text-decoration:none;">"→ View audit log"</a>
                                            </div>
                                        </div>
                                        <div class="card">
                                            <div class="card-hdr"><span class="card-title">"Per-App Overrides"</span></div>
                                            <div style="font-size:12px;color:var(--text-muted);padding:8px 0;">
                                                "Per-app settings are managed inside each app instance. Select an instance from the \"App Instances\" tab to configure."
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                "audit" => view! {
                                    <div style="display:flex;flex-direction:column;gap:14px;">
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"Security Audit Ledger"</span>
                                                <a href="/logs" class="btn btn-ghost btn-sm" style="text-decoration:none;">"→ Open Full Log"</a>
                                            </div>
                                            <div style="font-size:12px;color:var(--text-muted);padding:8px 0 4px;">
                                                "Immutable record of all state changes across this tenant's app instances. Audit entries span authentication, flag rollouts, billing events, and provisioning operations."
                                            </div>
                                            <div class="stat-row"><span class="s-label">"Coverage"</span><span class="s-value">"All "{total_apps.to_string()}" app instances"</span></div>
                                            <div class="stat-row"><span class="s-label">"Retention"</span><span class="s-value">"365 days"</span></div>
                                            <div class="stat-row"><span class="s-label">"Export"</span>
                                                <button
                                                    class="btn btn-ghost btn-sm opacity-40 cursor-not-allowed"
                                                    title="Audit log CSV export endpoint pending"
                                                    disabled
                                                    style="padding:0"
                                                >"Download CSV"</button>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                _ => view! {}.into_any()
                            }}
                        </div>
                    </div>
                }.into_any()
            })}
        </Suspense>
    }
}
