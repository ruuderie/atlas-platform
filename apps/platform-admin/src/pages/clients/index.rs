use crate::api::admin::{
    get_all_platform_apps, get_crm_accounts, get_tenant_stats, link_deployment_account,
    resume_instance, suspend_instance,
};
use crate::api::models::{AccountSummary, PlatformAppSummary, TenantStatModel};
/// # Clients — Subscription & Tenant Management
///
/// Route: /clients
///
/// This replaces the old "Network Instances" view for external customer deployments.
/// Each row is a **Tenant** (a paying client) with their linked app instance(s).
///
/// Internal/operator-managed deployments are intentionally excluded here —
/// see /internal-instances under Operations.
///
/// Data: get_tenant_stats() → TenantStatModel (tenant-centric)
///       get_all_platform_apps() → PlatformAppSummary (instance details)
use leptos::prelude::*;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn status_pill(s: &str) -> &'static str {
    // Returns inline style string for a .plan-badge span
    match s {
        "active" => "color:var(--green);border-color:var(--green);background:var(--green-dim)",
        "provisioning" => {
            "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)"
        }
        "beta" => "color:var(--amber);border-color:var(--amber);background:var(--amber-dim)",
        "suspended" => "color:var(--error);border-color:var(--error);background:var(--red-dim)",
        "cancelled" => "color:var(--text-muted);border-color:var(--border-default)",
        _ => "color:var(--text-muted);border-color:var(--border-default)",
    }
}

fn fmt_mrr(cents: i64) -> String {
    if cents == 0 {
        return "$0".to_string();
    }
    let dollars = cents / 100;
    format!("${}/mo", dollars)
}

fn app_type_label(t: &str) -> &'static str {
    match t {
        "property_management" | "folio" => "Folio",
        "anchor" => "Anchor",
        "meridian" => "Meridian",
        _ => "App",
    }
}

// ── Link Account Modal ────────────────────────────────────────────────────────
// A lightweight slide-in panel for linking a client deployment to a CRM Account.
// Shown when the operator clicks "Link Account" on a Clients row.

