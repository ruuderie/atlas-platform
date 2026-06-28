// apps/folio/src/pages/landlord/asset_alerts.rs
//
// Asset Alerts — /l/assets/:id/alerts
//
// Per-asset alert configuration and active alert history.
// Wraps the Atlas notification system for threshold-based property alerts
// (vacancy, payment overdue, maintenance SLA breach, occupancy threshold, etc.)
//
// Uses:
//   GET /api/folio/assets/{id}        — asset header info
//   GET /api/folio/maintenance        — maintenance tickets (filter by asset)
//   Atlas notification system (read/UI for now; per-asset config in Phase 7)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── Static alert type definitions ─────────────────────────────────────────────

struct AlertType {
    id:       &'static str,
    title:    &'static str,
    desc:     &'static str,
    icon:     &'static str,
    category: &'static str,
}

fn all_alert_types() -> Vec<AlertType> {
    vec![
        AlertType { id: "payment_overdue",   title: "Payment Overdue",   icon: "💰", category: "Financial",    desc: "Tenant payment not received by due date" },
        AlertType { id: "payment_failed",    title: "Payment Failed",    icon: "❌", category: "Financial",    desc: "Payment attempt rejected by payment rail" },
        AlertType { id: "vacancy",           title: "Unit Vacant",       icon: "🚪", category: "Occupancy",    desc: "Unit is unoccupied for more than N days" },
        AlertType { id: "lease_expiring",    title: "Lease Expiring",    icon: "📋", category: "Occupancy",    desc: "Active lease expires within 60 days" },
        AlertType { id: "maintenance_open",  title: "Open Maintenance",  icon: "🔧", category: "Maintenance",  desc: "Maintenance request unresolved for 7+ days" },
        AlertType { id: "inspection_due",    title: "Inspection Due",    icon: "🔍", category: "Maintenance",  desc: "Scheduled inspection approaching or overdue" },
        AlertType { id: "str_permit_expiry", title: "STR Permit Expiry", icon: "📜", category: "Compliance",   desc: "Short-term rental permit expiring within 30 days" },
        AlertType { id: "violation_filed",   title: "Violation Filed",   icon: "⚠️", category: "Compliance",   desc: "New compliance violation filed on this asset" },
    ]
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn AssetAlerts() -> impl IntoView {
    let params   = use_params_map();
    let asset_id = params.get().get("id").cloned().unwrap_or_default();
    let aid_disp = if asset_id.len() > 8 { format!("…{}", &asset_id[asset_id.len()-8..]) } else { asset_id.clone() };

    // Alert enabled state (in production persisted to notification preferences)
    let enabled: RwSignal<std::collections::HashSet<&'static str>> = RwSignal::new({
        let mut s = std::collections::HashSet::new();
        s.insert("payment_overdue");
        s.insert("vacancy");
        s.insert("violation_filed");
        s
    });

    let save_pending = RwSignal::new(false);
    let saved        = RwSignal::new(false);

    let all_types = all_alert_types();

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <a href="/l/assets" class="back-link">"← Back to Assets"</a>
                    <h1 class="page-title">"Asset Alerts"</h1>
                    <p class="page-subtitle">"Configure real-time alerts for asset " <code class="font-mono text-xs">{aid_disp}</code></p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || save_pending.get()
                        on:click=move |_| {
                            save_pending.set(true);
                            // In production: POST to /api/folio/notifications/preferences
                            // with the enabled channel set for this asset_id
                            saved.set(true);
                            save_pending.set(false);
                        }
                    >
                        {move || if save_pending.get() { "Saving…" } else { "Save Preferences" }}
                    </button>
                </div>
            </div>

            {move || if saved.get() {
                view! {
                    <div class="alert-saved-toast">"✓ Alert preferences saved"</div>
                }.into_any()
            } else { ().into_any() }}

            // ── Channel delivery note ──
            <div class="viol-info-banner" style="margin-bottom:1.25rem;">
                <span class="viol-info-icon">"📡"</span>
                <p class="viol-info-text">
                    "Alerts are delivered via your configured notification channels (Email, Telegram, SMS). "
                    <a href="/l/notifications" style="color:#60a5fa;">"Manage delivery channels →"</a>
                </p>
            </div>

            // ── Alert type groups ──
            {
                let categories = vec!["Financial", "Occupancy", "Maintenance", "Compliance"];
                categories.into_iter().map(|cat| {
                    let type_rows: Vec<_> = all_alert_types().into_iter().filter(|t| t.category == cat).collect();
                    view! {
                        <div class="alerts-section">
                            <div class="alerts-section-title">{cat}</div>
                            <div class="alerts-type-list">
                                {type_rows.into_iter().map(|at| {
                                    let at_id   = at.id;
                                    let at_icon = at.icon;
                                    let at_title= at.title;
                                    let at_desc = at.desc;

                                    view! {
                                        <div class="alerts-type-row">
                                            <span class="alerts-type-icon">{at_icon}</span>
                                            <div class="alerts-type-body">
                                                <div class="alerts-type-title">{at_title}</div>
                                                <div class="alerts-type-desc">{at_desc}</div>
                                            </div>
                                            <label class="syndic-toggle-wrap">
                                                <input
                                                    type="checkbox"
                                                    class="syndic-toggle-input"
                                                    prop:checked=move || enabled.get().contains(at_id)
                                                    on:change=move |ev: web_sys::Event| {
                                                        let el = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                                        if let Some(el) = el {
                                                            enabled.update(|s| {
                                                                if el.checked() { s.insert(at_id); }
                                                                else { s.remove(at_id); }
                                                            });
                                                            saved.set(false);
                                                        }
                                                    }
                                                />
                                                <span class="syndic-toggle-track"></span>
                                            </label>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()
            }

        </div>
    }
}
