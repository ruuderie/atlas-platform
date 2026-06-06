#![allow(dead_code, unused_imports)]
use super::helpers::{ensure_category, ensure_network_type, ensure_subcategory, record_seed_application};
use crate::traits::atlas_app::AppSeedPack;

const SEED_ID: &str = "transportation_logistics_starter";

pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "Transportation & Logistics Starter",
        description: "Seeds the Transportation & Logistics network type with a full category tree covering freight, warehousing, supply chain, courier services, and fleet management.",
        content_summary: "~5 parent categories, ~25 sub-categories",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                let nt_id = ensure_network_type(
                    &db,
                    "Transportation & Logistics",
                    "Network for transportation and logistics services",
                )
                .await?;

                // Parent categories and their sub-categories
                let categories: &[(&str, &str, &[&str])] = &[
                    ("Freight Services", "Services related to freight transportation", &[
                        "Air Freight", "Ocean Freight", "Road Freight", "Rail Freight", "Intermodal",
                    ]),
                    ("Warehousing", "Storage and warehousing services", &[
                        "Cold Storage", "Distribution Centers", "Fulfillment Centers",
                        "Cross-Docking", "Bonded Warehouses",
                    ]),
                    ("Supply Chain Management", "End-to-end supply chain solutions", &[
                        "Inventory Management", "Demand Planning", "Supplier Management",
                        "Logistics Technology", "Supply Chain Consulting",
                    ]),
                    ("Courier Services", "Package and document delivery services", &[
                        "Same-Day Delivery", "Next-Day Delivery", "International Shipping",
                        "Medical Courier", "Legal Document Delivery",
                    ]),
                    ("Fleet Management", "Vehicle fleet management services", &[
                        "Fleet Tracking", "Vehicle Maintenance", "Driver Management",
                        "Fuel Management", "Compliance & Safety",
                    ]),
                ];

                for (parent_name, parent_desc, subs) in categories {
                    let parent_id = ensure_category(&db, nt_id, parent_name, parent_desc).await?;
                    for sub in *subs {
                        ensure_subcategory(&db, nt_id, parent_id, sub).await?;
                    }
                }

                record_seed_application(&db, tenant_id, SEED_ID).await?;
                tracing::info!("Seed pack '{}' applied for tenant {}", SEED_ID, tenant_id);
                Ok(())
            })
        }),
    }
}
