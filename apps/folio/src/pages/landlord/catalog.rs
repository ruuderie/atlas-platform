//! Catalog / pricebook — `/l/catalog`
//!
//! Wired to `GET/POST /api/folio/catalog` (G-26).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;
use uuid::Uuid;

use crate::components::page_header::PageHeader;

// ── Domain enums (wire = snake_case, matches backend `types::pm`) ─────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogEntryType {
    RoomType,
    ServiceSlot,
    PackageTier,
    SubscriptionTier,
    CoverageOption,
    AddOn,
    EquipmentUnit,
}

impl CatalogEntryType {
    const ALL: &'static [Self] = &[
        Self::RoomType,
        Self::ServiceSlot,
        Self::PackageTier,
        Self::SubscriptionTier,
        Self::CoverageOption,
        Self::AddOn,
        Self::EquipmentUnit,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::RoomType => "room_type",
            Self::ServiceSlot => "service_slot",
            Self::PackageTier => "package_tier",
            Self::SubscriptionTier => "subscription_tier",
            Self::CoverageOption => "coverage_option",
            Self::AddOn => "add_on",
            Self::EquipmentUnit => "equipment_unit",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::RoomType => "Room type",
            Self::ServiceSlot => "Service slot",
            Self::PackageTier => "Package",
            Self::SubscriptionTier => "Subscription",
            Self::CoverageOption => "Coverage",
            Self::AddOn => "Add-on",
            Self::EquipmentUnit => "Equipment",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingInterval {
    Nightly,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Annually,
    PerUnit,
}

impl BillingInterval {
    const ALL: &'static [Self] = &[
        Self::Nightly,
        Self::Hourly,
        Self::Daily,
        Self::Weekly,
        Self::Monthly,
        Self::Annually,
        Self::PerUnit,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::Nightly => "nightly",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Annually => "annually",
            Self::PerUnit => "per_unit",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Nightly => "Nightly",
            Self::Hourly => "Hourly",
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Annually => "Annually",
            Self::PerUnit => "Per unit",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s)
    }
}

// ── API DTOs ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: Uuid,
    pub entry_type: String,
    pub name: String,
    pub description: Option<String>,
    pub asset_id: Option<Uuid>,
    pub base_price_cents: i64,
    pub currency: String,
    pub billing_interval: Option<String>,
    pub is_available: bool,
    pub min_quantity: i32,
    pub max_quantity: Option<i32>,
    #[serde(default)]
    pub catalog_metadata: serde_json::Value,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateCatalogBody {
    entry_type: CatalogEntryType,
    name: String,
    description: Option<String>,
    asset_id: Option<Uuid>,
    base_price_cents: i64,
    currency: String,
    billing_interval: Option<BillingInterval>,
    min_quantity: Option<i32>,
    max_quantity: Option<i32>,
    catalog_metadata: Option<serde_json::Value>,
    sort_order: Option<i32>,
    cover_image_attachment_id: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CatalogFilter {
    All,
    Available,
    Unavailable,
}

impl CatalogFilter {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Available => "Available",
            Self::Unavailable => "Unavailable",
        }
    }
}

fn fmt_price(cents: i64, currency: &str, interval: Option<&str>) -> String {
    let amount = cents as f64 / 100.0;
    let money = if currency.eq_ignore_ascii_case("USD") {
        format!("${amount:.2}")
    } else {
        format!("{amount:.2} {currency}")
    };
    match interval.and_then(BillingInterval::parse) {
        Some(bi) => format!("{money} / {}", bi.label().to_lowercase()),
        None => money,
    }
}

fn entry_type_label(raw: &str) -> String {
    CatalogEntryType::parse(raw)
        .map(|t| t.label().to_string())
        .unwrap_or_else(|| raw.replace('_', " "))
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListCatalogEntries, "/api")]
pub async fn list_catalog_entries() -> Result<Vec<CatalogEntry>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    crate::atlas_client::authenticated_get_with_headers(
        "/api/folio/catalog",
        &token,
        None,
        proxy,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Catalog list failed: {e}")))
}

