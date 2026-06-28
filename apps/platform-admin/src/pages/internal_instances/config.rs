/// # Internal Instance Config
///
/// Route: /internal-instances/:id/config
///
/// Dedicated configuration page for platform-team managed (InternalOperator) app instances.
/// Replaces the incorrect redirect to /network/:id (NetworkDetail) which was designed for
/// network-instance app types and showed hardcoded `.atlas-platform.com` domains.
///
/// The `:id` param is the app_instance_id. We load PlatformAppSummary from the platform
/// apps list (already in memory from the listing page), giving us the tenant name, real
/// domain, mode, purpose — all correct, none hardcoded.
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::api::admin::get_all_platform_apps;
use crate::api::client::api_get;
use serde::{Deserialize, Serialize};

// ── User model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserSummary {
    pub id:         String,
    pub email:      String,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
    pub role:       Option<String>,
    pub is_active:  bool,
}

async fn get_tenant_users(tenant_id: &str) -> Result<Vec<AdminUserSummary>, String> {
    api_get(&format!("api/admin/users?tenant_id={}", tenant_id)).await
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn purpose_badge_cls(p: &str) -> &'static str {
    match p {
        "demo"            => "bg-blue-500/10 border-blue-500/20 text-blue-400",
        "test"            => "bg-amber-500/10 border-amber-500/20 text-amber-400",
        "staging"         => "bg-purple-500/10 border-purple-500/20 text-purple-400",
        "managed_service" => "bg-emerald-500/10 border-emerald-500/20 text-emerald-400",
        _                 => "bg-outline-variant/20 border-outline-variant/30 text-on-surface-variant/70",
    }
}

fn purpose_label(p: &str) -> &'static str {
    match p {
        "demo"            => "Demo",
        "test"            => "Test",
        "staging"         => "Staging",
        "managed_service" => "Managed Service",
        _                 => "Internal",
    }
}

fn app_type_label(t: &str) -> &'static str {
    match t {
        "property_management" | "folio" => "Folio",
        "anchor"   => "Anchor",
        "meridian" => "Meridian",
        _          => "App",
    }
}

