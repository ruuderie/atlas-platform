use crate::auth::{ServerFnError, SessionInfo};
use leptos::prelude::*;

/// Owner portal dashboard — read-only view of the beneficial owner's portfolio.
/// Owners cannot create, edit, or delete any resource.
#[component]
pub fn OwnerDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || {
        session
            .get()
            .and_then(|r| r.ok())
            .and_then(|s| s.display_name)
            .unwrap_or_else(|| "there".into())
    };

    view! {
        <div class="page-header">
            <h1 class="page-title">{move || format!("Owner Portal - {}", name())}</h1>
            <p class="page-subtitle">
                "Read-only view of your property portfolio. Contact your property manager to make changes."
            </p>
        </div>
        <div class="stat-grid">
            <StatCard label="Properties" value="-" icon="🏠" href="/o/properties"/>
        </div>
        <div style="margin-top:20px;">
            <h2 style="font-size:18px;font-weight:700;margin:0 0 6px;">"Invite an owner"</h2>
            <p style="font-size:14px;color:#64748b;margin:0 0 14px;line-height:1.5;max-width:560px;">
                "Share this portal with fellow investors, or introduce Folio to a self-managed landlord in your circle."
            </p>
            {
                use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                view! {
                    <NetworkInvitePanel
                        actor_role="owner"
                        preferred_slug="owner_invite_peers"
                        angles=vec![
                            AngleCard {
                                icon: "star",
                                title: "Other managed owners",
                                body: "Fellow investors get the same visibility into statements and approvals.",
                                        benefit_icon: None,
                                        benefit_label: None,
                                    },
                            AngleCard {
                                icon: "apartment",
                                title: "Self-managed landlords",
                                body: "Share Folio with landlords who still track rent in spreadsheets.",
                                        benefit_icon: None,
                                        benefit_label: None,
                                    },
                        ]
                        allow_multi=false
                        send_label="Send invite".to_string()
                        show_history=true
                    />
                }
            }
        </div>
    }
}

#[component]
fn StatCard(
    label: &'static str,
    value: &'static str,
    icon: &'static str,
    href: &'static str,
) -> impl IntoView {
    view! {
        <a href=href class="stat-card stat-card--link">
            <span class="stat-icon">{icon}</span>
            <div class="stat-body">
                <p class="stat-label">{label}</p>
                <p class="stat-value">{value}</p>
            </div>
        </a>
    }
}
