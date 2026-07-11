//! Property Owner Lite — Dashboard
//!
//! Route: GET /po
//!
//! Shows the PO Lite home screen:
//!   - Property value summary card (latest valuation from value-history)
//!   - Upgrade-to-Landlord CTA banner
//!   - Quick links: Log Value, Find Vendor, My Reviews
//!
//! Data: GET /api/folio/properties/:id/value-history (first result for "latest")

use leptos::prelude::*;

/// Property Owner Lite dashboard — value summary + upgrade CTA + quick links.
#[component]
pub fn PropertyOwnerDashboard() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"My Property"</h1>
            <p class="page-subtitle">"Track your home's value and connect with trusted vendors."</p>
        </div>

        // ── Upgrade CTA banner ────────────────────────────────────────────
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

        // ── Quick stat cards ──────────────────────────────────────────────
        <div class="stat-grid stat-grid--3">
            <div class="stat-card">
                <span class="stat-icon ms msf">"home"</span>
                <div class="stat-body">
                    <p class="stat-label">"Est. Value"</p>
                    <p class="stat-value" id="po-stat-value">"-"</p>
                </div>
            </div>
            <div class="stat-card">
                <span class="stat-icon ms msf">"trending_up"</span>
                <div class="stat-body">
                    <p class="stat-label">"Since Purchase"</p>
                    <p class="stat-value" id="po-stat-change">"-"</p>
                </div>
            </div>
            <div class="stat-card">
                <span class="stat-icon ms msf">"star"</span>
                <div class="stat-body">
                    <p class="stat-label">"Reviews Submitted"</p>
                    <p class="stat-value" id="po-stat-reviews">"0"</p>
                </div>
            </div>
        </div>

        // ── Quick actions ─────────────────────────────────────────────────
        <div class="quick-actions">
            <a href="/po/value" class="quick-action-card" id="po-action-log-value">
                <span class="ms msf quick-action-card__icon">"add_chart"</span>
                <div class="quick-action-card__body">
                    <p class="quick-action-card__title">"Log a Valuation"</p>
                    <p class="quick-action-card__sub">"Record Zillow, appraisal, or your own estimate"</p>
                </div>
                <span class="ms">"chevron_right"</span>
            </a>
            <a href="/po/find-vendor" class="quick-action-card" id="po-action-find-vendor">
                <span class="ms msf quick-action-card__icon">"handyman"</span>
                <div class="quick-action-card__body">
                    <p class="quick-action-card__title">"Find a Vendor"</p>
                    <p class="quick-action-card__sub">"Browse and request service from trusted contractors"</p>
                </div>
                <span class="ms">"chevron_right"</span>
            </a>
        </div>

        // ── G-36 Network invites ──────────────────────────────────────────
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

        // ── Value history mini chart placeholder ──────────────────────────
        <div class="section-header" style="margin-top:28px">
            <h2 class="section-title">"Value History"</h2>
            <a href="/po/value" class="section-link" id="po-link-value-history">"View all →"</a>
        </div>
        <div class="page-placeholder" id="po-value-chart-placeholder">
            <p>"Connect your property to start tracking its value over time."</p>
            <a href="/po/value" class="btn btn-secondary btn-sm" style="margin-top:12px" id="po-cta-log-first">
                "Log first valuation"
            </a>
        </div>
    }
}
