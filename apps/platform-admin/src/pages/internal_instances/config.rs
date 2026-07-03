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
        "demo"            => "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)",
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
        "active"       => "color:var(--green)",
        "provisioning" => "color:var(--cobalt)",
        "beta"         => "color:var(--amber)",
        "suspended"    => "color:var(--error)",
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
        <div class="main-canvas">
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
                                            <a href="/internal-instances" style="color:var(--text-muted);text-decoration:none;">"App Instances"</a>
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
                                    {["overview", "domain", "users", "deployment", "danger"].map(|tab| {
                                        let label = match tab {
                                            "overview"   => "Overview",
                                            "domain"     => "Domain & SSL",
                                            "users"      => "Users & Roles",
                                            "deployment" => "Deployment",
                                            "danger"     => "⚠ Danger Zone",
                                            _            => tab,
                                        };
                                        view! {
                                            <button
                                                on:click=move |_| active_tab.set(tab.to_string())
                                                class=move || format!(
                                                    "px-4 py-2 text-xs font-semibold rounded transition-all {}",
                                                    if active_tab.get() == tab {
                                                        if tab == "danger" {
                                                            "bg-error/15 text-error"
                                                        } else {
                                                            "bg-primary/15 text-primary"
                                                        }
                                                    } else if tab == "danger" {
                                                        "text-error/60 hover:bg-error/10"
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
                                    "danger"     => view! { <DangerZoneTab instance_id=app.get_value().instance_id.clone() instance_name=app.get_value().name.clone() /> }.into_any(),
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
    let toast       = use_context::<crate::app::GlobalToast>().expect("toast context");
    let instance_id = app.instance_id.clone();
    let iid         = store_value(instance_id.clone());

    // Load live public config (has custom_domain + dns_instructions)
    let config_res = LocalResource::new(move || {
        let id = iid.get_value();
        async move {
            let uuid = uuid::Uuid::parse_str(&id).ok()?;
            crate::api::admin::get_public_config(uuid).await.ok()
        }
    });

    let saving        = RwSignal::new(false);
    let domain_input  = RwSignal::new(String::new());
    let saved_config  = RwSignal::<Option<crate::api::admin::PublicConfigResponse>>::new(None);

    // Sync input from loaded config once
    let synced = RwSignal::new(false);

    let on_save = {
        let iid2  = store_value(instance_id.clone());
        let toast = toast.clone();
        move |_| {
            let id = uuid::Uuid::parse_str(&iid2.get_value()).ok();
            let Some(id) = id else { return; };
            let domain = domain_input.get().trim().to_lowercase().to_string();
            if domain.is_empty() {
                toast.show_toast("Validation", "Domain cannot be empty.", "error");
                return;
            }
            saving.set(true);
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                match crate::api::admin::update_public_config(id, None, Some(domain)).await {
                    Ok(cfg) => {
                        saved_config.set(Some(cfg));
                        toast.show_toast("Domain saved", "Ingress provisioning started. DNS instructions shown below.", "success");
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
                saving.set(false);
            });
        }
    };

    view! {
        <Suspense fallback=move || view! {
            <div class="text-xs text-on-surface-variant/60 animate-pulse p-4">"Loading domain config…"</div>
        }>
            {move || {
                let cfg = saved_config.get().or_else(|| config_res.get().flatten());

                if !synced.get() {
                    if let Some(ref c) = cfg {
                        domain_input.set(c.custom_domain.clone().unwrap_or_default());
                        synced.set(true);
                    }
                }

                let current_domain = cfg.as_ref().and_then(|c| c.custom_domain.clone())
                    .unwrap_or_else(|| app.domain.clone());
                let dns  = cfg.as_ref().and_then(|c| c.dns_instructions.clone());
                let is_w = current_domain.ends_with(".dev.atlas.oply.co")
                    || current_domain.ends_with(".uat.atlas.oply.co")
                    || current_domain.ends_with(".atlas.oply.co");

                view! {
                    <div style="display:flex;flex-direction:column;gap:16px;max-width:640px;">

                        // ── Current domain + SSL status ──────────────────────────
                        <div class="section">
                            <div class="section-hdr"><span class="section-title">"Domain & SSL"</span></div>
                            <div style="padding:16px;display:flex;flex-direction:column;gap:12px;">
                                <div class="flex justify-between items-center">
                                    <span class="text-xs text-on-surface-variant/60">"Live Domain"</span>
                                    <div class="flex items-center gap-2">
                                        <code class="text-xs font-mono text-primary">{current_domain.clone()}</code>
                                        <a href=format!("https://{}", current_domain) target="_blank"
                                           class="text-[10px] text-on-surface-variant/50 hover:text-primary">"↗ Open"</a>
                                    </div>
                                </div>
                                <div class="flex justify-between items-center">
                                    <span class="text-xs text-on-surface-variant/60">"SSL"</span>
                                    {if is_w {
                                        view! { <span class="text-xs font-semibold text-emerald-400">"Wildcard cert active"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-xs font-semibold text-amber-400">"HTTP-01 auto (cert-manager)"</span> }.into_any()
                                    }}
                                </div>
                            </div>
                        </div>

                        // ── Assign domain form ───────────────────────────────────
                        <div class="section">
                            <div class="section-hdr">
                                <span class="section-title">"Assign Domain"</span>
                            </div>
                            <div style="padding:16px;display:flex;flex-direction:column;gap:12px;">
                                <p class="text-[11px] text-on-surface-variant/70 leading-relaxed">
                                    "Enter a platform subdomain ("
                                    <code class="text-xs">"folio.atlas.oply.co"</code>
                                    ") or a custom client domain ("
                                    <code class="text-xs">"app.clientco.com"</code>
                                    "). An Ingress is provisioned automatically. For custom domains, TLS is issued via Let's Encrypt HTTP-01."
                                </p>
                                <div style="display:flex;gap:8px;align-items:center;">
                                    <input
                                        type="text"
                                        class="input input-sm flex-1 font-mono"
                                        placeholder="e.g. app.clientco.com"
                                        prop:value=move || domain_input.get()
                                        on:input=move |e| { domain_input.set(event_target_value(&e)); }
                                        disabled=move || saving.get()
                                    />
                                    <button
                                        class="btn btn-primary btn-sm"
                                        disabled=move || saving.get()
                                        on:click=on_save.clone()
                                    >
                                        {move || if saving.get() { "Saving…" } else { "Save & Provision" }}
                                    </button>
                                </div>
                            </div>
                        </div>

                        // ── DNS instructions (custom domain only) ────────────────
                        {match dns {
                            Some(d) => view! {
                                <div class="section" style="border-left:3px solid var(--cobalt);">
                                    <div class="section-hdr">
                                        <span class="section-title">"DNS Configuration Required"</span>
                                        <span class="text-[10px] text-on-surface-variant/50">"Add this record at your client's DNS registrar"</span>
                                    </div>
                                    <div style="padding:16px;display:flex;flex-direction:column;gap:10px;">
                                        <p class="text-[11px] text-on-surface-variant/70 leading-relaxed">
                                            "TLS is provisioned automatically once the record propagates (~60 s for HTTP-01). \
                                             No further action is needed after adding the DNS record."
                                        </p>
                                        <div style="background:rgba(0,0,0,0.25);border-radius:6px;overflow:hidden;border:1px solid rgba(255,255,255,0.07);">
                                            <table style="width:100%;border-collapse:collapse;">
                                                <thead>
                                                    <tr style="border-bottom:1px solid rgba(255,255,255,0.06);">
                                                        <th style="text-align:left;padding:6px 12px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"Type"</th>
                                                        <th style="text-align:left;padding:6px 12px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"Name (Host)"</th>
                                                        <th style="text-align:left;padding:6px 12px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"Value (Target)"</th>
                                                        <th style="text-align:left;padding:6px 12px;font-size:9px;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);font-weight:700;">"TTL"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <tr>
                                                        <td style="padding:10px 12px;">
                                                            <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-cobalt/10 border border-cobalt/20" style="color:var(--cobalt);">
                                                                {d.record_type.clone()}
                                                            </span>
                                                        </td>
                                                        <td style="padding:10px 12px;">
                                                            <code class="text-xs font-mono text-on-surface">{d.name.clone()}</code>
                                                        </td>
                                                        <td style="padding:10px 12px;">
                                                            <code class="text-xs font-mono text-primary">{d.value.clone()}</code>
                                                        </td>
                                                        <td style="padding:10px 12px;">
                                                            <span class="text-xs text-on-surface-variant/50">"Auto"</span>
                                                        </td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>
                                        <div style="padding:10px 12px;background:rgba(99,179,237,0.06);border-radius:6px;border:1px solid rgba(99,179,237,0.15);">
                                            <p class="text-[11px] text-on-surface-variant/80 leading-relaxed">
                                                "ⓘ  "
                                                <strong>"Cloudflare users:"</strong>
                                                " set Proxy Status to DNS-only (grey cloud). cert-manager handles TLS — do not proxy this record."
                                            </p>
                                        </div>
                                    </div>
                                </div>
                                // ── Re-Provision button ────────────────────────────────────
                                <div class="section" style="border-left:3px solid var(--cobalt);">
                                    <div class="section-hdr">
                                        <span class="section-title">"Re-Provision Ingress"</span>
                                    </div>
                                    <div style="padding:12px 16px;display:flex;flex-direction:column;gap:8px;">
                                        <p class="text-[11px] text-on-surface-variant/70 leading-relaxed">
                                            "If the CNAME is set correctly but the site shows an NGINX error, the instance may have been created before the ingress-sidecar was deployed. \
                                             Clicking Re-Provision will re-fire the ingress + TLS provisioning event."
                                        </p>
                                        {
                                            let reprov_domain = app.clone();
                                            let toast_ref = use_context::<crate::app::GlobalToast>().expect("toast");
                                            let is_reprovisioning = RwSignal::new(false);
                                            view! {
                                                <button
                                                    class="btn btn-primary btn-sm self-start"
                                                    disabled=move || is_reprovisioning.get()
                                                    on:click=move |_| {
                                                        is_reprovisioning.set(true);
                                                        let iid = reprov_domain.instance_id.clone();
                                                        let tr = toast_ref.clone();
                                                        leptos::task::spawn_local(async move {
                                                            let url = crate::api::client::api_url(
                                                                &format!("api/admin/app-instances/{}/reprovision-domain", iid));
                                                            let res = crate::api::client::with_credentials(
                                                                crate::api::client::create_client().post(&url)
                                                            ).send().await;
                                                            is_reprovisioning.set(false);
                                                            match res {
                                                                Ok(r) if r.status().is_success() =>
                                                                    tr.show_toast("Reprovisioning", "Ingress re-provisioning triggered. Allow 60–120 s for cert-manager to issue the certificate.", "success"),
                                                                Ok(r) => tr.show_toast("Error", &format!("HTTP {} — check backend logs", r.status()), "error"),
                                                                Err(e) => tr.show_toast("Error", &e.to_string(), "error"),
                                                            }
                                                        });
                                                    }
                                                >
                                                    {move || if is_reprovisioning.get() { "Reprovisioning…" } else { "Re-Provision Ingress & TLS" }}
                                                </button>
                                            }
                                        }
                                    </div>
                                </div>
                            }.into_any(),
                            None if is_w => view! {
                                <div style="padding:10px 14px;border-left:3px solid rgba(52,211,153,0.5);border-radius:0 6px 6px 0;background:rgba(52,211,153,0.04);">
                                    <p style="font-size:11px;color:#34d399;font-weight:600;margin:0 0 2px;">"Wildcard domain — no DNS action needed"</p>
                                    <p class="muted" style="font-size:11px;margin:0;">"This subdomain is covered by the platform wildcard certificate. SSL is already active."</p>
                                </div>
                            }.into_any(),
                            None => view! { <></> }.into_any(),
                        }}
                    </div>
                }
            }}
        </Suspense>
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
                                            let active_cls = if u.is_active { "color:var(--green)" } else { "text-on-surface-variant/40" };
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
                                        let res = crate::api::client::with_credentials(
                                            crate::api::client::create_client().post(&url)
                                        ).send().await;
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

// ── Danger Zone ───────────────────────────────────────────────────────────────

#[component]
fn DangerZoneTab(instance_id: String, instance_name: String) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let iid = store_value(instance_id);
    let iname = store_value(instance_name);

    let confirm_archive = RwSignal::new(String::new());
    let confirm_reset   = RwSignal::new(String::new());
    let is_archiving    = RwSignal::new(false);
    let is_resetting    = RwSignal::new(false);

    let expected = Signal::derive(move || iname.get_value());

    view! {
        <div class="space-y-4" style="max-width:640px;">
            <div style="padding:12px 16px;border-left:3px solid var(--amber);border-radius:0 8px 8px 0;background:rgba(245,158,11,0.06);">
                <p style="font-size:12px;font-weight:700;color:var(--amber);margin:0 0 2px;">
                    "Destructive Actions — Proceed with Caution"
                </p>
                <p class="text-[11px] text-on-surface-variant/70 leading-relaxed" style="margin:0;">
                    "Archive removes the instance from active monitoring (data preserved). \
                    Reset re-queues the onboarding wizard without deleting configuration."
                </p>
            </div>

            // ── Reset ──────────────────────────────────────────────────────────
            <div class="section">
                <div class="section-hdr"><span class="section-title">"Reset Instance"</span></div>
                <div style="padding:16px;display:flex;flex-direction:column;gap:12px;">
                    <p class="text-[11px] text-on-surface-variant/70 leading-relaxed">
                        "Sets status back to " <code class="text-xs">"provisioning"</code>
                        " and re-displays the onboarding wizard. Config data is not deleted."
                    </p>
                    <p class="text-[11px] text-on-surface-variant/60">
                        "Type the instance name to confirm: "
                        <strong class="text-on-surface font-mono">{move || expected.get()}</strong>
                    </p>
                    <div style="display:flex;gap:8px;align-items:center;">
                        <input type="text" class="input input-sm flex-1"
                            placeholder="Type instance name to confirm"
                            prop:value=move || confirm_reset.get()
                            on:input=move |e| { confirm_reset.set(event_target_value(&e)); }
                        />
                        <button
                            class="btn btn-sm"
                            style="border-color:rgba(245,158,11,0.3);color:var(--amber);"
                            disabled=move || is_resetting.get() || confirm_reset.get() != expected.get()
                            on:click=move |_| {
                                is_resetting.set(true);
                                let iid2 = iid.get_value();
                                let tr = toast.clone();
                                leptos::task::spawn_local(async move {
                                    let url = crate::api::client::api_url(
                                        &format!("api/admin/app-instances/{}/reset", iid2));
                                    let res = crate::api::client::create_client()
                                        .post(&url).send().await;
                                    is_resetting.set(false);
                                    match res {
                                        Ok(r) if r.status().is_success() =>
                                            tr.show_toast("Reset", "Instance reset to provisioning state.", "success"),
                                        Ok(r) => tr.show_toast("Error", &format!("HTTP {}", r.status()), "error"),
                                        Err(e) => tr.show_toast("Error", &e.to_string(), "error"),
                                    }
                                });
                            }
                        >
                            {move || if is_resetting.get() { "Resetting…" } else { "Reset Instance" }}
                        </button>
                    </div>
                </div>
            </div>

            // ── Archive ────────────────────────────────────────────────────────
            <div class="section" style="border-left:3px solid var(--error);">
                <div class="section-hdr"><span class="section-title">"Archive Instance"</span></div>
                <div style="padding:16px;display:flex;flex-direction:column;gap:12px;">
                    <p class="text-[11px] text-on-surface-variant/70 leading-relaxed">
                        "Sets " <code class="text-xs">"instance_status = 'archived'"</code>
                        ". No longer appears in active listings. Data is retained. \
                        Ingress/DNS not auto-removed."
                    </p>
                    <p class="text-[11px] text-on-surface-variant/60">
                        "Type the instance name to confirm: "
                        <strong class="text-on-surface font-mono">{move || expected.get()}</strong>
                    </p>
                    <div style="display:flex;gap:8px;align-items:center;">
                        <input type="text" class="input input-sm flex-1"
                            placeholder="Type instance name to confirm"
                            prop:value=move || confirm_archive.get()
                            on:input=move |e| { confirm_archive.set(event_target_value(&e)); }
                        />
                        <button
                            class="btn btn-sm"
                            style="background:rgba(239,68,68,0.1);border-color:rgba(239,68,68,0.3);color:var(--error);"
                            disabled=move || is_archiving.get() || confirm_archive.get() != expected.get()
                            on:click=move |_| {
                                is_archiving.set(true);
                                let iid2 = iid.get_value();
                                let tr = toast.clone();
                                leptos::task::spawn_local(async move {
                                    let url = crate::api::client::api_url(
                                        &format!("api/admin/app-instances/{}", iid2));
                                    let res = crate::api::client::create_client()
                                        .delete(&url).send().await;
                                    is_archiving.set(false);
                                    match res {
                                        Ok(r) if r.status().is_success() =>
                                            tr.show_toast("Archived", "Instance archived successfully.", "success"),
                                        Ok(r) => tr.show_toast("Error", &format!("HTTP {}", r.status()), "error"),
                                        Err(e) => tr.show_toast("Error", &e.to_string(), "error"),
                                    }
                                });
                            }
                        >
                            {move || if is_archiving.get() { "Archiving…" } else { "Archive Instance" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
