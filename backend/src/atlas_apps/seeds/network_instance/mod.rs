#![allow(dead_code, unused_imports)]
pub mod automotive_sales;
pub mod beauty_care;
pub mod construction;
pub mod financial_services;
pub mod general_starter;
pub mod healthcare;
pub mod helpers;
pub mod transportation_logistics;

use crate::traits::atlas_app::AppSeedPack;

/// Returns all seed packs available for the NetworkInstance app.
pub fn all_packs() -> Vec<AppSeedPack> {
    vec![
        transportation_logistics::pack(),
        automotive_sales::pack(),
        construction::pack(),
        beauty_care::pack(),
        financial_services::pack(),
        healthcare::pack(),
        general_starter::pack(),
    ]
}
