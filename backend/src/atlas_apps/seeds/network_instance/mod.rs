#![allow(dead_code, unused_imports)]
pub mod helpers;
pub mod transportation_logistics;
pub mod automotive_sales;
pub mod construction;
pub mod beauty_care;
pub mod financial_services;
pub mod healthcare;
pub mod general_starter;

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