#[server(CreateCatalogEntry, "/api")]
pub async fn create_catalog_entry(
    name: String,
    entry_type: String,
    base_price_dollars: String,
    billing_interval: String,
    description: String,
) -> Result<CatalogEntry, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }
    let entry_type = CatalogEntryType::parse(entry_type.trim())
        .ok_or_else(|| ServerFnError::new("Invalid entry type"))?;
    let dollars: f64 = base_price_dollars
        .trim()
        .parse()
        .map_err(|_| ServerFnError::new("Price must be a number"))?;
    if dollars < 0.0 {
        return Err(ServerFnError::new("Price cannot be negative"));
    }
    let base_price_cents = (dollars * 100.0).round() as i64;
    let billing_interval = {
        let s = billing_interval.trim();
        if s.is_empty() || s == "one_time" {
            None
        } else {
            Some(
                BillingInterval::parse(s)
                    .ok_or_else(|| ServerFnError::new("Invalid billing interval"))?,
            )
        }
    };
    let description = {
        let d = description.trim();
        if d.is_empty() {
            None
        } else {
            Some(d.to_string())
        }
    };

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    let body = CreateCatalogBody {
        entry_type,
        name,
        description,
        asset_id: None,
        base_price_cents,
        currency: "USD".into(),
        billing_interval,
        min_quantity: Some(1),
        max_quantity: None,
        catalog_metadata: None,
        sort_order: None,
        cover_image_attachment_id: None,
    };
    crate::atlas_client::authenticated_post_with_headers(
        "/api/folio/catalog",
        &token,
        None,
        &body,
        proxy,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Create catalog entry failed: {e}")))
}

// ── Page ──────────────────────────────────────────────────────────────────────

