//! App Instance — Thin Dispatcher
//!
//! This file is the entry-point for `/apps/:id/instance`. It:
//!   1. Reads the `{id}` path param
//!   2. Calls `get_public_config` to resolve the canonical `app_slug`
//!   3. Dispatches to a type-specific sub-component:
//!      - `"property_management"` → `FolioInstance`
//!      - `"anchor"`             → `AnchorInstance`
//!      - `"network_instance"`   → `NetworkInstance`
//!      - anything else          → generic fallback
//!
//! All business logic, tab rendering, and platform-activity stats live in
//! the sub-components under `instance/`.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::admin::get_public_config;

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
    let instance_id_str = move || {
        params.with(|p| p.get("id").unwrap_or_default())
    };

    // Fetch the canonical public config once — this gives us app_slug, tenant_id,
    // folio_mode, billing_tier, and portal flags.
    let instance_config = LocalResource::new(move || {
        let id_str = instance_id_str();
        async move {
            match uuid::Uuid::parse_str(&id_str) {
                Ok(id) => get_public_config(id).await.ok(),
                Err(_) => None,
            }
        }
    });

    view! {
        <div class="w-full p-6">
            <Suspense fallback=move || view! {
                <div class="flex items-center justify-center h-64">
                    <div class="text-on-surface-variant text-sm animate-pulse">"Loading instance…"</div>
                </div>
            }>
                {move || {
                    let cfg_opt = instance_config.get().flatten();
                    match cfg_opt {
                        None => view! {
                            <div class="flex items-center justify-center h-64">
                                <div class="text-error text-sm">
                                    "Instance not found or still loading."
                                </div>
                            </div>
                        }.into_any(),
                        Some(cfg) => {
                            // Dispatch to the correct type-specific component.
                            // Use exact string matching — never `contains()`.
                            match cfg.app_slug.as_str() {
                                "property_management" => view! {
                                    <FolioInstance cfg=cfg />
                                }.into_any(),
                                "anchor" => view! {
                                    <AnchorInstance cfg=cfg />
                                }.into_any(),
                                "network_instance" => view! {
                                    <NetworkInstance cfg=cfg />
                                }.into_any(),
                                other => view! {
                                    // Generic fallback for unknown or future app types.
                                    // Shows identity card so ops can still inspect the instance.
                                    <div class="w-full space-y-6">
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
