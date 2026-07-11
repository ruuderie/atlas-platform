use leptos::prelude::*;

use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
use crate::pages::pmc::client_detail::{fetch_pmc_clients, ClientSummary};

/// PMC Client Book — managed client accounts + G-36 owner invites.
#[component]
pub fn ClientBook() -> impl IntoView {
    let clients_res = Resource::new(|| (), |_| fetch_pmc_clients());
    let search = RwSignal::new(String::new());

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

        <div class="section-header" style="margin-top:8px;margin-bottom:12px;display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap;">
            <h2 class="section-title" style="margin:0;font-size:16px;font-weight:700;">"Managed clients"</h2>
            <input
                class="wiz-inp"
                type="search"
                placeholder="Search clients…"
                style="max-width:240px;margin:0;"
                prop:value=move || search.get()
                on:input=move |e| search.set(event_target_value(&e))
            />
        </div>

        <Suspense fallback=|| view! { <div class="empty-state"><p>"Loading clients…"</p></div> }>
            {move || clients_res.get().map(|res| match res {
                Err(e) => view! {
                    <div class="empty-state">
                        <p>{format!("Could not load clients: {e}")}</p>
                    </div>
                }.into_any(),
                Ok(clients) if clients.is_empty() => view! {
                    <div class="empty-state">
                        <p>"No client accounts yet. Create a client account from onboarding, or invite an owner above."</p>
                    </div>
                }.into_any(),
                Ok(clients) => {
                    let q = search.get().trim().to_lowercase();
                    let filtered: Vec<ClientSummary> = if q.is_empty() {
                        clients
                    } else {
                        clients.into_iter().filter(|c| {
                            c.display_name.to_lowercase().contains(&q)
                                || c.contact_email.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                || c.contact_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                        }).collect()
                    };
                    if filtered.is_empty() {
                        return view! {
                            <div class="empty-state">
                                <p>"No clients match your search."</p>
                            </div>
                        }.into_any();
                    }
                    view! {
                        <div class="pmc-stmt-list" style="margin-bottom:24px;">
                            <For
                                each=move || filtered.clone()
                                key=|c| c.account_id
                                children=move |client| {
                                    let cid = client.account_id;
                                    let href = format!("/pmc/clients/{cid}");
                                    let name = client.display_name.clone();
                                    let email = client.contact_email.clone().unwrap_or_default();
                                    let contact = client.contact_name.clone().unwrap_or_default();
                                    let props = client.property_count.unwrap_or(0);
                                    let units = client.unit_count.unwrap_or(0);
                                    let leases = client.active_lease_count.unwrap_or(0);
                                    let occ = client.occupancy_pct
                                        .map(|p| format!("{p:.0}%"))
                                        .unwrap_or_else(|| "—".into());
                                    let initial = name.chars().next()
                                        .map(|ch| ch.to_uppercase().to_string())
                                        .unwrap_or_else(|| "?".into());
                                    view! {
                                        <a href=href class="pmc-stmt-row" style="text-decoration:none;color:inherit;">
                                            <div class="pmc-client-avatar" style="width:2rem;height:2rem;font-size:.9rem;flex-shrink:0;">
                                                {initial}
                                            </div>
                                            <div class="pmc-stmt-info">
                                                <div class="pmc-stmt-name">{name}</div>
                                                <div class="pmc-stmt-email">
                                                    {if !contact.is_empty() && !email.is_empty() {
                                                        format!("{contact} · {email}")
                                                    } else if !email.is_empty() {
                                                        email
                                                    } else if !contact.is_empty() {
                                                        contact
                                                    } else {
                                                        "No contact on file".into()
                                                    }}
                                                </div>
                                            </div>
                                            <div class="pmc-stmt-metrics">
                                                <span>{format!("{props} props")}</span>
                                                <span>{format!("{units} units")}</span>
                                                <span>{format!("{leases} leases")}</span>
                                                <span>{format!("{occ} occ")}</span>
                                            </div>
                                            <span class="ms" style="color:#94a3b8;">"chevron_right"</span>
                                        </a>
                                    }
                                }
                            />
                        </div>
                    }.into_any()
                }
            })}
        </Suspense>
    }
}