#[component]
fn LinkAccountModal(
    tenant_id: String,
    /// If Some, the current linked account id.
    current_account_id: Option<String>,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let err_msg = RwSignal::new(Option::<String>::None);
    let selected = RwSignal::new(current_account_id.clone());

    let accounts_res =
        LocalResource::new(|| async move { get_crm_accounts().await.unwrap_or_default() });

    let tid_save = tenant_id.clone();
    let on_save = move |_| {
        let tid = tid_save.clone();
        let acct = selected.get();
        saving.set(true);
        err_msg.set(None);
        let on_saved_cb = on_saved.clone();
        leptos::task::spawn_local(async move {
            let result = link_deployment_account(&tid, acct.as_deref()).await;
            saving.set(false);
            match result {
                Ok(_) => on_saved_cb.run(()),
                Err(e) => err_msg.set(Some(format!("Save failed: {}", e))),
            }
        });
    };

    let on_close_cb = on_close.clone();

    view! {
        // Backdrop
        <div
            class="fixed inset-0 bg-black/50 backdrop-blur-sm z-40"
            on:click=move |_| on_close_cb.run(())
        />
        // Panel
        <div class="fixed right-0 top-0 bottom-0 w-96 bg-surface-container border-l border-outline-variant/30 z-50 flex flex-col shadow-2xl">
            // Header
            <div class="flex items-center justify-between px-5 py-4 border-b border-outline-variant/20">
                <div>
                    <h2 class="text-sm font-bold text-on-surface">"Link CRM Account"</h2>
                    <p class="text-[10px] text-on-surface-variant/60 mt-0.5">{format!("Tenant: {}", &tenant_id[..8])}</p>
                </div>
                <button
                    on:click=move |_| on_close.run(())
                    class="btn btn-ghost btn-icon btn-sm"
                    title="Close"
                >
                    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" class="w-4 h-4">
                        <path d="M3 3l10 10M13 3L3 13"/>
                    </svg>
                </button>
            </div>

            // Search
            <div class="px-5 pt-4 pb-2">
                <input
                    type="text"
                    placeholder="Search accounts..."
                    class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
            </div>

            // Account list
            <div class="flex-1 overflow-y-auto px-5 pb-4">
                <div class="filter-nav">
                // "No link" option
                <button
                    class=move || {
                        if selected.get().is_none() {
                            "filter-nav-item active"
                        } else {
                            "filter-nav-item"
                        }
                    }
                    on:click=move |_| selected.set(None)
                >
                    <div class="flex items-center gap-2">
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" class="w-3 h-3 shrink-0">
                            <path d="M3 3l10 10M13 3L3 13"/>
                        </svg>
                        "Unlink (no account)"
                    </div>
                </button>

                <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant text-center py-4 animate-pulse">"Loading..."</p> }>
                    {move || {
                        let accounts = accounts_res.get().unwrap_or_default();
                        let q = search.get().to_lowercase();
                        let filtered: Vec<AccountSummary> = accounts.into_iter()
                            .filter(|a| q.is_empty() || a.name.to_lowercase().contains(&q))
                            .collect();
                        if filtered.is_empty() {
                            view! { <p class="text-xs text-on-surface-variant text-center py-4">"No accounts found"</p> }.into_any()
                        } else {
                            view! {
                                <>
                                    {filtered.into_iter().map(|acct| {
                                        let aid = acct.id.clone();
                                        let aid2 = aid.clone();
                                        let id_short = acct.id.chars().take(8).collect::<String>();
                                        view! {
                                            <button
                                                class=move || {
                                                    if selected.get().as_deref() == Some(&aid2) {
                                                        "filter-nav-item active"
                                                    } else {
                                                        "filter-nav-item"
                                                    }
                                                }
                                                on:click=move |_| selected.set(Some(aid.clone()))
                                            >
                                                <div class="font-medium">{acct.name.clone()}</div>
                                                <div class="text-[10px] text-on-surface-variant mt-0.5 font-mono">{id_short}</div>
                                            </button>
                                        }
                                    }).collect_view()}
                                </>
                            }.into_any()
                        }
                    }}
                </Suspense>
                </div>
            </div>

            // Footer
            <div class="px-5 py-4 border-t border-outline-variant/20 space-y-2">
                {move || err_msg.get().map(|e| view! {
                    <p class="text-xs text-red-400">{e}</p>
                })}
                <button
                    on:click=on_save
                    disabled=saving
                    class="btn btn-primary w-full disabled:opacity-50"
                >
                    {move || if saving.get() { "Saving..." } else { "Save Link" }}
                </button>
            </div>
        </div>
    }
}

// ── Page Component ────────────────────────────────────────────────────────────

