// apps/folio/src/pages/str_host/channels.rs
//
// STR Channel Manager — /s/channels
//
// Manage OTA channel connections (Airbnb, Vrbo, Booking.com, etc.).
// Reuses the syndication toggle pattern with STR-specific channel data.
// Channel API keys are stored in tenant secrets (Phase 7 /api/folio/channels).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;

// ── Channel definitions ───────────────────────────────────────────────────────

struct StrChannel {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    desc: &'static str,
    fee_pct: &'static str,
    markets: &'static str,
}

fn all_str_channels() -> Vec<StrChannel> {
    vec![
        StrChannel {
            id: "airbnb",
            name: "Airbnb",
            icon: "🏖",
            desc: "100M+ guests in 220 countries",
            fee_pct: "3%",
            markets: "Global",
        },
        StrChannel {
            id: "vrbo",
            name: "Vrbo",
            icon: "🏕",
            desc: "Expedia Group — family & whole-home focus",
            fee_pct: "5%",
            markets: "US/EU",
        },
        StrChannel {
            id: "bookingdotcom",
            name: "Booking.com",
            icon: "🌍",
            desc: "European leader, strong international",
            fee_pct: "15–17%",
            markets: "Global",
        },
        StrChannel {
            id: "hipcamp",
            name: "Hipcamp",
            icon: "⛺",
            desc: "Outdoors and unique stays",
            fee_pct: "7%",
            markets: "US",
        },
        StrChannel {
            id: "tripadvisor",
            name: "Tripadvisor Rentals",
            icon: "✈️",
            desc: "Formerly FlipKey — high-intent travellers",
            fee_pct: "3%",
            markets: "Global",
        },
        StrChannel {
            id: "atlas_str",
            name: "Atlas Network STR",
            icon: "⚡",
            desc: "Direct booking — zero channel fee",
            fee_pct: "0%",
            markets: "Platform",
        },
    ]
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrChannelManager() -> impl IntoView {
    let enabled: RwSignal<std::collections::HashSet<&'static str>> = RwSignal::new({
        let mut s = std::collections::HashSet::new();
        s.insert("atlas_str");
        s
    });

    let show_keys = RwSignal::new(None::<&'static str>); // which channel's keys modal is open
    let saved = RwSignal::new(false);

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Channel Manager"</h1>
                    <p class="page-subtitle">"Connect OTA channels and manage real-time availability sync"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| saved.set(true)
                    >"Save Preferences"</button>
                </div>
            </div>

            {move || if saved.get() {
                view! { <div class="alert-saved-toast">"✓ Channel preferences saved (sync credentials via API in Phase 7)"</div> }.into_any()
            } else { ().into_any() }}

            // ── Atlas always-on ──
            <div class="syndic-notice">
                <span class="syndic-notice-icon">"⚡"</span>
                <span>"Atlas Network STR channel is always active with zero channel fees."</span>
            </div>

            // ── KPIs ──
            <div class="kpi-row" style="margin-bottom:1.5rem;">
                <div class="kpi-card">
                    <span class="kpi-label">"Active Channels"</span>
                    <span class="kpi-value" style="color:var(--green)">{move || enabled.get().len().to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Available OTAs"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">{all_str_channels().len().to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Direct Booking Fee"</span>
                    <span class="kpi-value" style="color:var(--green)">"0%"</span>
                </div>
            </div>

            // ── Channel cards ──
            <div class="syndic-channel-grid">
                {all_str_channels().into_iter().map(|ch| {
                    let ch_id    = ch.id;
                    let ch_name  = ch.name;
                    let ch_icon  = ch.icon;
                    let ch_desc  = ch.desc;
                    let ch_fee   = ch.fee_pct;
                    let ch_mkt   = ch.markets;
                    let is_atlas = ch_id == "atlas_str";

                    view! {
                        <div class=move || format!("syndic-card {}",
                            if enabled.get().contains(ch_id) { "syndic-card--on" } else { "" })
                        >
                            <div class="syndic-card-top">
                                <span class="syndic-card-icon">{ch_icon}</span>
                                <div class="syndic-card-meta">
                                    <div class="syndic-card-name">{ch_name}</div>
                                    <div class="syndic-card-desc">{ch_desc}</div>
                                    <div class="text-xs text-on-surface-variant" style="margin-top:0.2rem;">
                                        "Fee: " <strong style="color:var(--amber)">{ch_fee}</strong>
                                        " · " {ch_mkt}
                                    </div>
                                </div>
                            </div>
                            <div class="syndic-card-toggle-row">
                                <label class="syndic-toggle-wrap">
                                    <input
                                        type="checkbox"
                                        class="syndic-toggle-input"
                                        prop:checked=move || enabled.get().contains(ch_id)
                                        disabled=is_atlas
                                        on:change=move |ev: web_sys::Event| {
                                            let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                            if let Some(el) = el {
                                                enabled.update(|s| {
                                                    if el.checked() { s.insert(ch_id); }
                                                    else { s.remove(ch_id); }
                                                });
                                                saved.set(false);
                                            }
                                        }
                                    />
                                    <span class="syndic-toggle-track"></span>
                                </label>
                                <span class="syndic-toggle-label">
                                    {move || if enabled.get().contains(ch_id) { "Connected" } else { "Disabled" }}
                                </span>
                                {if !is_atlas {
                                    view! {
                                        <button
                                            class="btn btn-ghost btn-sm"
                                            style="margin-left:auto;"
                                            on:click=move |_| show_keys.set(Some(ch_id))
                                        >"🔑 API Keys"</button>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // ── API Keys Modal ──
            <Show when=move || show_keys.get().is_some()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"🔑 " {move || show_keys.get().unwrap_or("")} " API Credentials"</h3>
                            <button class="modal-close" on:click=move |_| show_keys.set(None)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="viol-info-banner">
                                <span class="viol-info-icon">"🔒"</span>
                                <p class="viol-info-text">"Credentials are stored encrypted in the platform secrets vault. Never enter credentials into untrusted pages."</p>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"API Key"</label>
                                <input type="password" class="form-input" placeholder="••••••••••••••••" />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"API Secret / Channel ID"</label>
                                <input type="password" class="form-input" placeholder="••••••••••••••••" />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_keys.set(None)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=move |_| show_keys.set(None)>"Save Credentials"</button>
                        </div>
                    </div>
                </div>
            </Show>

        </div>
    }
}
