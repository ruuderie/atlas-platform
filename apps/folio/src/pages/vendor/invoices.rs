//! Vendor invoices — `/v/invoices`
//! Wired to `GET /api/folio/vendor/invoices`.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorInvoiceSummary {
    pub id: Uuid,
    pub gross_amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "paid" => StatusPillTone::Ok,
        "pending" => StatusPillTone::Warn,
        "overdue" | "failed" => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn VendorInvoices() -> impl IntoView {
    let filter = RwSignal::new("all".to_string());
    let invoices = Resource::new(
        move || filter.get(),
        |status| async move { list_vendor_invoices(status).await },
    );

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Invoices".to_string())
                subtitle=Signal::derive(|| "Submitted invoices and payment status.".to_string())
            />

            <div class="landlord-filter-chips" style="margin-bottom:1rem;">
                {[("all", "All"), ("pending", "Pending"), ("paid", "Paid")]
                    .into_iter()
                    .map(|(v, label)| {
                        let v_for_class = v.to_string();
                        let v_for_click = v.to_string();
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    if filter.get() == v_for_class {
                                        "landlord-chip landlord-chip--active"
                                    } else {
                                        "landlord-chip"
                                    }
                                }
                                on:click=move |_| filter.set(v_for_click.clone())
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading invoices…"</p></div>
            }>
                {move || invoices.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load invoices"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"receipt_long"</span>
                            <p class="folio-empty__heading">"No invoices"</p>
                            <p class="folio-empty__sub">"Invoices you submit for completed work appear here."</p>
                        </div>
                    }.into_any(),
                    Ok(items) => view! {
                        <For
                            each=move || items.clone()
                            key=|i| i.id
                            children=move |i| {
                                let amt = format!("${:.2}", i.gross_amount_cents as f64 / 100.0);
                                let due = i.due_date
                                    .map(|d| d.format("%b %d, %Y").to_string())
                                    .unwrap_or_else(|| "No due date".into());
                                view! {
                                    <div class="hub-activity-rail__row">
                                        <StatusPill label=i.status.clone() tone=tone(&i.status)/>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{format!("{amt} · {}", i.currency)}</p>
                                            <p class="hub-activity-rail__row-meta">{due}</p>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListVendorInvoices, "/api")]
pub async fn list_vendor_invoices(
    status: String,
) -> Result<Vec<VendorInvoiceSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let path = format!("/api/folio/vendor/invoices?status={status}");
    crate::atlas_client::authenticated_get::<Vec<VendorInvoiceSummary>>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Vendor invoices failed: {e}")))
}