#[component]
pub fn ClientsPage() -> impl IntoView {
    // Refresh trigger: increment to force resource re-fetch
    let refresh = RwSignal::new(0u32);
    let clients_error: RwSignal<Option<String>> = RwSignal::new(None);
    let tenants_res = LocalResource::new(move || async move {
        let _ = refresh.get();
        match get_tenant_stats().await {
            Ok(v) => {
                clients_error.set(None);
                v
            }
            Err(e) => {
                clients_error.set(Some(e));
                vec![]
            }
        }
    });
    let apps_res = LocalResource::new(move || async move {
        let _ = refresh.get();
        get_all_platform_apps().await.unwrap_or_default()
    });

    let search = RwSignal::new(String::new());
    let filter_status = RwSignal::new("all".to_string());
    // Modal state: Some(tenant_id) = modal open for that tenant
    let modal_tenant_id = RwSignal::new(Option::<String>::None);
    let modal_account_id = RwSignal::new(Option::<String>::None);

    view! {
        <div class="main-canvas">

        // ── Link Account Modal (rendered outside main scroll) ───────────
        {move || modal_tenant_id.get().map(|tid| {
            let current_acct = modal_account_id.get();
            view! {
                <LinkAccountModal
                    tenant_id=tid
                    current_account_id=current_acct
                    on_close=Callback::new(move |_| modal_tenant_id.set(None))
                    on_saved=Callback::new(move |_| {
                        modal_tenant_id.set(None);
                        refresh.update(|n| *n += 1);
                    })
                />
            }
        })}

            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Clients"</h1>
                    <p class="page-subtitle">
                        "Active subscriber tenants and their deployments. "
                        "Each client gets a dedicated app instance. "
                        <a href="/internal-instances" style="color:var(--cobalt)">"Internal Instances"</a>
                        " managed separately."
                    </p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-icon" title="Refresh" on:click=move |_| refresh.update(|n| *n += 1)>
                        <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M2 8a6 6 0 0 1 6-6 6 6 0 0 1 4.2 1.8L14 6"/><path d="M14 2v4h-4"/>
                        </svg>
                    </button>
                    <a href="/network/new" class="btn btn-primary">"+ Provision Client"</a>
                </div>
            </div>

            // ── Error Banner ───────────────────────────────────────────────────
            {move || clients_error.get().map(|e| crate::utils::inline_error(&e))}

            // ── KPI Row ──
            <Suspense fallback=|| view! { <div style="height:72px;border-radius:8px;background:var(--bg-elevated);" /> }>
                {move || {
                    let tenants = tenants_res.get().unwrap_or_default();
                    let apps    = apps_res.get().unwrap_or_default();
                    let active  = apps.iter().filter(|a| a.site_status == "active").count();
                    let provisioning = apps.iter().filter(|a| a.site_status == "provisioning").count();
                    let total_mrr: i64 = tenants.iter().filter_map(|t| t.mrr_cents).sum();

                    view! {
                        <div class="kpi-row">
                            <div class="kpi-card">
                                <div class="kpi-label">"Total Clients"</div>
                                <div class="kpi-value mono">{tenants.len().to_string()}</div>
                            </div>
                            <div class="kpi-card">
                                <div class="kpi-label">"Live Instances"</div>
                                <div class="kpi-value mono" style="color:var(--green)">{active.to_string()}</div>
                            </div>
                            <div class="kpi-card">
                                <div class="kpi-label">"Provisioning"</div>
                                <div class="kpi-value mono" style="color:var(--cobalt)">{provisioning.to_string()}</div>
                            </div>
                            <div class="kpi-card">
                                <div class="kpi-label">"Monthly Revenue"</div>
                                <div class="kpi-value mono" style="color:var(--cobalt)">{fmt_mrr(total_mrr)}</div>
                            </div>
                        </div>
                    }
                }}
            </Suspense>

            // ── Controls ─────────────────────────────────────────────────────
            <div class="flex items-center gap-3 flex-wrap">
                <input
                    type="text"
                    placeholder="Search clients..."
                    class="bg-surface-container-low border border-outline-variant/30 text-xs rounded-lg px-3 py-2 focus:border-primary/60 outline-none transition-all text-on-surface w-56"
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
                <div class="flex gap-1">
                    {["all", "active", "provisioning", "suspended"].iter().map(|f| {
                        let f = f.to_string();
                        let f2 = f.clone();
                        view! {
                            <button
                                class=move || if filter_status.get() == f {
                                    "pill active"
                                } else {
                                    "pill"
                                }
                                on:click=move |_| filter_status.set(f2.clone())
                            >{f.clone()}</button>
                        }
                    }).collect_view()}
                </div>
            </div>

            // ── Client Table ─────────────────────────────────────────────────
            <Suspense fallback=|| view! {
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-8 text-center text-xs text-on-surface-variant/60 animate-pulse">
                    "Loading clients..."
                </div>
            }>
                {move || {
                    let tenants = tenants_res.get().unwrap_or_default();
                    let apps    = apps_res.get().unwrap_or_default();
                    let q       = search.get().to_lowercase();
                    let f_status = filter_status.get();

                    // Filter to Standard mode (paying subscriber clients) only.
                    // InternalOperator instances are visible under /internal-instances.
                    let filtered: Vec<TenantStatModel> = tenants.into_iter()
                        .filter(|t| {
                            // Determine mode from matching app
                            let app_mode = apps.iter()
                                .find(|a| a.tenant_id == t.tenant_id)
                                .map(|a| a.mode.as_str())
                                .unwrap_or("standard");
                            let is_client = app_mode == "standard";
                            let name_match = q.is_empty() || t.name.to_lowercase().contains(&q);
                            let status_match = f_status == "all"
                                || t.site_status.as_deref().unwrap_or("") == f_status;
                            is_client && name_match && status_match
                        })
                        .collect();

                    if filtered.is_empty() {
                        view! {
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-10 text-center">
                                <svg class="w-10 h-10 text-on-surface-variant/20 mx-auto mb-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/></svg>
                                <p class="text-sm text-on-surface-variant/60">"No clients match your filter."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                <div class="px-5 py-3 border-b border-outline-variant/15 bg-surface-container-high/20 flex items-center justify-between">
                                    <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">
                                        {format!("{} client{}", filtered.len(), if filtered.len() == 1 { "" } else { "s" })}
                                    </span>
                                </div>
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse text-xs">
                                        <thead>
                                            <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/10">
                                                <th class="py-3 px-5 font-semibold">"Client"</th>
                                                <th class="py-3 px-5 font-semibold">"CRM Account"</th>
                                                <th class="py-3 px-5 font-semibold">"App Type"</th>
                                                <th class="py-3 px-5 font-semibold">"Status"</th>
                                                <th class="py-3 px-5 font-semibold">"MRR"</th>
                                                <th class="py-3 px-5 font-semibold">"Joined"</th>
                                                <th class="py-3 px-5 font-semibold">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-outline-variant/5">
                                            {filtered.into_iter().map(|t| {
                                                // Find matching app instances for this tenant
                                                let tenant_apps: Vec<&PlatformAppSummary> = apps.iter()
                                                    .filter(|a| a.tenant_id == t.tenant_id)
                                                    .collect();
                                                let primary_app = tenant_apps.first().cloned();
                                                let crm_account_id   = primary_app.and_then(|a| a.platform_account_id.clone());
                                                let app_type         = primary_app.map(|a| app_type_label(&a.app_type).to_string()).unwrap_or("—".to_string());
                                                let instance_status  = t.site_status.clone().unwrap_or("unknown".to_string());
                                                let mrr_str          = t.mrr_cents.map(|c| fmt_mrr(c)).unwrap_or("—".to_string());
                                                let anchor_id        = t.anchor_instance_id.clone();
                                                let tenant_id_str    = t.tenant_id.clone();
                                                let joined_str       = t.joined_at.as_deref()
                                                    .and_then(|s| s.get(..10))
                                                    .unwrap_or("—")
                                                    .to_string();

                                                // Avatar: colored circle with initials, color based on status
                                                let initial = t.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                                                let avatar_style = match instance_status.as_str() {
                                                    "active" => "width:26px;height:26px;border-radius:50%;background:var(--cobalt-dim);border:1px solid var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:9px;font-weight:700;color:var(--cobalt);flex-shrink:0;",
                                                    "suspended" => "width:26px;height:26px;border-radius:50%;background:var(--red-dim);border:1px solid var(--red);display:flex;align-items:center;justify-content:center;font-size:9px;font-weight:700;color:var(--red);flex-shrink:0;",
                                                    "provisioning" => "width:26px;height:26px;border-radius:50%;background:var(--amber-dim);border:1px solid var(--amber);display:flex;align-items:center;justify-content:center;font-size:9px;font-weight:700;color:var(--amber);flex-shrink:0;",
                                                    _ => "width:26px;height:26px;border-radius:50%;background:var(--cobalt-dim);border:1px solid var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:9px;font-weight:700;color:var(--cobalt);flex-shrink:0;",
                                                };

                                                view! {
                                                    <tr class="hover:bg-surface-bright/5 transition-colors cursor-pointer">
                                                        // Client avatar + name + ID
                                                        <td class="py-3.5 px-5">
                                                            <div style="display:flex;align-items:center;gap:8px;">
                                                                <div style=avatar_style>{initial}</div>
                                                                {if let Some(ref aid) = anchor_id {
                                                                    let aid2 = aid.clone();
                                                                    view! {
                                                                        <a href=format!("/apps/{}", aid2) class="block group/link">
                                                                            <div class="font-semibold text-on-surface group-hover/link:text-primary transition-colors" style="font-size:12px;">{t.name.clone()}</div>
                                                                            <div class="text-[9px] font-mono text-on-surface-variant mt-0.5">
                                                                                {tenant_id_str.clone().chars().take(8).collect::<String>()}
                                                                                "..."
                                                                            </div>
                                                                        </a>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <div>
                                                                            <div class="font-semibold text-on-surface" style="font-size:12px;">{t.name.clone()}</div>
                                                                            <div class="text-[9px] font-mono text-on-surface-variant mt-0.5">
                                                                                {tenant_id_str.clone().chars().take(8).collect::<String>()}
                                                                                "..."
                                                                            </div>
                                                                        </div>
                                                                    }.into_any()
                                                                }}
                                                            </div>
                                                        </td>
                                                        // CRM Account
                                                        <td class="py-3.5 px-5">
                                                            {crm_account_id.as_ref().map(|acct_id| {
                                                                let acct_id = acct_id.clone();
                                                                view! {
                                                                    <a href={format!("/crm/accounts/{}", acct_id)}
                                                                        style="color:var(--cobalt);font-size:11px;text-decoration:none;"
                                                                        title="View CRM Account"
                                                                    >{if acct_id.len() >= 8 { acct_id[..8].to_string() + "…" } else { acct_id.clone() }}</a>
                                                                }.into_any()
                                                            }).unwrap_or_else(|| view! { <span style="color:var(--text-muted);font-size:11px">"—"</span> }.into_any())}
                                                        </td>
                                                        // App type
                                                        <td class="py-3.5 px-5">
                                                            <span class="px-2 py-0.5 rounded bg-surface-container border border-outline-variant/20 text-[9px] font-mono text-on-surface-variant uppercase">
                                                                {app_type}
                                                            </span>
                                                        </td>
                                                        // Status
                                                        <td class="py-3.5 px-5">
                                                            <span class="plan-badge" style=status_pill(&instance_status)>
                                                                {instance_status.clone()}
                                                            </span>
                                                        </td>
                                                        // MRR
                                                        <td class="py-3.5 px-5">
                                                            <span class="font-mono font-semibold text-on-surface">{mrr_str}</span>
                                                        </td>
                                                        // Joined
                                                        <td class="py-3.5 px-5 text-[11px] text-on-surface-variant font-mono">
                                                            {joined_str}
                                                        </td>
                                                        // Actions
                                                        <td class="py-3.5 px-5">
                                                            <div class="flex items-center gap-2">
                                                                // View instance — always shown when anchor_id is set
                                                                {anchor_id.clone().map(|aid| view! {
                                                                    <a href={format!("/apps/{}", aid)}
                                                                        class="btn btn-ghost btn-sm"
                                                                        style="text-decoration:none;"
                                                                    >"View →"</a>
                                                                })}
                                                                // Link Account
                                                                <button
                                                                    class="btn btn-ghost btn-sm"
                                                                    on:click={
                                                                        let tid = tenant_id_str.clone();
                                                                        let acct = crm_account_id.clone();
                                                                        move |_| {
                                                                            modal_account_id.set(acct.clone());
                                                                            modal_tenant_id.set(Some(tid.clone()));
                                                                        }
                                                                    }
                                                                >"Link Acct"</button>
                                                                <a href={format!("/tenants/{}", tenant_id_str.clone())}
                                                                    class="btn btn-ghost btn-sm"
                                                                    style="text-decoration:none;"
                                                                    title="View Tenant Detail"
                                                                >"Tenant →"</a>
                                                                <a href={format!("/billing/tenant/{}", tenant_id_str)}
                                                                    class="btn btn-ghost btn-sm"
                                                                    style="text-decoration:none;"
                                                                >"Billing"</a>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>

            // ── Footer note ──────────────────────────────────────────────────
            <p class="text-[10px] text-on-surface-variant text-center">
                "Showing subscriber tenants. For internally managed deployments, see "
                <a href="/internal-instances" class="text-primary hover:underline">"Internal Instances"</a>
                ". For infrastructure-level instance control, see "
                <a href="/apps" class="text-primary hover:underline">"Apps"</a>
                "."
            </p>
        </div>
    }
}
