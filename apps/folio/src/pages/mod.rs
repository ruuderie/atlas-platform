pub mod landlord;
pub mod tenant;
pub mod vendor;
pub mod pmc;
pub mod owner;
pub mod str_host;
pub mod agent;
pub mod broker;
pub mod login;
pub mod verify;
pub mod not_found;
pub mod marketing;  // zero-auth SSR landing pages served at /lp/*

pub mod settings;
pub mod auth;         // passkey_setup + future auth flows
pub mod onboarding;   // first-run wizard
pub mod property_owner; // Property Owner Lite — free-tier self-registered owners
pub mod r#pub;           // Zero-auth public pages: /help, /review/:id, etc.
