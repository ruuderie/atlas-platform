# PRD: Atlas Platform Lead Generation Engine

## 1. Executive Summary
**Objective:** Evolve the Atlas Platform from a passive business directory into an active, monetized lead distribution and client management engine targeting high-ticket local service businesses (e.g., Lawyers, Surgeons). 
**Key Goals:**
- Capture high CPC/CPA budgets by providing exclusive, verified leads or premium directory positioning.
- Prevent vendor lock-in for critical infrastructure (telephony/communications).
- Maximize Customer Lifetime Value (LTV) through native, subtle cross-selling of additional B2B services inside the client portal.

---

## 2. Core Feature Requirements

### 2.1 Lead Ingestion & Verification Engine
**Description:** The system must securely accept, sanitize, and verify lead data before it enters the routing logic.
*   **Req 2.1.1 - Unified Ingestion API:** A robust REST endpoint (`/api/v1/leads/ingest`) that accepts standard payloads (Name, Phone, Email, Zip Code, Intent/Notes) from web forms, webhooks (e.g., ManyChat, Zapier), or direct manual entry.
*   **Req 2.1.2 - Deduplication & Exclusivity:** The system must check the database to ensure the lead is unique (based on phone/email) within a customizable timeframe (e.g., 30 days) to guarantee exclusivity to the buyer.
*   **Req 2.1.3 - Spam Processing:** Integration with a basic spam/bot-protection layer (e.g., reCAPTCHA scores on forms, honeypot fields).

### 2.2 Geographic Routing & Assignment Logic
**Description:** Leads must be instantly routed to the correct paying client based on their geographic location.
*   **Req 2.2.1 - Spatial Data Models:** Business profiles require a defined "Service Area" (array of zip codes or a radius from a lat/lng centroid using PostGIS).
*   **Req 2.2.2 - Instant Matchmaking:** Upon ingestion, the engine queries for active, paying clients whose service area overlaps the lead's zip code.
*   **Req 2.2.3 - Priority Queuing:** If multiple clients match a zip code, the system must support routing logic (e.g., Round-Robin, Highest Bidder, or "First to Claim" SMS broadcast).

### 2.3 Automated Billing & Wallet System (Stripe)
**Description:** Frictionless, automated payment collection based on lead delivery or subscription status.
*   **Req 2.3.1 - Managed Wallets:** Clients must add a credit card on file upon registration. The system stores a Stripe Customer ID and Payment Method.
*   **Req 2.3.2 - Usage-Based Billing:** When a lead is successfully assigned to a client, an event triggers an asynchronous Stripe charge for the predefined CPL (Cost Per Lead).
*   **Req 2.3.3 - Ledger & Transparency:** The client portal must display a ledger of all charges matched to the specific lead ID to prevent chargebacks and build trust.

---

## 3. Technical Architecture: Communications Abstraction Layer

To avoid lock-in with unified communications providers (like Twilio, Plivo, Telnyx), the platform must utilize an abstraction layer.

*   **Req 3.1.1 - Generic Provider Trait (Adapter Pattern):** Create a generic interface in the Rust backend for telephony operations. Business logic must *only* interact with this interface, never the specific provider's SDK.
    ```rust
    // Conceptual Example
    pub trait TelephonyProvider {
        async fn provision_number(&self, area_code: &str) -> Result<PhoneNumber, Error>;
        async fn send_sms(&self, to: &str, body: &str) -> Result<(), Error>;
        async fn get_call_logs(&self, number: &str, since: DateTime) -> Result<Vec<CallLog>, Error>;
    }
    ```
*   **Req 3.1.2 - Configurable Adapters:** Implement the trait for multiple providers (e.g., `TwilioAdapter`, `TelnyxAdapter`, `SignalWireAdapter`).
*   **Req 3.1.3 - Environment-Driven Provider Selection:** The active provider must be determined by environment variables or database settings on app boot. This allows seamless switching to a cheaper provider (like Telnyx) without altering business logic.
*   **Req 3.1.4 - Standardized Webhook Ingestion:** Create an intermediary webhook normalizer. When a provider sends a "Call Completed" webhook, the normalizer translates the vendor-specific JSON payload into a standard internal `CallEvent` struct before passing it to the billing engine.

---

## 4. Monetization & Upsell: Subtle Native B2B Advertising

**Description:** Native ad placements within the Atlas admin portal to cross-sell your other B2B services (e.g., SEO, Legal Receptionists, Website Design) without feeling "spammy".

*   **Req 4.1.1 - Contextual "Zero-State" Banners:**
    *   *Trigger:* When a client has no leads or has exhausted their monthly budget.
    *   *Placement:* Main dashboard.
    *   *Copy Example:* "Looking to dominate local SEO so you don't have to buy leads from us forever? View our Website Optimization packages."
*   **Req 4.1.2 - The "Recommended Partners" Side-Panel:**
    *   *Trigger:* Permanent fixture in the navigation sidebar or dashboard sidebar.
    *   *Concept:* A curated list of tools/services. Looks like a helpful resource list, but acts as a direct upsell channel.
    *   *Example:* "Need help answering these calls? We recommend [Your White-Labeled Receptionist Service]."
*   **Req 4.1.3 - Lifecycle Milestone Modals:**
    *   *Trigger:* When a client successfully buys their 10th lead.
    *   *Concept:* A dismissible congratulatory modal that packages a pitch.
    *   *Example:* "You've successfully secured 10 leads! Make sure you are closing them at the highest rate possible with our *Automated SMS Follow-Up CRM* module. Enable it for $49/mo."
*   **Req 4.1.4 - Action-Driven Upsells:**
    *   *Trigger:* When viewing an unclosed lead detail page.
    *   *Concept:* Subtle text link below the lead's contact info.
    *   *Example:* "? Having trouble reaching this lead? Let our dedicated intake team chase them down for you." 
*   **Req 4.1.5 - Ad Tracking & Analytics:** The backend must log clicks and impressions on these internal promotions to determine which upsells convert best.
