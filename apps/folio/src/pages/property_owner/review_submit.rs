//! Public review submit page — zero-auth cold traffic.
//!
//! Route: GET /review/:invite_id  (public, no session required)
//!
//! Flow:
//!   1. Load invite context: GET /api/pub/review/:invite_id
//!      → vendor name + G-27 scorecard dimensions
//!   2. OTP inline gate: user enters email → receives 6-digit code → verifies
//!   3. Review form: one input per dimension (scale_type-matched UI)
//!      rating      → segmented 1–10 (or scale_min..scale_max)
//!      boolean     → Yes/No toggle pair
//!      absolute    → numeric input
//!   4. Submit: POST /api/pub/review/:invite_id/submit
//!      → testimonial textarea + dimension scores
//!   5. Confirmation screen
//!
//! Design: split-panel (vendor info left, form right). Inline OTP — no redirect.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// Step in the review flow.
#[derive(Clone, PartialEq)]
enum ReviewStep {
    /// Awaiting OTP email entry.
    EmailGate,
    /// OTP code entry.
    OtpVerify,
    /// Review form.
    ReviewForm,
    /// Submitted — thank-you screen.
    Done,
}

/// Public review submit page — no session required.
#[component]
pub fn ReviewSubmitPage() -> impl IntoView {
    let params    = use_params_map();
    let invite_id = move || params.get().get("invite_id").unwrap_or_default();

    let (step, set_step)           = signal(ReviewStep::EmailGate);
    let (email, set_email)         = signal(String::new());
    let (otp, set_otp)             = signal(String::new());
    let (testimonial, set_testimonial) = signal(String::new());
    let (otp_error, set_otp_error) = signal(Option::<String>::None);

    view! {
        // ── Public header ─────────────────────────────────────────────────
        <div class="public-shell">
            <header class="public-header">
                <div class="public-header__logo">
                    <span class="ms msf" style="color:#0d1421;font-size:20px">"apartment"</span>
                    <span class="public-header__wordmark">"Folio"</span>
                </div>
            </header>

            // ── Split panel ───────────────────────────────────────────────
            <div class="review-split">

                // Left — vendor context card
                <aside class="review-split__vendor" id="review-vendor-panel">
                    <div class="vendor-context-card">
                        <div class="vendor-context-card__avatar" id="review-vendor-avatar">
                            <span class="ms msf">"apartment"</span>
                        </div>
                        <h2 class="vendor-context-card__name" id="review-vendor-name">
                            "Loading…"
                        </h2>
                        <p class="vendor-context-card__trade" id="review-vendor-trade">"—"</p>
                        <div class="vendor-context-card__cta">
                            <p class="vendor-context-card__cta-text">
                                "Your review helps other property owners find trusted vendors in the Folio network."
                            </p>
                        </div>
                    </div>
                </aside>

                // Right — form panel
                <main class="review-split__form" id="review-form-panel">
                    <div class="card review-card">

                        // ── Step: Email gate ──────────────────────────────
                        <Show when=move || step.get() == ReviewStep::EmailGate>
                            <div id="review-step-email">
                                <h1 class="card-section-title">"Leave a Review"</h1>
                                <p class="card-section-sub">
                                    "We'll send a one-time code to verify your identity before you submit."
                                </p>
                                <div class="form-group" style="margin-top:20px">
                                    <label class="form-label" for="review-email">"Your email address"</label>
                                    <input
                                        id="review-email"
                                        type="email"
                                        class="form-input"
                                        placeholder="you@example.com"
                                        on:input=move |e| set_email.set(event_target_value(&e))
                                    />
                                </div>
                                <button
                                    id="review-email-submit"
                                    type="button"
                                    class="btn btn-primary w-full"
                                    on:click=move |_| {
                                        // TODO: POST /api/otp/send  { email }
                                        let _ = email.get();
                                        set_step.set(ReviewStep::OtpVerify);
                                    }
                                >
                                    "Send verification code →"
                                </button>
                            </div>
                        </Show>

                        // ── Step: OTP verify ──────────────────────────────
                        <Show when=move || step.get() == ReviewStep::OtpVerify>
                            <div id="review-step-otp">
                                <h1 class="card-section-title">"Enter your code"</h1>
                                <p class="card-section-sub">
                                    "We sent a 6-digit code to "
                                    <strong>{move || email.get()}</strong>
                                    ". Enter it below."
                                </p>
                                <div class="otp-input-wrap" style="margin-top:20px">
                                    <input
                                        id="review-otp-input"
                                        type="text"
                                        inputmode="numeric"
                                        maxlength="6"
                                        pattern="[0-9]*"
                                        placeholder="000000"
                                        class="form-input otp-input"
                                        on:input=move |e| set_otp.set(event_target_value(&e))
                                    />
                                </div>
                                <Show when=move || otp_error.get().is_some()>
                                    <p class="form-error" id="review-otp-error">
                                        {move || otp_error.get().unwrap_or_default()}
                                    </p>
                                </Show>
                                <button
                                    id="review-otp-submit"
                                    type="button"
                                    class="btn btn-primary w-full"
                                    style="margin-top:16px"
                                    on:click=move |_| {
                                        // TODO: POST /api/otp/verify { email, code }
                                        if otp.get().len() == 6 {
                                            set_otp_error.set(None);
                                            set_step.set(ReviewStep::ReviewForm);
                                        } else {
                                            set_otp_error.set(Some("Please enter all 6 digits.".to_string()));
                                        }
                                    }
                                >
                                    "Verify →"
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-ghost w-full"
                                    id="review-otp-back"
                                    style="margin-top:8px"
                                    on:click=move |_| set_step.set(ReviewStep::EmailGate)
                                >
                                    "← Back"
                                </button>
                            </div>
                        </Show>

                        // ── Step: Review form ─────────────────────────────
                        // Dimensions are loaded from /api/pub/review/:invite_id on mount.
                        // Until wired: shows testimonial textarea + a placeholder rating.
                        <Show when=move || step.get() == ReviewStep::ReviewForm>
                            <div id="review-step-form">
                                <h1 class="card-section-title">"Rate your experience"</h1>
                                <p class="card-section-sub">
                                    "Your feedback is published after a brief review."
                                </p>

                                // Dimension placeholder — wire to G-27 dimensions from API
                                <div class="dimension-group" id="review-dimensions">
                                    // Will be populated with dimension-specific inputs:
                                    // rating     → <RatingSegmented />
                                    // boolean    → <BooleanToggle />
                                    // absolute   → <input type="number">
                                    <p class="placeholder-label">"Loading review dimensions…"</p>
                                </div>

                                // Testimonial
                                <div class="form-group" style="margin-top:20px">
                                    <label class="form-label" for="review-testimonial">
                                        "Write a review (optional)"
                                    </label>
                                    <textarea
                                        id="review-testimonial"
                                        class="form-input"
                                        rows="4"
                                        placeholder="Share your experience working with this vendor…"
                                        on:input=move |e| set_testimonial.set(event_target_value(&e))
                                    />
                                </div>

                                <button
                                    id="review-form-submit"
                                    type="button"
                                    class="btn btn-primary w-full"
                                    on:click=move |_| {
                                        // TODO: POST /api/pub/review/:invite_id/submit
                                        let _ = (invite_id(), testimonial.get());
                                        set_step.set(ReviewStep::Done);
                                    }
                                >
                                    "Submit Review →"
                                </button>
                            </div>
                        </Show>

                        // ── Step: Done ────────────────────────────────────
                        <Show when=move || step.get() == ReviewStep::Done>
                            <div class="success-state" id="review-step-done">
                                <span class="ms msf success-state__icon" style="color:#10b981;font-size:48px">
                                    "check_circle"
                                </span>
                                <h1 class="success-state__title">"Review submitted!"</h1>
                                <p class="success-state__sub">
                                    "Thank you. Your review is under a brief moderation check "
                                    "before it appears on the vendor's public profile."
                                </p>
                            </div>
                        </Show>

                    </div>
                </main>
            </div>
        </div>
    }
}
