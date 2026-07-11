// apps/folio/src/pages/vendor/network_profile.rs
//
// Network Profile page — /v/profile
//
// Allows a vendor to view and manage their business profile, listing information,
// and payment configurations.
//
// Data sources:
//   - GET   /api/folio/vendor/profile
//   - PATCH /api/folio/vendor/profile

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorProfileDetail {
    pub id:                      uuid::Uuid,
    pub business_name:           Option<String>,
    pub preferred_payment_rail:  Option<String>,
    pub btc_wallet_address:      Option<String>,
    pub stripe_connect_id:       Option<String>,
    pub is_insured:              bool,
    pub is_bonded:               bool,
    pub is_marketplace_visible:  bool,
    pub marketplace_bio:         Option<String>,
    pub marketplace_trade_types: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProfileInput {
    pub business_name:           Option<String>,
    pub preferred_payment_rail:  Option<String>,
    pub btc_wallet_address:      Option<String>,
    pub stripe_connect_id:       Option<String>,
    pub is_insured:              Option<bool>,
    pub is_bonded:               Option<bool>,
    pub is_marketplace_visible:  Option<bool>,
    pub marketplace_bio:         Option<String>,
    pub marketplace_trade_types: Option<Vec<String>>,
}

#[component]
pub fn VendorNetworkProfile() -> impl IntoView {
    // ── Resources ────────────────────────────────────────────────────────────
    let profile = Resource::new(|| (), |_| async move { get_vendor_profile().await });

    // ── Form State Signals ───────────────────────────────────────────────────
    let (business_name, set_business_name) = signal(String::new());
    let (bio, set_bio) = signal(String::new());
    let (is_insured, set_is_insured) = signal(false);
    let (is_bonded, set_is_bonded) = signal(false);
    let (is_visible, set_is_visible) = signal(false);
    let (payment_rail, set_payment_rail) = signal(String::new());
    let (btc_address, set_btc_address) = signal(String::new());
    let (stripe_id, set_stripe_id) = signal(String::new());
    let (selected_trades, set_selected_trades) = signal(Vec::<String>::new());

    // ── Notification State ───────────────────────────────────────────────────
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (is_saving, set_is_saving) = signal(false);

    // Trade type list matching backend enums/slugs
    let available_trades = vec![
        ("plumbing", "Plumbing"),
        ("electrical", "Electrical"),
        ("hvac", "HVAC"),
        ("structural", "Structural"),
        ("pest", "Pest Control"),
        ("appliance", "Appliance Repair"),
        ("roofing", "Roofing"),
        ("general", "General Maintenance"),
    ];

    // Effect to populate form inputs when profile resource loads
    Effect::new(move |_| {
        if let Some(Ok(p)) = profile.get() {
            set_business_name.set(p.business_name.unwrap_or_default());
            set_bio.set(p.marketplace_bio.unwrap_or_default());
            set_is_insured.set(p.is_insured);
            set_is_bonded.set(p.is_bonded);
            set_is_visible.set(p.is_marketplace_visible);
            set_payment_rail.set(p.preferred_payment_rail.unwrap_or_else(|| "stripe".to_string()));
            set_btc_address.set(p.btc_wallet_address.unwrap_or_default());
            set_stripe_id.set(p.stripe_connect_id.unwrap_or_default());
            set_selected_trades.set(p.marketplace_trade_types);
        }
    });

    // ── Actions ──────────────────────────────────────────────────────────────
    let save_profile = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_saving.set(true);
        set_success_msg.set(None);
        set_error_msg.set(None);

        let input = UpdateProfileInput {
            business_name:           Some(business_name.get()),
            preferred_payment_rail:  Some(payment_rail.get()),
            btc_wallet_address:      Some(btc_address.get()),
            stripe_connect_id:       Some(stripe_id.get()),
            is_insured:              Some(is_insured.get()),
            is_bonded:               Some(is_bonded.get()),
            is_marketplace_visible:  Some(is_visible.get()),
            marketplace_bio:         Some(bio.get()),
            marketplace_trade_types: Some(selected_trades.get()),
        };

        spawn_local(async move {
            match update_vendor_profile(input).await {
                Ok(_) => {
                    set_success_msg.set(Some("Profile saved successfully!".to_string()));
                    profile.refetch();
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to save profile: {e}")));
                }
            }
            set_is_saving.set(false);
        });
    };

    let toggle_trade = move |slug: &'static str| {
        let current = selected_trades.get();
        if current.contains(&slug.to_string()) {
            set_selected_trades.set(current.into_iter().filter(|s| s != slug).collect());
        } else {
            let mut updated = current;
            updated.push(slug.to_string());
            set_selected_trades.set(updated);
        }
    };

    view! {
        <div class="vp-page">
            <div class="vp-header">
                <h1 class="vp-title">"Network Profile"</h1>
                <p class="vp-subtitle">"Manage your service listing, insurance status, and payout credentials. Invite clients and contractors below."</p>
            </div>

            <div style="margin-bottom:20px;">
                <h3 class="vp-section-title" style="margin-bottom:6px;">"Grow your network"</h3>
                <p style="font-size:14px;color:#64748b;margin:0 0 14px;line-height:1.5;max-width:560px;">
                    "Invite past clients so you can log jobs and reviews, and invite contractors you already collaborate with."
                </p>
                {
                    use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                    view! {
                        <NetworkInvitePanel
                            actor_role="vendor"
                            preferred_slug="vendor_invite_clients"
                            angles=vec![
                                AngleCard {
                                    icon: "home_work",
                                    title: "Past clients & owners",
                                    body: "Invite an owner from a recent job to log the work, request a review, and stay visible for the next dispatch.",
                                },
                                AngleCard {
                                    icon: "engineering",
                                    title: "Other contractors",
                                    body: "Invite trades you trust. When a job needs a second specialty, refer each other inside Folio.",
                                },
                            ]
                            show_stats=true
                            show_history=true
                        />
                    }
                }
            </div>

            <Suspense fallback=|| view! { <div class="vp-loading-skeleton" /> }>
                {move || profile.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="vp-error-state">
                            <span class="material-symbols-outlined vp-error-icon">"error"</span>
                            <p class="vp-error-text">{format!("Could not load profile: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(_) => view! {
                        <form class="vp-form" on:submit=save_profile>
                            // ── Notifications ─────────────────────────────────
                            {move || success_msg.get().map(|msg| view! {
                                <div class="vp-alert vp-alert--success">
                                    <span class="material-symbols-outlined">"check_circle"</span>
                                    <span>{msg}</span>
                                </div>
                            })}
                            {move || error_msg.get().map(|msg| view! {
                                <div class="vp-alert vp-alert--error">
                                    <span class="material-symbols-outlined">"error"</span>
                                    <span>{msg}</span>
                                </div>
                            })}

                            // ── Core Business Info ────────────────────────────
                            <div class="vp-section">
                                <h3 class="vp-section-title">"Business Identity"</h3>
                                <div class="vp-field">
                                    <label class="vp-label" for="biz-name">"Business Name"</label>
                                    <input
                                        id="biz-name"
                                        class="vp-input"
                                        type="text"
                                        required
                                        value=business_name
                                        on:input=move |e| set_business_name.set(event_target_value(&e))
                                    />
                                </div>
                                <div class="vp-field-row">
                                    <label class="vp-checkbox-label">
                                        <input
                                            class="vp-checkbox"
                                            type="checkbox"
                                            checked=is_insured
                                            on:change=move |e| set_is_insured.set(event_target_checked(&e))
                                        />
                                        <span>"Insured"</span>
                                    </label>
                                    <label class="vp-checkbox-label">
                                        <input
                                            class="vp-checkbox"
                                            type="checkbox"
                                            checked=is_bonded
                                            on:change=move |e| set_is_bonded.set(event_target_checked(&e))
                                        />
                                        <span>"Bonded"</span>
                                    </label>
                                </div>
                            </div>

                            // ── Marketplace Visibility & Bio ──────────────────
                            <div class="vp-section">
                                <h3 class="vp-section-title">"Marketplace Settings"</h3>
                                <div class="vp-field">
                                    <label class="vp-checkbox-label vp-checkbox-label--large">
                                        <input
                                            class="vp-checkbox"
                                            type="checkbox"
                                            checked=is_visible
                                            on:change=move |e| set_is_visible.set(event_target_checked(&e))
                                        />
                                        <div>
                                            <span class="vp-checkbox-heading">"List in Marketplace"</span>
                                            <span class="vp-checkbox-sub">"Make your profile visible to landlords across the network."</span>
                                        </div>
                                    </label>
                                </div>
                                <div class="vp-field">
                                    <label class="vp-label" for="biz-bio">"Public Biography"</label>
                                    <textarea
                                        id="biz-bio"
                                        class="vp-textarea"
                                        placeholder="Describe your specialties, service area, and experience..."
                                        rows=4
                                        maxlength=500
                                        on:input=move |e| set_bio.set(event_target_value(&e))
                                    >{move || bio.get()}</textarea>
                                    <span class="vp-field-helper">
                                        {move || format!("{}/500 characters max", bio.get().len())}
                                    </span>
                                </div>
                                <div class="vp-field">
                                    <label class="vp-label">"Advertised Trades / Specialties"</label>
                                    <div class="vp-trades-grid">
                                        {available_trades.iter().map(|&(slug, label)| {
                                            let is_selected = move || selected_trades.get().contains(&slug.to_string());
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        if is_selected() {
                                                            "vp-trade-btn vp-trade-btn--active"
                                                        } else {
                                                            "vp-trade-btn"
                                                        }
                                                    }
                                                    on:click=move |_| toggle_trade(slug)
                                                >
                                                    {label}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            </div>

                            // ── Payment Configurations ────────────────────────
                            <div class="vp-section">
                                <h3 class="vp-section-title">"Payout Settings"</h3>
                                <div class="vp-field">
                                    <label class="vp-label" for="payment-rail">"Preferred Payout Rail"</label>
                                    <select
                                        id="payment-rail"
                                        class="vp-select"
                                        on:change=move |e| set_payment_rail.set(event_target_value(&e))
                                    >
                                        <option value="stripe" selected=move || payment_rail.get() == "stripe">"Stripe Direct"</option>
                                        <option value="bitcoin" selected=move || payment_rail.get() == "bitcoin">"Bitcoin (On-chain / Lightning)"</option>
                                    </select>
                                </div>
                                {move || (payment_rail.get() == "bitcoin").then(|| view! {
                                    <div class="vp-field vp-field-fade">
                                        <label class="vp-label" for="btc-addr">"Bitcoin Wallet Address"</label>
                                        <input
                                            id="btc-addr"
                                            class="vp-input vp-input--mono"
                                            type="text"
                                            placeholder="bc1..."
                                            value=btc_address
                                            on:input=move |e| set_btc_address.set(event_target_value(&e))
                                        />
                                    </div>
                                })}
                                {move || (payment_rail.get() == "stripe").then(|| view! {
                                    <div class="vp-field vp-field-fade">
                                        <label class="vp-label" for="stripe-id">"Stripe Connect Account ID"</label>
                                        <input
                                            id="stripe-id"
                                            class="vp-input vp-input--mono"
                                            type="text"
                                            placeholder="acct_..."
                                            value=stripe_id
                                            on:input=move |e| set_stripe_id.set(event_target_value(&e))
                                        />
                                    </div>
                                })}
                            </div>

                            <button
                                type="submit"
                                class="vp-submit-btn"
                                disabled=is_saving
                            >
                                {move || if is_saving.get() { "Saving..." } else { "Save Profile" }}
                            </button>
                        </form>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

// ── Server Functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|part| {
                        part.trim()
                            .strip_prefix("atlas_session=")
                            .map(|t| t.to_string())
                    })
                })
        })
}

#[server(GetVendorProfile, "/api")]
pub async fn get_vendor_profile() -> Result<VendorProfileDetail, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<VendorProfileDetail>(
        "/api/folio/vendor/profile",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Profile fetch failed: {e}")))
}

#[server(UpdateVendorProfile, "/api")]
pub async fn update_vendor_profile(
    input: UpdateProfileInput,
) -> Result<VendorProfileDetail, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_patch::<UpdateProfileInput, VendorProfileDetail>(
        "/api/folio/vendor/profile",
        &token,
        input,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Profile save failed: {e}")))
}
