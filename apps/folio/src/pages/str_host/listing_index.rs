// apps/folio/src/pages/str_host/listing_index.rs
//
// STR Host — Listing Index — /s/listings
//
// Grid view of all the STR host's listings (across all channels).
// The sidebar "Listings" nav link lands here; individual listing detail is at
// /s/listings/:id (StrListingDetail, already implemented).
//
// Data: GET /api/folio/str/listings
//   Returns [{id, name, status, base_rate_cents, channel_count, photo_url,
//             rating, review_count, next_checkin, occupancy_30d}]
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrListingSummary {
    pub id: String,
    pub name: String,
    pub address: String,
    pub listing_type: String, // "apartment" | "house" | "cabin" | "villa" | etc.
    pub status: String,       // "active" | "draft" | "unlisted" | "blocked"
    pub base_rate_cents: i64,
    pub channel_count: u32,
    pub rating: Option<f64>,
    pub review_count: Option<u32>,
    pub next_checkin: Option<String>,
    pub occupancy_30d: Option<f64>,
    pub photo_url: Option<String>,
}

#[server(FetchStrListingIndex, "/api")]
pub async fn fetch_str_listing_index(
) -> Result<Vec<StrListingSummary>, server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let token = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                s.split(';').find_map(|p| {
                    let p = p.trim();
                    p.strip_prefix("session=").map(|t| t.to_string())
                })
            })
            .ok_or_else(|| server_fn::error::ServerFnError::new("No session"))?;
        crate::atlas_client::authenticated_get::<Vec<StrListingSummary>>(
            "/api/folio/str/listings",
            &token,
            None,
        )
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "active" => "ph-badge ph-badge--paid",
        "draft" => "ph-badge ph-badge--default",
        "unlisted" => "ph-badge ph-badge--overdue",
        "blocked" => "ph-badge ph-badge--overdue",
        _ => "ph-badge ph-badge--default",
    }
}

fn listing_type_icon(t: &str) -> &'static str {
    match t {
        "cabin" => "🌲",
        "villa" => "🏰",
        "apartment" => "🏢",
        "boat" => "⛵",
        "house" => "🏡",
        _ => "🏠",
    }
}

fn fmt_nightly(cents: i64) -> String {
    format!("${}/nt", cents / 100)
}

