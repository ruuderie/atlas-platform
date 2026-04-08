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