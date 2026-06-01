#![allow(dead_code, unused_imports)]
use super::helpers::{ensure_category, ensure_network_type, ensure_subcategory, record_seed_application};
use crate::traits::atlas_app::AppSeedPack;

const SEED_ID: &str = "beauty_care_starter";

pub fn pack() -> AppSeedPack {
    AppSeedPack {
        id: SEED_ID,
        title: "Beauty & Personal Care Starter",
        description: "Seeds the Beauty & Personal Care network type with categories covering hair care, skin care, nail care, makeup, and spa & wellness services.",
        content_summary: "~5 parent categories, ~24 sub-categories",
        apply: Box::new(|db, tenant_id, _app_instance_id| {
            Box::pin(async move {
                let nt_id = ensure_network_type(
                    &db,
                    "Beauty & Personal Care",
                    "Network for beauty and personal care services",
                )
                .await?;

                let categories: &[(&str, &str, &[&str])] = &[
                    ("Hair Care", "Hair styling and treatment services", &[
                        "Hair Cutting", "Hair Coloring", "Hair Styling", "Hair Extensions", "Hair Treatments",
                    ]),
                    ("Skin Care", "Skin treatment and maintenance services", &[
                        "Facials", "Acne Treatments", "Anti-Aging Treatments", "Waxing", "Tanning",
                    ]),
                    ("Nail Care", "Nail styling and treatment services", &[
                        "Manicures", "Pedicures", "Nail Extensions", "Nail Art", "Nail Repair",
                    ]),
                    ("Makeup Services", "Makeup application and consultation", &[
                        "Bridal Makeup", "Special Event Makeup", "Makeup Lessons", "Permanent Makeup",
                    ]),
                    ("Spa & Wellness", "Relaxation and wellness services", &[
                        "Massage Therapy", "Body Treatments", "Hydrotherapy", "Aromatherapy", "Meditation Classes",
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
