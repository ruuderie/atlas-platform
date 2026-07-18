//! m20261024_folio_marketing_section_blocks — Folio marketing section overlays
//!
//! Seeds `blocks_payload` with typed section blocks consumed as overlays by the
//! Folio marketing pages. The frontend still renders the polished hardcoded
//! stacks and uses these blocks only to replace matching section content.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r##"
            WITH seed(app_id, blocks) AS (
                VALUES
                (
                    'folio',
                    $json$
                    [
                      {
                        "type": "nav_sections",
                        "items": [
                          {"label": "Features", "href": "#features"},
                          {"label": "How it works", "href": "#app-preview"},
                          {"label": "Pricing", "href": "#pricing"}
                        ]
                      },
                      {
                        "type": "stats",
                        "items": [
                          {"value": "5 min", "label": "Average setup time"},
                          {"value": "1 login", "label": "For your whole portfolio"},
                          {"value": "3", "label": "Countries at launch"},
                          {"value": "$0", "label": "Setup fee · no contracts"}
                        ]
                      },
                      {
                        "type": "personas",
                        "eyebrow": "Built for every role",
                        "heading": "One platform. Every person in the deal.",
                        "subhead": "Folio issues role-based portals so every role sees exactly what they need.",
                        "items": [
                          {"icon": "🏠", "title": "Independent Landlord", "subhead": "1–20 units", "accent": "coral", "bullets": ["Dashboard and reports", "Automated rent reminders", "Lease templates & e-sign", "Maintenance dispatch"]},
                          {"icon": "💼", "title": "Property Manager", "subhead": "Any size", "accent": "teal", "bullets": ["Multi-client portfolio", "Owner statements & reports", "Branded tenant portal", "Owner disbursement & fees"]},
                          {"icon": "🏨", "title": "Vacation Rental Host", "subhead": "Airbnb + direct", "accent": "gold", "bullets": ["Booking calendar", "Channel sync", "Guest messaging", "Vacation rental licensing & compliance"]},
                          {"icon": "🏡", "title": "Tenant", "subhead": "Renter portal", "accent": "green", "bullets": ["Pay rent online", "Submit maintenance requests", "View & sign lease", "Track move-in docs"]},
                          {"icon": "🔧", "title": "Vendor / Contractor", "subhead": "Work order portal", "accent": "orange", "bullets": ["Receive job dispatches", "Submit invoices", "Schedule management", "Marketplace profile"]}
                        ]
                      },
                      {
                        "type": "feature_grid",
                        "eyebrow": "What's included",
                        "heading": "From first lease to tax season — covered.",
                        "items": [
                          {"icon": "payments", "title": "Rent collection", "description": "Bank transfer, card, and international payment methods. Automatically records every payment."},
                          {"icon": "description", "title": "Lease management", "description": "Create, send, e-sign, renew, and store leases with templates for required disclosures."},
                          {"icon": "build", "title": "Maintenance tracking", "description": "Tenants report issues, you approve work, contractors receive jobs and send invoices."},
                          {"icon": "calendar_month", "title": "Vacation rental calendar", "description": "Airbnb, VRBO, Booking.com and direct bookings in one calendar."},
                          {"icon": "analytics", "title": "Portfolio dashboard", "description": "See income, vacancies, and maintenance costs across every property."}
                        ]
                      },
                      {
                        "type": "tenant_portal",
                        "eyebrow": "Tenant experience",
                        "heading": "Happy tenants pay on time. Give them a portal worth logging into.",
                        "subhead": "Tenants can pay rent, report problems, sign leases, and track move-in documents without calling you.",
                        "items": [
                          {"icon": "payments", "title": "Pay rent online", "desc": "Bank transfer, card, or local payment method."},
                          {"icon": "build", "title": "Maintenance requests", "desc": "Tenants describe the issue, upload a photo, and you get notified instantly."},
                          {"icon": "description", "title": "Lease & documents", "desc": "Tenants can read, sign, and download their lease anytime."}
                        ]
                      },
                      {
                        "type": "str_section",
                        "eyebrow": "Vacation rentals",
                        "heading": "Your vacation rental, fully under control.",
                        "subhead": "One calendar, one inbox, one platform for short-term rentals.",
                        "items": [
                          {"icon": "calendar_month", "title": "Booking calendar", "desc": "Direct and OTA bookings in one drag-and-drop calendar."},
                          {"icon": "verified_user", "title": "Permits & compliance", "desc": "Permit tracking, renewal reminders, and local registration support."},
                          {"icon": "payments", "title": "Collect directly from guests", "desc": "Take deposits, damage holds, and nightly rates from guests."}
                        ]
                      },
                      {
                        "type": "markets",
                        "eyebrow": "Where Folio works",
                        "heading": "Built for the Americas. Expanding from there.",
                        "items": [
                          {"flag": "🇺🇸", "name": "United States", "desc": "All 50 states · Federal fair housing · ACH + card"},
                          {"flag": "🇨🇦", "name": "Canada", "desc": "ON · BC · QC · PIPEDA-compliant · EFT rails"},
                          {"flag": "🇧🇷", "name": "Brazil", "desc": "LGPD-compliant · PIX payment rail"}
                        ]
                      },
                      {
                        "type": "payment_rails",
                        "eyebrow": "Rent collection",
                        "heading": "Rent collected. Split. Reported.",
                        "items": [
                          {"icon": "💳", "name": "ACH / EFT", "desc": "US and Canada bank transfers."},
                          {"icon": "⚡", "name": "PIX", "desc": "Brazil's instant payment rail."},
                          {"icon": "💰", "name": "Card", "desc": "Tenant pays the processing fee."}
                        ]
                      },
                      {
                        "type": "pricing_intro",
                        "eyebrow": "For landlords · your own properties",
                        "heading": "Simple. Transparent. No surprises.",
                        "subtitle": "Start free. Pay as you grow. Built for landlords managing their own portfolio.",
                        "audience_callout": "Managing properties for other owners? See Property Manager pricing."
                      },
                      {
                        "type": "cta",
                        "eyebrow": "Limited beta spots available",
                        "heading": "Be one of the first landlords inside.",
                        "subhead": "Join the waitlist now and get exclusive early access to Folio before we open to the public.",
                        "button_label": "Reserve my beta spot →",
                        "button_href": "#waitlist-wrap"
                      },
                      {
                        "type": "beta_strip",
                        "title": "Apply for the Folio Beta Program",
                        "body": "Get free access during beta in exchange for real feedback.",
                        "button_label": "Apply now",
                        "button_href": "/beta"
                      },
                      {
                        "type": "footer",
                        "tagline": "Modern Landlord OS",
                        "links": [
                          {"label": "Sign in", "href": "/login"},
                          {"label": "Pricing", "href": "#pricing"},
                          {"label": "Features", "href": "#features"}
                        ]
                      }
                    ]
                    $json$::jsonb
                ),
                (
                    'folio-broker',
                    $json$
                    [
                      {"type": "nav_sections", "items": [{"label": "Features", "href": "#broker-features"}, {"label": "How it works", "href": "#broker-app-preview"}, {"label": "Pricing", "href": "#broker-pricing"}]},
                      {"type": "feature_grid", "eyebrow": "The platform", "heading": "Built for the way brokerages actually run.", "items": [
                        {"icon": "home_work", "title": "Listing management", "description": "Manage active, pending, and closed listings in one place."},
                        {"icon": "group", "title": "Buyer & seller CRM", "description": "Track every client timeline, preference, offer, and communication."},
                        {"icon": "payments", "title": "Commission tracking", "description": "Define splits and keep a running ledger for every deal."}
                      ]},
                      {"type": "cta", "eyebrow": "Limited beta spots available", "heading": "Be one of the first brokerages inside.", "subhead": "Join the waitlist for exclusive early access.", "button_label": "Reserve my beta spot →", "button_href": "/#waitlist-wrap"},
                      {"type": "beta_strip", "title": "Apply for the Folio Beta Program", "body": "Get discounted access during beta in exchange for real feedback.", "button_label": "Apply now", "button_href": "/beta"},
                      {"type": "footer", "tagline": "Modern Landlord OS · Broker Edition", "links": [{"label": "Main page", "href": "/"}, {"label": "Pricing", "href": "#broker-pricing"}, {"label": "Features", "href": "#broker-features"}]}
                    ]
                    $json$::jsonb
                ),
                (
                    'folio-pm',
                    $json$
                    [
                      {"type": "nav_sections", "items": [{"label": "Features", "href": "#pm-features"}, {"label": "How it works", "href": "#pm-app-preview"}, {"label": "Pricing", "href": "#pm-pricing"}]},
                      {"type": "feature_grid", "eyebrow": "Platform capabilities", "heading": "Built for PMCs. Not adapted from something else.", "items": [
                        {"icon": "account_tree", "title": "Multi-portfolio management", "description": "Manage client portfolios from a single dashboard."},
                        {"icon": "receipt_long", "title": "Owner portals & statements", "description": "Branded portals and monthly statements generated automatically."},
                        {"icon": "account_balance", "title": "Trust accounting", "description": "Security deposit ledgers, reserve funds, disbursements, and reconciliation."}
                      ]},
                      {"type": "cta", "eyebrow": "Limited beta spots available", "heading": "Stop managing with spreadsheets. Start running a real business.", "subhead": "Join the waitlist for exclusive early access.", "button_label": "Reserve my beta spot →", "button_href": "#pm-waitlist"},
                      {"type": "beta_strip", "title": "Apply for the Folio Beta Program", "body": "Get discounted access during beta in exchange for real feedback.", "button_label": "Apply now", "button_href": "/beta"},
                      {"type": "footer", "tagline": "Modern Landlord OS · Property Manager Edition", "links": [{"label": "For Landlords", "href": "/"}, {"label": "For Brokers", "href": "/brokers"}, {"label": "For Vendors", "href": "/vendors"}]}
                    ]
                    $json$::jsonb
                ),
                (
                    'folio-vendor',
                    $json$
                    [
                      {"type": "nav_sections", "items": [{"label": "Features", "href": "#vendor-features"}, {"label": "How it works", "href": "#vendor-how"}, {"label": "Pricing", "href": "#vendor-pricing"}]},
                      {"type": "feature_grid", "eyebrow": "Platform features", "heading": "Built for tradespeople, not accountants.", "items": [
                        {"icon": "search", "title": "Marketplace listing", "description": "Your profile surfaces to landlords and PMs by trade, location, and availability."},
                        {"icon": "assignment", "title": "Instant job dispatch", "description": "Receive jobs with photos, descriptions, and full property context."},
                        {"icon": "receipt_long", "title": "One-tap invoicing", "description": "Build and send invoices directly from the job."}
                      ]},
                      {"type": "cta", "eyebrow": "Open to all trades", "heading": "Stop waiting for referrals. Start getting jobs.", "subhead": "Join the Folio vendor marketplace and get connected to property managers and landlords.", "button_label": "Join the marketplace →", "button_href": "#vendor-signup"},
                      {"type": "footer", "tagline": "The Landlord OS · Vendor Marketplace", "links": [{"label": "For Landlords", "href": "/"}, {"label": "For Property Managers", "href": "/property-managers"}, {"label": "For Brokers", "href": "/brokers"}]}
                    ]
                    $json$::jsonb
                )
            )
            UPDATE product_page_templates t
            SET blocks_payload = seed.blocks,
                updated_at = NOW()
            FROM platform_products p
            JOIN seed ON seed.app_id = p.slug
            WHERE t.product_id = p.id
              AND (
                t.blocks_payload IS NULL
                OR t.blocks_payload = '{}'::jsonb
                OR t.blocks_payload = '[]'::jsonb
              );
            "##,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
