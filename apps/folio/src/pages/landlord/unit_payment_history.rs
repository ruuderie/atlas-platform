//! Unit payment history — `/l/assets/:id/history/payments`
//! Single charge, period batch, and void series.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::leases::list_leases;
use crate::pages::landlord::ledger::{list_ledger_entries, LedgerEntrySummary};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerKind {
    Charge,
    Payment,
    ExternalMarkedPaid,
}

impl LedgerKind {
    const ALL: &'static [Self] = &[Self::Charge, Self::Payment, Self::ExternalMarkedPaid];

    fn as_str(self) -> &'static str {
        match self {
            Self::Charge => "charge",
            Self::Payment => "payment",
            Self::ExternalMarkedPaid => "external_marked_paid",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Charge => "Charge",
            Self::Payment => "Payment",
            Self::ExternalMarkedPaid => "External marked paid",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryMode {
    Single,
    Period,
}

#[derive(Serialize)]
struct PeriodChargesBody {
    billable_entity_type: String,
    billable_entity_id: Uuid,
    kind: String,
    description: String,
    gross_amount_cents: i64,
    currency: String,
    method: Option<String>,
    start_month: String,
    end_month: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PeriodChargesResp {
    series_id: Uuid,
    entry_ids: Vec<Uuid>,
    months: Vec<String>,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

/// Parse `[period_series:{uuid}]` tags written into ledger notes.
pub fn series_id_from_note(note: &str) -> Option<Uuid> {
    const PREFIX: &str = "[period_series:";
    let start = note.find(PREFIX)? + PREFIX.len();
    let rest = &note[start..];
    let end = rest.find(']')?;
    Uuid::parse_str(&rest[..end]).ok()
}

/// Inclusive calendar-month count between YYYY-MM-01 dates (client preview).
pub fn inclusive_month_count(start: &str, end: &str) -> Option<u32> {
    let parse = |s: &str| -> Option<(i32, u32)> {
        let parts: Vec<_> = s.split('-').collect();
        if parts.len() < 2 {
            return None;
        }
        let y: i32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        if (1..=12).contains(&m) {
            Some((y, m))
        } else {
            None
        }
    };
    let (ys, ms) = parse(start)?;
    let (ye, me) = parse(end)?;
    let start_i = ys * 12 + (ms as i32 - 1);
    let end_i = ye * 12 + (me as i32 - 1);
    if end_i < start_i {
        return None;
    }
    Some((end_i - start_i + 1) as u32)
}

#[server(CreatePeriodCharges, "/api")]
async fn create_period_charges(
    billable_entity_id: Uuid,
    kind: String,
    description: String,
    gross_amount_cents: i64,
    currency: String,
    method: Option<String>,
    start_month: String,
    end_month: String,
) -> Result<PeriodChargesResp, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let to_date = |ym: String| -> String {
        if ym.len() == 7 {
            format!("{ym}-01")
        } else {
            ym
        }
    };
    let body = PeriodChargesBody {
        billable_entity_type: "atlas_contract".into(),
        billable_entity_id,
        kind,
        description,
        gross_amount_cents,
        currency,
        method,
        start_month: to_date(start_month),
        end_month: to_date(end_month),
    };
    crate::atlas_client::authenticated_post(
        "/api/folio/ledger/period-charges",
        &token,
        None,
        &body,
    )
    .await
    .map_err(ServerFnError::new)
}

#[server(VoidPeriodSeries, "/api")]
async fn void_period_series(series_id: Uuid) -> Result<usize, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let resp: serde_json::Value = crate::atlas_client::authenticated_post(
        &format!("/api/folio/ledger/period-charges/{series_id}/void"),
        &token,
        None,
        &serde_json::json!({}),
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp
        .get("voided")
        .or_else(|| resp.get("count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize)
}

#[server(VoidLedgerEntry, "/api")]
async fn void_ledger_entry(entry_id: Uuid) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let _: serde_json::Value = crate::atlas_client::authenticated_post(
        &format!("/api/folio/ledger/{entry_id}/void"),
        &token,
        None,
        &serde_json::json!({}),
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(())
}

#[component]
pub fn UnitPaymentHistory() -> impl IntoView {
    let params = use_params_map();
    let asset_id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or(Uuid::nil())
    });

    let refresh = RwSignal::new(0u32);
    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let ledger = Resource::new(
        move || refresh.get(),
        |_| async move { list_ledger_entries().await },
    );
    let lease_id = RwSignal::new(String::new());
    let mode = RwSignal::new(EntryMode::Period);
    let kind = RwSignal::new(LedgerKind::Charge);
    let amount = RwSignal::new(String::new());
    let currency = RwSignal::new("USD".to_string());
    let description = RwSignal::new("Rent".to_string());
    let method = RwSignal::new(String::new());
    let start_month = RwSignal::new(String::new());
    let end_month = RwSignal::new(String::new());
    let last_series = RwSignal::new(None::<Uuid>);
    let error = RwSignal::new(None::<String>);
    let success = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);

    let history_href = Memo::new(move |_| {
        FolioRoute::LandlordUnitHistory
            .path()
            .replace(":id", &asset_id.get().to_string())
    });

    let month_preview = Memo::new(move |_| {
        inclusive_month_count(&start_month.get(), &end_month.get())
    });

    Effect::new(move |_| {
        let aid = asset_id.get();
        if let Some(Ok(all)) = leases.get() {
            if lease_id.get().is_empty() {
                if let Some(l) = all.into_iter().find(|l| l.asset_id == Some(aid)) {
                    lease_id.set(l.id.to_string());
                }
            }
        }
    });

    view! {
        <div class="folio-form-page">
            <PageHeader
                title=Signal::derive(|| "Payment history".to_string())
                subtitle=Signal::derive(|| {
                    "Backfill charges or payments for leases on this unit.".to_string()
                })
            >
                <a class="folio-btn folio-btn--ghost press" href=move || history_href.get()>
                    "Back to History"
                </a>
            </PageHeader>

            <Suspense fallback=|| view! { <div class="folio-empty">"Loading leases…"</div> }>
                {move || {
                    let aid = asset_id.get();
                    let unit_leases: Vec<_> = leases
                        .get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|l| l.asset_id == Some(aid))
                        .collect();
                    if unit_leases.is_empty() {
                        return view! {
                            <div class="folio-empty">
                                <p>"No leases on this unit yet."</p>
                                <a
                                    class="folio-btn folio-btn--primary press"
                                    style="margin-top:0.75rem;display:inline-block;"
                                    href=FolioRoute::LandlordHistoricalLease
                                        .path()
                                        .replace(":id", &aid.to_string())
                                >
                                    "Add historical lease"
                                </a>
                            </div>
                        }.into_any();
                    }
                    let lease_options = unit_leases.clone();
                    let lease_ids: std::collections::HashSet<_> =
                        unit_leases.into_iter().map(|l| l.id).collect();
                    view! {
                        <form
                            class="folio-form"
                            on:submit=move |ev| {
                                ev.prevent_default();
                                error.set(None);
                                success.set(None);
                                let Ok(lid) = Uuid::parse_str(lease_id.get().trim()) else {
                                    error.set(Some("Select a lease.".into()));
                                    return;
                                };
                                let cents = match amount.get().trim().parse::<f64>() {
                                    Ok(v) if v >= 0.0 => (v * 100.0).round() as i64,
                                    _ => {
                                        error.set(Some("Enter amount (e.g. 1850).".into()));
                                        return;
                                    }
                                };
                                let desc = description.get().trim().to_string();
                                if desc.is_empty() {
                                    error.set(Some("Description is required.".into()));
                                    return;
                                }
                                let cur = currency.get();
                                let k = kind.get();
                                let m = mode.get();
                                pending.set(true);
                                spawn_local(async move {
                                    // Single = one-month period series (same ledger path as batch).
                                    let (start, end) = match m {
                                        EntryMode::Single => {
                                            let sm = start_month.get();
                                            if sm.is_empty() {
                                                error.set(Some(
                                                    "Pick the month for this single entry.".into(),
                                                ));
                                                pending.set(false);
                                                return;
                                            }
                                            (sm.clone(), sm)
                                        }
                                        EntryMode::Period => {
                                            let start = start_month.get();
                                            let end = end_month.get();
                                            if start.is_empty() || end.is_empty() {
                                                error.set(Some(
                                                    "Start and end month are required.".into(),
                                                ));
                                                pending.set(false);
                                                return;
                                            }
                                            (start, end)
                                        }
                                    };
                                    let method_opt = {
                                        let s = method.get().trim().to_string();
                                        if s.is_empty() {
                                            None
                                        } else {
                                            Some(s)
                                        }
                                    };
                                    let result = create_period_charges(
                                        lid,
                                        k.as_str().into(),
                                        desc,
                                        cents,
                                        cur,
                                        method_opt,
                                        start,
                                        end,
                                    )
                                    .await
                                    .map(|r| {
                                        last_series.set(Some(r.series_id));
                                        format!("Saved {} ledger entries.", r.entry_ids.len())
                                    });
                                    match result {
                                        Ok(msg) => {
                                            success.set(Some(msg));
                                            refresh.update(|n| *n += 1);
                                            pending.set(false);
                                        }
                                        Err(e) => {
                                            error.set(Some(e.to_string()));
                                            pending.set(false);
                                        }
                                    }
                                });
                            }
                        >
                            <label class="folio-field__label">
                                "Lease"
                                <select
                                    class="folio-input"
                                    prop:value=move || lease_id.get()
                                    on:change=move |ev| lease_id.set(event_target_value(&ev))
                                >
                                    {lease_options.into_iter().map(|l| {
                                        let id = l.id.to_string();
                                        let label = format!(
                                            "{} · {} · {}",
                                            l.status,
                                            l.monthly_rent_cents
                                                .map(|c| format!("${:.0}", c as f64 / 100.0))
                                                .unwrap_or_else(|| "—".into()),
                                            l.start_date
                                                .map(|d| d.to_string())
                                                .unwrap_or_else(|| "—".into())
                                        );
                                        view! { <option value=id>{label}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </label>

                            <div class="unit-actions" style="margin-bottom:1rem;">
                                <button
                                    type="button"
                                    class=move || if mode.get() == EntryMode::Period {
                                        "folio-btn folio-btn--primary press"
                                    } else {
                                        "folio-btn folio-btn--ghost press"
                                    }
                                    on:click=move |_| mode.set(EntryMode::Period)
                                >
                                    "Period series"
                                </button>
                                <button
                                    type="button"
                                    class=move || if mode.get() == EntryMode::Single {
                                        "folio-btn folio-btn--primary press"
                                    } else {
                                        "folio-btn folio-btn--ghost press"
                                    }
                                    on:click=move |_| mode.set(EntryMode::Single)
                                >
                                    "Single entry"
                                </button>
                            </div>

                            <label class="folio-field__label">
                                "Kind"
                                <select
                                    class="folio-input"
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        if let Some(k) = LedgerKind::ALL.iter().copied().find(|k| k.as_str() == v) {
                                            kind.set(k);
                                        }
                                    }
                                >
                                    {LedgerKind::ALL.iter().map(|k| {
                                        view! { <option value=k.as_str()>{k.label()}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </label>

                            <label class="folio-field__label">
                                "Amount (per month if period)"
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || amount.get()
                                    on:input=move |ev| amount.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Description"
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || description.get()
                                    on:input=move |ev| description.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Currency"
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || currency.get()
                                    on:input=move |ev| currency.set(event_target_value(&ev))
                                />
                            </label>

                            <label class="folio-field__label">
                                {move || if mode.get() == EntryMode::Single {
                                    "Month"
                                } else {
                                    "Start month"
                                }}
                                <input
                                    class="folio-input"
                                    type="month"
                                    prop:value=move || start_month.get()
                                    on:input=move |ev| start_month.set(event_target_value(&ev))
                                />
                            </label>
                            <Show when=move || mode.get() == EntryMode::Period>
                                <label class="folio-field__label">
                                    "End month"
                                    <input
                                        class="folio-input"
                                        type="month"
                                        prop:value=move || end_month.get()
                                        on:input=move |ev| end_month.set(event_target_value(&ev))
                                    />
                                </label>
                                <p class="proj-section__hint">
                                    {move || match month_preview.get() {
                                        Some(n) => format!("Will create {n} month(s)."),
                                        None => "Select a valid month range.".into(),
                                    }}
                                </p>
                            </Show>
                            <label class="folio-field__label">
                                "Method (optional)"
                                <input
                                    class="folio-input"
                                    type="text"
                                    placeholder="cash, check, ach…"
                                    prop:value=move || method.get()
                                    on:input=move |ev| method.set(event_target_value(&ev))
                                />
                            </label>

                            {move || error.get().map(|e| view! {
                                <p style="color:#b91c1c;">{e}</p>
                            })}
                            {move || success.get().map(|s| view! {
                                <p style="color:#15803d;">{s}</p>
                            })}

                            <button
                                type="submit"
                                class="folio-btn folio-btn--primary press"
                                disabled=move || pending.get()
                            >
                                {move || if pending.get() { "Saving…" } else { "Save to ledger" }}
                            </button>
                        </form>

                        {move || last_series.get().map(|sid| view! {
                            <div class="unit-actions" style="margin-top:1rem;">
                                <button
                                    type="button"
                                    class="folio-btn folio-btn--ghost press"
                                    on:click=move |_| {
                                        spawn_local(async move {
                                            match void_period_series(sid).await {
                                                Ok(n) => {
                                                    success.set(Some(format!("Voided {n} entries in that series.")));
                                                    last_series.set(None);
                                                    refresh.update(|n| *n += 1);
                                                }
                                                Err(e) => error.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                >
                                    "Void series just saved"
                                </button>
                            </div>
                        })}

                        <section class="proj-section" style="margin-top:2rem;">
                            <h3 class="proj-section__title">"Ledger on this unit"</h3>
                            <p class="proj-section__hint">"Void a charge or an entire period series from the row."</p>
                            <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading ledger…"</div> }>
                                {move || {
                                    let ids = lease_ids.clone();
                                    let rows: Vec<LedgerEntrySummary> = ledger
                                        .get()
                                        .and_then(|r| r.ok())
                                        .unwrap_or_default()
                                        .into_iter()
                                        .filter(|e| {
                                            e.billable_entity_type == "atlas_contract"
                                                && ids.contains(&e.billable_entity_id)
                                        })
                                        .take(40)
                                        .collect();
                                    if rows.is_empty() {
                                        return view! {
                                            <div class="folio-empty--compact">"No ledger entries for this unit’s leases yet."</div>
                                        }.into_any();
                                    }
                                    view! {
                                        <For
                                            each=move || rows.clone()
                                            key=|e| e.id
                                            children=move |e: LedgerEntrySummary| {
                                                let eid = e.id;
                                                let series = e
                                                    .description
                                                    .as_deref()
                                                    .and_then(series_id_from_note)
                                                    .or_else(|| {
                                                        e.reconciliation_note
                                                            .as_deref()
                                                            .and_then(series_id_from_note)
                                                    });
                                                let amt = format!(
                                                    "${:.2} {}",
                                                    e.gross_amount_cents as f64 / 100.0,
                                                    e.currency
                                                );
                                                let label = e
                                                    .description
                                                    .clone()
                                                    .unwrap_or_else(|| e.status.clone());
                                                let can_void = e.status != "voided"
                                                    && e.status != "refunded"
                                                    && e.status != "waived";
                                                view! {
                                                    <div class="hub-activity-rail__row">
                                                        <div class="hub-activity-rail__body">
                                                            <p class="hub-activity-rail__row-title">{amt}</p>
                                                            <p class="hub-activity-rail__row-meta">
                                                                {format!("{} · {}", e.status, label)}
                                                            </p>
                                                        </div>
                                                        <Show when=move || can_void>
                                                            <div class="unit-actions">
                                                                <button
                                                                    type="button"
                                                                    class="folio-btn folio-btn--ghost press"
                                                                    on:click=move |_| {
                                                                        spawn_local(async move {
                                                                            match void_ledger_entry(eid).await {
                                                                                Ok(()) => {
                                                                                    success.set(Some("Entry voided.".into()));
                                                                                    refresh.update(|n| *n += 1);
                                                                                }
                                                                                Err(err) => error.set(Some(err.to_string())),
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    "Void"
                                                                </button>
                                                                {series.map(|sid| view! {
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost press"
                                                                        on:click=move |_| {
                                                                            spawn_local(async move {
                                                                                match void_period_series(sid).await {
                                                                                    Ok(n) => {
                                                                                        success.set(Some(format!("Voided {n} entries.")));
                                                                                        refresh.update(|n| *n += 1);
                                                                                    }
                                                                                    Err(err) => error.set(Some(err.to_string())),
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "Void series"
                                                                    </button>
                                                                })}
                                                            </div>
                                                        </Show>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }}
                            </Suspense>
                        </section>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::inclusive_month_count;

    #[test]
    fn month_count_six_months() {
        assert_eq!(
            inclusive_month_count("2025-01", "2025-06"),
            Some(6)
        );
    }

    #[test]
    fn month_count_rejects_inverted() {
        assert_eq!(inclusive_month_count("2025-06", "2025-01"), None);
    }

    #[test]
    fn parses_period_series_tag() {
        let note = "[period_series:11111111-1111-1111-1111-111111111111][charge][cash] Rent (2025-01)";
        assert_eq!(
            series_id_from_note(note),
            Some(Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap())
        );
    }
}
