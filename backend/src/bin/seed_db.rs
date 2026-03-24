use sea_orm::{ActiveModelTrait, Database, Set};
use atlas_backend::entities::{directory, directory_type, user, account, profile, listing, user_account, category, template, customer, contact, lead, deal, case, activity, note, feed};
use uuid::Uuid;
use chrono::Utc;
use dotenv::dotenv;
use atlas_backend::auth::hash_password;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await?;

    tracing::info!("Connected to the database. Beginning Seed Process.");

    // 1. Create Directory Type
    let dir_type_id = Uuid::new_v4();
    let dir_type = directory_type::ActiveModel {
        id: Set(dir_type_id),
        name: Set("Contractor Directory".to_string()),
        description: Set("Directory for state-level contractors".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    dir_type.insert(&db).await?;
    tracing::info!("Created DirectoryType: {}", dir_type_id);

    // 2. Create Directory
    // Attempt to clear old mock string if valid UUID, otherwise generate new
    // Our DB ID column is UUID, so the frontend's 'mock-dir...'' strings must become a valid uuid in production.
    let dir_uuid = Uuid::new_v4();
    
    // Provide the theme and custom settings that mirror app.rs payload
    let theme_json = serde_json::json!({
        "brand_primary": "#f97316",
        "bg_surface": "#ffffff",
        "radius_ui": "6px",
        "font_heading": "Inter, sans-serif"
    });

    let custom_settings = serde_json::json!({
        "hero_headline": "Connecticut's Most Trusted Home Renovation Pros.",
        "hero_subtitle": "Find licensed contractors, handymen, and renovation specialists — vetted and reviewed by your neighbors.",
        "search_placeholder_keyword": "Kitchen remodel, plumber, handyman...",
        "search_placeholder_location": "Hartford, Stamford, New Haven...",
        "categories": [
            { "slug": "kitchen-bath", "label": "Kitchen & Bath", "subtitle": "Remodels & Upgrades", "icon": "countertops" },
            { "slug": "general-handyman", "label": "General Handyman", "subtitle": "Repairs & Odd Jobs", "icon": "handyman" },
            { "slug": "roofing-siding", "label": "Roofing & Siding", "subtitle": "Exterior Specialists", "icon": "roofing" },
            { "slug": "electrical", "label": "Electrical", "subtitle": "Licensed Electricians", "icon": "electrical_services" },
            { "slug": "painting", "label": "Painting", "subtitle": "Professional Painter", "icon": "professional_painter" }
        ],
        "process_steps": [
            { "number": "1", "title": "Search", "description": "Browse our curated list of professionals by category or location." },
            { "number": "2", "title": "Compare", "description": "Read verified reviews and compare qualifications." },
            { "number": "3", "title": "Connect", "description": "Contact pros directly to get quotes and start your project." }
        ],
        "host_page_content": {
            "hero_headline": "Grow Your Business with CT Build Pros",
            "hero_subtitle": "Join Connecticut's premier network of verified contractors. Get discovered by homeowners actively looking for your services.",
            "form_category_options": ["Kitchen & Bath", "Roofing", "Electrical", "Plumbing", "General Contracting"],
            "trust_heading": "Why Pros Choose Us",
            "trust_subtitle": "We connect you with high-intent homeowners ready to start their projects.",
            "testimonial_quote": "\"Since listing on CT Build Pros, my lead volume has doubled. The quality of clients is significantly better than other platforms.\"",
            "testimonial_name": "Mike Sullivan",
            "testimonial_title": "Owner, Apex Renovation",
            "cta_headline": "Ready to get more jobs?",
            "cta_subtitle": "Join hundreds of successful contractors."
        },
        "featured_listings": []
    });

    let directory = directory::ActiveModel {
        id: Set(dir_uuid),
        directory_type_id: Set(dir_type_id),
        name: Set("CT Build Pros".to_string()),
        domain: Set("directory.localhost".to_string()),
        description: Set("The premier directory for top-rated construction and renovation services across Connecticut.".to_string()),
        enabled_modules: Set(1),
        theme: Set(Some(theme_json.to_string())),
        custom_settings: Set(Some(custom_settings)),
        site_status: Set("ACTIVE".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        subdomain: Set(Some("ct-build-pros".to_string())),
        custom_domain: Set(None),
        logo: Set(None),
        favicon: Set(None),
        header_scripts: Set(None),
        footer_scripts: Set(None),
        google_analytics_id: Set(None),
        google_site_verification: Set(None),
        meta_description: Set(None),
        meta_keywords: Set(None),
        meta_title: Set(None),
        page_title: Set(None),
        page_description: Set(None),
        page_keywords: Set(None),
        canonical_url: Set(None),
    };

    directory.insert(&db).await?;
    tracing::info!("Created Directory: CT Build Pros ({})", dir_uuid);

    // 3. Create Categories
    let cat_id_1 = Uuid::new_v4();
    let cat1 = category::ActiveModel {
        id: Set(cat_id_1),
        directory_type_id: Set(dir_type_id),
        parent_category_id: Set(None),
        name: Set("Kitchen & Bath".to_string()),
        description: Set("Remodels & Upgrades".to_string()),
        icon: Set(Some("countertops".to_string())),
        slug: Set(Some("kitchen-bath".to_string())),
        is_custom: Set(false),
        is_active: Set(true),
        directory_id: Set(Some(dir_uuid)),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    cat1.insert(&db).await?;

    let cat_id_2 = Uuid::new_v4();
    let cat2 = category::ActiveModel {
        id: Set(cat_id_2),
        directory_type_id: Set(dir_type_id),
        parent_category_id: Set(None),
        name: Set("Roofing & Siding".to_string()),
        description: Set("Exterior Specialists".to_string()),
        icon: Set(Some("roofing".to_string())),
        slug: Set(Some("roofing-siding".to_string())),
        is_custom: Set(false),
        is_active: Set(true),
        directory_id: Set(Some(dir_uuid)),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    cat2.insert(&db).await?;

    let cat_id_3 = Uuid::new_v4();
    let cat3 = category::ActiveModel {
        id: Set(cat_id_3),
        directory_type_id: Set(dir_type_id),
        parent_category_id: Set(None),
        name: Set("Home Services".to_string()),
        description: Set("Plumbers, electricians, and more".to_string()),
        icon: Set(Some("home_repair_service".to_string())),
        slug: Set(Some("home-services".to_string())),
        is_custom: Set(false),
        is_active: Set(true),
        directory_id: Set(Some(dir_uuid)),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    cat3.insert(&db).await?;

    let cat_id_4 = Uuid::new_v4();
    let cat4 = category::ActiveModel {
        id: Set(cat_id_4),
        directory_type_id: Set(dir_type_id),
        parent_category_id: Set(None),
        name: Set("Professional Services".to_string()),
        description: Set("Accountants, Lawyers, Consulting".to_string()),
        icon: Set(Some("work".to_string())),
        slug: Set(Some("professional-services".to_string())),
        is_custom: Set(false),
        is_active: Set(true),
        directory_id: Set(Some(dir_uuid)),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    cat4.insert(&db).await?;

    // 4. Create Templates
    let tpl_id_1 = Uuid::new_v4();
    let tpl1 = template::ActiveModel {
        id: Set(tpl_id_1),
        directory_id: Set(dir_uuid),
        category_id: Set(cat_id_1),
        name: Set("Contractor Profile Schema".to_string()),
        description: Set("Default fields for home service pros".to_string()),
        template_type: Set("business".to_string()),
        is_active: Set(true),
        attributes_schema: Set(Some(serde_json::json!({
            "license_number": "String",
            "insurance_provider": "String",
            "years_in_business": "Number",
            "emergency_services": "Boolean"
        }))),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    tpl1.insert(&db).await?;

    let tpl_id_2 = Uuid::new_v4();
    let tpl2 = template::ActiveModel {
        id: Set(tpl_id_2),
        directory_id: Set(dir_uuid),
        category_id: Set(cat_id_3),
        name: Set("Standard Business".to_string()),
        description: Set("Standard fields for a business".to_string()),
        template_type: Set("business".to_string()),
        is_active: Set(true),
        attributes_schema: Set(Some(serde_json::json!({
            "website": "String",
            "operating_hours": "String"
        }))),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    tpl2.insert(&db).await?;

    let tpl_id_3 = Uuid::new_v4();
    let tpl3 = template::ActiveModel {
        id: Set(tpl_id_3),
        directory_id: Set(dir_uuid),
        category_id: Set(cat_id_4),
        name: Set("Premium Listing".to_string()),
        description: Set("Extended fields for premium users".to_string()),
        template_type: Set("business".to_string()),
        is_active: Set(true),
        attributes_schema: Set(Some(serde_json::json!({
            "featured_video": "String",
            "certifications": "Array"
        }))),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    tpl3.insert(&db).await?;

    // 5. Generate 10 Users + Accounts + Profiles + Listings
    let mut admin_user_id = Uuid::nil();
    let first_names = ["John", "Sarah", "Mike", "Emily", "David", "Jessica", "Robert", "Lisa", "James", "Anna"];
    let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez"];
    let business_names = [
        "Apex Renovation", "ProPlumb CT", "Hartford Electric", 
        "Elite Siding", "Master Painters", "Green Lawns", 
        "Secure Gates", "Crystal Clear Windows", "Top Tier Roofing", "Precision Builders"
    ];

    let password_hash = hash_password("Password123!")?;

    for i in 0..10 {
        // User
        let user_id = Uuid::new_v4();
        let first_name = first_names[i].to_string();
        let last_name = last_names[i].to_string();
        let email = format!("{}.{}@example.com", first_name.to_lowercase(), last_name.to_lowercase());
        
        let u = user::ActiveModel {
            id: Set(user_id),
            username: Set(email.clone()),
            first_name: Set(first_name.clone()),
            last_name: Set(last_name.clone()),
            email: Set(email.clone()),
            phone: Set(format!("555-010{}", i)),
            password_hash: Set(password_hash.clone()),
            last_login: Set(None),
            is_admin: Set(i == 0),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        u.insert(&db).await?;
        if i == 0 { admin_user_id = user_id; }

        // Account
        let acct_id = Uuid::new_v4();
        let acct = account::ActiveModel {
            id: Set(acct_id),
            directory_id: Set(dir_uuid),
            name: Set(business_names[i].to_string()),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        acct.insert(&db).await?;

        // UserAccount Link
        let ua = user_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            account_id: Set(acct_id),
            role: Set(user_account::UserRole::Owner),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        ua.insert(&db).await?;

        // Profile
        let profile_id = Uuid::new_v4();
        let prof = profile::ActiveModel {
            id: Set(profile_id),
            account_id: Set(acct_id),
            directory_id: Set(dir_uuid),
            profile_type: Set(profile::ProfileType::Business),
            display_name: Set(format!("{} {}", first_name, last_name)),
            contact_info: Set(email.clone()),
            business_name: Set(Some(business_names[i].to_string())),
            business_address: Set(Some("100 Main St, Hartford, CT 06103".to_string())),
            business_phone: Set(Some(format!("555-010{}", i))),
            business_website: Set(None),
            additional_info: Set(Some(serde_json::json!({
                "rating": 4.5 + (i as f32) * 0.05,
                "review_count": i as i32 * 5 + 10
            }))),
            properties: Set(Some(serde_json::json!({
                "contractor_license_number": format!("CT-{:06}", i * 1024),
                "insurance_provider": "Travelers",
                "years_in_business": i + 5
            }))),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        prof.insert(&db).await?;

        // Listing
        let listing_id = Uuid::new_v4();
        let lst = listing::ActiveModel {
            id: Set(listing_id),
            directory_id: Set(dir_uuid),
            profile_id: Set(profile_id),
            title: Set(business_names[i].to_string()),
            description: Set(format!("Professional services by {}", business_names[i])),
            status: Set(atlas_backend::models::listing::ListingStatus::Approved),
            category_id: Set(Some(if i % 2 == 0 { cat_id_1 } else { cat_id_2 })), 
            listing_type: Set("SERVICE".to_string()),
            price: Set(None),
            price_type: Set(None),
            country: Set(Some("US".to_string())),
            state: Set(Some("CT".to_string())),
            city: Set(Some("Hartford".to_string())),
            neighborhood: Set(None),
            latitude: Set(None),
            longitude: Set(None),
            slug: Set(Some(business_names[i].to_lowercase().replace(" ", "-"))),
            additional_info: Set(Some(serde_json::json!({
                "tags": ["verified", "licensed"]
            }))),
            properties: Set(Some(serde_json::json!({
                "service_radius_miles": 50,
                "emergency_services": i % 2 == 0,
                "free_estimates": true
            }))),
            is_featured: Set(i < 3), // Make first 3 featured
            is_based_on_template: Set(true),
            based_on_template_id: Set(Some(tpl_id_1)),
            is_ad_placement: Set(false),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        
        lst.insert(&db).await?;
        
        tracing::info!("Created User & Listing for {}", business_names[i]);
    }

    
    // 6. CRM Entities
    // We will generate 3 Customers, some Contacts, Leads, Deals, Cases, Activities and Notes
    let mut cust_ids = Vec::new();
    let statuses = ["Prospecting", "Qualification", "Closed Won", "Closed Lost"];
    let stages = ["Initial Contact", "Meeting Scheduled", "Proposal Sent", "Contract Signed"];

    for j in 0..3 {
        let cust_id = Uuid::new_v4();
        cust_ids.push(cust_id);
        
        // Customer
        let cust = customer::ActiveModel {
            id: Set(cust_id),
            name: Set(format!("Mock Enterprise {}", j)),
            primary_contact_id: Set(None),
            customer_type: Set(customer::CustomerType::BusinessEntity),
            attributes: Set(customer::CustomerAttributes {
                shipper: false, carrier: false, loan_seeker: false, loan_broker: false, 
                software_vendor: false, tenant: false, software_development_client: false, 
                salesforce_client: false, web3_client: false, bitcoiner: false, zk: false, 
                lender: false, advertiser: false, gp: false, construction_contractor: true, 
                construction_client: false, landlord: false
            }),
            cpf: Set(None), cnpj: Set(None), tin: Set(None),
            email: Set(Some(format!("contact@enterprise{}.com", j))),
            phone: Set(Some(format!("555-020{}", j))),
            whatsapp: Set(None), telegram: Set(None), twitter: Set(None), instagram: Set(None), facebook: Set(None),
            website: Set(Some(format!("https://enterprise{}.com", j))),
            annual_revenue: Set(Some(1000000.0 * (j as f64 + 1.0))),
            employee_count: Set(Some(10 * (j + 1))),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            billing_address: Set(None),
            shipping_address: Set(None),
            directory_id: Set(Some(dir_uuid)),
            properties: Set(None),
        };
        cust.insert(&db).await?;

        // Contact
        let cnt_id = Uuid::new_v4();
        let cnt = contact::ActiveModel {
            id: Set(cnt_id),
            customer_id: Set(Some(cust_id)),
            name: Set(format!("Contact Name {}", j)),
            first_name: Set(Some("Contact".to_string())),
            last_name: Set(Some(format!("Name {}", j))),
            email: Set(Some(format!("contact{}@enterprise{}.com", j, j))),
            phone: Set(Some(format!("555-030{}", j))),
            whatsapp: Set(None), telegram: Set(None), twitter: Set(None), instagram: Set(None), facebook: Set(None),
            billing_address: Set(None),
            shipping_address: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            directory_id: Set(Some(dir_uuid)),
            properties: Set(None),
        };
        cnt.insert(&db).await?;

        // Deal
        let d_id = Uuid::new_v4();
        let dl = deal::ActiveModel {
            id: Set(d_id),
            customer_id: Set(cust_id),
            name: Set(format!("Renovation Deal {}", j)),
            amount: Set(5000.0 * (j as f64 + 1.0)),
            status: Set(statuses[(j as usize) % statuses.len()].to_string()),
            stage: Set(stages[(j as usize) % stages.len()].to_string()),
            close_date: Set(Some(Utc::now())),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            directory_id: Set(Some(dir_uuid)),
            properties: Set(None),
        };
        dl.insert(&db).await?;

        // Case
        let cs_id = Uuid::new_v4();
        let cs = case::ActiveModel {
            id: Set(cs_id),
            customer_id: Set(cust_id),
            title: Set(format!("Support Ticket #{}", j)),
            description: Set("Issue with the latest service.".to_string()),
            status: Set((if j % 2 == 0 { "Open" } else { "Closed" }).to_string()),
            priority: Set("High".to_string()),
            assigned_to: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            closed_at: Set(None),
            properties: Set(None),
        };
        cs.insert(&db).await?;

        // Activity
        let act_id = Uuid::new_v4();
        let act = activity::ActiveModel {
            id: Set(act_id),
            account_id: Set(None),
            deal_id: Set(Some(d_id)),
            customer_id: Set(Some(cust_id)),
            lead_id: Set(None),
            contact_id: Set(Some(cnt_id)),
            case_id: Set(None),
            activity_type: Set(activity::ActivityType::PhoneCall),
            title: Set("Follow-up Call".to_string()),
            description: Set(Some("Discussed the proposal specs.".to_string())),
            status: Set(activity::ActivityStatus::Completed),
            due_date: Set(Some(Utc::now())),
            completed_at: Set(Some(Utc::now())),
            associated_entities: Set(serde_json::json!([])),
            created_by: Set(admin_user_id),
            assigned_to: Set(Some(admin_user_id)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        act.insert(&db).await?;

        // Note
        let nt_id = Uuid::new_v4();
        let nt = note::ActiveModel {
            id: Set(nt_id),
            content: Set("Client seemed very interested in premium options.".to_string()),
            created_by: Set(admin_user_id),
            entity_type: Set("Deal".to_string()),
            entity_id: Set(d_id),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        nt.insert(&db).await?;
        
        tracing::info!("Created CRM entries for enterprise {}", j);
    }

    // Leads
    for j in 0..5 {
        let ld_id = Uuid::new_v4();
        let ld = lead::ActiveModel {
            id: Set(ld_id),
            name: Set(format!("Website Lead {}", j)),
            listing_id: Set(None),
            account_id: Set(None),
            first_name: Set(Some(format!("LeadFirst{}", j))),
            last_name: Set(Some(format!("LeadLast{}", j))),
            email: Set(Some(format!("lead{}@example.com", j))),
            phone: Set(Some(format!("555-040{}", j))),
            whatsapp: Set(None), telegram: Set(None), twitter: Set(None), instagram: Set(None), facebook: Set(None),
            billing_address: Set(None),
            shipping_address: Set(None),
            message: Set(Some("I would like a quote.".to_string())),
            source: Set(Some("Website".to_string())),
            is_converted: Set(false),
            converted_to_contact: Set(false),
            associated_deal_id: Set(None),
            converted_customer_id: Set(None),
            converted_contact_id: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            directory_id: Set(Some(dir_uuid)),
            properties: Set(None),
        };
        ld.insert(&db).await?;
    }

    // 7. CMS Feed
    let feed_id = Uuid::new_v4();
    let fd = feed::ActiveModel {
        id: Set(feed_id),
        directory_id: Set(dir_uuid),
        title: Set("Main Directory Blog".to_string()),
        description: Set("News and updates for contractors.".to_string()),
        feed_url: Set("https://directory.localhost/feed".to_string()),
        home_page_url: Set("https://directory.localhost".to_string()),
        icon: Set(None),
        favicon: Set(None),
        author: Set(Some("Admin".to_string())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    fd.insert(&db).await?;
    tracing::info!("Created Feed: {}", feed_id);

tracing::info!("Database successfully seeded!");
    Ok(())
}
