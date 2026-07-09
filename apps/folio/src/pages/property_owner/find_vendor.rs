//! Property Owner Lite — Find a Vendor
//!
//! Route: GET /po/find-vendor
//!
//! PO Lite can browse the vendor marketplace and submit a service request
//! (which becomes a source of business for the vendor).
//! This page reuses the G-34 marketplace data — no PO-specific tables needed.
//!
//! API: GET /api/folio/marketplace/vendors?trade_type=&search=

use leptos::prelude::*;

/// Find a vendor — searchable marketplace list with service request modal.
#[component]
pub fn FindVendorPage() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (show_request, set_show_request) = signal(false);
    let (selected_vendor, set_selected_vendor) = signal(Option::<String>::None);
    let (request_desc, set_request_desc) = signal(String::new());
    let (request_sent, set_request_sent) = signal(false);

    view! {
        <div class="page-header">
            <div>
                <h1 class="page-title">"Find a Vendor"</h1>
                <p class="page-subtitle">
                    "Browse verified contractors and service providers in the Folio network."
                </p>
            </div>
        </div>

        // ── Search bar ────────────────────────────────────────────────────
        <div class="search-bar-wrap" style="margin-bottom:20px">
            <span class="ms search-bar__icon">"search"</span>
            <input
                id="po-vendor-search"
                type="search"
                placeholder="Search by trade, name, or location…"
                class="form-input search-bar__input"
                on:input=move |e| set_search.set(event_target_value(&e))
            />
        </div>

        // ── Vendor list (placeholder cards — wire to API on mount) ────────
        <div class="vendor-grid" id="po-vendor-grid">
            // Placeholder cards until server_fn wires up
            {(0..4).map(|i| view! {
                <div class="vendor-card" id=format!("po-vendor-card-{}", i)>
                    <div class="vendor-card__avatar">
                        <span class="ms msf">"handyman"</span>
                    </div>
                    <div class="vendor-card__body">
                        <p class="vendor-card__name">"— Vendor —"</p>
                        <p class="vendor-card__trade">"Trade type"</p>
                        <div class="vendor-card__rating">
                            <span class="rating-stars">"★★★★☆"</span>
                            <span class="rating-count">"(0 reviews)"</span>
                        </div>
                    </div>
                    <button
                        type="button"
                        class="btn btn-secondary btn-sm"
                        id=format!("po-vendor-request-{}", i)
                        on:click=move |_| {
                            set_selected_vendor.set(Some(format!("vendor-{}", i)));
                            set_show_request.set(true);
                        }
                    >
                        "Request Service"
                    </button>
                </div>
            }).collect_view()}
        </div>

        // ── Service Request Modal ─────────────────────────────────────────
        <Show when=move || show_request.get()>
            <div class="modal-bg" id="po-vendor-request-modal">
                <div class="modal">
                    <div class="modal-header">
                        <span class="modal-title">"Request a Service"</span>
                        <button
                            type="button"
                            class="btn-icon"
                            id="po-vendor-modal-close"
                            on:click=move |_| set_show_request.set(false)
                        >
                            <span class="ms">"close"</span>
                        </button>
                    </div>
                    <div class="modal-body">
                        <Show
                            when=move || request_sent.get()
                            fallback=move || view! {
                                <div class="form-group">
                                    <label class="form-label">"Vendor"</label>
                                    <p class="form-value" id="po-request-vendor-name">
                                        {move || selected_vendor.get().unwrap_or_else(|| "—".to_string())}
                                    </p>
                                </div>
                                <div class="form-group">
                                    <label class="form-label" for="po-request-desc">"Describe the work needed"</label>
                                    <textarea
                                        id="po-request-desc"
                                        class="form-input"
                                        rows="4"
                                        placeholder="e.g. HVAC tune-up, plumbing leak in kitchen…"
                                        on:input=move |e| set_request_desc.set(event_target_value(&e))
                                    />
                                </div>
                            }
                        >
                            <div class="success-state" id="po-request-sent-state">
                                <span class="ms msf success-state__icon">"check_circle"</span>
                                <p class="success-state__title">"Request sent!"</p>
                                <p class="success-state__sub">
                                    "The vendor has been notified and will follow up shortly."
                                </p>
                            </div>
                        </Show>
                    </div>
                    <Show when=move || !request_sent.get()>
                        <div class="modal-footer">
                            <button
                                type="button"
                                class="btn btn-ghost"
                                id="po-request-cancel"
                                on:click=move |_| set_show_request.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="btn btn-primary"
                                id="po-request-submit"
                                on:click=move |_| {
                                    // TODO: wire to POST /api/folio/service-requests
                                    let _ = request_desc.get();
                                    set_request_sent.set(true);
                                }
                            >
                                "Send Request"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
