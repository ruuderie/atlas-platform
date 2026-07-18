//! Property Owner Lite — Dashboard — `/po`
//! Est. value from first property's value-history; honest empty when none.

use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pages::landlord::assets::{list_assets, AssetSummary};
use crate::pages::property_owner::property_value::{fetch_value_history, ValueHistoryEntry};

#[component]
pub fn PropertyOwnerDashboard() -> impl IntoView {
    let summary = Resource::new(|| (), |_| async move {
        let assets = list_assets().await?;
        let property_id = assets
            .into_iter()
            .find(|a: &AssetSummary| a.parent_asset_id.is_none())
            .map(|a| a.id);
        let Some(pid) = property_id else {
            return Ok::<PoDashSummary, server_fn::error::ServerFnError>(PoDashSummary {
                property_id: None,
                latest_value_cents: None,
                change_pct: None,
                history_count: 0,
            });
        };
        let history = fetch_value_history(pid).await.unwrap_or_default();
        let latest = history.first().map(|e: &ValueHistoryEntry| e.value_cents);
        let change_pct = compute_change_pct(&history);
        Ok(PoDashSummary {
            property_id: Some(pid),
            latest_value_cents: latest,
            change_pct,
            history_count: history.len(),
        })
    });

    view! {
        <div class="page-header">
            <h1 class="page-title">"My Property"</h1>
            <p class="page-subtitle">"Track your home's value and connect with trusted vendors."</p>
        </div>

        <div class="upgrade-banner">
            <div class="upgrade-banner__icon">
                <span class="ms msf">"rocket_launch"</span>
            </div>
            <div class="upgrade-banner__body">
                <p class="upgrade-banner__title">"Unlock the full landlord suite"</p>
                <p class="upgrade-banner__sub">
                    "Add tenants, manage leases, collect rent, and automate maintenance, "
                    "all in one place. Upgrade to Landlord for $X/mo."
                </p>
            </div>
            <a href="/po/upgrade" class="btn btn-primary btn-sm">
                "Upgrade →"
            </a>
        </div>

        <Suspense fallback=|| view! {
            <div class="stat-grid stat-grid--3">
                <div class="stat-card"><div class="stat-body"><p class="stat-label">"Est. Value"</p><p class="stat-value">"…"</p></div></div>
            </div>
        }>
            {move || match summary.get() {
                Some(Err(e)) => view! {
                    <div class="folio-empty">
                        <p class="folio-empty__heading">"Could not load summary"</p>
                        <p class="folio-empty__sub">{e.to_string()}</p>
                    </div>
                }.into_any(),
                Some(Ok(s)) => {
                    let value_label = s
                        .latest_value_cents
                        .map(|c| format!("${:.0}", c as f64 / 100.0))
                        .unwrap_or_else(|| "—".into());
                    let change_label = s
                        .change_pct
                        .map(|p| format!("{p:+.1}%"))
                        .unwrap_or_else(|| "—".into());
                    let has_history = s.history_count > 0;
                    view! {
                        <div class="stat-grid stat-grid--3">
                            <div class="stat-card">
                                <span class="stat-icon ms msf">"home"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Est. Value"</p>
                                    <p class="stat-value">{value_label}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <span class="stat-icon ms msf">"trending_up"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Since earliest log"</p>
                                    <p class="stat-value">{change_label}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <span class="stat-icon ms msf">"timeline"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Valuations logged"</p>
                                    <p class="stat-value">{s.history_count.to_string()}</p>
                                </div>
                            </div>
                        </div>

                        <div class="quick-actions">
                            <A href="/po/value" attr:class="quick-action-card">
                                <span class="ms msf quick-action-card__icon">"add_chart"</span>
                                <div class="quick-action-card__body">
                                    <p class="quick-action-card__title">"Log a Valuation"</p>
                                    <p class="quick-action-card__sub">"Record Zillow, appraisal, or your own estimate"</p>
                                </div>
                                <span class="ms">"chevron_right"</span>
                            </A>
                            <A href="/po/find-vendor" attr:class="quick-action-card">
                                <span class="ms msf quick-action-card__icon">"handyman"</span>
                                <div class="quick-action-card__body">
                                    <p class="quick-action-card__title">"Find a Vendor"</p>
                                    <p class="quick-action-card__sub">"Browse and request service from trusted contractors"</p>
                                </div>
                                <span class="ms">"chevron_right"</span>
                            </A>
                        </div>

                        <div class="section-header" style="margin-top:28px">
                            <h2 class="section-title">"Grow your network"</h2>
                        </div>
                        <p style="font-size:14px;color:#64748b;margin:0 0 14px;line-height:1.5;max-width:560px;">
                            "Invite other owners and landlords you know, and vendors you trust. Optional anytime."
                        </p>
                        {
                            use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                            view! {
                                <NetworkInvitePanel
                                    actor_role="property_owner"
                                    preferred_slug="property_owner_invite_peers"
                                    angles=vec![
                                        AngleCard {
                                            icon: "apartment",
                                            title: "Other owners & landlords",
                                            body: "Share Folio with owners in your circle so they can track value and vendors the same way you do.",
                                            benefit_icon: None,
                                            benefit_label: None,
                                        },
                                        AngleCard {
                                            icon: "handyman",
                                            title: "Vendors you recommend",
                                            body: "Invite a contractor you trust. The next job stays on Folio with shared history and reviews.",
                                            benefit_icon: None,
                                            benefit_label: None,
                                        },
                                    ]
                                    show_history=true
                                />
                            }
                        }

                        <div class="section-header" style="margin-top:28px">
                            <h2 class="section-title">"Value History"</h2>
                            <A href="/po/value" attr:class="section-link">"View all →"</A>
                        </div>
                        {if has_history {
                            view! {
                                <div class="card">
                                    <p class="folio-empty__sub" style="padding:1rem;">
                                        {format!(
                                            "{} valuation(s) on file. Open Property Value for the full timeline.",
                                            s.history_count
                                        )}
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="folio-empty">
                                    <p class="folio-empty__heading">"No valuations yet"</p>
                                    <p class="folio-empty__sub">
                                        {if s.property_id.is_some() {
                                            "Log a valuation to start tracking equity over time."
                                        } else {
                                            "Complete onboarding to attach a property, then log valuations."
                                        }}
                                    </p>
                                    <A href="/po/value" attr:class="btn btn-secondary btn-sm">"Log first valuation"</A>
                                </div>
                            }.into_any()
                        }}
                    }.into_any()
                }
                None => view! { <div /> }.into_any(),
            }}
        </Suspense>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PoDashSummary {
    property_id: Option<Uuid>,
    latest_value_cents: Option<i64>,
    change_pct: Option<f64>,
    history_count: usize,
}

fn compute_change_pct(history: &[ValueHistoryEntry]) -> Option<f64> {
    if history.len() < 2 {
        return None;
    }
    let latest = history.first()?.value_cents as f64;
    let earliest = history.last()?.value_cents as f64;
    if earliest == 0.0 {
        return None;
    }
    Some(((latest - earliest) / earliest) * 100.0)
}
