// apps/folio/src/pages/marketing/inquiry_confirm.rs
//
// Inquiry Confirmation — /inquiry/thanks
//
// Post-form thank-you page shown after a prospective tenant submits an inquiry
// from a Network Instance listing, the lead portal, or an embedded inquiry form.
// No auth, no server call — pure SSR confirmation with optional lead-ref from
// the `ref` query param. Phase 7 may add a lightweight status poll.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn InquiryConfirm() -> impl IntoView {
    let query = use_query_map();
    let lead_ref = query.get().get("ref");
    let name = query.get().get("name");
    let property = query.get().get("property");
    let action = query
        .get()
        .get("action")
        .unwrap_or_else(|| "contact".to_string());

    let (title, subtitle, icon) = if action == "schedule" {
        (
            "Showing Requested!",
            "We've received your showing request and will confirm a time with you shortly.",
            "📅",
        )
    } else {
        (
            "Message Sent!",
            "Your inquiry has been received. Expect a response within 1 business day.",
            "✉",
        )
    };

    view! {
        <div class="apply-layout">
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas"</div>
            </div>

            <div class="lead-portal-card" style="max-width:28rem;text-align:center;">
                <div class="wiz-success-icon" style="font-size:3.5rem;margin-bottom:.5rem;">{icon}</div>
                <div class="wiz-success-title">{title}</div>
                <div class="wiz-success-sub">{subtitle}</div>

                {name.map(|n| view! {
                    <div class="inquiry-confirm-name">"Hi " {n} "! We'll be in touch."</div>
                })}

                {property.map(|p| view! {
                    <div class="inquiry-confirm-property">
                        <span class="lead-hero-chip">"🏠 " {p}</span>
                    </div>
                })}

                {lead_ref.map(|r| view! {
                    <div class="inquiry-confirm-ref">"Reference: " <code>{r}</code></div>
                })}

                <div class="inquiry-confirm-next">
                    <div class="inquiry-confirm-next-title">"What happens next?"</div>
                    <div class="inquiry-confirm-step">
                        <span class="inquiry-confirm-step-num">"1"</span>
                        <span>"The property manager reviews your inquiry"</span>
                    </div>
                    <div class="inquiry-confirm-step">
                        <span class="inquiry-confirm-step-num">"2"</span>
                        <span>{if action == "schedule" { "A showing slot will be confirmed by email" } else { "You'll receive a reply within 1 business day" }}</span>
                    </div>
                    <div class="inquiry-confirm-step">
                        <span class="inquiry-confirm-step-num">"3"</span>
                        <span>"Apply online when you're ready — it takes less than 5 minutes"</span>
                    </div>
                </div>

                <div style="margin-top:1.5rem;display:flex;gap:.75rem;justify-content:center;flex-wrap:wrap;">
                    <button
                        class="btn btn-ghost btn-sm"
                        on:click=move |_| {
                            let _ = web_sys::window().and_then(|w| w.history().ok()).map(|h| h.back());
                        }
                    >"← Back to Listing"</button>
                </div>
            </div>
        </div>
    }
}
