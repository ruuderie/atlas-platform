//! Property Owner Lite — Property Value Tracker
//!
//! Route: GET /po/value
//!
//! Two-panel page:
//!   Left:  Source-keyed line chart of all logged valuations (Recharts-style in Leptos)
//!   Right: "Log a Valuation" form — source picker + value + date + note
//!
//! API:
//!   GET  /api/folio/properties/:id/value-history
//!   POST /api/folio/properties/:id/value

use leptos::prelude::*;

/// Source options that match the backend PropertyValueSource enum.
/// Displayed as a segmented control so the user understands the provenance.
const VALUE_SOURCES: &[(&str, &str, &str)] = &[
    ("manual", "My Estimate", "edit"),
    ("purchase_price", "Purchase Price", "sell"),
    ("zillow_avm", "Zillow AVM", "bar_chart"),
    ("county_record", "County Record", "account_balance"),
    ("certified_appraisal", "Appraisal", "verified"),
    ("bank_appraisal", "Bank Appraisal", "domain"),
    ("agent_cma", "Agent CMA", "real_estate_agent"),
];

/// Property value tracker — source-keyed chart + log form.
#[component]
pub fn PropertyValuePage() -> impl IntoView {
    // Selected source for the log form (local signal)
    let (selected_source, set_source) = signal("manual");
    let (value_input, set_value) = signal(String::new());
    let (date_input, set_date) = signal(String::new());
    let (note_input, set_note) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (success_msg, set_success) = signal(Option::<String>::None);

    view! {
        <div class="page-header">
            <div>
                <h1 class="page-title">"Property Value"</h1>
                <p class="page-subtitle">
                    "Log valuations from any source and watch your equity grow over time."
                </p>
            </div>
        </div>

        // ── Two-panel layout ──────────────────────────────────────────────
        <div class="split-panel">

            // Left — chart panel
            <div class="split-panel__main">
                <div class="card" style="height:360px;display:flex;flex-direction:column">
                    <div class="card-header">
                        <span class="card-title">"Value Over Time"</span>
                        <div class="source-legend" id="po-value-legend">
                            // Legend dots per source (shown when data loads)
                            {VALUE_SOURCES.iter().map(|(slug, label, _)| view! {
                                <span class="legend-dot" data-source={*slug}>{*label}</span>
                            }).collect_view()}
                        </div>
                    </div>
                    // Chart placeholder — will be wired to real data via server_fn
                    <div class="chart-placeholder" id="po-value-chart" style="flex:1">
                        <div class="chart-empty-state">
                            <span class="ms msf chart-empty-state__icon">"show_chart"</span>
                            <p>"No valuations logged yet."</p>
                            <p class="chart-empty-state__sub">"Log your first entry using the form →"</p>
                        </div>
                    </div>
                </div>

                // History table
                <div class="card" style="margin-top:16px">
                    <div class="card-header">
                        <span class="card-title">"History"</span>
                    </div>
                    <table class="data-table" id="po-value-history-table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Source"</th>
                                <th>"Value"</th>
                                <th>"Ref / Note"</th>
                            </tr>
                        </thead>
                        <tbody id="po-value-history-body">
                            // Populated by JS/server_fn on mount
                            <tr class="empty-row">
                                <td colspan="4">"No entries yet"</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            // Right — log form panel
            <div class="split-panel__aside">
                <div class="card">
                    <div class="card-header">
                        <span class="card-title">"Log a Valuation"</span>
                    </div>
                    <div class="card-body">

                        // Source picker — segmented grid
                        <div class="form-group">
                            <label class="form-label">"Valuation Source"</label>
                            <div class="source-grid" id="po-source-grid">
                                {VALUE_SOURCES.iter().map(|(slug, label, icon)| {
                                    let slug_str = *slug;
                                    let is_sel = move || selected_source.get() == slug_str;
                                    view! {
                                        <button
                                            type="button"
                                            id=format!("po-source-{}", slug)
                                            class:source-btn=true
                                            class:source-btn--selected=is_sel
                                            on:click=move |_| set_source.set(slug_str)
                                        >
                                            <span class="ms msf source-btn__icon">{*icon}</span>
                                            <span class="source-btn__label">{*label}</span>
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Value input (dollars)
                        <div class="form-group">
                            <label class="form-label" for="po-value-input">"Estimated Value (USD)"</label>
                            <div class="input-prefix-wrap">
                                <span class="input-prefix">"$"</span>
                                <input
                                    id="po-value-input"
                                    type="number"
                                    min="1"
                                    step="1000"
                                    placeholder="450,000"
                                    class="form-input input-with-prefix"
                                    on:input=move |e| set_value.set(event_target_value(&e))
                                />
                            </div>
                        </div>

                        // Date
                        <div class="form-group">
                            <label class="form-label" for="po-value-date">"Valuation Date"</label>
                            <input
                                id="po-value-date"
                                type="date"
                                class="form-input"
                                on:input=move |e| set_date.set(event_target_value(&e))
                            />
                        </div>

                        // Optional note
                        <div class="form-group">
                            <label class="form-label" for="po-value-note">"Note (optional)"</label>
                            <input
                                id="po-value-note"
                                type="text"
                                placeholder="e.g. after roof replacement"
                                class="form-input"
                                on:input=move |e| set_note.set(event_target_value(&e))
                            />
                        </div>

                        // Success feedback
                        <Show when=move || success_msg.get().is_some()>
                            <div class="alert alert-success" id="po-value-success">
                                {move || success_msg.get().unwrap_or_default()}
                            </div>
                        </Show>

                        // Submit
                        <button
                            id="po-value-submit"
                            type="button"
                            class="btn btn-primary w-full"
                            disabled=move || submitting.get()
                            on:click=move |_| {
                                // TODO: wire to server_fn → POST /api/folio/properties/:id/value
                                let _ = (selected_source.get(), value_input.get(), date_input.get(), note_input.get());
                                set_submitting.set(true);
                                set_success.set(Some("Valuation logged! Chart will update.".to_string()));
                                set_submitting.set(false);
                            }
                        >
                            <Show when=move || submitting.get() fallback=|| view! { "Log Valuation" }>
                                "Saving…"
                            </Show>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
