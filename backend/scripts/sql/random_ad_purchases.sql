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