-- Transportation & Logistics Categories
WITH tl_parent_categories AS (
    INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
    VALUES
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Transportation & Logistics'), 'Freight Services', 'Services related to freight transportation', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Transportation & Logistics'), 'Warehousing', 'Storage and warehousing services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Transportation & Logistics'), 'Supply Chain Management', 'End-to-end supply chain solutions', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Transportation & Logistics'), 'Courier Services', 'Package and document delivery services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Transportation & Logistics'), 'Fleet Management', 'Vehicle fleet management services', false, true, NOW(), NOW())
    RETURNING id, name
)
INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type WHERE name = 'Transportation & Logistics'),
    pc.id,
    sub.name,
    'Subcategory of ' || pc.name,
    false,
    true,
    NOW(),
    NOW()
FROM tl_parent_categories pc
CROSS JOIN LATERAL (
    VALUES 
        ('Freight Services', 'Air Freight'),
        ('Freight Services', 'Ocean Freight'),
        ('Freight Services', 'Road Freight'),
        ('Freight Services', 'Rail Freight'),
        ('Freight Services', 'Intermodal'),
        ('Warehousing', 'Cold Storage'),
        ('Warehousing', 'Bulk Storage'),
        ('Warehousing', 'Distribution Centers'),
        ('Warehousing', 'Fulfillment Services'),
        ('Supply Chain Management', 'Inventory Management'),
        ('Supply Chain Management', 'Demand Planning'),
        ('Supply Chain Management', 'Logistics Consulting'),
        ('Supply Chain Management', 'Reverse Logistics'),
        ('Courier Services', 'Express Delivery'),
        ('Courier Services', 'Same-Day Delivery'),
        ('Courier Services', 'International Shipping'),
        ('Courier Services', 'Specialized Handling'),
        ('Fleet Management', 'Vehicle Tracking'),
        ('Fleet Management', 'Maintenance Services'),
        ('Fleet Management', 'Fuel Management'),
        ('Fleet Management', 'Driver Management')
) AS sub(parent_name, name)
WHERE pc.name = sub.parent_name;

-- Automotive Sales Categories
WITH as_parent_categories AS (
    INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
    VALUES
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Automotive Sales'), 'New Vehicles', 'Sales of new automobiles', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Automotive Sales'), 'Used Vehicles', 'Sales of pre-owned automobiles', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Automotive Sales'), 'Auto Parts', 'Automotive parts and accessories', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Automotive Sales'), 'Specialty Vehicles', 'Sales of specialized automobiles', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Automotive Sales'), 'Vehicle Services', 'Automotive-related services', false, true, NOW(), NOW())
    RETURNING id, name
)
INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type WHERE name = 'Automotive Sales'),
    pc.id,
    sub.name,
    'Subcategory of ' || pc.name,
    false,
    true,
    NOW(),
    NOW()
FROM as_parent_categories pc
CROSS JOIN LATERAL (
    VALUES 
        ('New Vehicles', 'Sedans'),
        ('New Vehicles', 'SUVs'),
        ('New Vehicles', 'Trucks'),
        ('New Vehicles', 'Electric Vehicles'),
        ('New Vehicles', 'Luxury Cars'),
        ('Used Vehicles', 'Certified Pre-Owned'),
        ('Used Vehicles', 'Economy Cars'),
        ('Used Vehicles', 'Classic Cars'),
        ('Used Vehicles', 'Off-Lease Vehicles'),
        ('Auto Parts', 'Engine Parts'),
        ('Auto Parts', 'Body Parts'),
        ('Auto Parts', 'Interior Accessories'),
        ('Auto Parts', 'Performance Parts'),
        ('Specialty Vehicles', 'Commercial Vehicles'),
        ('Specialty Vehicles', 'RVs'),
        ('Specialty Vehicles', 'Motorcycles'),
        ('Specialty Vehicles', 'Off-Road Vehicles'),
        ('Vehicle Services', 'Financing'),
        ('Vehicle Services', 'Insurance'),
        ('Vehicle Services', 'Extended Warranties'),
        ('Vehicle Services', 'Vehicle Inspections')
) AS sub(parent_name, name)
WHERE pc.name = sub.parent_name;

