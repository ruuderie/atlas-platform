//! App Instance — Thin Dispatcher
//!
//! This file is the entry-point for `/apps/:id/instance`. It:
//!   1. Reads the `{id}` path param (may be deployment_config_id OR tenant_id)
//!   2. Resolves the canonical deployment_config_id via the dirs context
//!   3. Calls `get_public_config` with the resolved ID to get the `app_slug`
//!   4. Dispatches to a type-specific sub-component:
//!      - `"property_management"` → `FolioInstance`
//!      - `"anchor"`             → `AnchorInstance`
//!      - `"network_instance"`   → `NetworkInstance`
//!      - anything else          → generic fallback
//!
//! The URL :id param is always treated as ambiguous: we first try matching it
//! as an instance_id (deployment_config_id), then fall back to matching it as
//! a tenant_id. This handles both correctly-generated links and manually typed
//! or bookmarked URLs that may use the tenant UUID.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::admin::get_public_config;
use crate::api::models::PlatformAppModel;

// Sub-components — each handles a specific app type's UI.
// Rust looks for these in `pages/apps/instance/` relative to this file.
pub mod folio_instance;
pub mod anchor_instance;
pub mod network_instance;

use folio_instance::FolioInstance;
use anchor_instance::AnchorInstance;
use network_instance::NetworkInstance;

// ── Thin loader / dispatcher ──────────────────────────────────────────────────

#[component]
pub fn AppInstance() -> impl IntoView {
    let params = use_params_map();
    let url_id = move || params.with(|p| p.get("id").unwrap_or_default());

    // The URL :id may be either a deployment_config_id (normal listing links)
    // or a tenant_id (bookmarked / manually typed URLs). Resolve both by
    // looking up the dirs context first — O(n) over a tiny list, always in memory.
    let dirs = use_context::<LocalResource<Vec<PlatformAppModel>>>()
        .expect("dirs context must be provided by AuthenticatedLayout");

    // Derive the canonical deployment_config_id from the URL id.
    // Priority: match by instance_id first, then match by tenant_id.
    let resolved_config_id = Signal::derive(move || {
        let id = url_id();
        dirs.get().and_then(|apps| {
            // 1. Direct match: url_id IS the deployment_config_id
            apps.iter()
                .find(|a| a.instance_id == id)
                .map(|a| a.instance_id.clone())
                // 2. Fallback: url_id is the tenant_id — find the first instance
                .or_else(|| {
                    apps.iter()
                        .find(|a| a.tenant_id == id)
                        .map(|a| a.instance_id.clone())
                })
        })
    });

    // Fetch public config using the resolved deployment_config_id.
    let instance_config = LocalResource::new(move || {
        let maybe_id = resolved_config_id.get();
        async move {
            match maybe_id {
                Some(id_str) => match uuid::Uuid::parse_str(&id_str) {
                    Ok(id) => get_public_config(id).await.ok(),
                    Err(_) => None,
                },
                None => None,
            }
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
                    let cfg_opt = instance_config.get().flatten();
                    match cfg_opt {
                        None => view! {
                            <div class="flex flex-col items-center justify-center h-64 gap-4">
                                <span class="material-symbols-outlined text-4xl text-on-surface-variant/40">"deployed_code"</span>
                                <p class="text-on-surface-variant text-sm">"Instance not found. It may still be loading or the URL may be invalid."</p>
                                <a href="/apps" class="btn btn-ghost btn-sm">"← Back to Tenants"</a>
                            </div>
                        }.into_any(),
                        Some(cfg) => {
                            // Dispatch to the correct type-specific component.
                            // Use exact string matching — never `contains()`.
                            match cfg.app_slug.as_str() {
                                "property_management" => view! {
                                    <div class="w-full space-y-4">
                                        <crate::pages::billing::scorecard_panel::ScorecardPanel
                                            entity_type="app_instance".to_string()
                                            entity_id=cfg.instance_id.to_string()
                                            tenant_id=cfg.tenant_id.to_string()
                                            subject_label=cfg.tenant_name.clone()
                                        />
                                        <FolioInstance cfg=cfg />
                                    </div>
                                }.into_any(),
                                "anchor" => view! {
                                    <div class="w-full space-y-4">
                                        <crate::pages::billing::scorecard_panel::ScorecardPanel
                                            entity_type="app_instance".to_string()
                                            entity_id=cfg.instance_id.to_string()
                                            tenant_id=cfg.tenant_id.to_string()
                                            subject_label=cfg.tenant_name.clone()
                                        />
                                        <AnchorInstance cfg=cfg />
                                    </div>
                                }.into_any(),
                                "network_instance" => view! {
                                    <div class="w-full space-y-4">
                                        <crate::pages::billing::scorecard_panel::ScorecardPanel
                                            entity_type="app_instance".to_string()
                                            entity_id=cfg.instance_id.to_string()
                                            tenant_id=cfg.tenant_id.to_string()
                                            subject_label=cfg.tenant_name.clone()
                                        />
                                        <NetworkInstance cfg=cfg />
                                    </div>
                                }.into_any(),
                                other => view! {
                                    // Generic fallback for unknown or future app types.
                                    // Shows identity card so ops can still inspect the instance.
                                    <div class="w-full space-y-6">
                                        <crate::pages::billing::scorecard_panel::ScorecardPanel
                                            entity_type="app_instance".to_string()
                                            entity_id=cfg.instance_id.to_string()
                                            tenant_id=cfg.tenant_id.to_string()
                                            subject_label=cfg.tenant_name.clone()
                                        />
                                        <div class="bg-amber-500/10 border border-amber-500/20 rounded-xl p-5 text-xs text-amber-400">
                                            "Unknown app_slug: " {other.to_string()}
                                            ". No type-specific UI is registered. Please add a new instance component."
                                        </div>
                                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                            <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"App Instance Identity"</h3>
                                            </div>
                                            <div class="divide-y divide-outline-variant/10 text-xs">
                                                <div class="flex justify-between items-center px-5 py-3">
                                                    <span class="text-on-surface-variant">"app_slug"</span>
                                                    <span class="font-mono text-on-surface/80">{cfg.app_slug.clone()}</span>
                                                </div>
                                                <div class="flex justify-between items-center px-5 py-3">
                                                    <span class="text-on-surface-variant">"Instance ID"</span>
                                                    <span class="font-mono text-on-surface/80 text-[10px]">{cfg.instance_id.to_string()}</span>
                                                </div>
                                                <div class="flex justify-between items-center px-5 py-3">
                                                    <span class="text-on-surface-variant">"Tenant"</span>
                                                    <span class="font-mono text-on-surface/80 text-[10px]">{cfg.tenant_id.to_string()}</span>
                                                </div>
                                                <div class="flex justify-between items-center px-5 py-3">
                                                    <span class="text-on-surface-variant">"Status"</span>
                                                    <span class="font-mono text-on-surface/80">{cfg.instance_status.clone()}</span>
                                                </div>
                                                <div class="flex justify-between items-center px-5 py-3">
                                                    <span class="text-on-surface-variant">"Billing Tier"</span>
                                                    <span class="font-mono text-on-surface/80">{cfg.billing_tier.clone()}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                            }
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}
