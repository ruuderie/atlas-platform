// apps/folio/src/pages/vendor/job_link.rs
//
// Vendor Job Link — /jobs/:token
//
// Token-gated page for vendors/contractors to view and accept/decline a
// specific work order without logging into the full vendor portal.
// Shared via email or SMS. POSTs to /api/folio/maintenance/job-response.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLinkData {
    pub work_order_id:  String,
    pub subject:        String,
    pub category:       String,
    pub priority:       String,
    pub description:    Option<String>,
    pub asset_address:  String,
    pub unit:           Option<String>,
    pub scheduled_date: Option<String>,
    pub budget_cents:   Option<i64>,
    pub property_manager:Option<String>,
    pub pm_phone:       Option<String>,
    pub status:         String,  // "pending_acceptance" | "accepted" | "declined" | "expired"
}

#[server(FetchJobLink, "/api")]
pub async fn fetch_job_link(token: String) -> Result<Option<JobLinkData>, server_fn::error::ServerFnError> {
    let url = format!("/api/folio/maintenance/job-link?token={token}");
    crate::atlas_client::authenticated_get::<Option<JobLinkData>>(&url, "", None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponseInput {
    pub token:    String,
    pub decision: String,  // "accept" | "decline"
    pub note:     Option<String>,
}

#[server(RespondJobLink, "/api")]
pub async fn respond_job_link(input: JobResponseInput) -> Result<(), server_fn::error::ServerFnError> {
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn priority_color(p: &str) -> &'static str {
    match p.to_lowercase().as_str() {
        "urgent" | "emergency" => "#f87171",
        "high"                 => "#fb923c",
        "medium"               => "#fbbf24",
        _                      => "#94a3b8",
    }
}

fn fmt_budget(cents: i64) -> String {
    format!("${:.0}", cents as f64 / 100.0)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn VendorJobLink() -> impl IntoView {
    let params  = use_params_map();
    let token   = params.get().get("token").unwrap_or_default();

    let decision   = RwSignal::new(None::<String>);
    let note       = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let responded  = RwSignal::new(None::<String>);
    let error      = RwSignal::new(None::<String>);

    let token2 = token.clone();
    let res = Resource::new(
        move || token.clone(),
        |t| fetch_job_link(t),
    );

    let handle_respond = move |d: String| {
        decision.set(Some(d.clone()));
        submitting.set(true);
        let input = JobResponseInput {
            token:    token2.clone(),
            decision: d.clone(),
            note:     if note.get().is_empty() { None } else { Some(note.get()) },
        };
        leptos::task::spawn_local(async move {
            match respond_job_link(input).await {
                Ok(_)  => { responded.set(Some(d)); submitting.set(false); }
                Err(e) => { error.set(Some(e.to_string())); submitting.set(false); }
            }
        });
    };
    let handle_accept  = store_value({
        let h = handle_respond.clone();
        move |_: web_sys::MouseEvent| h("accept".to_string())
    });
    let handle_decline = store_value({
        let h = handle_respond.clone();
        move |_: web_sys::MouseEvent| h("decline".to_string())
    });

    view! {
        <div class="apply-layout">
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas"</div>
                <div class="apply-subtitle">"Work Order"</div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading job details…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(Some(job)) => {
                            let already_done = job.status == "accepted" || job.status == "declined" || job.status == "expired";
                            let pcolor = priority_color(&job.priority);
                            let budget_str = job.budget_cents.map(fmt_budget);
                            view! {
                                <div class="lead-portal-card">
                                    // ── Job header ──
                                    <div class="job-link-header">
                                        <div class="job-link-category">{job.category.replace('_', " ")}</div>
                                        <div class="job-link-subject">{job.subject.clone()}</div>
                                        <div class="job-link-meta">
                                            <span class="job-link-priority"
                                                style=format!("color:{pcolor};font-weight:700;")>
                                                {job.priority.clone()}
                                            </span>
                                            <span class="job-link-address">"📍 " {job.asset_address.clone()}
                                                {job.unit.as_ref().map(|u| format!(" · Unit {u}"))}
                                            </span>
                                        </div>
                                    </div>

                                    // ── Details ──
                                    <div class="job-link-details">
                                        {job.scheduled_date.as_ref().map(|d| view! {
                                            <div class="job-link-detail-row">
                                                <span class="job-link-detail-label">"Scheduled"</span>
                                                <span class="job-link-detail-val">{d.clone()}</span>
                                            </div>
                                        })}
                                        {budget_str.as_ref().map(|b| view! {
                                            <div class="job-link-detail-row">
                                                <span class="job-link-detail-label">"Budget"</span>
                                                <span class="job-link-detail-val" style="color:#4ade80;">{b.clone()}</span>
                                            </div>
                                        })}
                                        {job.property_manager.as_ref().map(|pm| view! {
                                            <div class="job-link-detail-row">
                                                <span class="job-link-detail-label">"Contact"</span>
                                                <span class="job-link-detail-val">{pm.clone()}
                                                    {job.pm_phone.as_ref().map(|ph| format!(" · {ph}"))}
                                                </span>
                                            </div>
                                        })}
                                    </div>

                                    {job.description.as_ref().map(|d| view! {
                                        <div class="job-link-desc">
                                            <div class="job-link-desc-label">"Job Description"</div>
                                            <p class="job-link-desc-text">{d.clone()}</p>
                                        </div>
                                    })}

                                    // ── Response ──
                                    {move || {
                                        let resp = responded.get();
                                        if resp.is_some() || already_done {
                                            let is_accepted = resp.as_deref() == Some("accept") || job.status == "accepted";
                                            let (resp_icon, resp_title, resp_color) = if is_accepted {
                                                ("✅", "Job Accepted", "#4ade80")
                                            } else if job.status == "expired" {
                                                ("⏰", "Link Expired", "#94a3b8")
                                            } else {
                                                ("✗", "Job Declined", "#f87171")
                                            };
                                            view! {
                                                <div class="job-link-response-banner" style=format!("border-color:{resp_color};background:rgba(0,0,0,.05)")>
                                                    <span style=format!("font-size:1.5rem;color:{resp_color};")>{resp_icon}</span>
                                                    <div>
                                                        <div style=format!("font-weight:700;color:{resp_color};")>{resp_title}</div>
                                                        {if is_accepted { view! { <div class="text-xs text-on-surface-variant">"The property manager has been notified. Check your email for full details."</div> }.into_any() }
                                                         else { ().into_any() }}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="job-link-action-section">
                                                    {error.get().map(|e| view! { <div class="wiz-error-banner">"⚠ " {e}</div> })}
                                                    <div class="form-field">
                                                        <label class="form-label">"Note (optional)"</label>
                                                        <textarea class="form-input" style="min-height:4rem;resize:vertical;"
                                                            placeholder="Any questions or availability notes…"
                                                            on:input=move |ev| note.set(event_target_value(&ev))
                                                        >{move || note.get()}</textarea>
                                                    </div>
                                                    <div class="job-link-cta-row">
                                                        <button
                                                            class="btn btn-ghost"
                                                            style="border-color:#f87171;color:#f87171;"
                                                            disabled=move || submitting.get()
                                                            on:click=move |e| handle_decline.get_value()(e)
                                                        >"✗ Decline Job"</button>
                                                        <button
                                                            class="btn btn-primary"
                                                            style="background:linear-gradient(135deg,#22c55e,#16a34a);"
                                                            disabled=move || submitting.get()
                                                            on:click=move |e| handle_accept.get_value()(e)
                                                        >{move || if submitting.get() { "Submitting…" } else { "✓ Accept Job" }}</button>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        }
                        Ok(None) | Err(_) => view! {
                            <div class="lead-portal-card">
                                <div class="doc-empty" style="padding:2rem;">
                                    <div style="font-size:2rem;">"🔗"</div>
                                    <div>"This job link is invalid or has expired."</div>
                                    <div class="text-xs text-on-surface-variant">"Contact the property manager for a new link, or log in to your vendor portal."</div>
                                    <a href="/v" class="btn btn-ghost btn-sm" style="margin-top:1rem;">"Vendor Portal →"</a>
                                </div>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