-- Construction & Contracting Categories
WITH cc_parent_categories AS (
    INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
    VALUES
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Construction & Contracting'), 'Residential Construction', 'Home building and renovation services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Construction & Contracting'), 'Commercial Construction', 'Business and industrial construction services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Construction & Contracting'), 'Specialized Contracting', 'Specialized construction services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Construction & Contracting'), 'Construction Equipment', 'Construction machinery and tools', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Construction & Contracting'), 'Construction Management', 'Project management and consulting services', false, true, NOW(), NOW())
    RETURNING id, name
)
INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type WHERE name = 'Construction & Contracting'),
    pc.id,
    sub.name,
    'Subcategory of ' || pc.name,
    false,
    true,
    NOW(),
    NOW()
FROM cc_parent_categories pc
CROSS JOIN LATERAL (
    VALUES 
        ('Residential Construction', 'Custom Home Building'),
        ('Residential Construction', 'Home Renovations'),
        ('Residential Construction', 'Kitchen Remodeling'),
        ('Residential Construction', 'Bathroom Remodeling'),
        ('Residential Construction', 'Roofing'),
        ('Commercial Construction', 'Office Buildings'),
        ('Commercial Construction', 'Retail Spaces'),
        ('Commercial Construction', 'Industrial Facilities'),
        ('Commercial Construction', 'Healthcare Facilities'),
        ('Specialized Contracting', 'Electrical'),
        ('Specialized Contracting', 'Plumbing'),
        ('Specialized Contracting', 'HVAC'),
        ('Specialized Contracting', 'Landscaping'),
        ('Specialized Contracting', 'Painting'),
        ('Construction Equipment', 'Heavy Machinery'),
        ('Construction Equipment', 'Power Tools'),
        ('Construction Equipment', 'Safety Equipment'),
        ('Construction Equipment', 'Equipment Rental'),
        ('Construction Management', 'Project Planning'),
        ('Construction Management', 'Cost Estimation'),
        ('Construction Management', 'Quality Control'),
        ('Construction Management', 'Safety Management')
) AS sub(parent_name, name)
WHERE pc.name = sub.parent_name;

-- Beauty & Personal Care Categories
WITH bp_parent_categories AS (
    INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
    VALUES
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Beauty & Personal Care'), 'Hair Care', 'Hair styling and treatment services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Beauty & Personal Care'), 'Skin Care', 'Skin treatment and maintenance services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Beauty & Personal Care'), 'Nail Care', 'Nail styling and treatment services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Beauty & Personal Care'), 'Makeup Services', 'Makeup application and consultation', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Beauty & Personal Care'), 'Spa & Wellness', 'Relaxation and wellness services', false, true, NOW(), NOW())
    RETURNING id, name
)
INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type WHERE name = 'Beauty & Personal Care'),
    pc.id,
    sub.name,
    'Subcategory of ' || pc.name,
    false,
    true,
    NOW(),
    NOW()
FROM bp_parent_categories pc
CROSS JOIN LATERAL (
    VALUES 
        ('Hair Care', 'Hair Cutting'),
        ('Hair Care', 'Hair Coloring'),
        ('Hair Care', 'Hair Styling'),
        ('Hair Care', 'Hair Extensions'),
        ('Hair Care', 'Hair Treatments'),
        ('Skin Care', 'Facials'),
        ('Skin Care', 'Acne Treatments'),
        ('Skin Care', 'Anti-Aging Treatments'),
        ('Skin Care', 'Waxing'),
        ('Skin Care', 'Tanning'),
        ('Nail Care', 'Manicures'),
        ('Nail Care', 'Pedicures'),
        ('Nail Care', 'Nail Extensions'),
        ('Nail Care', 'Nail Art'),
        ('Nail Care', 'Nail Repair'),
        ('Makeup Services', 'Bridal Makeup'),
        ('Makeup Services', 'Special Event Makeup'),
        ('Makeup Services', 'Makeup Lessons'),
        ('Makeup Services', 'Permanent Makeup'),
        ('Spa & Wellness', 'Massage Therapy'),
        ('Spa & Wellness', 'Body Treatments'),
        ('Spa & Wellness', 'Hydrotherapy'),
        ('Spa & Wellness', 'Aromatherapy'),
        ('Spa & Wellness', 'Meditation Classes')
) AS sub(parent_name, name)
WHERE pc.name = sub.parent_name;

