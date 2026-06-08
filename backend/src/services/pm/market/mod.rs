//! Folio — Market Configuration System
//!
//! Phase 1: infrastructure defined. Phase 2: wired to FolioApp::provision()
//! via MarketRegistry::build(). Until then, dead_code warnings are suppressed.
#![allow(dead_code, unused_imports)]

//! Defines the trait contract (`market_config`) and provides one concrete
//! implementation per operating market.
//!
//! # Module Structure
//!
//! ```text
//! services/pm/market/
//!   mod.rs           ← this file — public re-exports
//!   market_config.rs ← MarketConfig trait + sub-traits + MarketRegistry
//!   brazil.rs        ← BrazilMarket (Lei do Inquilinato, Serasa, BRL, IRRF)
//!   miami.rs         ← MiamiDadeMarket (FHA, TDT, STR Ordinance 2023-89)
//!   usvi.rs          ← UsViMarket (FHA, USVI Hotel Room Tax)
//! ```
//!
//! # Adding a new market
//!
//! 1. Create `{market}.rs` in this directory
//! 2. Implement `MarketConfig` (and required sub-traits)
//! 3. Add `pub mod {market};` below
//! 4. Push `Box::new({Market})` in `market_config::MarketRegistry::build()`
//! 5. Nothing else changes

pub mod market_config;
pub mod brazil;
pub mod miami;
pub mod usvi;

// Convenience re-exports — service code imports from here, not from sub-modules
pub use market_config::{
    MarketConfig, MarketRegistry,
    TenancyLaw, AntiDiscriminationLaw, StrRegulation, TaxEngine, CreditBureau,
};
