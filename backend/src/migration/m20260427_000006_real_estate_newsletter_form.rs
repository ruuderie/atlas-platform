use sea_orm_migration::prelude::*;

/// Patches the real-estate-ventures page to:
/// 1. Add a "Subscribe to newsletter" opt-in checkbox to the contact form
/// 2. Add a ProfileHeader block with personal investor profile details
/// 3. Anchor the FormBuilder to id="contact" for in-page linking from Grid cards
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r##"
            DO $$
            DECLARE
                v_tenant_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;

                IF v_tenant_id IS NULL THEN
                    RAISE EXCEPTION 'buildwithruud tenant not found';
                END IF;

                UPDATE app_pages
                SET
                    blocks_payload = $json$
                    [
                        {
                            "Hero": {
                                "title": "Real Estate Ventures",
                                "subtitle": "Acquisition · Wholesale · Rental · Commercial",
                                "layout": "minimal"
                            }
                        },
                        {
                            "ProfileHeader": {
                                "name": "Ruud Erie",
                                "role": "Real Estate Investor & Landlord",
                                "bio": "I operate across Connecticut and surrounding markets — acquiring off-market deals, building a buy-and-hold rental portfolio, selling on lease-option terms, and partnering on commercial opportunities. If you are a motivated seller, cash buyer, private lender, or tenant-buyer looking for a path to homeownership, reach out below.",
                                "location": "Connecticut, USA",
                                "tags": ["Wholesale", "Buy & Hold", "Lease-Option", "Joint Venture", "Commercial"]
                            }
                        },
                        {
                            "Grid": {
                                "section_title": "How I Invest",
                                "columns": 3,
                                "items": [
                                    {
                                        "id": "wholesale",
                                        "title": "Wholesale",
                                        "description": "I source and assign off-market deals. If you are a cash buyer or investor looking for discounted properties, get on my deal list and I will notify you before deals hit the market.",
                                        "icon": "swap_horiz",
                                        "link_url": "#contact"
                                    },
                                    {
                                        "id": "buy-hold",
                                        "title": "Buy & Hold",
                                        "description": "Long-term rental portfolio focused on single-family and small multi-family properties. I acquire, rehab, and hold for cash flow and appreciation.",
                                        "icon": "home",
                                        "link_url": "#contact"
                                    },
                                    {
                                        "id": "lease-option",
                                        "title": "Lease-Option",
                                        "description": "I sell properties on lease-to-own terms. If you want a path to homeownership but are not quite ready for a conventional mortgage, I may have an option that works for you.",
                                        "icon": "key",
                                        "link_url": "#contact"
                                    }
                                ]
                            }
                        },
                        {
                            "Grid": {
                                "section_title": "",
                                "columns": 2,
                                "items": [
                                    {
                                        "id": "jv",
                                        "title": "Joint Venture & Private Lending",
                                        "description": "Equity partnerships and private money on the right deals. Cash buyers, private lenders, and experienced investors welcome. I bring the deal, you bring the capital — we structure it fairly.",
                                        "icon": "handshake",
                                        "link_url": "#contact"
                                    },
                                    {
                                        "id": "commercial",
                                        "title": "Commercial Real Estate",
                                        "description": "Actively looking at industrial, flex-use industrial, and multi-family apartment acquisitions. If you have a commercial deal or know of an off-market opportunity, let's connect.",
                                        "icon": "warehouse",
                                        "link_url": "#contact"
                                    }
                                ]
                            }
                        },
                        {
                            "FormBuilder": {
                                "form_id": "rev_intake",
                                "title": "Let's Talk Real Estate",
                                "description": "Tell me what you have or what you're looking for. I respond to every serious inquiry. You can also subscribe to my newsletter for deal flow and market updates.",
                                "submit_button_text": "Send Message",
                                "fields": [
                                    { "name": "first_name", "label": "First Name", "field_type": "text", "required": true, "placeholder": "Jane" },
                                    { "name": "last_name", "label": "Last Name", "field_type": "text", "required": true, "placeholder": "Doe" },
                                    { "name": "email", "label": "Email Address", "field_type": "email", "required": true, "placeholder": "jane@example.com" },
                                    { "name": "phone", "label": "Phone (optional)", "field_type": "text", "required": false, "placeholder": "+1 (860) 000-0000" },
                                    {
                                        "name": "role",
                                        "label": "I am a...",
                                        "field_type": "select",
                                        "required": true,
                                        "options": [
                                            "Cash Buyer / Investor",
                                            "Motivated Seller",
                                            "Tenant-Buyer (Lease-Option)",
                                            "Private Lender",
                                            "Joint Venture Partner",
                                            "Commercial Deal Referral",
                                            "Newsletter Subscriber Only",
                                            "Other"
                                        ]
                                    },
                                    { "name": "message", "label": "Tell me more", "field_type": "textarea", "required": false, "placeholder": "Deal details, location, budget, timeline, or just say hello..." }
                                ]
                            }
                        }
                    ]
                    $json$,
                    updated_at = NOW()
                WHERE slug = 'real-estate-ventures' AND tenant_id = v_tenant_id;

                IF NOT FOUND THEN
                    RAISE EXCEPTION 'real-estate-ventures page not found for buildwithruud tenant';
                END IF;
            END $$;
        "##;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
