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