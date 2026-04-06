-- Create random networks
INSERT INTO network (id, network_type_id, name, domain, description, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type ORDER BY RANDOM() LIMIT 1),
    'Network ' || generate_series(1, 10),
    lower('network' || generate_series(1, 10) || '.com'),
    'Description for Network ' || generate_series(1, 10),
    NOW() - (random() * interval '365 days'),
    NOW()
FROM generate_series(1, 10);

-- Create random profiles
INSERT INTO profile (id, account_id, network_id, profile_type, display_name, contact_info, business_name, business_address, business_phone, business_website, additional_info, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM account ORDER BY RANDOM() LIMIT 1),
    (SELECT id FROM network ORDER BY RANDOM() LIMIT 1),
    CASE WHEN random() < 0.5 THEN 'Individual' ELSE 'Business' END,
    'Profile ' || generate_series(1, 100),
    'contact' || generate_series(1, 100) || '@example.com',
    CASE WHEN random() < 0.5 THEN 'Business ' || generate_series(1, 100) ELSE NULL END,
    CASE WHEN random() < 0.5 THEN (floor(random() * 9999) + 1)::text || ' Main St, City ' || generate_series(1, 100) || ', State' ELSE NULL END,
    CASE WHEN random() < 0.5 THEN '+1 ' || floor(random() * 999 + 100)::text || '-' || floor(random() * 999 + 100)::text || '-' || floor(random() * 9999 + 1000)::text ELSE NULL END,
    CASE WHEN random() < 0.5 THEN 'www.business' || generate_series(1, 100) || '.com' ELSE NULL END,
    '{"key": "value"}',
    true,
    NOW() - (random() * interval '365 days'),
    NOW()
FROM generate_series(1, 100);

-- Create random listings
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM profile ORDER BY RANDOM() LIMIT 1),
    (SELECT id FROM network ORDER BY RANDOM() LIMIT 1),
    (SELECT id FROM category ORDER BY RANDOM() LIMIT 1),
    'Listing ' || generate_series(1, 500),
    'Description for Listing ' || generate_series(1, 500),
    (ARRAY['Service', 'Product', 'Event'])[floor(random() * 3 + 1)],
    CASE WHEN random() < 0.8 THEN floor(random() * 10000)::bigint ELSE NULL END,
    CASE WHEN random() < 0.8 THEN (ARRAY['Fixed', 'Hourly', 'Daily'])[floor(random() * 3 + 1)] ELSE NULL END,
    'United States',
    (ARRAY['CA', 'NY', 'TX', 'FL', 'IL'])[floor(random() * 5 + 1)],
    (ARRAY['Los Angeles', 'New York', 'Houston', 'Miami', 'Chicago'])[floor(random() * 5 + 1)],
    CASE WHEN random() < 0.5 THEN 'Neighborhood ' || generate_series(1, 500) ELSE NULL END,
    CASE WHEN random() < 0.5 THEN random() * 180 - 90 ELSE NULL END,
    CASE WHEN random() < 0.5 THEN random() * 360 - 180 ELSE NULL END,
    '{"key": "value"}',
    (ARRAY['pending', 'approved', 'rejected'])[floor(random() * 3 + 1)],
    random() < 0.1,
    random() < 0.2,
    CASE WHEN random() < 0.2 THEN (SELECT id FROM template ORDER BY RANDOM() LIMIT 1) ELSE NULL END,
    random() < 0.05,
    true,
    NOW() - (random() * interval '365 days'),
    NOW()
FROM generate_series(1, 500);

-- Create random listing attributes
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    CASE WHEN random() < 0.8 THEN (SELECT id FROM listing ORDER BY RANDOM() LIMIT 1) ELSE NULL END,
    CASE WHEN random() < 0.2 THEN (SELECT id FROM template ORDER BY RANDOM() LIMIT 1) ELSE NULL END,
    (ARRAY['text', 'number', 'boolean', 'date'])[floor(random() * 4 + 1)],
    'Attribute ' || generate_series(1, 1000),
    CASE 
        WHEN (ARRAY['text', 'number', 'boolean', 'date'])[floor(random() * 4 + 1)] = 'text' THEN 'Sample text'
        WHEN (ARRAY['text', 'number', 'boolean', 'date'])[floor(random() * 4 + 1)] = 'number' THEN floor(random() * 100)::text
        WHEN (ARRAY['text', 'number', 'boolean', 'date'])[floor(random() * 4 + 1)] = 'boolean' THEN (ARRAY['true', 'false'])[floor(random() * 2 + 1)]
        ELSE NOW()::date::text
    END,
    NOW() - (random() * interval '365 days'),
    NOW()
FROM generate_series(1, 1000);

-- Create random ad purchases
INSERT INTO ad_purchase (id, listing_id, profile_id, start_date, end_date, status, amount, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM listing ORDER BY RANDOM() LIMIT 1),
    (SELECT id FROM profile ORDER BY RANDOM() LIMIT 1),
    NOW() + (random() * interval '30 days'),
    NOW() + (random() * interval '60 days'),
    (ARRAY['pending', 'active', 'completed', 'cancelled'])[floor(random() * 4 + 1)],
    floor(random() * 1000)::numeric(10, 2),
    NOW() - (random() * interval '365 days'),
    NOW()
FROM generate_series(1, 200);