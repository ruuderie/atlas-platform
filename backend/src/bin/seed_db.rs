use sea_orm::{ActiveModelTrait, Database, Set};
use business_directory_backend::entities::{directory, directory_type, user, account, profile, listing, user_account};
use uuid::Uuid;
use chrono::Utc;
use dotenv::dotenv;
use business_directory_backend::auth::hash_password;
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

    // 3. Generate 10 Users + Accounts + Profiles + Listings
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
            is_admin: Set(false),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        u.insert(&db).await?;

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
            status: Set(business_directory_backend::models::listing::ListingStatus::Approved),
            category_id: Set(None), 
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
            is_featured: Set(i < 3), // Make first 3 featured
            is_based_on_template: Set(false),
            based_on_template_id: Set(None),
            is_ad_placement: Set(false),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        
        lst.insert(&db).await?;
        
        tracing::info!("Created User & Listing for {}", business_names[i]);
    }

    tracing::info!("Database successfully seeded!");
    Ok(())
}
