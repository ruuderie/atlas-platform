pub mod agent;
pub mod broker;
pub mod landlord;
pub mod login;
pub mod marketing;
pub mod not_found;
pub mod owner;
pub mod pmc;
pub mod str_host;
pub mod tenant;
pub mod vendor;
pub mod verify; // zero-auth SSR landing pages served at /lp/*

pub mod auth; // passkey_setup + future auth flows
pub mod onboarding; // first-run wizard
pub mod property_owner; // Property Owner Lite — free-tier self-registered owners
pub mod r#pub;
pub mod settings; // Zero-auth public pages: /help, /review/:id, etc.
