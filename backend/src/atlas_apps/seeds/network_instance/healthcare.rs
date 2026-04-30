use super::helpers::{ensure_category, ensure_network_type, ensure_subcategory, record_seed_application};
use crate::traits::atlas_app::AppSeedPack;

const SEED_ID: &str = "healthcare_starter";

pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "Healthcare Starter",
        description: "Seeds the Healthcare network type with categories covering medical services, pharmaceuticals, medical equipment, health insurance, and healthcare consulting.",
        content_summary: "~5 parent categories, ~16 sub-categories",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                let nt_id = ensure_network_type(
                    &db,
                    "Healthcare",
                    "Network for healthcare services and medical professionals",
                )
                .await?;

                let categories: &[(&str, &str, &[&str])] = &[
                    ("Medical Services", "Healthcare services and providers", &[
                        "Primary Care", "Specialty Care", "Emergency Services",
                        "Urgent Care", "Mental Health Services",
                        "Physical Therapy", "Chiropractic Services", "Dental Services",
                    ]),
                    ("Pharmaceuticals", "Pharmaceutical products and services", &[
                        "Pharmaceutical Manufacturing", "Pharmaceutical Distribution", "Pharmaceutical Retail",
                    ]),
                    ("Medical Equipment", "Medical equipment and supplies", &[
                        "Medical Imaging Equipment", "Medical Laboratory Equipment",
                        "Medical Supplies", "Medical Furniture",
                    ]),
                    ("Health Insurance", "Health insurance products and services", &[
                        "Individual Plans", "Group Plans", "Medicare Supplements", "Dental & Vision",
                    ]),
                    ("Healthcare Consulting", "Consulting services for healthcare organizations", &[
                        "Revenue Cycle Management", "Compliance Consulting",
                        "Healthcare IT", "Practice Management",
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
