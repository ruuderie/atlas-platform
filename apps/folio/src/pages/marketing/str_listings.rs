// apps/folio/src/pages/marketing/str_listings.rs
//
// STR Embedded Listings — /listings/str
//
// Public listing search page for Short-Term Rentals. Embeddable via
// <iframe> in Network Instance sites (add ?embed=1).
// Features: date-range picker, guest count, listing card grid.
// Links to /leads/:token for inquiry, /s/listings/:id for host detail.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrListingSummary {
    pub asset_id:        String,
    pub token:           String,
    pub name:            String,
    pub address:         String,
    pub city:            String,
    pub state:           String,
    pub listing_type:    String,    // "cabin" | "apartment" | "villa" | "house" | "boat" | "other"
    pub max_guests:      u32,
    pub bedrooms:        u32,
    pub bathrooms:       f64,
    pub base_rate_cents: i64,
    pub rating:          Option<f64>,
    pub review_count:    Option<u32>,
    pub amenities:       Vec<String>,
    pub photo_url:       Option<String>,
    pub is_available:    bool,
    pub platform_tags:   Vec<String>,  // "pet_friendly", "pool", "beachfront" etc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrSearchResult {
    pub listings:   Vec<StrListingSummary>,
    pub total_count: i64,
    pub page:        i64,
    pub per_page:    i64,
}

