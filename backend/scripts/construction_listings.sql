DO $$
BEGIN
    -- Create attribute_type enum if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'attribute_type') THEN
        CREATE TYPE attribute_type AS ENUM (
            'ServiceDetail', 'ProductDetail', 'EventDetail', 'Location', 'BusinessHours',
            'Custom', 'Fees', 'Payment', 'Media', 'Amenity', 'Tag'
        );
    END IF;

    -- Create attribute_key enum if it doesn't exist
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
    builder_connect_id UUID;
    profile_record RECORD;
    listing_record RECORD;
BEGIN
    -- Get the Builder Connect network ID
    SELECT id INTO builder_connect_id FROM network WHERE name = 'Builder Connect';

    IF builder_connect_id IS NULL THEN
        RAISE EXCEPTION 'Builder Connect network not found';
    END IF;

    -- First, create all listings
    FOR profile_record IN (SELECT id, COALESCE(business_name, 'Construction Company') AS business_name FROM profile WHERE network_id = builder_connect_id)
    LOOP
        -- Create 2 listings for each profile
        FOR i IN 1..2 LOOP
            INSERT INTO listing (
                id, profile_id, network_id, category_id, title, description, listing_type,
                price, price_type, country, state, city, neighborhood, latitude, longitude,
                additional_info, status, is_featured, is_based_on_template, is_ad_placement, is_active,
                created_at, updated_at
            )
            VALUES (
                gen_random_uuid(), profile_record.id, builder_connect_id,
                (SELECT id FROM category WHERE network_type_id = (SELECT network_type_id FROM network WHERE id = builder_connect_id) ORDER BY RANDOM() LIMIT 1),
                (ARRAY['Home Renovation Project', 'Commercial Building Construction', 'Kitchen Remodeling', 'Bathroom Renovation', 'Roofing Services', 'Landscaping Design', 'Basement Finishing', 'Home Addition', 'Deck Construction', 'Flooring Installation'])[floor(random() * 10 + 1)],
                'Professional ' || profile_record.business_name || ' offering high-quality construction services. We specialize in delivering exceptional results for your building needs.',
                'service',
                floor(random() * 10000 + 1000),
                (ARRAY['fixed', 'hourly', 'square_foot'])[floor(random() * 3 + 1)],
                'USA',
                (ARRAY['NY', 'CA', 'IL', 'TX', 'AZ', 'PA', 'FL', 'OH', 'GA', 'NC'])[floor(random() * 10 + 1)],
                (ARRAY['New York', 'Los Angeles', 'Chicago', 'Houston', 'Phoenix', 'Philadelphia', 'San Antonio', 'San Diego', 'Dallas', 'San Jose'])[floor(random() * 10 + 1)],
                (ARRAY['Downtown', 'Midtown', 'Uptown', 'Suburbs', 'Business District'])[floor(random() * 5 + 1)],
                random() * 180 - 90,
                random() * 360 - 180,
                json_build_object(
                    'project_duration', (ARRAY['1-3 months', '3-6 months', '6-12 months', '1-2 years'])[floor(random() * 4 + 1)],
                    'team_size', floor(random() * 20 + 5),
                    'services_offered', ARRAY[(ARRAY['General Contracting', 'Design-Build', 'Project Management', 'Sustainable Building', 'Historic Restoration', 'Tenant Improvements', 'Site Development', 'Concrete Work', 'Steel Framing', 'Carpentry'])[floor(random() * 10 + 1)], (ARRAY['Electrical', 'Plumbing', 'HVAC', 'Painting', 'Roofing', 'Masonry', 'Drywall', 'Flooring', 'Windows and Doors', 'Insulation'])[floor(random() * 10 + 1)]]
                ),
                (ARRAY['active', 'pending', 'completed'])[floor(random() * 3 + 1)],
                random() < 0.2,
                false,
                false,
                true,
                NOW(),
                NOW()
            );
        END LOOP;
    END LOOP;

    -- Then, create attributes for all listings
    FOR listing_record IN (SELECT id FROM listing WHERE network_id = builder_connect_id)
    LOOP
        -- Create 5 listing attributes for each listing
        INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
        VALUES
            (gen_random_uuid(), listing_record.id, 'ServiceDetail'::attribute_type, 'Specialization'::attribute_key, 
             to_jsonb((ARRAY['Residential Construction', 'Commercial Construction', 'Industrial Construction', 'Green Building', 'Historic Renovation', 'Disaster Reconstruction', 'Luxury Home Building', 'Modular Construction', 'Design-Build Services', 'Construction Management'])[floor(random() * 10 + 1)]::text), 
             NOW(), NOW()),
            (gen_random_uuid(), listing_record.id, 'ServiceDetail'::attribute_type, 'Experience'::attribute_key, 
             to_jsonb((floor(random() * 30 + 5) || ' years')::text), 
             NOW(), NOW()),
            (gen_random_uuid(), listing_record.id, 'ServiceDetail'::attribute_type, 'Certification'::attribute_key, 
             to_jsonb((ARRAY['Licensed General Contractor', 'LEED Certified', 'OSHA 30-Hour Certified', 'EPA Lead-Safe Certified', 'Energy Star Partner', 'NAHB Certified Green Professional', 'AIA Contractor', 'NARI Certified Remodeler', 'DBIA Certified', 'ICC Certified Building Official'])[floor(random() * 10 + 1)]::text), 
             NOW(), NOW()),
            (gen_random_uuid(), listing_record.id, 'BusinessHours'::attribute_type, 'DaysAvailable'::attribute_key, 
             to_jsonb(ARRAY['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']::text[]), 
             NOW(), NOW()),
            (gen_random_uuid(), listing_record.id, 'BusinessHours'::attribute_type, 'HoursAvailable'::attribute_key, 
             to_jsonb('7:00 AM - 6:00 PM'::text), 
             NOW(), NOW());
    END LOOP;
END $$;