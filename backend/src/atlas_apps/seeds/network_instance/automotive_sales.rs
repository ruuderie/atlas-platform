#![allow(dead_code, unused_imports)]
use super::helpers::{ensure_category, ensure_network_type, ensure_subcategory, record_seed_application};
use crate::traits::atlas_app::AppSeedPack;

const SEED_ID: &str = "automotive_sales_starter";

pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "Automotive Sales Starter",
        description: "Seeds the Automotive Sales network type with category trees for new vehicles, used vehicles, parts, services, and financing.",
        content_summary: "~5 parent categories, ~25 sub-categories",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                let nt_id = ensure_network_type(
                    &db,
                    "Automotive Sales",
                    "Network for automotive sales and dealerships",
                )
                .await?;

                let categories: &[(&str, &str, &[&str])] = &[
                    ("New Vehicles", "Brand new vehicle sales", &[
                        "Sedans", "SUVs & Crossovers", "Trucks", "Electric Vehicles", "Luxury Vehicles",
                    ]),
                    ("Used Vehicles", "Pre-owned vehicle sales", &[
                        "Certified Pre-Owned", "Economy Cars", "Classic Cars", "Commercial Vehicles", "Motorcycles",
                    ]),
                    ("Parts & Accessories", "Vehicle parts and accessories", &[
                        "OEM Parts", "Aftermarket Parts", "Tires & Wheels", "Audio & Electronics", "Performance Parts",
                    ]),
                    ("Auto Services", "Vehicle maintenance and repair", &[
                        "Oil Changes", "Brake Service", "Transmission Repair", "Body Shop", "Detailing",
                    ]),
                    ("Financing & Insurance", "Vehicle financing and insurance products", &[
                        "Auto Loans", "Lease Options", "Gap Insurance", "Extended Warranties", "Trade-In Evaluation",
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
