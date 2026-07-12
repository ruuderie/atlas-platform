pub mod agent_wizard; // AgentWizard — /onboard/agent
pub mod broker_wizard; // BrokerWizard — /onboard/broker
pub mod cohost_wizard; // CohostWizard — /onboard/cohost
pub mod invite_codes_client; // Shared AcceptInviteCode server fn (used by all wizards)
pub mod invite_join; // InviteJoin — /join/:code
pub mod landlord_wizard; // LandlordWizard — /onboarding
pub mod otp_client;
pub mod owner_wizard; // OwnerWizard — /onboard/owner
pub mod pmc_wizard; // PmcWizard — /onboard/pmc
pub mod property_owner_wizard; // PropertyOwnerWizard — /onboard/property-owner
pub mod str_guest_wizard; // StrGuestWizard — /onboard/str-guest
pub mod tenant_wizard; // TenantApplicantWizard — /onboard/tenant (pending-onboard)
pub mod vendor_wizard; // VendorWizard — /onboard/vendor
pub mod wizard; // Legacy OnboardingWizard (kept for compatibility)

#[cfg(test)]
#[path = "email_contract_tests.rs"]
mod email_contract_tests;