#[server(SearchStrListings, "/api")]
pub async fn search_str_listings(
    city:       String,
    check_in:   String,
    check_out:  String,
    guests:     u32,
    listing_type: String,
    page:       i64,
) -> Result<StrSearchResult, server_fn::error::ServerFnError> {
    let q = format!(
        "/api/pub/listings/str?city={city}&check_in={check_in}&check_out={check_out}&guests={guests}&listing_type={listing_type}&page={page}&per_page=12"
    );
    crate::atlas_client::authenticated_get::<StrSearchResult>(&q, "", None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn str_type_icon(t: &str) -> &'static str {
    match t {
        "cabin"     => "🌲",
        "villa"     => "🏰",
        "boat"      => "⛵",
        "apartment" => "🏢",
        "house"     => "🏡",
        _           => "🏠",
    }
}

fn render_stars(rating: f64) -> String {
    let full  = rating.floor() as usize;
    let half  = if rating - rating.floor() >= 0.5 { 1 } else { 0 };
    let empty = 5usize.saturating_sub(full + half);
    "★".repeat(full) + &"½".repeat(half) + &"☆".repeat(empty)
}

fn fmt_nightly(cents: i64) -> String {
    format!("${}/nt", cents / 100)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrListings() -> impl IntoView {
    let query = use_query_map();
    let q = query.get();
    let is_embed  = q.get("embed").map(|v| v == "1").unwrap_or(false);

    let city = RwSignal::new(q.get("city").unwrap_or_default());
    let check_in = RwSignal::new(q.get("check_in").unwrap_or_default());
    let check_out = RwSignal::new(q.get("check_out").unwrap_or_default());
    let guests        = RwSignal::new(2u32);
    let listing_type  = RwSignal::new("".to_string());
    let page          = RwSignal::new(1i64);

    let res = Resource::new(
        move || (city.get(), check_in.get(), check_out.get(), guests.get(), listing_type.get(), page.get()),
        |(c, ci, co, g, lt, p)| search_str_listings(c, ci, co, g, lt, p),
    );

    view! {
        <div class=if is_embed { "listings-embed" } else { "listings-standalone" }>
            {if !is_embed { view! {
                <div class="listings-hero" style="background:linear-gradient(135deg,rgba(10,132,255,.15),rgba(139,92,246,.1));">
                    <div class="listings-hero-title">"Short-Term Rentals"</div>
                    <div class="listings-hero-sub">"Unique stays for every occasion"</div>
                </div>
            }.into_any() } else { ().into_any() }}

            // ── Filter bar ──
            <div class="listings-filter-bar str-filter-bar">
                <div class="listings-filter-group">
                    <input type="text" class="form-input listings-filter-input" placeholder="Destination…"
                        prop:value=move || city.get()
                        on:input=move |ev| { city.set(event_target_value(&ev)); page.set(1); }
                    />
                </div>
                <div class="listings-filter-group">
                    <input type="date" class="form-input listings-filter-input" placeholder="Check-in"
                        prop:value=move || check_in.get()
                        on:input=move |ev| { check_in.set(event_target_value(&ev)); page.set(1); }
                    />
                </div>
                <div class="listings-filter-group">
                    <input type="date" class="form-input listings-filter-input" placeholder="Check-out"
                        prop:value=move || check_out.get()
                        on:input=move |ev| { check_out.set(event_target_value(&ev)); page.set(1); }
                    />
                </div>
                <div class="listings-filter-group" style="max-width:6rem;">
                    <input type="number" class="form-input listings-filter-input" placeholder="Guests" min="1" max="30"
                        prop:value=move || guests.get().to_string()
                        on:input=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<u32>() { guests.set(n); }
                            page.set(1);
                        }
                    />
                </div>
                <div class="listings-filter-group">
                    <select class="form-select listings-filter-input"
                        on:change=move |ev| { listing_type.set(event_target_value(&ev)); page.set(1); }
                    >
                        <option value="">"All types"</option>
                        <option value="cabin">"🌲 Cabin"</option>
                        <option value="villa">"🏰 Villa"</option>
                        <option value="apartment">"🏢 Apartment"</option>
                        <option value="house">"🏡 House"</option>
                        <option value="boat">"⛵ Boat"</option>
                    </select>
                </div>
                <button class="btn btn-primary listings-search-btn"
                    on:click=move |_| { page.set(1); res.refetch(); }
                >"Search"</button>
            </div>

            // ── Results ──
            <Suspense fallback=|| view! { <div class="listings-loading">"Searching stays…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(data) => {
                            let total = data.total_count;
                            let pages = (total as f64 / data.per_page as f64).ceil() as i64;
                            let cur_page = data.page;
                            view! {
                                <div>
                                    <div class="listings-result-count">{total.to_string()} " stay" {if total != 1 { "s" } else { "" }} " available"</div>
                                    <div class="listings-grid">
                                        {data.listings.iter().map(|l| {
                                            let icon       = str_type_icon(&l.listing_type);
                                            let nightly    = fmt_nightly(l.base_rate_cents);
                                            let stars_str  = l.rating.map(|r| render_stars(r)).unwrap_or_default();
                                            let rev_str    = l.review_count.map(|n| format!(" ({n})")).unwrap_or_default();
                                            let beds_str   = format!("{} bd · {:.0} ba · {} guests", l.bedrooms, l.bathrooms, l.max_guests);
                                            let name       = l.name.clone();
                                            let city_str   = format!("{}, {}", l.city, l.state);
                                            let amenities: Vec<String> = l.amenities.iter().take(4).cloned().collect();
                                            let tags: Vec<String>      = l.platform_tags.iter().take(2).cloned().collect();
                                            let contact_href = format!("/leads/{}", l.token);
                                            let avail = l.is_available;
                                            view! {
                                                <div class=if avail { "listing-card" } else { "listing-card listing-card--unavail" }>
                                                    {if !avail { view! { <div class="listing-card-unavail-overlay">"Unavailable for selected dates"</div> }.into_any() } else { ().into_any() }}
                                                    <div class="listing-card-photo str-listing-card-photo">
                                                        <div class="listing-card-type-icon str-listing-icon">{icon}</div>
                                                        {tags.iter().map(|t| view! {
                                                            <span class="str-listing-tag">{t.replace('_', " ")}</span>
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                    <div class="listing-card-body">
                                                        <div class="listing-card-address str-listing-name">{name}</div>
                                                        <div class="listing-card-city">{city_str}</div>
                                                        <div class="str-listing-beds">{beds_str}</div>
                                                        {if l.rating.is_some() { view! {
                                                            <div class="str-listing-rating">
                                                                <span class="str-stars">{stars_str}</span>
                                                                <span class="str-rev-count text-xs">{rev_str}</span>
                                                            </div>
                                                        }.into_any() } else { ().into_any() }}
                                                        {if !amenities.is_empty() {
                                                            view! {
                                                                <div class="listing-card-amenities">
                                                                    {amenities.iter().map(|a| view! { <span class="listing-amenity-chip">{a.clone()}</span> }).collect::<Vec<_>>()}
                                                                </div>
                                                            }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                    <div class="listing-card-footer">
                                                        <div class="listing-card-rent">{nightly}</div>
                                                        <div class="listing-card-actions">
                                                            <a href=contact_href class=if avail { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }>"Inquire"</a>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>

                                    {if total == 0 { view! {
                                        <div class="listings-empty">
                                            <div class="listings-empty-icon">"🏝"</div>
                                            <div class="listings-empty-title">"No stays available for your search"</div>
                                            <div class="text-sm text-on-surface-variant">"Try different dates or expand your destination."</div>
                                        </div>
                                    }.into_any() } else { ().into_any() }}

                                    {if pages > 1 { view! {
                                        <div class="listings-pagination">
                                            <button class="btn btn-ghost btn-sm" disabled=move || cur_page <= 1
                                                on:click=move |_| page.update(|p| *p -= 1)>"← Prev"</button>
                                            <span class="listings-page-info">"Page " {cur_page.to_string()} " of " {pages.to_string()}</span>
                                            <button class="btn btn-ghost btn-sm" disabled=move || cur_page >= pages
                                                on:click=move |_| page.update(|p| *p += 1)>"Next →"</button>
                                        </div>
                                    }.into_any() } else { ().into_any() }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <div class="listings-empty">
                                <div>"Could not load listings. Please try again."</div>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            {if !is_embed { view! {
                <div class="listings-footer">
                    "Powered by " <a href="/lp" class="listings-footer-link">"Atlas Platform"</a>
                </div>
            }.into_any() } else { ().into_any() }}
        </div>
    }
}
