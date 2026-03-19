-- Create custom attribute enums if they don't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'attribute_type') THEN
        CREATE TYPE attribute_type AS ENUM (
            'ServiceDetail', 'ProductDetail', 'EventDetail', 'Location', 'BusinessHours',
            'Custom', 'Fees', 'Payment', 'Media', 'Amenity', 'Tag'
        );
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'attribute_key') THEN
        CREATE TYPE attribute_key AS ENUM (
            'Specialization', 'Experience', 'Certification', 'Brand', 'Condition',
            'Warranty', 'EventDate', 'Venue', 'Capacity', 'Address', 'City', 'State',
            'Country', 'PostalCode', 'DaysAvailable', 'HoursAvailable'
        );
    END IF;
END $$;

DO $$
DECLARE
    ct_directory_id UUID;
    admin_user_id UUID;
    admin_account_id UUID;
    cat_construction UUID;
    cat_plumbing UUID;
    cat_hvac UUID;
    cat_electric UUID;
    cat_cleaning UUID;
    cat_landscaping UUID;
    dir_type_id UUID;
    new_profile_id UUID;
    new_listing_id UUID;
    i INTEGER;
    j INTEGER;
    business_names TEXT[] := ARRAY[
        'Apex CT Renovations', 'Elite HVAC Professionals', 'Prime Wiring & Electric', 'Sparkle Commercial Cleaning', 
        'Scenic Views Outdoor', 'Precision Plumbing CT', 'Stamford Roofing Co.', 'Greenwich Build Partners', 
        'New Haven Hardwood', 'Bridgeport Masonry', 'Fairfield Landscapes', 'Hartford Heating & Air',
        'Waterbury Waterproofing', 'Danbury Drywall Pro', 'Norwalk Painting Inc', 'Milford Mechanical',
        'Westport Windows & Doors', 'Trumbull Tree Service', 'Shelton Steel Works', 'Glastonbury Glass',
        'Southington Siding', 'Middletown Remodeling', 'Meriden Paving', 'Bristol Builders Group'
    ];
    categories TEXT[] := ARRAY['Contractors', 'Plumbing', 'HVAC', 'Electricians', 'Cleaning', 'Landscaping'];
    categories_uuids UUID[];
