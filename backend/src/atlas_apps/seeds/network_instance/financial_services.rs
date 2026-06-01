#![allow(dead_code, unused_imports)]
use super::helpers::{ensure_category, ensure_network_type, ensure_subcategory, record_seed_application};
use crate::traits::atlas_app::AppSeedPack;

const SEED_ID: &str = "financial_services_starter";

pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "Financial Services Starter",
        description: "Seeds the Financial Services network type with categories covering business banking, loans & lending, and insurance products.",
        content_summary: "~3 parent categories, ~15 sub-categories",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                let nt_id = ensure_network_type(
                    &db,
                    "Financial Services",
                    "Network for various financial and lending services",
                )
                .await?;

                let categories: &[(&str, &str, &[&str])] = &[
                    ("Business Banking", "Banking services for businesses", &[
                        "Business Checking", "Business Savings", "Merchant Services",
                        "Payroll Services", "Business Credit Cards",
                    ]),
                    ("Loans & Lending", "Various loan and lending services", &[
                        "Home Loans", "Auto Loans", "Student Loans",
                        "Small Business Loans", "Commercial Real Estate Loans",
                    ]),
                    ("Insurance Services", "Various insurance products", &[
                        "Life Insurance", "Health Insurance", "Auto Insurance",
                        "Home Insurance", "Business Insurance",
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