fn status_cls(s: &str) -> &'static str {
    match s {
        "active"       => "text-emerald-400",
        "provisioning" => "text-blue-400",
        "beta"         => "text-amber-400",
        "suspended"    => "text-red-400",
        _              => "text-on-surface-variant/50",
    }
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn InternalInstanceConfig() -> impl IntoView {
    let params = use_params_map();
    let instance_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let active_tab = RwSignal::new("overview".to_string());

    let app_res = LocalResource::new(move || {
        let id = instance_id();
        async move {
            get_all_platform_apps().await.ok()
                .and_then(|apps| apps.into_iter().find(|a| a.instance_id == id))
        }
    });

    view! {
        <div class="main-area">
            <Suspense fallback=move || view! {
                <div class="flex items-center justify-center h-64">
                    <div class="text-on-surface-variant text-sm animate-pulse">"Loading instance…"</div>
                </div>
            }>
                {move || {
                    let app_opt = app_res.get().flatten();
                    match app_opt {
                        None => view! {
                            <div class="flex flex-col items-center justify-center h-64 gap-3">
                                <p class="text-error text-sm">"Instance not found."</p>
                                <a href="/internal-instances" class="btn btn-ghost btn-sm">"← Back to Internal Instances"</a>
                            </div>
                        }.into_any(),
                        Some(app) => {
                            let app = store_value(app);
                            view! {
                                <div class="page-header">
                                    <div>
                                        <div style="font-size:10px;color:var(--text-muted);margin-bottom:4px;display:flex;align-items:center;gap:4px;">
                                            <a href="/internal-instances" style="color:var(--text-muted);text-decoration:none;">"Internal Instances"</a>
                                            <span>"›"</span>
                                            <span style="color:var(--on-surface);">{move || app.get_value().name.clone()}</span>
                                        </div>
                                        <div class="page-title" style="display:flex;align-items:center;gap:10px;">
                                            {move || app.get_value().name.clone()}
                                            <span class=move || format!("text-[10px] font-semibold {}", status_cls(&app.get_value().site_status))>
                                                {move || format!("● {}", app.get_value().site_status)}
                                            </span>
                                            {move || app.get_value().purpose.as_ref().map(|p| {
                                                let cls = purpose_badge_cls(p).to_string();
                                                let lbl = purpose_label(p).to_string();
                                                view! {
                                                    <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase border {}", cls)>
                                                        {lbl}
                                                    </span>
                                                }
                                            })}
                                        </div>
                                        <div class="page-subtitle" style="display:flex;align-items:center;gap:8px;margin-top:2px;">
                                            <span>{move || format!("{} instance", app_type_label(&app.get_value().app_type))}</span>
                                            <span style="opacity:0.4;">"·"</span>
                                            <code style="font-size:10px;color:var(--cobalt);">{move || app.get_value().domain.clone()}</code>
                                            <a href=move || format!("https://{}", app.get_value().domain)
                                               target="_blank"
                                               style="font-size:10px;color:var(--text-muted);text-decoration:none;">"↗"</a>
                                        </div>
                                    </div>
                                    <div class="page-actions">
                                        <a href="/internal-instances" class="btn btn-ghost btn-sm">"← Back"</a>
                                    </div>
                                </div>

                                <div style="display:flex;gap:2px;padding:0 0 16px;border-bottom:1px solid rgba(255,255,255,0.08);margin-bottom:20px;">
                                    {["overview", "domain", "users", "deployment"].map(|tab| {
                                        let label = match tab {
                                            "overview"   => "Overview",
                                            "domain"     => "Domain & SSL",
                                            "users"      => "Users & Roles",
                                            "deployment" => "Deployment",
                                            _            => tab,
                                        };
                                        view! {
                                            <button
                                                on:click=move |_| active_tab.set(tab.to_string())
                                                class=move || format!(
                                                    "px-4 py-2 text-xs font-semibold rounded transition-all {}",
                                                    if active_tab.get() == tab {
                                                        "bg-primary/15 text-primary"
                                                    } else {
                                                        "text-on-surface-variant hover:bg-surface-container-high/40"
                                                    }
                                                )
                                            >{label}</button>
                                        }
                                    }).collect_view()}
                                </div>

                                {move || match active_tab.get().as_str() {
                                    "overview"   => view! { <OverviewTab   app=app.get_value() /> }.into_any(),
                                    "domain"     => view! { <DomainTab    app=app.get_value() /> }.into_any(),
                                    "users"      => view! { <UsersTab     tenant_id=app.get_value().tenant_id.clone() /> }.into_any(),
                                    "deployment" => view! { <DeploymentTab app=app.get_value() /> }.into_any(),
                                    _            => view! { <></> }.into_any(),
                                }}
                            }.into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}

// ── Overview ──────────────────────────────────────────────────────────────────

#[component]
fn OverviewTab(app: crate::api::models::PlatformAppSummary) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="section">
                <div class="section-hdr"><span class="section-title">"Instance Details"</span></div>
                <div style="padding:12px 16px;display:flex;flex-direction:column;gap:10px;">
                    <div class="flex justify-between items-center">
                        <span class="text-xs text-on-surface-variant/60">"Name"</span>
                        <span class="text-xs font-semibold text-on-surface">{app.name.clone()}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-xs text-on-surface-variant/60">"App Type"</span>
                        <span class="text-xs font-semibold text-on-surface">{app_type_label(&app.app_type)}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-xs text-on-surface-variant/60">"Mode"</span>
                        <code class="text-xs font-mono text-on-surface">{app.mode.clone()}</code>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-xs text-on-surface-variant/60">"Status"</span>
                        <span class=format!("text-xs font-semibold {}", status_cls(&app.site_status))>
                            {app.site_status.clone()}
                        </span>
                    </div>
                    {app.purpose.as_ref().map(|p| {
                        let cls = purpose_badge_cls(p).to_string();
                        let lbl = purpose_label(p).to_string();
                        view! {
                            <div class="flex justify-between items-center">
                                <span class="text-xs text-on-surface-variant/60">"Purpose"</span>
                                <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase border {}", cls)>{lbl}</span>
                            </div>
                        }
                    })}
                </div>
            </div>

            <div class="section">
                <div class="section-hdr"><span class="section-title">"Identifiers"</span></div>
                <div style="padding:12px 16px;display:flex;flex-direction:column;gap:10px;">
                    <div>
                        <span class="text-[9px] uppercase tracking-wider text-on-surface-variant/50 block mb-1">"Tenant ID"</span>
                        <code class="text-[10px] font-mono text-on-surface-variant break-all">{app.tenant_id.clone()}</code>
                    </div>
                    <div>
                        <span class="text-[9px] uppercase tracking-wider text-on-surface-variant/50 block mb-1">"Instance ID"</span>
                        <code class="text-[10px] font-mono text-on-surface-variant break-all">{app.instance_id.clone()}</code>
                    </div>
                    <div>
                        <span class="text-[9px] uppercase tracking-wider text-on-surface-variant/50 block mb-1">"Domain"</span>
                        <a href=format!("https://{}", app.domain)
                           target="_blank"
                           class="text-[10px] font-mono text-primary hover:underline">
                            {app.domain.clone()}
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── Domain & SSL ──────────────────────────────────────────────────────────────

#[component]
fn DomainTab(app: crate::api::models::PlatformAppSummary) -> impl IntoView {
    let is_wildcard = app.domain.ends_with(".dev.atlas.oply.co");
    let ssl_status = if is_wildcard {
        "Covered by wildcard cert (*.dev.atlas.oply.co)"
    } else {
        "Custom domain — cert-manager Certificate resource required in k3s"
    };
    let ssl_cls = if is_wildcard { "text-emerald-400" } else { "text-amber-400" };

    view! {
        <div class="section" style="max-width:640px;">
            <div class="section-hdr"><span class="section-title">"Domain & SSL"</span></div>
            <div style="padding:16px;display:flex;flex-direction:column;gap:16px;">
                <div class="flex justify-between items-center">
                    <span class="text-xs text-on-surface-variant/60">"Live Domain"</span>
                    <div class="flex items-center gap-2">
                        <code class="text-xs font-mono text-primary">{app.domain.clone()}</code>
                        <a href=format!("https://{}", app.domain)
                           target="_blank"
                           class="text-[10px] text-on-surface-variant/50 hover:text-primary">"↗ Open"</a>
                    </div>
                </div>
                <div class="flex justify-between items-start">
                    <span class="text-xs text-on-surface-variant/60">"SSL Status"</span>
                    <span class=format!("text-xs font-semibold text-right {}", ssl_cls)>{ssl_status}</span>
                </div>
                {if !is_wildcard {
                    view! {
                        <div class="card" style="padding:10px 14px;border-left:3px solid var(--amber);">
                            <p style="font-size:11px;color:var(--amber);font-weight:600;margin:0 0 4px;">"Action required"</p>
                            <p class="muted" style="font-size:11px;margin:0;">
                                "Add a k3s Ingress manifest with host "
                                <code style="font-size:10px;">{app.domain.clone()}</code>
                                " pointing to the correct service, and a cert-manager Certificate resource. "
                                "See NixForge/docs/ for the ingress pattern."
                            </p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="card" style="padding:10px 14px;border-left:3px solid rgba(52,211,153,0.5);">
                            <p style="font-size:11px;color:#34d399;font-weight:600;margin:0 0 2px;">"Wildcard domain — no action needed"</p>
                            <p class="muted" style="font-size:11px;margin:0;">
                                "SSL is provisioned automatically via the k3s ingress wildcard rule."
                            </p>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

// ── Users & Roles ─────────────────────────────────────────────────────────────

#[component]
fn UsersTab(tenant_id: String) -> impl IntoView {
    let tid = store_value(tenant_id);
    let users_res = LocalResource::new(move || async move {
        get_tenant_users(&tid.get_value()).await
    });

    view! {
        <div class="section" style="max-width:760px;">
            <div class="section-hdr"><span class="section-title">"Users & Roles"</span></div>
            <Suspense fallback=move || view! {
                <div class="text-xs text-on-surface-variant/60 animate-pulse p-4">"Loading users…"</div>
            }>
                {move || {
                    match users_res.get() {
                        None => view! {
                            <div class="text-xs text-on-surface-variant/60 animate-pulse p-4">"Loading…"</div>
                        }.into_any(),
                        Some(Err(e)) => view! {
                            <div class="p-4">{crate::utils::inline_error(&e)}</div>
                        }.into_any(),
                        Some(Ok(users)) if users.is_empty() => view! {
                            <div class="p-8 text-center text-sm text-on-surface-variant/60">
                                "No users on this tenant yet. "
                                <a href="/user-access" class="text-primary hover:underline">"Manage via User Access →"</a>
                            </div>
                        }.into_any(),
                        Some(Ok(users)) => view! {
                            <div>
                                <table style="width:100%;border-collapse:collapse;">
                                    <thead>
                                        <tr style="border-bottom:1px solid rgba(255,255,255,0.06);">
                                            <th style="text-align:left;padding:6px 16px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"User"</th>
                                            <th style="text-align:left;padding:6px 16px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"Email"</th>
                                            <th style="text-align:left;padding:6px 16px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"Role"</th>
                                            <th style="text-align:left;padding:6px 16px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"Status"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {users.into_iter().map(|u| {
                                            let display_name = match (&u.first_name, &u.last_name) {
                                                (Some(f), Some(l)) => format!("{} {}", f, l),
                                                (Some(f), None) => f.clone(),
                                                _ => u.email.clone(),
                                            };
                                            let role = u.role.clone().unwrap_or_else(|| "User".into());
                                            let active_cls = if u.is_active { "text-emerald-400" } else { "text-on-surface-variant/40" };
                                            let status_label = if u.is_active { "Active" } else { "Inactive" };
                                            view! {
                                                <tr style="border-bottom:1px solid rgba(255,255,255,0.04);">
                                                    <td style="padding:8px 16px;">
                                                        <span class="text-xs font-semibold text-on-surface">{display_name}</span>
                                                    </td>
                                                    <td style="padding:8px 16px;">
                                                        <span class="text-xs font-mono text-on-surface-variant/70">{u.email}</span>
                                                    </td>
                                                    <td style="padding:8px 16px;">
                                                        <span class="text-[10px] font-semibold px-2 py-0.5 rounded bg-surface-container-high/40 border border-outline-variant/20 text-on-surface-variant">{role}</span>
                                                    </td>
                                                    <td style="padding:8px 16px;">
                                                        <span class=format!("text-[10px] font-semibold {}", active_cls)>{status_label}</span>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                                <div style="padding:10px 16px;border-top:1px solid rgba(255,255,255,0.04);">
                                    <a href="/user-access" class="text-[10px] text-on-surface-variant/60 hover:text-primary">
                                        "Manage roles & permissions in User Access →"
                                    </a>
                                </div>
                            </div>
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

// ── Deployment ────────────────────────────────────────────────────────────────

#[component]
fn DeploymentTab(app: crate::api::models::PlatformAppSummary) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let is_busy = RwSignal::new(false);
    let is_suspended = app.site_status == "suspended";

    view! {
        <div class="section" style="max-width:640px;">
            <div class="section-hdr"><span class="section-title">"Deployment Settings"</span></div>
            <div style="padding:16px;display:flex;flex-direction:column;gap:16px;">
                <div class="flex justify-between items-center">
                    <span class="text-xs text-on-surface-variant/60">"Deployment Mode"</span>
                    <code class="text-xs font-mono text-on-surface">{app.mode.clone()}</code>
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-xs text-on-surface-variant/60">"Instance Status"</span>
                    <span class=format!("text-xs font-semibold {}", status_cls(&app.site_status))>
                        {app.site_status.clone()}
                    </span>
                </div>

                <div style="padding-top:8px;border-top:1px solid rgba(255,255,255,0.06);">
                    {if is_suspended {
                        let iid = app.instance_id.clone();
                        let toast_ref = toast.clone();
                        view! {
                            <button
                                class="btn btn-ghost btn-sm"
                                disabled=move || is_busy.get()
                                on:click=move |_| {
                                    is_busy.set(true);
                                    let iid2 = iid.clone();
                                    let tr = toast_ref.clone();
                                    leptos::task::spawn_local(async move {
                                        let url = crate::api::client::api_url(
                                            &format!("api/admin/app-instances/{}/resume", iid2));
                                        let res = crate::api::client::create_client().post(&url).send().await;
                                        is_busy.set(false);
                                        match res {
                                            Ok(r) if r.status().is_success() =>
                                                tr.show_toast("Resumed", "Instance resumed.", "success"),
                                            Ok(r) => tr.show_toast("Error", &format!("HTTP {}", r.status()), "error"),
                                            Err(e) => tr.show_toast("Error", &e.to_string(), "error"),
                                        }
                                    });
                                }
                            >{move || if is_busy.get() { "Resuming…" } else { "Resume Instance" }}</button>
                        }.into_any()
                    } else {
                        let iid = app.instance_id.clone();
                        let toast_ref = toast.clone();
                        view! {
                            <button
                                class="btn btn-ghost btn-sm"
                                style="color:var(--error);border-color:rgba(239,68,68,0.2);"
                                disabled=move || is_busy.get()
                                on:click=move |_| {
                                    is_busy.set(true);
                                    let iid2 = iid.clone();
                                    let tr = toast_ref.clone();
                                    leptos::task::spawn_local(async move {
                                        let url = crate::api::client::api_url(
                                            &format!("api/admin/app-instances/{}/suspend", iid2));
                                        let res = crate::api::client::create_client()
                                            .post(&url)
                                            .json(&serde_json::json!({"reason":"Suspended via Internal Instances config"}))
                                            .send().await;
                                        is_busy.set(false);
                                        match res {
                                            Ok(r) if r.status().is_success() =>
                                                tr.show_toast("Suspended", "Instance suspended.", "success"),
                                            Ok(r) => tr.show_toast("Error", &format!("HTTP {}", r.status()), "error"),
                                            Err(e) => tr.show_toast("Error", &e.to_string(), "error"),
                                        }
                                    });
                                }
                            >{move || if is_busy.get() { "Suspending…" } else { "Suspend Instance" }}</button>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