fn render_stars(r: f64) -> String {
    let full = r.floor() as usize;
    "★".repeat(full) + &"☆".repeat(5usize.saturating_sub(full))
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── fmt_nightly ──────────────────────────────────────────────────────────

    #[test]
    fn fmt_nightly_typical() {
        assert_eq!(fmt_nightly(8500), "$85/nt");
    }

    #[test]
    fn fmt_nightly_round_hundred() {
        assert_eq!(fmt_nightly(10000), "$100/nt");
    }

    #[test]
    fn fmt_nightly_zero() {
        assert_eq!(fmt_nightly(0), "$0/nt");
    }

    #[test]
    fn fmt_nightly_sub_dollar_truncates() {
        // 99 cents → $0/nt (integer division intentional)
        assert_eq!(fmt_nightly(99), "$0/nt");
    }

    #[test]
    fn fmt_nightly_large_value() {
        assert_eq!(fmt_nightly(100_000), "$1000/nt");
    }

    // ── render_stars ─────────────────────────────────────────────────────────

    #[test]
    fn render_stars_five_full() {
        assert_eq!(render_stars(5.0), "★★★★★");
    }

    #[test]
    fn render_stars_zero() {
        assert_eq!(render_stars(0.0), "☆☆☆☆☆");
    }

    #[test]
    fn render_stars_four() {
        assert_eq!(render_stars(4.0), "★★★★☆");
    }

    #[test]
    fn render_stars_fractional_floors() {
        // 3.9 should floor to 3 filled stars
        assert_eq!(render_stars(3.9), "★★★☆☆");
    }

    #[test]
    fn render_stars_one() {
        assert_eq!(render_stars(1.0), "★☆☆☆☆");
    }

    #[test]
    fn render_stars_total_len_always_five() {
        for r in [0.0f64, 1.0, 2.5, 3.9, 5.0] {
            let s = render_stars(r);
            // Each ★/☆ is 3 bytes in UTF-8, so total chars = 5
            assert_eq!(s.chars().count(), 5, "failed for r={r}: got {s:?}");
        }
    }

    // ── status_badge_class ───────────────────────────────────────────────────

    #[test]
    fn status_badge_class_active() {
        assert_eq!(status_badge_class("active"), "ph-badge ph-badge--paid");
    }

    #[test]
    fn status_badge_class_draft() {
        assert_eq!(status_badge_class("draft"), "ph-badge ph-badge--default");
    }

    #[test]
    fn status_badge_class_unlisted() {
        assert_eq!(status_badge_class("unlisted"), "ph-badge ph-badge--overdue");
    }

    #[test]
    fn status_badge_class_blocked() {
        assert_eq!(status_badge_class("blocked"), "ph-badge ph-badge--overdue");
    }

    #[test]
    fn status_badge_class_unknown_falls_back() {
        // Any unknown status gets the default badge
        assert_eq!(
            status_badge_class("some_future_status"),
            "ph-badge ph-badge--default"
        );
    }

    // ── listing_type_icon ────────────────────────────────────────────────────

    #[test]
    fn listing_type_icon_known_types() {
        assert_eq!(listing_type_icon("cabin"), "🌲");
        assert_eq!(listing_type_icon("villa"), "🏰");
        assert_eq!(listing_type_icon("apartment"), "🏢");
        assert_eq!(listing_type_icon("boat"), "⛵");
        assert_eq!(listing_type_icon("house"), "🏡");
    }

    #[test]
    fn listing_type_icon_unknown_falls_back() {
        assert_eq!(listing_type_icon("yurt"), "🏠");
        assert_eq!(listing_type_icon(""), "🏠");
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrListingIndex() -> impl IntoView {
    let filter = RwSignal::new("all".to_string());
    let navigate = use_navigate();

    let listings_res = Resource::new(|| (), |_| fetch_str_listing_index());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"My Listings"</h1>
                    <p class="page-subtitle">"Manage your short-term rental properties"</p>
                </div>
                <div class="page-header-actions">
                    <a href="/s/listings/new" class="btn btn-primary">"+ New Listing"</a>
                </div>
            </div>

            // ── Status filter tabs ──
            <div class="owner-tabs" style="margin-bottom:1rem;">
                {["all", "active", "draft", "unlisted"].iter().map(|s| {
                    let s = *s;
                    let label = match s {
                        "all"      => "All",
                        "active"   => "✓ Active",
                        "draft"    => "⟳ Draft",
                        "unlisted" => "⊘ Unlisted",
                        _          => s,
                    };
                    view! {
                        <button
                            class=move || format!("owner-tab {}", if filter.get() == s { "owner-tab--active" } else { "" })
                            on:click=move |_| filter.set(s.to_string())
                        >{label}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading listings…"</div> }>
                {move || listings_res.get().map(|res| {
                    match res {
                        Ok(listings) => {
                            let status_q = filter.get();
                            let visible: Vec<_> = listings.iter()
                                .filter(|l| status_q == "all" || l.status == status_q)
                                .collect();

                            if visible.is_empty() {
                                return view! {
                                    <div class="doc-empty">
                                        <div class="doc-empty-icon">"🏠"</div>
                                        <div>"No listings found. Create your first listing to get started."</div>
                                        <a href="/s/listings/new" class="btn btn-primary" style="margin-top:.75rem;">"+ New Listing"</a>
                                    </div>
                                }.into_any();
                            }

                            view! {
                                <div class="str-listing-index-grid">
                                    {visible.iter().map(|l| {
                                        let icon      = listing_type_icon(&l.listing_type);
                                        let nightly   = fmt_nightly(l.base_rate_cents);
                                        let status_cls = status_badge_class(&l.status);
                                        let status_lbl = l.status.clone();
                                        let name       = l.name.clone();
                                        let addr       = l.address.clone();
                                        let channels   = l.channel_count;
                                        let stars      = l.rating.map(|r| render_stars(r)).unwrap_or_default();
                                        let revs       = l.review_count.map(|n| format!(" ({n})")).unwrap_or_default();
                                        let occ        = l.occupancy_30d.map(|o| format!("{:.0}%", o * 100.0)).unwrap_or_else(|| "—".to_string());
                                        let checkin    = l.next_checkin.clone().unwrap_or_else(|| "No upcoming bookings".to_string());
                                        let detail_href = format!("/s/listings/{}", l.id);
                                        view! {
                                            <div class="str-listing-index-card">
                                                <div class="str-listing-index-photo">
                                                    <span class="str-listing-index-icon">{icon}</span>
                                                    <span class={status_cls} style="position:absolute;top:.5rem;right:.5rem;font-size:.7rem;">{status_lbl}</span>
                                                </div>
                                                <div class="str-listing-index-body">
                                                    <div class="str-listing-index-name">{name}</div>
                                                    <div class="str-listing-index-addr text-xs text-on-surface-variant">{addr}</div>

                                                    {if !stars.is_empty() { view! {
                                                        <div class="str-listing-index-rating">
                                                            <span style="color:#fbbf24;font-size:.85rem;">{stars}</span>
                                                            <span class="text-xs text-on-surface-variant">{revs}</span>
                                                        </div>
                                                    }.into_any() } else { ().into_any() }}

                                                    <div class="str-listing-index-stats">
                                                        <div class="str-listing-stat">
                                                            <span class="str-listing-stat-label">"Rate"</span>
                                                            <span class="str-listing-stat-val" style="color:#4ade80;">{nightly}</span>
                                                        </div>
                                                        <div class="str-listing-stat">
                                                            <span class="str-listing-stat-label">"Occ. (30d)"</span>
                                                            <span class="str-listing-stat-val">{occ}</span>
                                                        </div>
                                                        <div class="str-listing-stat">
                                                            <span class="str-listing-stat-label">"Channels"</span>
                                                            <span class="str-listing-stat-val">{channels.to_string()}</span>
                                                        </div>
                                                    </div>

                                                    <div class="str-listing-index-checkin text-xs">
                                                        <span style="color:var(--on-surface-variant);">"Next check-in: "</span>
                                                        {checkin}
                                                    </div>
                                                </div>
                                                <div class="str-listing-index-footer">
                                                    <a href=detail_href class="btn btn-primary btn-sm">"Manage →"</a>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <div class="doc-empty">"Could not load listings. Please try again."</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
