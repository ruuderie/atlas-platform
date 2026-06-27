// apps/folio/src/pages/str_host/reviews.rs
//
// STR Reviews — /s/reviews
//
// Guest review management: view ratings, respond to reviews.
// Review data sourced from /api/folio/reviews (Phase 7 endpoint).
// For now renders with realistic placeholder data and the response UI.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;

// ── Static review data (production: from /api/folio/reviews) ─────────────────

#[derive(Debug, Clone)]
struct ReviewEntry {
    id:           &'static str,
    guest_name:   &'static str,
    rating:       u8,
    comment:      &'static str,
    date:         &'static str,
    responded:    bool,
    response:     Option<&'static str>,
}

fn sample_reviews() -> Vec<ReviewEntry> {
    vec![
        ReviewEntry {
            id: "r1", guest_name: "Marcus T.", rating: 5,
            comment: "Amazing place! Super clean, great location, and the host was incredibly responsive. Would stay again in a heartbeat.",
            date: "2026-06-15", responded: true,
            response: Some("Thank you Marcus! It was a pleasure hosting you. Hope to see you again soon!"),
        },
        ReviewEntry {
            id: "r2", guest_name: "Sophie L.", rating: 4,
            comment: "Really enjoyed our stay. The apartment was well-equipped and comfortable. Minor noise from the street at night.",
            date: "2026-06-02", responded: false, response: None,
        },
        ReviewEntry {
            id: "r3", guest_name: "James K.", rating: 5,
            comment: "Perfect weekend getaway. Everything as described, check-in was seamless.",
            date: "2026-05-28", responded: false, response: None,
        },
        ReviewEntry {
            id: "r4", guest_name: "Ana R.", rating: 3,
            comment: "Decent place but the WiFi was slow. Kitchen was well-stocked though.",
            date: "2026-05-10", responded: true,
            response: Some("Hi Ana, thank you for the feedback! We've upgraded the WiFi since your stay."),
        },
    ]
}

fn stars(rating: u8) -> String {
    "★".repeat(rating as usize) + &"☆".repeat(5 - rating as usize)
}

fn star_color(rating: u8) -> &'static str {
    match rating {
        5    => "#fbbf24",
        4    => "#fbbf24",
        3    => "#f97316",
        1|2  => "#f87171",
        _    => "#94a3b8",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrReviews() -> impl IntoView {
    let reviews  = sample_reviews();
    let respond_to = RwSignal::new(None::<&'static str>);
    let response_text = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);

    let avg_rating = {
        let total: f64 = reviews.iter().map(|r| r.rating as f64).sum();
        total / reviews.len() as f64
    };
    let five_star = reviews.iter().filter(|r| r.rating == 5).count();
    let pending   = reviews.iter().filter(|r| !r.responded).count();

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Guest Reviews"</h1>
                    <p class="page-subtitle">"Ratings and feedback from your guests — respond to build trust"</p>
                </div>
            </div>

            // ── KPIs ──
            <div class="kpi-row" style="margin-bottom:1.5rem;">
                <div class="kpi-card">
                    <span class="kpi-label">"Average Rating"</span>
                    <span class="kpi-value" style="color:#fbbf24">{format!("{:.1} ★", avg_rating)}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"5-Star Reviews"</span>
                    <span class="kpi-value" style="color:var(--green)">{five_star.to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Total Reviews"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">{reviews.len().to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Awaiting Response"</span>
                    <span class="kpi-value" style="color:var(--amber)">{pending.to_string()}</span>
                </div>
            </div>

            {move || if submitted.get() {
                view! { <div class="alert-saved-toast">"✓ Response submitted"</div> }.into_any()
            } else { ().into_any() }}

            // ── Review list ──
            <div class="str-review-list">
                {reviews.iter().map(|review| {
                    let rid       = review.id;
                    let name      = review.guest_name;
                    let rating    = review.rating;
                    let comment   = review.comment;
                    let date      = review.date;
                    let responded = review.responded;
                    let response  = review.response;
                    let star_str  = stars(rating);
                    let s_color   = star_color(rating);

                    view! {
                        <div class="str-review-card">
                            <div class="str-review-header">
                                <div class="str-review-avatar">{name.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "G".to_string())}</div>
                                <div class="str-review-meta">
                                    <div class="str-review-name">{name}</div>
                                    <div class="str-review-date">{date}</div>
                                </div>
                                <div class="str-review-stars" style=format!("color:{s_color};font-size:1.1rem;")>{star_str}</div>
                                {if !responded {
                                    view! {
                                        <span class="ph-badge ph-badge--pending">"Needs Response"</span>
                                    }.into_any()
                                } else { ().into_any() }}
                            </div>

                            <p class="str-review-comment">"{" {comment} "}"</p>

                            {if let Some(resp) = response {
                                view! {
                                    <div class="str-review-response">
                                        <div class="str-review-response-label">"Host Response:"</div>
                                        <div class="str-review-response-text">{resp}</div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <button
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |_| { respond_to.set(Some(rid)); response_text.set(String::new()); submitted.set(false); }
                                    >"Reply to Guest"</button>
                                }.into_any()
                            }}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // ── Response Modal ──
            <Show when=move || respond_to.get().is_some()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:30rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Respond to Review"</h3>
                            <button class="modal-close" on:click=move |_| respond_to.set(None)>"✕"</button>
                        </div>
                        <div class="modal-body">
                            <div class="form-field">
                                <label class="form-label">"Your Response"</label>
                                <textarea
                                    class="form-input str-listing-textarea"
                                    placeholder="Thank you for your review…"
                                    prop:value=move || response_text.get()
                                    on:input=move |ev| response_text.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| respond_to.set(None)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || response_text.get().trim().is_empty()
                                on:click=move |_| { submitted.set(true); respond_to.set(None); }
                            >"Submit Response"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