-- Financial Services Categories
WITH fs_parent_categories AS (
    INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
    VALUES
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Financial Services'), 'Business Banking', 'Banking services for businesses', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Financial Services'), 'Loans & Lending', 'Various loan and lending services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Financial Services'), 'Insurance Services', 'Various insurance products', false, true, NOW(), NOW())
    RETURNING id, name
)
INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type WHERE name = 'Financial Services'),
    pc.id,
    sub.name,
    'Subcategory of ' || pc.name,
    false,
    true,
    NOW(),
    NOW()
FROM fs_parent_categories pc
CROSS JOIN LATERAL (
    VALUES 
        ('Business Banking', 'Business Checking'),
        ('Business Banking', 'Business Savings'),
        ('Business Banking', 'Merchant Services'),
        ('Business Banking', 'Payroll Services'),
        ('Business Banking', 'Business Credit Cards'),
        ('Loans & Lending', 'Home Loans'),
        ('Loans & Lending', 'Auto Loans'),
        ('Loans & Lending', 'Student Loans'),
        ('Loans & Lending', 'Small Business Loans'),
        ('Loans & Lending', 'Commercial Real Estate Loans'),
        ('Insurance Services', 'Life Insurance'),
        ('Insurance Services', 'Health Insurance'),
        ('Insurance Services', 'Auto Insurance'),
        ('Insurance Services', 'Home Insurance'),
        ('Insurance Services', 'Business Insurance')
) AS sub(parent_name, name)
WHERE pc.name = sub.parent_name;

-- Healthcare Categories
WITH hc_parent_categories AS (
    INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
    VALUES
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Healthcare'), 'Medical Services', 'Healthcare services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Healthcare'), 'Pharmaceuticals', 'Pharmaceutical products and services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Healthcare'), 'Medical Equipment', 'Medical equipment and supplies', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Healthcare'), 'Health Insurance', 'Health insurance products and services', false, true, NOW(), NOW()),
    (gen_random_uuid(), (SELECT id FROM network_type WHERE name = 'Healthcare'), 'Healthcare Consulting', 'Healthcare consulting services', false, true, NOW(), NOW())
    RETURNING id, name
)
INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    (SELECT id FROM network_type WHERE name = 'Healthcare'),
    pc.id,
    sub.name,
    'Subcategory of ' || pc.name,
    false,
    true,
    NOW(),
    NOW()
FROM hc_parent_categories pc
CROSS JOIN LATERAL (
    VALUES 
        ('Medical Services', 'Primary Care'),
        ('Medical Services', 'Specialty Care'),
        ('Medical Services', 'Emergency Services'),
        ('Medical Services', 'Urgent Care'),
        ('Medical Services', 'Mental Health Services'),
        ('Medical Services', 'Physical Therapy'),
        ('Medical Services', 'Chiropractic Services'),
        ('Medical Services', 'Dental Services'),
        ('Pharmaceuticals', 'Pharmaceutical Manufacturing'),
        ('Pharmaceuticals', 'Pharmaceutical Distribution'),
        ('Pharmaceuticals', 'Pharmaceutical Retail'),
        ('Medical Equipment', 'Medical Imaging Equipment'),
        ('Medical Equipment', 'Medical Laboratory Equipment'),
        ('Medical Equipment', 'Medical Supplies'),
        ('Medical Equipment', 'Medical Furniture'),
        ('Medical Equipment', 'Medical Furniture')
) AS sub(parent_name, name)
WHERE pc.name = sub.parent_name;