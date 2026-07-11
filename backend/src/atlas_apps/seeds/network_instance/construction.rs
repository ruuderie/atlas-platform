#![allow(dead_code, unused_imports)]
use super::helpers::{
    ensure_category, ensure_network_type, ensure_subcategory, record_seed_application,
};
use crate::traits::atlas_app::AppSeedPack;
use sea_orm::{ConnectionTrait, Statement};
use uuid::Uuid;

const SEED_ID: &str = "construction_starter";

pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "Construction & Contracting Starter",
        description: "Seeds the Construction & Contracting network type with a full category tree, plus 24 sample Connecticut-based contractor listings covering HVAC, plumbing, electrical, and renovation services.",
        content_summary: "~5 parent categories, ~20 sub-categories, 24 sample listings",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                let nt_id = ensure_network_type(
                    &db,
                    "Construction & Contracting",
                    "Network for construction and contracting services",
                )
                .await?;

                // ── Categories ────────────────────────────────────────────────
                let categories: &[(&str, &str, &[&str])] = &[
                    (
                        "Residential Construction",
                        "Home building and renovation services",
                        &[
                            "New Home Construction",
                            "Home Renovations",
                            "Kitchen Remodeling",
                            "Bathroom Remodeling",
                            "Roofing",
                        ],
                    ),
                    (
                        "Commercial Construction",
                        "Commercial building services",
                        &[
                            "Office Buildings",
                            "Retail Spaces",
                            "Industrial Facilities",
                            "Healthcare Facilities",
                        ],
                    ),
                    (
                        "Specialized Contracting",
                        "Trade-specific contracting services",
                        &["Electrical", "Plumbing", "HVAC", "Landscaping", "Painting"],
                    ),
                    (
                        "Construction Equipment",
                        "Equipment sales and rental",
                        &[
                            "Heavy Machinery",
                            "Power Tools",
                            "Safety Equipment",
                            "Equipment Rental",
                        ],
                    ),
                    (
                        "Construction Management",
                        "Project and site management services",
                        &[
                            "Project Planning",
                            "Cost Estimation",
                            "Quality Control",
                            "Safety Management",
                        ],
                    ),
                ];

                let mut cat_ids: std::collections::HashMap<&str, Uuid> =
                    std::collections::HashMap::new();
                for (parent_name, parent_desc, subs) in categories {
                    let parent_id = ensure_category(&db, nt_id, parent_name, parent_desc).await?;
                    cat_ids.insert(parent_name, parent_id);
                    for sub in *subs {
                        ensure_subcategory(&db, nt_id, parent_id, sub).await?;
                    }
                }

                // ── Get or create a network scoped to this tenant ──────────────
                let network_id: Uuid = {
                    let existing = db.query_one(Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!(
                            "SELECT id FROM network WHERE tenant_id = '{tenant_id}' AND network_type_id = '{nt_id}' LIMIT 1"
                        ),
                    ))
                    .await
                    .map_err(|e| e.to_string())?;

                    if let Some(row) = existing {
                        row.try_get("", "id").map_err(|e| e.to_string())?
                    } else {
                        let row = db.query_one(Statement::from_string(
                            sea_orm::DatabaseBackend::Postgres,
                            format!(
                                "INSERT INTO network (id, tenant_id, network_type_id, name, description, is_active, requires_approval, allow_reviews, created_at, updated_at)
                                 VALUES (gen_random_uuid(), '{tenant_id}', '{nt_id}',
                                         'CT Build Pros', 'The premier network for top-rated construction and renovation services across Connecticut.',
                                         true, false, true, NOW(), NOW())
                                 RETURNING id"
                            ),
                        ))
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or_else(|| "Failed to insert network".to_string())?;
                        row.try_get("", "id").map_err(|e| e.to_string())?
                    }
                };

                // ── Sample listings ────────────────────────────────────────────
                let specialized_cat = cat_ids.get("Specialized Contracting").copied();
                let residential_cat = cat_ids.get("Residential Construction").copied();

                let businesses: &[(&str, &str, &str, &str)] = &[
                    (
                        "Apex CT Renovations",
                        "Premium renovation services across Connecticut. Licensed and fully insured.",
                        "New Haven",
                        "Residential Construction",
                    ),
                    (
                        "Elite HVAC Professionals",
                        "24/7 HVAC installation, repair, and maintenance for residential and commercial.",
                        "Stamford",
                        "Specialized Contracting",
                    ),
                    (
                        "Prime Wiring & Electric",
                        "Licensed electricians serving all of Connecticut. Commercial & residential.",
                        "Bridgeport",
                        "Specialized Contracting",
                    ),
                    (
                        "Sparkle Commercial Cleaning",
                        "Professional commercial cleaning services. Verified and insured.",
                        "Hartford",
                        "Specialized Contracting",
                    ),
                    (
                        "Scenic Views Outdoor",
                        "Full-service landscaping and outdoor living solutions.",
                        "Waterbury",
                        "Specialized Contracting",
                    ),
                    (
                        "Precision Plumbing CT",
                        "Emergency plumbing services available 24/7 across CT.",
                        "Danbury",
                        "Specialized Contracting",
                    ),
                    (
                        "Stamford Roofing Co.",
                        "Expert roofing installation and repair. 20+ years experience.",
                        "Stamford",
                        "Residential Construction",
                    ),
                    (
                        "Greenwich Build Partners",
                        "High-end residential and commercial construction.",
                        "Greenwich",
                        "Residential Construction",
                    ),
                    (
                        "New Haven Hardwood",
                        "Custom hardwood flooring installation and refinishing.",
                        "New Haven",
                        "Residential Construction",
                    ),
                    (
                        "Bridgeport Masonry",
                        "Expert masonry, concrete, and stonework services.",
                        "Bridgeport",
                        "Commercial Construction",
                    ),
                    (
                        "Fairfield Landscapes",
                        "Comprehensive landscape design and lawn care.",
                        "Fairfield",
                        "Specialized Contracting",
                    ),
                    (
                        "Hartford Heating & Air",
                        "Full HVAC services for homes and businesses.",
                        "Hartford",
                        "Specialized Contracting",
                    ),
                ];

                for (name, desc, city, category_name) in businesses {
                    let cat_id = cat_ids
                        .get(category_name)
                        .or(specialized_cat.as_ref())
                        .or(residential_cat.as_ref())
                        .copied()
                        .ok_or_else(|| format!("No category found for '{category_name}'"))?;

                    db.execute(Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!(
                            "INSERT INTO listing (id, network_id, category_id, title, description,
                                                  listing_type, price, price_type, country, state, city,
                                                  status, is_active, created_at, updated_at)
                             VALUES (gen_random_uuid(), '{network_id}', '{cat_id}',
                                     $name${}$name$, $desc${}$desc$,
                                     'service', 0, 'quote', 'USA', 'CT', $city${}$city$,
                                     'approved', true, NOW(), NOW())
                             ON CONFLICT DO NOTHING",
                            name, desc, city
                        ),
                    ))
                    .await
                    .map_err(|e| format!("listing insert error for '{name}': {e}"))?;
                }

                record_seed_application(&db, tenant_id, SEED_ID).await?;
                tracing::info!("Seed pack '{}' applied for tenant {}", SEED_ID, tenant_id);
                Ok(())
            })
        }),
    }
}
