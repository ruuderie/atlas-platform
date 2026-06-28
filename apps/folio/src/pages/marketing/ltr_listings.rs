// apps/folio/src/pages/marketing/ltr_listings.rs
//
// LTR Embedded Listings — /listings/ltr
//
// Public listing search/browse page for Long-Term Rentals. Embeddable via
// <iframe> in Network Instance sites. Also served directly at the Folio domain
// for operators who don't yet have a Network Instance.
//
// Features:
//   - Location / bed / bath / price filter bar
//   - Responsive listing card grid
//   - Pagination footer (page-based, not infinite scroll — better SSR)
//   - "Apply Now" deep link → /apply/:property_id
//   - "Contact" deep link → /leads/:listing_token
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LtrListing {
    pub asset_id:     String,
    pub token:        String,
    pub address:      String,
    pub city:         String,
    pub state:        String,
    pub bedrooms:     Option<u32>,
    pub bathrooms:    Option<f64>,
    pub sqft:         Option<u32>,
    pub rent_cents:   i64,
    pub available:    Option<String>,
    pub listing_type: String,    // "apartment" | "house" | "condo" | "townhome"
    pub amenities:    Vec<String>,
    pub photo_url:    Option<String>,
    pub is_featured:  bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LtrSearchResult {
    pub listings:     Vec<LtrListing>,
    pub total_count:  i64,
    pub page:         i64,
    pub per_page:     i64,
}

#[derive(Debug, Clone, Default)]
struct Filters {
    city:       String,
    beds:       String,
    max_rent:   String,
    page:       i64,
}

#[server(SearchLtrListings, "/api")]
pub async fn search_ltr_listings(
    city: String, beds: String, max_rent: String, page: i64,
) -> Result<LtrSearchResult, server_fn::error::ServerFnError> {
    let q = format!(
        "/api/pub/listings/ltr?city={city}&beds={beds}&max_rent_cents={max_rent}&page={page}&per_page=12"
    );
    crate::atlas_client::authenticated_get::<LtrSearchResult>(&q, "", None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_rent(cents: i64) -> String {
    let dollars = cents / 100;
    if dollars >= 1000 {
        format!("${},{:03}/mo", dollars / 1000, dollars % 1000)
    } else {
        format!("${}/mo", dollars)
    }
}

fn listing_type_icon(t: &str) -> &'static str {
    match t {
        "apartment"  => "🏢",
        "house"      => "🏡",
        "condo"      => "🏙",
        "townhome"   => "🏘",
        _            => "🏠",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LtrListings() -> impl IntoView {
    let query = use_query_map();
    let q = query.get();
    let init_city = q.get(0).unwrap_or_default();
    let init_beds = q.get(0).unwrap_or_default();
    let init_rent = q.get(0).unwrap_or_default();
    let is_embed  = q.get("embed").map(|v| v == "1").unwrap_or(false);

    let city     = RwSignal::new(init_city);
    let beds     = RwSignal::new(init_beds);
    let max_rent = RwSignal::new(init_rent);
    let page     = RwSignal::new(1i64);

    let res = Resource::new(
        move || (city.get(), beds.get(), max_rent.get(), page.get()),
        |(c, b, r, p)| search_ltr_listings(c, b, r, p),
    );

    view! {
        <div class=if is_embed { "listings-embed" } else { "listings-standalone" }>
            {if !is_embed { view! {
                <div class="listings-hero">
                    <div class="listings-hero-title">"Find Your Next Home"</div>
                    <div class="listings-hero-sub">"Long-term rentals powered by Atlas Platform"</div>
                </div>
            }.into_any() } else { ().into_any() }}

            // ── Filter bar ──
            <div class="listings-filter-bar">
                <div class="listings-filter-group">
                    <input type="text" class="form-input listings-filter-input" placeholder="City or zip…"
                        prop:value=move || city.get()
                        on:input=move |ev| { city.set(event_target_value(&ev)); page.set(1); }
                    />
                </div>
                <div class="listings-filter-group">
                    <select class="form-select listings-filter-input"
                        on:change=move |ev| { beds.set(event_target_value(&ev)); page.set(1); }
                    >
                        <option value="">"Any beds"</option>
                        <option value="1">"1+ bed"</option>
                        <option value="2">"2+ beds"</option>
                        <option value="3">"3+ beds"</option>
                        <option value="4">"4+ beds"</option>
                    </select>
                </div>
                <div class="listings-filter-group">
                    <select class="form-select listings-filter-input"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            // Convert display value to cents string
                            let cents = match v.as_str() {
                                "1500" => "150000", "2000" => "200000", "2500" => "250000",
                                "3000" => "300000", "4000" => "400000", "5000" => "500000",
                                _      => "",
                            };
                            max_rent.set(cents.to_string());
                            page.set(1);
                        }
                    >
                        <option value="">"Any price"</option>
                        <option value="1500">"Up to $1,500/mo"</option>
                        <option value="2000">"Up to $2,000/mo"</option>
                        <option value="2500">"Up to $2,500/mo"</option>
                        <option value="3000">"Up to $3,000/mo"</option>
                        <option value="4000">"Up to $4,000/mo"</option>
                        <option value="5000">"Up to $5,000/mo"</option>
                    </select>
                </div>
                <button class="btn btn-primary listings-search-btn"
                    on:click=move |_| { page.set(1); res.refetch(); }
                >"Search"</button>
            </div>

            // ── Results ──
            <Suspense fallback=|| view! { <div class="listings-loading">"Searching listings…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(data) => {
                            let total = data.total_count;
                            let pages = (total as f64 / data.per_page as f64).ceil() as i64;
                            let cur_page = data.page;
                            view! {
                                <div>
                                    <div class="listings-result-count">{total.to_string()} " rental" {if total != 1 { "s" } else { "" }} " found"</div>
                                    <div class="listings-grid">
                                        {data.listings.iter().map(|l| {
                                            let icon    = listing_type_icon(&l.listing_type);
                                            let rent    = fmt_rent(l.rent_cents);
                                            let beds_s  = l.bedrooms.map(|b| format!("{b} bd")).unwrap_or_default();
                                            let baths_s = l.bathrooms.map(|b| format!("{b} ba")).unwrap_or_default();
                                            let sqft_s  = l.sqft.map(|s| format!("{s} sqft")).unwrap_or_default();
                                            let avail   = l.available.clone().unwrap_or_else(|| "Now".to_string());
                                            let apply_href   = format!("/apply/{}", l.asset_id);
                                            let contact_href = format!("/leads/{}", l.token);
                                            let is_feat = l.is_featured;
                                            let city_s  = format!("{}, {}", l.city, l.state);
                                            let addr    = l.address.clone();
                                            let amenities: Vec<String> = l.amenities.iter().take(3).cloned().collect();
                                            view! {
                                                <div class=if is_feat { "listing-card listing-card--featured" } else { "listing-card" }>
                                                    {if is_feat { view! { <div class="listing-card-featured-badge">"⭐ Featured"</div> }.into_any() } else { ().into_any() }}
                                                    <div class="listing-card-photo">
                                                        <div class="listing-card-type-icon">{icon}</div>
                                                    </div>
                                                    <div class="listing-card-body">
                                                        <div class="listing-card-address">{addr}</div>
                                                        <div class="listing-card-city">{city_s}</div>
                                                        <div class="listing-card-meta">
                                                            {if !beds_s.is_empty()  { view! { <span class="listing-meta-chip">{beds_s}</span>  }.into_any() } else { ().into_any() }}
                                                            {if !baths_s.is_empty() { view! { <span class="listing-meta-chip">{baths_s}</span> }.into_any() } else { ().into_any() }}
                                                            {if !sqft_s.is_empty()  { view! { <span class="listing-meta-chip">{sqft_s}</span>  }.into_any() } else { ().into_any() }}
                                                        </div>
                                                        {if !amenities.is_empty() {
                                                            view! {
                                                                <div class="listing-card-amenities">
                                                                    {amenities.iter().map(|a| view! { <span class="listing-amenity-chip">{a.clone()}</span> }).collect::<Vec<_>>()}
                                                                </div>
                                                            }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                    <div class="listing-card-footer">
                                                        <div class="listing-card-rent">{rent}</div>
                                                        <div class="listing-card-avail">"Available: " {avail}</div>
                                                        <div class="listing-card-actions">
                                                            <a href=contact_href class="btn btn-ghost btn-sm">"Contact"</a>
                                                            <a href=apply_href  class="btn btn-primary btn-sm">"Apply"</a>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>

                                    {if total == 0 {
                                        view! {
                                            <div class="listings-empty">
                                                <div class="listings-empty-icon">"🏚"</div>
                                                <div class="listings-empty-title">"No listings match your filters"</div>
                                                <div class="text-sm text-on-surface-variant">"Try adjusting your search — expand the city, beds, or price range."</div>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}

                                    {if pages > 1 {
                                        view! {
                                            <div class="listings-pagination">
                                                <button
                                                    class="btn btn-ghost btn-sm"
                                                    disabled=move || cur_page <= 1
                                                    on:click=move |_| page.update(|p| *p -= 1)
                                                >"← Prev"</button>
                                                <span class="listings-page-info">"Page " {cur_page.to_string()} " of " {pages.to_string()}</span>
                                                <button
                                                    class="btn btn-ghost btn-sm"
                                                    disabled=move || cur_page >= pages
                                                    on:click=move |_| page.update(|p| *p += 1)
                                                >"Next →"</button>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <div class="listings-empty">
                                <div class="listings-empty-icon">"⚡"</div>
                                <div>"Could not load listings. Please try again."</div>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            {if !is_embed { view! {
                <div class="listings-footer">
                    "Powered by " <a href="/lp" class="listings-footer-link">"Atlas Platform"</a> " · " <a href="/apply" class="listings-footer-link">"Apply Online"</a>
                </div>
            }.into_any() } else { ().into_any() }}
        </div>
    }
}
