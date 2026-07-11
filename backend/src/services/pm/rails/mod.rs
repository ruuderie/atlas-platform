//! PM payment rail adapter implementations.
//!
//! Each module implements [`crate::services::pm::payment_rail::PaymentRailAdapter`]
//! for a specific credential type stored in `atlas_payment_credentials`.
//!
//! | Module            | Credential type          | Market              |
//! |-------------------|--------------------------|---------------------|
//! | `stripe_connect`  | stripe_connect_express / standard | US, global |
//! | `infinitepay`     | pix_key                  | Brazil (BR)         |
//! | `bitcoin_onchain` | btc_onchain_address      | Global              |
//! | `lightning`       | btc_lightning_node       | Global              |
//! | `kelviq`          | kelviq                   | Caribbean / USVI    |

pub mod bitcoin_onchain;
pub mod infinitepay;
pub mod kelviq;
pub mod lightning;
pub mod stripe_connect;
