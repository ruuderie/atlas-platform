#![allow(dead_code, unused_imports)]
use super::helpers::{ensure_category, ensure_network_type, record_seed_application};
use crate::traits::atlas_app::AppSeedPack;
use sea_orm::{ConnectionTrait, Statement};

const SEED_ID: &str = "general_starter";

/// Cross-industry starter pack — seeds 3 network types and populates each with
/// a handful of representative listings. Useful for demos where you want to show
/// a multi-industry platform immediately without committing to one vertical.
pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "General Multi-Industry Starter",
        description: "Seeds three industry networks (Transportation & Logistics, Automotive Sales, Construction & Contracting) with sample listings across each. Ideal for platform demos.",
        content_summary: "3 network types, ~9 sample listings",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                // ── Transportation & Logistics ────────────────────────────────
                let tl_id = ensure_network_type(
                    &db,
                    "Transportation & Logistics",
                    "Network for transportation and logistics services",
                )
                .await?;
                let freight_id = ensure_category(
                    &db,
                    tl_id,
                    "Freight Services",
                    "Services related to freight transportation",
                )
                .await?;

                // ── Automotive ────────────────────────────────────────────────
                let auto_id = ensure_network_type(
                    &db,
                    "Automotive Sales",
                    "Network for automotive sales and dealerships",
                )
                .await?;
                let used_id =
                    ensure_category(&db, auto_id, "Used Vehicles", "Pre-owned vehicle sales")
                        .await?;

                // ── Construction ──────────────────────────────────────────────
                let con_id = ensure_network_type(
                    &db,
                    "Construction & Contracting",
                    "Network for construction and contracting services",
                )
                .await?;
                let spec_id = ensure_category(
                    &db,
                    con_id,
                    "Specialized Contracting",
                    "Trade-specific contracting services",
                )
                .await?;

                // Networks
                let networks: &[(&str, &str, uuid::Uuid)] = &[
                    (
                        "Global Logistics Network",
                        "Connecting logistics professionals worldwide",
                        tl_id,
                    ),
                    (
                        "Auto Dealer Hub",
                        "Connecting car buyers with trusted dealerships",
                        auto_id,
                    ),
                    (
                        "Builder Connect",
                        "Connecting construction professionals and clients",
                        con_id,
                    ),
                ];

                let mut network_ids: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new(); // (network_id, cat_id)

                for (name, desc, nt_id) in networks {
                    let default_cat = if *nt_id == tl_id {
                        freight_id
                    } else if *nt_id == auto_id {
                        used_id
                    } else {
                        spec_id
                    };

                    let row = db.query_one(Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!(
                            "INSERT INTO network (id, tenant_id, network_type_id, name, description, is_active, requires_approval, allow_reviews, created_at, updated_at)
                             VALUES (gen_random_uuid(), '{tenant_id}', '{nt_id}',
                                     $n${name}$n$, $d${desc}$d$,
                                     true, false, true, NOW(), NOW())
                             ON CONFLICT (tenant_id, name) DO UPDATE SET name = EXCLUDED.name
                             RETURNING id"
                        ),
                    ))
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("No row returned for network '{name}'"))?;

                    let nid: uuid::Uuid = row.try_get("", "id").map_err(|e| e.to_string())?;
                    network_ids.push((nid, default_cat));
                }

                // Sample listings — 3 per network
                let sample_listings: &[(&str, &str)] = &[
                    (
                        "Nationwide Freight Brokerage",
                        "Full-service freight brokerage connecting shippers and carriers across the US.",
                    ),
                    (
                        "2022 Toyota Camry LE",
                        "Clean title, 28k miles. One owner. Full service history available.",
                    ),
                    (
                        "CT Licensed Master Electrician",
                        "Commercial and residential electrical services. 24/7 emergency response.",
                    ),
                ];

                for (i, (nid, cat_id)) in network_ids.iter().enumerate() {
                    if let Some((title, desc)) = sample_listings.get(i) {
                        db.execute(Statement::from_string(
                            sea_orm::DatabaseBackend::Postgres,
                            format!(
                                "INSERT INTO listing (id, network_id, category_id, title, description,
                                                      listing_type, price, price_type, country, state, city,
                                                      status, is_active, created_at, updated_at)
                                 VALUES (gen_random_uuid(), '{nid}', '{cat_id}',
                                         $t${title}$t$, $d${desc}$d$,
                                         'service', 0, 'quote', 'USA', 'CT', 'Hartford',
                                         'approved', true, NOW(), NOW())
                                 ON CONFLICT DO NOTHING"
                            ),
                        ))
                        .await
                        .map_err(|e| format!("listing insert error: {e}"))?;
                    }
                }

                record_seed_application(&db, tenant_id, SEED_ID).await?;
                tracing::info!("Seed pack '{}' applied for tenant {}", SEED_ID, tenant_id);
                Ok(())
            })
        }),
    }
}
