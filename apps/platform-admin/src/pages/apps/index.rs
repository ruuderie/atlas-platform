use leptos::prelude::*;
use crate::api::models::PlatformAppModel;
use crate::api::networks::get_networks;

#[component]
pub fn Apps() -> impl IntoView {
    let (trigger_fetch, _set_trigger_fetch) = signal(0);
    let active_tab = RwSignal::new("apps".to_string());
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let active_network = use_context::<ReadSignal<Option<uuid::Uuid>>>().expect("active network context");

    let networks = LocalResource::new(
        move || { 
            trigger_fetch.get();
            async move { get_networks().await.unwrap_or_default() }
        }
    );

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

                let (tenant_id, tenant_name, apps) = selected.or_else(|| grouped_vec.first().cloned()).unwrap_or_else(|| {
                    (
                        "t_8a91f3d2".to_string(),
                        "Nexus Property Group".to_string(),
                        vec![
                            PlatformAppModel {
                                tenant_id: "t_8a91f3d2".to_string(),
                                instance_id: "inst_1".to_string(),
                                name: "Atlas PM — Residential".to_string(),
                                app_type: "PM".to_string(),
                                domain: "nexus-pm.atlas.app".to_string(),
                                site_status: "Active".to_string(),
                                description: "Residential Property Management".to_string(),
                            },
                            PlatformAppModel {
                                tenant_id: "t_8a91f3d2".to_string(),
                                instance_id: "inst_2".to_string(),
                                name: "Atlas STR — Miami".to_string(),
                                app_type: "STR".to_string(),
                                domain: "nexus-str.atlas.app".to_string(),
                                site_status: "Active".to_string(),
                                description: "Short Term Rentals".to_string(),
                            },
                            PlatformAppModel {
                                tenant_id: "t_8a91f3d2".to_string(),
                                instance_id: "inst_3".to_string(),
                                name: "Atlas Commercial".to_string(),
                                app_type: "COM".to_string(),
                                domain: "nexus-com.atlas.app".to_string(),
                                site_status: "Beta".to_string(),
                                description: "Commercial properties".to_string(),
                            },
                        ]
                    )
                });

                let apps_val = StoredValue::new(apps);
                let total_apps = apps_val.with_value(|a| a.len());
                let live_count = apps_val.with_value(|a| a.iter().filter(|x| x.site_status.to_lowercase() == "active").count());
                let beta_count = total_apps - live_count;

                view! {
                    <div class="main-area" style="padding: 0; gap: 0;">
                        // ── Tenant Hero ──
                        <div class="tenant-hero">
                            <div>
                                <div class="breadcrumb">
                                    <a href="/">"Platform"</a>" › "<a href="/apps">"Tenants"</a>" › "{tenant_name.clone()}
                                </div>
                                <div class="t-identity">
                                    <div class="t-avatar">{tenant_name.chars().next().unwrap_or('N')}</div>
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
                                <button class="btn btn-ghost" on:click=move |_| {
                                    toast.message.set(Some("Impersonating tenant...".to_string()));
                                }>"Impersonate"</button>
                                <button class="btn btn-ghost" on:click=move |_| {
                                    toast.message.set(Some("Provision App Instance modal loaded.".to_string()));
                                } font-weight="500">"+ New App Instance"</button>
                                <button class="btn btn-primary" on:click=move |_| {
                                    toast.message.set(Some("Edit Tenant modal loaded.".to_string()));
                                }>"Edit Tenant"</button>
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
                                <div class="kpi-value mono">"$6,400"</div>
                                <div class="kpi-sub">"+$1,600 MoM"</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Total Leads"</div>
                                <div class="kpi-value mono">"487"</div>
                                <div class="kpi-sub">"Cross-app"</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Accounts"</div>
                                <div class="kpi-value mono">"112"</div>
                                <div class="kpi-sub">"Unique orgs"</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Contacts"</div>
                                <div class="kpi-value mono">"284"</div>
                                <div class="kpi-sub">"Unique people"</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Open Opps"</div>
                                <div class="kpi-value mono">"38"</div>
                                <div class="kpi-sub">"$2.4M pipeline"</div>
                            </div>
                            <div class="kpi">
                                <div class="kpi-label">"Health Score"</div>
                                <div style="margin-top:4px">
                                    <div class="score-badge" style="font-size:15px;padding:4px 10px">
                                        <span class="score-dot" style="background:#00CC44;width:8px;height:8px"></span>
                                        <span>"9.2"</span>
                                        <span class="score-tier">"Outstanding"</span>
                                    </div>
                                </div>
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
                                            <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                toast.message.set(Some("Provision app instance dialog loaded.".to_string()));
                                            }>"+ Provision New Instance"</button>
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
                                                            <div class="app-stats">
                                                                <div class="app-stat">
                                                                    <div class="app-stat-label">"Leads"</div>
                                                                    <div class="app-stat-value">"342"</div>
                                                                </div>
                                                                <div class="app-stat">
                                                                    <div class="app-stat-label">"Properties"</div>
                                                                    <div class="app-stat-value">"87"</div>
                                                                </div>
                                                                <div class="app-stat">
                                                                    <div class="app-stat-label">"MRR"</div>
                                                                    <div class="app-stat-value">"$4.8k"</div>
                                                                </div>
                                                            </div>
                                                            <div class="app-modules">
                                                                <span class="mod-chip on">"Listings"</span>
                                                                <span class="mod-chip on">"Payments"</span>
                                                                <span class="mod-chip on">"Analytics"</span>
                                                                <span class="mod-chip">"Messaging"</span>
                                                                <span class="mod-chip on">"Events"</span>
                                                                <span class="mod-chip on">"Custom Fields"</span>
                                                            </div>
                                                            <div class="app-card-footer">
                                                                <span class="app-footer-meta">"Last activity: 12 min ago · 14 users"</span>
                                                                <div class="app-footer-actions">
                                                                    <button class="btn btn-ghost btn-sm" on:click=move |e| {
                                                                        e.stop_propagation();
                                                                        toast.message.set(Some("App settings loaded.".to_string()));
                                                                    }>"Config"</button>
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

                                        // ── Summary Grid ──
                                        <div class="summary-grid">
                                            <div class="sum-card">
                                                <div class="sum-label">"Pipeline Value (All Apps)"</div>
                                                <div class="sum-value mono">"$2.4M"</div>
                                                <div class="sum-sub">"38 open opportunities"</div>
                                            </div>
                                            <div class="sum-card">
                                                <div class="sum-label">"Conversion Rate"</div>
                                                <div class="sum-value">"11.4%"</div>
                                                <div class="sum-sub">"204 converted leads"</div>
                                            </div>
                                            <div class="sum-card">
                                                <div class="sum-label">"Top Lead Source"</div>
                                                <div class="sum-value" style="font-size:18px">"FMCSA"</div>
                                                <div class="sum-sub">"61% of all leads"</div>
                                            </div>
                                            <div class="sum-card">
                                                <div class="sum-label">"Avg G-27 Score"</div>
                                                <div class="sum-value" style="color:#88CC00">"8.1"</div>
                                                <div class="sum-sub">"Above the bar"</div>
                                            </div>
                                        </div>

                                        // ── Recent Leads Table ──
                                        <div class="card">
                                            <div class="card-hdr">
                                                <span class="card-title">"Recent Leads — All App Instances"</span>
                                                <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                    let navigate = leptos_router::hooks::use_navigate();
                                                    navigate("/crm?tab=leads", Default::default());
                                                }>"View All →"</button>
                                            </div>
                                            <table>
                                                <thead>
                                                    <tr>
                                                        <th>"Lead"</th>
                                                        <th>"App Instance"</th>
                                                        <th>"Source"</th>
                                                        <th>"G-27 Score"</th>
                                                        <th>"Stage"</th>
                                                        <th>"Last Activity"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <tr on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/crm?tab=leads", Default::default());
                                                    }>
                                                        <td>
                                                            <div style="font-weight:500">"João Silva"</div>
                                                            <div style="font-size:10.5px;color:var(--text-muted)">"Logística Meridional S.A."</div>
                                                        </td>
                                                        <td>
                                                            <span class="app-chip">
                                                                <span style="width:5px;height:5px;border-radius:50%;background:var(--cobalt);display:inline-block;margin-right:4px;"></span>
                                                                "PM Residential"
                                                            </span>
                                                        </td>
                                                        <td>
                                                            <span style="font-size:10px;font-weight:600;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 5px;background:var(--cobalt-dim)">"FMCSA"</span>
                                                        </td>
                                                        <td>
                                                            <div class="score-badge">
                                                                <span class="score-dot" style="background:#00CC44"></span>
                                                                <span>"9.3"</span>
                                                                <span class="score-tier">"Outstanding"</span>
                                                            </div>
                                                        </td>
                                                        <td style="color:var(--cobalt)">"Qualified"</td>
                                                        <td class="muted">"12 min ago"</td>
                                                    </tr>
                                                    <tr on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/crm?tab=leads", Default::default());
                                                    }>
                                                        <td>
                                                            <div style="font-weight:500">"Marcus Thompson"</div>
                                                            <div style="font-size:10.5px;color:var(--text-muted)">"Thompson Freight Inc."</div>
                                                        </td>
                                                        <td>
                                                            <span class="app-chip">
                                                                <span style="width:5px;height:5px;border-radius:50%;background:var(--amber);display:inline-block;margin-right:4px;"></span>
                                                                "STR Miami"
                                                            </span>
                                                        </td>
                                                        <td>
                                                            <span style="font-size:10px;font-weight:600;color:var(--amber);border:1px solid var(--amber);border-radius:3px;padding:1px 5px;background:var(--amber-dim)">"DOT"</span>
                                                        </td>
                                                        <td>
                                                            <div class="score-badge">
                                                                <span class="score-dot" style="background:#88CC00"></span>
                                                                <span>"7.6"</span>
                                                                <span class="score-tier">"Above"</span>
                                                            </div>
                                                        </td>
                                                        <td>"New"</td>
                                                        <td class="muted">"5h ago"</td>
                                                    </tr>
                                                    <tr on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/crm?tab=leads", Default::default());
                                                    }>
                                                        <td>
                                                            <div style="font-weight:500">"Priya Bhat"</div>
                                                            <div style="font-size:10.5px;color:var(--text-muted)">"Bhat Holdings LLC"</div>
                                                        </td>
                                                        <td>
                                                            <span class="app-chip">
                                                                <span style="width:5px;height:5px;border-radius:50%;background:var(--violet);display:inline-block;margin-right:4px;"></span>
                                                                "Commercial"
                                                            </span>
                                                        </td>
                                                        <td>
                                                            <span style="font-size:10px;font-weight:600;color:var(--violet);border:1px solid var(--violet);border-radius:3px;padding:1px 5px;background:var(--violet-dim)">"Campaign"</span>
                                                        </td>
                                                        <td>
                                                            <div class="score-badge">
                                                                <span class="score-dot" style="background:#88CC00;opacity:0.5"></span>
                                                                <span style="font-style:italic;color:var(--text-muted)">"~7.2"</span>
                                                                <span class="score-tier">"Est."</span>
                                                            </div>
                                                        </td>
                                                        <td>"New"</td>
                                                        <td class="muted">"2d ago"</td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                }.into_any(),
                                "crm" => view! {
                                    <div>
                                        <p style="color:var(--text-muted);font-size:12px;margin-bottom:16px">"Cross-app CRM view — all accounts, contacts, and opportunities across all 3 app instances for this tenant."</p>
                                        <div class="two-col">
                                            <div class="card">
                                                <div class="card-hdr">
                                                    <span class="card-title">"Recent Accounts"</span>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/crm?tab=accounts", Default::default());
                                                    }>"View All →"</button>
                                                </div>
                                                <table>
                                                    <thead>
                                                        <tr>
                                                            <th>"Account"</th>
                                                            <th>"App"</th>
                                                            <th>"Type"</th>
                                                            <th>"Contacts"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <tr on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/crm?tab=accounts", Default::default());
                                                        }>
                                                            <td><div style="font-weight:500">"Logística Meridional S.A."</div></td>
                                                            <td><span class="app-chip">"PM"</span></td>
                                                            <td style="color:var(--text-muted)">"Carrier"</td>
                                                            <td class="mono">"4"</td>
                                                        </tr>
                                                        <tr on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/crm?tab=accounts", Default::default());
                                                        }>
                                                            <td><div style="font-weight:500">"Carvalho Imóveis Ltda"</div></td>
                                                            <td><span class="app-chip">"PM"</span></td>
                                                            <td style="color:var(--text-muted)">"Real Estate"</td>
                                                            <td class="mono">"2"</td>
                                                        </tr>
                                                        <tr on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/crm?tab=accounts", Default::default());
                                                        }>
                                                            <td><div style="font-weight:500">"Bhat Holdings LLC"</div></td>
                                                            <td><span class="app-chip">"COM"</span></td>
                                                            <td style="color:var(--text-muted)">"Investor"</td>
                                                            <td class="mono">"1"</td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                            <div class="card">
                                                <div class="card-hdr">
                                                    <span class="card-title">"Open Opportunities"</span>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                        let navigate = leptos_router::hooks::use_navigate();
                                                        navigate("/crm?tab=opportunities", Default::default());
                                                    }>"View All →"</button>
                                                </div>
                                                <table>
                                                    <thead>
                                                        <tr>
                                                            <th>"Opportunity"</th>
                                                            <th>"Stage"</th>
                                                            <th>"Value"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <tr on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/crm?tab=opportunities", Default::default());
                                                        }>
                                                            <td><div style="font-weight:500">"Meridional Fleet Depot"</div></td>
                                                            <td style="color:var(--cobalt)">"Proposal"</td>
                                                            <td class="mono">"$4.2M"</td>
                                                        </tr>
                                                        <tr on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/crm?tab=opportunities", Default::default());
                                                        }>
                                                            <td><div style="font-weight:500">"Carvalho Portfolio Mgmt"</div></td>
                                                            <td style="color:var(--amber)">"Qualified"</td>
                                                            <td class="mono">"$800k"</td>
                                                        </tr>
                                                        <tr on:click=move |_| {
                                                            let navigate = leptos_router::hooks::use_navigate();
                                                            navigate("/crm?tab=opportunities", Default::default());
                                                        }>
                                                            <td><div style="font-weight:500">"Bhat Commercial Block"</div></td>
                                                            <td>"New"</td>
                                                            <td class="mono">"$1.1M"</td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                "billing" => view! {
                                    <div>
                                        <p style="color:var(--text-muted);font-size:12px">"Consolidated billing across all app instances. "<a href="/billing" style="color:var(--text-link)">"View detailed billing →"</a></p>
                                    </div>
                                }.into_any(),
                                "config" => view! {
                                    <div>
                                        <p style="color:var(--text-muted);font-size:12px">"Tenant-level config applies to all instances. Per-app overrides managed inside each app instance. "<a href="/developer" style="color:var(--text-link)">"Open full config →"</a></p>
                                    </div>
                                }.into_any(),
                                "audit" => view! {
                                    <div>
                                        <p style="color:var(--text-muted);font-size:12px">"Audit log spans all app instances. "<a href="/logs" style="color:var(--text-link)">"Open full audit log →"</a></p>
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
