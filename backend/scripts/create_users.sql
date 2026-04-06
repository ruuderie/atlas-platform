DO $$
DECLARE
    network_record RECORD;
    account_id UUID;
    user_id UUID;
    profile_id UUID;
    user_account_id UUID;
    username TEXT;
    email TEXT;
    display_name TEXT;
    phone_number TEXT;
    website TEXT;
    first_name TEXT;
    last_name TEXT;
    domain TEXT;
BEGIN
    -- Loop through all existing networks
    FOR network_record IN SELECT id, name FROM network
    LOOP
        -- Create 3 users for each network
        FOR i IN 1..3 LOOP
            -- Generate user data
            first_name := (ARRAY['James', 'John', 'Robert', 'Michael', 'William', 'David', 'Richard', 'Joseph', 'Thomas', 'Charles', 'Mary', 'Patricia', 'Jennifer', 'Linda', 'Elizabeth', 'Barbara', 'Margaret', 'Susan', 'Dorothy', 'Lisa'])[floor(random() * 20 + 1)];
            last_name := (ARRAY['Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Rodriguez', 'Martinez', 'Hernandez', 'Lopez', 'Gonzalez', 'Wilson', 'Anderson', 'Thomas', 'Taylor', 'Moore', 'Jackson', 'Martin'])[floor(random() * 20 + 1)];
            
            username := lower(first_name || '.' || last_name || floor(random() * 100)::text);
            display_name := first_name || ' ' || last_name;
            
            domain := (ARRAY['gmail.com', 'yahoo.com', 'outlook.com', 'hotmail.com', 'example.com'])[floor(random() * 5 + 1)];
            email := username || '@' || domain;
            
            -- Generate phone number
            phone_number := '+1 ' || 
                            LPAD(CAST(floor(random() * 900 + 100) AS TEXT), 3, '0') || '-' ||
                            LPAD(CAST(floor(random() * 900 + 100) AS TEXT), 3, '0') || '-' ||
                            LPAD(CAST(floor(random() * 9000 + 1000) AS TEXT), 4, '0');

            -- Generate website
            website := 'www.' || lower(regexp_replace(network_record.name, '\s+', '', 'g')) || '.com';

            -- Insert user
            INSERT INTO "user" (id, username, email, password_hash, is_admin, is_active, created_at, updated_at)
            VALUES (gen_random_uuid(), username, email, 'hashed_password', false, true, NOW(), NOW())
            RETURNING id INTO user_id;

            -- Insert account
            INSERT INTO account (id, network_id, name, is_active, created_at, updated_at)
            VALUES (gen_random_uuid(), network_record.id, last_name || ' ' || network_record.name || ' Account', true, NOW(), NOW())
            RETURNING id INTO account_id;

            -- Insert user_account
            INSERT INTO user_account (id, user_id, account_id, role, is_active, created_at, updated_at)
            VALUES (gen_random_uuid(), user_id, account_id, 
                    CASE WHEN i = 1 THEN 'Owner'
                         WHEN i = 2 THEN 'Admin'
                         ELSE 'Member'
                    END, 
                    true, NOW(), NOW())
            RETURNING id INTO user_account_id;

            -- Insert profile
            INSERT INTO profile (
                id, account_id, network_id, profile_type, display_name, contact_info, 
                business_name, business_address, business_phone, business_website, 
                additional_info, is_active, created_at, updated_at
            )
            VALUES (
                gen_random_uuid(), account_id, network_record.id, 
                CASE WHEN random() < 0.7 THEN 'Business' ELSE 'Individual' END,
                display_name, email, 
                CASE WHEN random() < 0.7 THEN last_name || ' ' || network_record.name ELSE NULL END,
                CASE WHEN random() < 0.7 THEN 
                    floor(random() * 9999 + 1)::text || ' ' || 
                    (ARRAY['Main', 'Oak', 'Pine', 'Maple', 'Cedar'])[floor(random() * 5 + 1)] || ' ' ||
                    (ARRAY['St', 'Ave', 'Blvd', 'Rd', 'Ln'])[floor(random() * 5 + 1)] || ', ' ||
                    (ARRAY['New York', 'Los Angeles', 'Chicago', 'Houston', 'Phoenix', 'Philadelphia', 'San Antonio', 'San Diego', 'Dallas', 'San Jose'])[floor(random() * 10 + 1)] || ', ' ||
                    (ARRAY['NY', 'CA', 'IL', 'TX', 'AZ', 'PA', 'TX', 'CA', 'TX', 'CA'])[floor(random() * 10 + 1)]
                ELSE NULL END,
                CASE WHEN random() < 0.7 THEN phone_number ELSE NULL END,
                CASE WHEN random() < 0.5 THEN website ELSE NULL END,
                json_build_object(
                    'years_of_experience', floor(random() * 30 + 1),
                    'specialties', ARRAY[
                        (ARRAY['Residential', 'Commercial', 'Industrial', 'Luxury', 'Foreclosures'])[floor(random() * 5 + 1)],
                        (ARRAY['Sales', 'Rentals', 'Property Management', 'Investment', 'Development'])[floor(random() * 5 + 1)]
                    ]
                ),
                true, NOW(), NOW()
            )
            RETURNING id INTO profile_id;

        END LOOP;
    END LOOP;
END $$;