BEGIN
    -- 1. Get or Create Directory
    SELECT id INTO ct_directory_id FROM directory WHERE name = 'CT Build Pros' OR domain = 'localhost';
    IF ct_directory_id IS NULL THEN
        -- Need a directory type first
        SELECT id INTO dir_type_id FROM directory_type LIMIT 1;
        IF dir_type_id IS NULL THEN
            dir_type_id := gen_random_uuid();
            INSERT INTO directory_type (id, name, description, is_active, created_at, updated_at)
            VALUES (dir_type_id, 'Service Professionals', 'Directory of local service professionals', true, NOW(), NOW());
        END IF;

        ct_directory_id := gen_random_uuid();
        INSERT INTO directory (id, directory_type_id, name, description, is_active, requires_approval, allow_reviews, created_at, updated_at, domain)
        VALUES (ct_directory_id, dir_type_id, 'CT Build Pros', 'The premier directory for top-rated construction and renovation services across Connecticut.', true, false, true, NOW(), NOW(), 'localhost');
    ELSE
        SELECT directory_type_id INTO dir_type_id FROM directory WHERE id = ct_directory_id;
    END IF;

    -- 2. Ensure Categories Exist
    -- Get or create category IDs for our 6 main types
    FOR i IN 1..array_length(categories, 1) LOOP
        DECLARE
            cat_id UUID;
        BEGIN
            SELECT id INTO cat_id FROM category WHERE directory_type_id = dir_type_id AND name = categories[i];
            IF cat_id IS NULL THEN
                cat_id := gen_random_uuid();
                INSERT INTO category (id, directory_type_id, name, description, icon, is_active, created_at, updated_at)
                VALUES (cat_id, dir_type_id, categories[i], 'Category for ' || categories[i], 'tool', true, NOW(), NOW());
            END IF;
            categories_uuids := array_append(categories_uuids, cat_id);
        END;
    END LOOP;

    -- 3. Get an Admin User / Account (for profiles)
    SELECT id INTO admin_user_id FROM "user" LIMIT 1;
    IF admin_user_id IS NULL THEN
        admin_user_id := gen_random_uuid();
        INSERT INTO "user" (id, email, password_hash, first_name, last_name, is_active, created_at, updated_at)
        VALUES (admin_user_id, 'admin@ctbuild.com', 'hashedpassword', 'CT', 'Admin', true, NOW(), NOW());
    END IF;

    SELECT id INTO admin_account_id FROM account LIMIT 1;
    IF admin_account_id IS NULL THEN
        admin_account_id := gen_random_uuid();
        INSERT INTO account (id, company_name, is_active, created_at, updated_at)
        VALUES (admin_account_id, 'CT Build Pros LLC', true, NOW(), NOW());
        
        -- Link user to account
        INSERT INTO user_account (user_id, account_id, role, created_at, updated_at)
        VALUES (admin_user_id, admin_account_id, 'admin', NOW(), NOW());
    END IF;

    -- 4. Create 50+ Listings
    -- We loop 55 times to generate 55 listings
    FOR i IN 1..55 LOOP
        new_profile_id := gen_random_uuid();
        
        -- Create Profile
        INSERT INTO profile (
            id, account_id, directory_id, business_name, description, contact_email, 
            phone_number, website, address, city, state, country, zip_code, 
            profile_type, status, created_at, updated_at
        ) VALUES (
            new_profile_id, admin_account_id, ct_directory_id, 
            business_names[floor(random() * array_length(business_names, 1) + 1)] || ' ' || i,
            'Premium Connecticut-based service provider delivering exceptional results.',
            'contact' || i || '@example.com', '203-555-' || lpad(i::text, 4, '0'),
            'https://example.com/biz' || i, '123 Main St Suite ' || i, 
            (ARRAY['New Haven', 'Stamford', 'Bridgeport', 'Hartford', 'Waterbury', 'Danbury', 'Norwalk'])[floor(random() * 7 + 1)], 
            'CT', 'USA', '06' || floor(random() * 900 + 100)::text,
            'business', 'active', NOW(), NOW()
        );

        new_listing_id := gen_random_uuid();

        -- Create Listing
        INSERT INTO listing (
            id, profile_id, directory_id, category_id, title, description, listing_type,
            price, price_type, country, state, city, neighborhood, latitude, longitude,
            additional_info, status, is_featured, is_based_on_template, is_ad_placement, is_active,
            created_at, updated_at
        ) VALUES (
            new_listing_id, new_profile_id, ct_directory_id,
            categories_uuids[floor(random() * array_length(categories_uuids, 1) + 1)],
            (ARRAY['Premium Renovation', '24/7 HVAC Service', 'Trusted Plumber', 'Commercial Cleaning', 'Luxury Landscaping', 'Electrical Systems', 'Roofing Pros', 'Paving & Masonry'])[floor(random() * 8 + 1)] || ' - Top Rated ' || i,
            'We provide industry-leading services directly to residential and commercial clients across our service area. Reliable, insured, and verified professionals.',
            (ARRAY['Contractors', 'Plumbers', 'HVAC', 'Cleaning', 'Landscaping', 'Electricians'])[floor(random() * 6 + 1)],
            floor(random() * 5000 + 100),
            (ARRAY['fixed', 'hourly', 'quote'])[floor(random() * 3 + 1)],
            'USA', 'CT', 
            (ARRAY['New Haven', 'Stamford', 'Bridgeport', 'Hartford', 'Waterbury', 'Danbury', 'Norwalk'])[floor(random() * 7 + 1)], 
            (ARRAY['Downtown', 'Suburbs', 'North End', 'West Side', 'Business Park'])[floor(random() * 5 + 1)],
            random() * 1.5 + 41.0, -- CT Lat
            random() * 1.5 - 73.5, -- CT Lng
            json_build_object(
                'hero_headline', 'Your Local CT Experts',
                'team_size', floor(random() * 50 + 2),
                'verified', true,
                'rating', (floor(random() * 10) + 40) / 10.0 -- 4.0 to 4.9
            ),
            'approved',
            random() < 0.1, -- 10% chance to be featured
            false,
            false,
            true,
            NOW() - (random() * 90 || ' days')::INTERVAL, -- randomize creation date within last 90 days
            NOW()
        );

        -- Create 3 Listing Attributes (Metadata) per listing
        INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
        VALUES
            (gen_random_uuid(), new_listing_id, 'ServiceDetail'::attribute_type, 'Experience'::attribute_key, 
             to_jsonb((floor(random() * 25 + 5) || ' years')::text), NOW(), NOW()),
            (gen_random_uuid(), new_listing_id, 'BusinessHours'::attribute_type, 'DaysAvailable'::attribute_key, 
             to_jsonb(ARRAY['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday']::text[]), NOW(), NOW()),
            (gen_random_uuid(), new_listing_id, 'ServiceDetail'::attribute_type, 'Certification'::attribute_key, 
             to_jsonb((ARRAY['Licensed Pro', 'Fully Insured', 'OSHA Certified', 'A+ BBB Rated'])[floor(random() * 4 + 1)]::text), NOW(), NOW());
    END LOOP;

    RAISE NOTICE 'Successfully seeded 55 directory listings into CT Build Pros.';
END $$;