#[component]
pub fn Catalog() -> impl IntoView {
    let entries = Resource::new(
        || (),
        |_| async move { list_catalog_entries().await },
    );
    let (filter, set_filter) = signal(CatalogFilter::All);
    let (search, set_search) = signal(String::new());
    let (show_create, set_show_create) = signal(false);
    let (creating, set_creating) = signal(false);
    let (create_err, set_create_err) = signal(Option::<String>::None);

    let name = RwSignal::new(String::new());
    let entry_type = RwSignal::new(CatalogEntryType::RoomType.as_str().to_string());
    let price = RwSignal::new(String::new());
    let interval = RwSignal::new(BillingInterval::Monthly.as_str().to_string());
    let description = RwSignal::new(String::new());

    let title = Signal::derive(|| "Catalog".to_string());
    let subtitle = Signal::derive(|| {
        "Product and service pricebook — listings, add-ons, and rate plans.".to_string()
    });

    let submit_create = move |_| {
        if creating.get() {
            return;
        }
        set_creating.set(true);
        set_create_err.set(None);
        let n = name.get();
        let et = entry_type.get();
        let p = price.get();
        let bi = interval.get();
        let d = description.get();
        leptos::task::spawn_local(async move {
            match create_catalog_entry(n, et, p, bi, d).await {
                Ok(_) => {
                    name.set(String::new());
                    price.set(String::new());
                    description.set(String::new());
                    set_show_create.set(false);
                    entries.refetch();
                }
                Err(e) => set_create_err.set(Some(e.to_string())),
            }
            set_creating.set(false);
        });
    };

    view! {
        <div class="landlord-list-page">
            <PageHeader title=title subtitle=subtitle>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary"
                    on:click=move |_| {
                        set_create_err.set(None);
                        set_show_create.update(|v| *v = !*v);
                    }
                >
                    <span class="material-symbols-outlined">"add"</span>
                    {move || if show_create.get() { "Cancel" } else { "Add entry" }}
                </button>
            </PageHeader>

            {move || show_create.get().then(|| view! {
                <div class="landlord-create-panel">
                    <h2 class="landlord-create-panel__title">"New catalog entry"</h2>
                    <div class="landlord-create-grid">
                        <label class="landlord-field">
                            <span>"Name"</span>
                            <input
                                type="text"
                                prop:value=move || name.get()
                                on:input=move |e| name.set(event_target_value(&e))
                                placeholder="e.g. 2BR unit — LTR"
                            />
                        </label>
                        <label class="landlord-field">
                            <span>"Type"</span>
                            <select
                                prop:value=move || entry_type.get()
                                on:change=move |e| entry_type.set(event_target_value(&e))
                            >
                                {CatalogEntryType::ALL.iter().map(|t| {
                                    let v = t.as_str();
                                    let l = t.label();
                                    view! { <option value=v>{l}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="landlord-field">
                            <span>"Base price (USD)"</span>
                            <input
                                type="number"
                                min="0"
                                step="0.01"
                                prop:value=move || price.get()
                                on:input=move |e| price.set(event_target_value(&e))
                                placeholder="1850"
                            />
                        </label>
                        <label class="landlord-field">
                            <span>"Billing"</span>
                            <select
                                prop:value=move || interval.get()
                                on:change=move |e| interval.set(event_target_value(&e))
                            >
                                <option value="one_time">"One-time"</option>
                                {BillingInterval::ALL.iter().map(|b| {
                                    let v = b.as_str();
                                    let l = b.label();
                                    view! { <option value=v>{l}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="landlord-field landlord-field--wide">
                            <span>"Description"</span>
                            <input
                                type="text"
                                prop:value=move || description.get()
                                on:input=move |e| description.set(event_target_value(&e))
                                placeholder="Optional"
                            />
                        </label>
                    </div>
                    {move || create_err.get().map(|e| view! {
                        <p class="landlord-create-error">{e}</p>
                    })}
                    <button
                        type="button"
                        class="folio-btn folio-btn--primary"
                        disabled=move || creating.get()
                        on:click=submit_create
                    >
                        {move || if creating.get() { "Saving…" } else { "Save entry" }}
                    </button>
                </div>
            })}

            <div class="landlord-filter-bar">
                <div class="landlord-search-wrap">
                    <span class="material-symbols-outlined landlord-search-icon">"search"</span>
                    <input
                        class="landlord-search-input"
                        type="search"
                        placeholder="Search by name or type…"
                        on:input=move |e| set_search.set(event_target_value(&e))
                    />
                </div>
                <div class="landlord-filter-chips">
                    {[CatalogFilter::All, CatalogFilter::Available, CatalogFilter::Unavailable]
                        .into_iter()
                        .map(|f| view! {
                            <button
                                type="button"
                                class=move || if filter.get() == f {
                                    "landlord-chip landlord-chip--active"
                                } else {
                                    "landlord-chip"
                                }
                                on:click=move |_| set_filter.set(f)
                            >
                                {f.label()}
                            </button>
                        })
                        .collect_view()}
                </div>
            </div>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading catalog…"</p></div>
            }>
                {move || entries.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load catalog"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let q = search.get().to_lowercase();
                        let f = filter.get();
                        let filtered: Vec<_> = all.into_iter().filter(|e| {
                            let avail_ok = match f {
                                CatalogFilter::All => true,
                                CatalogFilter::Available => e.is_available,
                                CatalogFilter::Unavailable => !e.is_available,
                            };
                            let search_ok = q.is_empty()
                                || e.name.to_lowercase().contains(&q)
                                || e.entry_type.to_lowercase().contains(&q)
                                || e.description.as_deref().unwrap_or("").to_lowercase().contains(&q);
                            avail_ok && search_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"menu_book"</span>
                                    <p class="folio-empty__heading">"No catalog entries yet"</p>
                                    <p class="folio-empty__sub">
                                        "Add a room type, add-on, or service so quotes and listings can pull prices from the pricebook."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="landlord-table-wrap">
                                    <table class="landlord-table">
                                        <thead>
                                            <tr>
                                                <th>"Name"</th>
                                                <th>"Type"</th>
                                                <th>"Price"</th>
                                                <th>"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {filtered.into_iter().map(|e| {
                                                let type_label = entry_type_label(&e.entry_type);
                                                let price = fmt_price(
                                                    e.base_price_cents,
                                                    &e.currency,
                                                    e.billing_interval.as_deref(),
                                                );
                                                let status_class = if e.is_available {
                                                    "landlord-pill landlord-pill--ok"
                                                } else {
                                                    "landlord-pill landlord-pill--muted"
                                                };
                                                let status = if e.is_available { "Available" } else { "Unavailable" };
                                                let name = e.name.clone();
                                                let desc = e.description.clone().unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <div class="landlord-table__primary">{name}</div>
                                                            {(!desc.is_empty()).then(|| view! {
                                                                <div class="landlord-table__meta">{desc}</div>
                                                            })}
                                                        </td>
                                                        <td>{type_label}</td>
                                                        <td>{price}</td>
                                                        <td><span class=status_class>{status}</span></td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
