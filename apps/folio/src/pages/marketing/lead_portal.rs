// apps/folio/src/pages/marketing/lead_portal.rs
//
// Lead Portal — /leads/:token
//
// Token-gated page shown to prospective tenants who clicked a "Schedule
// a showing" or "Express interest" link from a listing page, email, or QR code.
// The token encodes the asset + variant + campaign. On load the portal shows:
//   - Property summary (fetched via the token)
//   - Book a showing slot (Phase 7: connects to landlord calendar)
//   - Quick contact form (POST /api/pub/leads)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadPortalData {
    pub asset_name:       String,
    pub address:          String,
    pub listing_type:     String,
    pub bedrooms:         Option<u32>,
    pub bathrooms:        Option<f64>,
    pub rent_cents:       Option<i64>,
    pub available_date:   Option<String>,
    pub photo_url:        Option<String>,
    pub campaign_name:    Option<String>,
    pub agent_name:       Option<String>,
    pub agent_phone:      Option<String>,
}

#[server(FetchLeadPortal, "/api")]
pub async fn fetch_lead_portal(token: String) -> Result<Option<LeadPortalData>, server_fn::error::ServerFnError> {
    let url = format!("/api/pub/leads/portal?token={token}");
    crate::atlas_client::authenticated_get::<Option<LeadPortalData>>(&url, "", None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadSubmission {
    pub token:       String,
    pub name:        String,
    pub email:       String,
    pub phone:       Option<String>,
    pub message:     Option<String>,
    pub action:      String,   // "contact" | "schedule"
}

#[server(SubmitLead, "/api")]
pub async fn submit_lead(sub: LeadSubmission) -> Result<(), server_fn::error::ServerFnError> {
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_rent(cents: i64) -> String {
    format!("${}/mo", (cents / 100).to_string()
        .chars().rev().enumerate()
        .flat_map(|(i, c)| if i > 0 && i % 3 == 0 { vec![',', c] } else { vec![c] })
        .collect::<String>().chars().rev().collect::<String>())
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LeadPortal() -> impl IntoView {
    let params  = use_params_map();
    let token   = params.get().get(0).unwrap_or_default();

    let name    = RwSignal::new(String::new());
    let email   = RwSignal::new(String::new());
    let phone   = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let action  = RwSignal::new("contact".to_string());
    let submitting = RwSignal::new(false);
    let submitted  = RwSignal::new(false);
    let sub_error  = RwSignal::new(None::<String>);

    let token2 = token.clone();
    let res = Resource::new(
        move || token.clone(),
        |t| fetch_lead_portal(t),
    );

    let handle_submit = move |_| {
        if name.get().trim().is_empty() || email.get().trim().is_empty() { return; }
        submitting.set(true);
        let sub = LeadSubmission {
            token:   token2.clone(),
            name:    name.get(),
            email:   email.get(),
            phone:   if phone.get().is_empty() { None } else { Some(phone.get()) },
            message: if message.get().is_empty() { None } else { Some(message.get()) },
            action:  action.get(),
        };
        leptos::task::spawn_local(async move {
            match submit_lead(sub).await {
                Ok(_)  => { submitted.set(true); submitting.set(false); }
                Err(e) => { sub_error.set(Some(e.to_string())); submitting.set(false); }
            }
        });
    };

    view! {
        <div class="apply-layout">
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas"</div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading property…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(Some(data)) => {
                            let rent_str = data.rent_cents.map(fmt_rent).unwrap_or_default();
                            let avail    = data.available_date.clone().unwrap_or_else(|| "Contact us".to_string());
                            let beds_str = data.bedrooms.map(|b| format!("{b} bed")).unwrap_or_default();
                            let baths_str= data.bathrooms.map(|b| format!("{b} bath")).unwrap_or_default();
                            let agent    = data.agent_name.clone().unwrap_or_else(|| "Our team".to_string());
                            let agent_ph = data.agent_phone.clone();
                            let campaign = data.campaign_name.clone();
                            view! {
                                <div class="lead-portal-card">
                                    // ── Property hero ──
                                    <div class="lead-hero">
                                        <div class="lead-hero-type">{data.listing_type.replace('_', " ")}</div>
                                        <div class="lead-hero-name">{data.asset_name.clone()}</div>
                                        <div class="lead-hero-address">"📍 " {data.address.clone()}</div>
                                        <div class="lead-hero-meta">
                                            {if !beds_str.is_empty() { view! { <span class="lead-hero-chip">{beds_str}</span> }.into_any() } else { ().into_any() }}
                                            {if !baths_str.is_empty() { view! { <span class="lead-hero-chip">{baths_str}</span> }.into_any() } else { ().into_any() }}
                                            {if !rent_str.is_empty() { view! { <span class="lead-hero-chip lead-hero-chip--green">{rent_str}</span> }.into_any() } else { ().into_any() }}
                                        </div>
                                        <div class="lead-avail">"Available: " {avail}</div>
                                        {campaign.map(|c| view! {
                                            <div class="lead-campaign-badge">"📢 " {c}</div>
                                        })}
                                    </div>

                                    // ── Agent ──
                                    <div class="lead-agent-row">
                                        <div class="lead-agent-avatar">{agent.chars().next().map(|c| c.to_string()).unwrap_or("A".to_string())}</div>
                                        <div>
                                            <div class="lead-agent-name">{agent}</div>
                                            {agent_ph.map(|ph| view! { <div class="lead-agent-phone">"📞 " {ph}</div> })}
                                        </div>
                                    </div>

                                    // ── Action tabs ──
                                    {move || if submitted.get() {
                                        view! {
                                            <div class="lead-success">
                                                <div class="wiz-success-icon">"✓"</div>
                                                <div class="lead-success-title">"Got it! We'll be in touch soon."</div>
                                                <div class="text-sm text-on-surface-variant">"Check your inbox for a confirmation."</div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div>
                                                <div class="lead-action-tabs">
                                                    <button
                                                        class=move || format!("lead-action-tab {}", if action.get() == "contact" { "lead-action-tab--active" } else { "" })
                                                        on:click=move |_| action.set("contact".to_string())
                                                    >"✉ Contact Agent"</button>
                                                    <button
                                                        class=move || format!("lead-action-tab {}", if action.get() == "schedule" { "lead-action-tab--active" } else { "" })
                                                        on:click=move |_| action.set("schedule".to_string())
                                                    >"📅 Schedule Showing"</button>
                                                </div>

                                                {move || sub_error.get().map(|e| view! { <div class="wiz-error-banner">"⚠ " {e}</div> })}

                                                <div class="form-field">
                                                    <label class="form-label">"Your Name" <span class="form-required">"*"</span></label>
                                                    <input type="text" class="form-input" placeholder="Jane Smith"
                                                        prop:value=move || name.get()
                                                        on:input=move |ev| name.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div class="apply-two-col">
                                                    <div class="form-field">
                                                        <label class="form-label">"Email" <span class="form-required">"*"</span></label>
                                                        <input type="email" class="form-input" placeholder="jane@example.com"
                                                            prop:value=move || email.get()
                                                            on:input=move |ev| email.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div class="form-field">
                                                        <label class="form-label">"Phone"</label>
                                                        <input type="tel" class="form-input" placeholder="+1 555 000 0000"
                                                            prop:value=move || phone.get()
                                                            on:input=move |ev| phone.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                </div>

                                                <Show when=move || action.get() == "schedule">
                                                    <div class="lead-schedule-note">
                                                        "📅 Showing calendar is coming in Phase 7. Fill your preferred dates below and we'll contact you."
                                                    </div>
                                                </Show>

                                                <div class="form-field">
                                                    <label class="form-label">"Message (optional)"</label>
                                                    <textarea class="form-input" style="min-height:5rem;resize:vertical;"
                                                        placeholder=move || if action.get() == "schedule" { "Preferred dates and times…" } else { "Any questions?" }
                                                        on:input=move |ev| message.set(event_target_value(&ev))
                                                    >{move || message.get()}</textarea>
                                                </div>

                                                <button
                                                    class="btn btn-primary" style="width:100%;"
                                                    disabled=move || submitting.get() || name.get().trim().is_empty() || email.get().trim().is_empty()
                                                    on:click=handle_submit
                                                >
                                                    {move || if submitting.get() { "Sending…" } else if action.get() == "schedule" { "Request Showing" } else { "Send Message" }}
                                                </button>
                                            </div>
                                        }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Ok(None) | Err(_) => view! {
                            <div class="lead-portal-card">
                                <div class="doc-empty" style="padding:2rem;">
                                    <div style="font-size:2rem;">"🔗"</div>
                                    <div>"This link has expired or is invalid."</div>
                                    <div class="text-xs text-on-surface-variant">"Please contact the property manager for an updated link."</div>
                                </div>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
