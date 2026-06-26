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
use crate::api::admin::{
    get_tenant_stats, get_all_platform_apps,
    link_deployment_account, get_crm_accounts,
    suspend_instance, resume_instance,
};
use crate::api::models::{TenantStatModel, PlatformAppSummary, AccountSummary};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn status_pill(s: &str) -> &'static str {
    match s {
        "active"      => "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase bg-emerald-500/10 border border-emerald-500/20 text-emerald-400",
        "provisioning"=> "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase bg-blue-500/10 border border-blue-500/20 text-blue-400",
        "beta"        => "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase bg-amber-500/10 border border-amber-500/20 text-amber-400",
        "suspended"   => "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase bg-red-500/10 border border-red-500/20 text-red-400",
        "cancelled"   => "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase bg-outline-variant/20 border border-outline-variant/30 text-on-surface-variant/50",
        _             => "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase bg-outline-variant/20 border border-outline-variant/30 text-on-surface-variant/60",
    }
}

fn fmt_mrr(cents: i64) -> String {
    if cents == 0 { return "$0".to_string(); }
    let dollars = cents / 100;
    format!("${}/mo", dollars)
}

fn app_type_label(t: &str) -> &'static str {
    match t {
        "property_management" | "folio" => "Folio",
        "anchor"   => "Anchor",
        "meridian" => "Meridian",
        _          => "App",
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
    let search   = RwSignal::new(String::new());
    let saving   = RwSignal::new(false);
    let err_msg  = RwSignal::new(Option::<String>::None);
    let selected = RwSignal::new(current_account_id.clone());

    let accounts_res = LocalResource::new(|| async move {
        get_crm_accounts().await.unwrap_or_default()
    });

    let tid_save    = tenant_id.clone();
    let on_save = move |_| {
        let tid  = tid_save.clone();
        let acct = selected.get();
        saving.set(true);
        err_msg.set(None);
        let on_saved_cb = on_saved.clone();
        leptos::task::spawn_local(async move {
            let result = link_deployment_account(
                &tid,
                acct.as_deref(),
            ).await;
            saving.set(false);
            match result {
                Ok(_)  => on_saved_cb.run(()),
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
                    class="p-1.5 rounded hover:bg-surface-container-high/50 text-on-surface-variant transition-colors"
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
            <div class="flex-1 overflow-y-auto px-5 pb-4 space-y-1">
                // "No link" option
                <button
                    class=move || {
                        if selected.get().is_none() {
                            "w-full text-left px-3 py-2.5 rounded-lg text-xs font-medium bg-primary/10 border border-primary/30 text-primary"
                        } else {
                            "w-full text-left px-3 py-2.5 rounded-lg text-xs text-on-surface-variant hover:bg-surface-container-high/40 transition-colors"
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

                <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant/50 text-center py-4 animate-pulse">"Loading..."</p> }>
                    {move || {
                        let accounts = accounts_res.get().unwrap_or_default();
                        let q = search.get().to_lowercase();
                        let filtered: Vec<AccountSummary> = accounts.into_iter()
                            .filter(|a| q.is_empty() || a.name.to_lowercase().contains(&q))
                            .collect();
                        if filtered.is_empty() {
                            view! { <p class="text-xs text-on-surface-variant/50 text-center py-4">"No accounts found"</p> }.into_any()
                        } else {
                            view! {
                                <div class="space-y-1">
                                    {filtered.into_iter().map(|acct| {
                                        let aid = acct.id.clone();
                                        let aid2 = aid.clone();
                                        let id_short = acct.id.chars().take(8).collect::<String>();
                                        view! {
                                            <button
                                                class=move || {
                                                    if selected.get().as_deref() == Some(&aid2) {
                                                        "w-full text-left px-3 py-2.5 rounded-lg text-xs font-medium bg-primary/10 border border-primary/30 text-primary"
                                                    } else {
                                                        "w-full text-left px-3 py-2.5 rounded-lg text-xs text-on-surface hover:bg-surface-container-high/40 transition-colors"
                                                    }
                                                }
                                                on:click=move |_| selected.set(Some(aid.clone()))
                                            >
                                                <div class="font-medium">{acct.name.clone()}</div>
                                                <div class="text-[10px] text-on-surface-variant/50 mt-0.5 font-mono">{id_short}</div>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </Suspense>
            </div>

            // Footer
            <div class="px-5 py-4 border-t border-outline-variant/20 space-y-2">
                {move || err_msg.get().map(|e| view! {
                    <p class="text-xs text-red-400">{e}</p>
                })}
                <button
                    on:click=on_save
                    disabled=saving
                    class="w-full px-4 py-2 bg-primary text-on-primary rounded-lg text-xs font-semibold hover:bg-primary/90 disabled:opacity-50 transition-colors"
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
    let tenants_res  = LocalResource::new(move || async move { get_tenant_stats().await.unwrap_or_default() });
    let apps_res     = LocalResource::new(move || async move { get_all_platform_apps().await.unwrap_or_default() });

    let search        = RwSignal::new(String::new());
    let filter_status = RwSignal::new("all".to_string());
    // Modal state: Some(tenant_id) = modal open for that tenant
    let modal_tenant_id  = RwSignal::new(Option::<String>::None);
    let modal_account_id = RwSignal::new(Option::<String>::None);
    // Refresh trigger: increment to force resource re-fetch
    let refresh = RwSignal::new(0u32);

    view! {
        <div class="p-8 max-w-screen-2xl mx-auto space-y-6">

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

            // ── Page Header ──────────────────────────────────────────────────
            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">"Clients"</h1>
                    <p class="text-sm text-on-surface-variant mt-1 max-w-xl">
                        "Active subscriber tenants and their deployments. "
                        "Each client gets a dedicated Folio (or other app) instance. "
                        "Internal deployments are managed under "
                        <a href="/internal-instances" class="text-primary hover:underline">"Internal Instances"</a>
                        "."
                    </p>
                </div>
                <a href="/network/new"
                    class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold flex items-center gap-2"
                >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="2" x2="8" y2="14"/><line x1="2" y1="8" x2="14" y2="8"/></svg>
                    "Provision Client"
                </a>
            </div>

            // ── KPI Row ──────────────────────────────────────────────────────
            <Suspense fallback=|| view! { <div class="h-20 animate-pulse bg-surface-container-low rounded-xl" /> }>
                {move || {
                    let tenants = tenants_res.get().unwrap_or_default();
                    let apps    = apps_res.get().unwrap_or_default();
                    let active  = apps.iter().filter(|a| a.site_status == "active").count();
                    let provisioning = apps.iter().filter(|a| a.site_status == "provisioning").count();
                    let total_mrr: i64 = tenants.iter().filter_map(|t| t.mrr_cents).sum();

                    view! {
                        <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                            {[
                                ("Total Clients", tenants.len().to_string(), "text-on-surface"),
                                ("Live Instances", active.to_string(), "text-emerald-400"),
                                ("Provisioning", provisioning.to_string(), "text-blue-400"),
                                ("Monthly Revenue", fmt_mrr(total_mrr), "text-primary"),
                            ].iter().map(|(label, val, color)| {
                                let val = val.clone();
                                let color = color.to_string();
                                let label = label.to_string();
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-5 py-4">
                                        <div class=format!("text-xl font-extrabold font-mono {}", color)>{val}</div>
                                        <div class="text-[9px] uppercase tracking-wider text-on-surface-variant/60 mt-1">{label}</div>
                                    </div>
                                }
                            }).collect_view()}
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
                                    "px-3 py-1.5 rounded text-[10px] font-bold uppercase tracking-wider bg-primary/20 border border-primary/30 text-primary"
                                } else {
                                    "px-3 py-1.5 rounded text-[10px] font-bold uppercase tracking-wider bg-surface-container-low border border-outline-variant/20 text-on-surface-variant hover:text-on-surface"
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
                                                <th class="py-3 px-5 font-semibold">"App"</th>
                                                <th class="py-3 px-5 font-semibold">"Status"</th>
                                                <th class="py-3 px-5 font-semibold">"Plan / MRR"</th>
                                                <th class="py-3 px-5 font-semibold">"Domain"</th>
                                                <th class="py-3 px-5 font-semibold">"Usage"</th>
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
                                                let instance_id = primary_app.map(|a| a.instance_id.clone()).unwrap_or_default();
                                                let domain = primary_app.map(|a| a.domain.clone()).unwrap_or("—".to_string());
                                                let app_type = primary_app.map(|a| app_type_label(&a.app_type).to_string()).unwrap_or("—".to_string());
                                                let instance_status = t.site_status.clone().unwrap_or("unknown".to_string());
                                                let mrr_str = t.mrr_cents.map(|c| fmt_mrr(c)).unwrap_or("—".to_string());
                                                let plan_str = t.plan.clone().unwrap_or("—".to_string());
                                                let anchor_id = t.anchor_instance_id.clone();
                                                let tenant_id_str = t.tenant_id.clone();

                                                view! {
                                                    <tr class="hover:bg-surface-bright/5 transition-colors group">
                                                        // Client name + ID
                                                        <td class="py-3.5 px-5">
                                                            <div class="font-semibold text-on-surface">{t.name.clone()}</div>
                                                            <div class="text-[9px] font-mono text-on-surface-variant/40 mt-0.5">
                                                                {tenant_id_str.clone().chars().take(8).collect::<String>()}
                                                                "..."
                                                            </div>
                                                        </td>
                                                        // App type
                                                        <td class="py-3.5 px-5">
                                                            <span class="px-2 py-0.5 rounded bg-surface-container border border-outline-variant/20 text-[9px] font-mono text-on-surface-variant uppercase">
                                                                {app_type}
                                                            </span>
                                                        </td>
                                                        // Status
                                                        <td class="py-3.5 px-5">
                                                            <span class=status_pill(&instance_status)>
                                                                {instance_status.clone()}
                                                            </span>
                                                        </td>
                                                        // Plan / MRR
                                                        <td class="py-3.5 px-5">
                                                            <div class="font-semibold text-on-surface">{mrr_str}</div>
                                                            <div class="text-[9px] text-on-surface-variant/50 mt-0.5">{plan_str}</div>
                                                        </td>
                                                        // Domain
                                                        <td class="py-3.5 px-5 font-mono text-[10px] text-on-surface-variant/70 max-w-[200px] truncate">
                                                            {domain}
                                                        </td>
                                                        // Usage
                                                        <td class="py-3.5 px-5">
                                                            <div class="flex gap-3 text-[10px] text-on-surface-variant/70">
                                                                <span title="Listings">{format!("{} listings", t.listing_count)}</span>
                                                                <span class="text-outline-variant/30">"·"</span>
                                                                <span title="Profiles">{format!("{} users", t.profile_count)}</span>
                                                            </div>
                                                        </td>
                                                        // Actions
                                                        <td class="py-3.5 px-5">
                                                            <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                // View Account — visible when platform_account_id is already set
                                                                {primary_app.and_then(|a| a.platform_account_id.clone()).map(|acct_id| view! {
                                                                    <a href=format!("/crm/accounts/{}", acct_id)
                                                                        class="px-2.5 py-1 bg-primary/10 border border-primary/20 rounded text-[9px] font-semibold text-primary hover:bg-primary/20 transition-colors"
                                                                        title="View CRM Account"
                                                                    >"CRM Account"</a>
                                                                })}
                                                                // Link Account — always visible, opens the modal
                                                                <button
                                                                    class="px-2.5 py-1 bg-surface-container-high/50 border border-outline-variant/30 rounded text-[9px] font-semibold text-on-surface-variant hover:text-on-surface hover:border-primary/40 transition-colors"
                                                                    on:click={
                                                                        let tid = tenant_id_str.clone();
                                                                        let acct = primary_app.and_then(|a| a.platform_account_id.clone());
                                                                        move |_| {
                                                                            modal_account_id.set(acct.clone());
                                                                            modal_tenant_id.set(Some(tid.clone()));
                                                                        }
                                                                    }
                                                                >"Link Account"</button>
                                                                {anchor_id.clone().map(|aid| view! {
                                                                    <a href=format!("/apps/{}/instance", aid)
                                                                        class="px-2.5 py-1 bg-surface-container-high/50 border border-outline-variant/30 rounded text-[9px] font-semibold text-on-surface-variant hover:text-on-surface transition-colors"
                                                                    >"Instance"</a>
                                                                })}
                                                                <a href=format!("/billing/tenant/{}", tenant_id_str)
                                                                    class="px-2.5 py-1 bg-surface-container-high/50 border border-outline-variant/30 rounded text-[9px] font-semibold text-on-surface-variant hover:text-on-surface transition-colors"
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
            <p class="text-[10px] text-on-surface-variant/40 text-center">
                "Showing subscriber tenants. For internally managed deployments, see "
                <a href="/internal-instances" class="text-primary/60 hover:text-primary">"Internal Instances"</a>
                ". For infrastructure-level instance control, see "
                <a href="/apps" class="text-primary/60 hover:text-primary">"Apps"</a>
                "."
            </p>
        </div>
    }
}
