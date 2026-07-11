use leptos::prelude::*;

/// PMC Client Book — invite landlord clients onto Folio (G-36) and manage Owner portal access.
#[component]
pub fn ClientBook() -> impl IntoView {
    use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
    view! {
        <div class="page-header">
            <h1 class="page-title">"Client Book"</h1>
            <p class="page-subtitle">
                "Bring landlord clients onto Folio so they get an Owner portal with statements and reporting. Your PMC stays the hub."
            </p>
        </div>
        <NetworkInvitePanel
            actor_role="property_manager"
            preferred_slug="pmc_invite_clients"
            angles=vec![
                AngleCard {
                    icon: "apartment",
                    title: "Existing owner clients",
                    body: "Invite landlords you already manage. They see statements here while you keep operations centralized.",
                },
                AngleCard {
                    icon: "campaign",
                    title: "Prospects & referrals",
                    body: "When pitching a new owner, send a Folio invite instead of a PDF.",
                },
            ]
            section_title="Invite a new client".to_string()
            send_label="Send Owner Invite".to_string()
            show_note=true
            allow_multi=false
            show_stats=true
            show_history=true
        />
        <div class="empty-state" style="margin-top:8px;">
            <p>"Client list will appear here as owners join."</p>
        </div>
    }
}
