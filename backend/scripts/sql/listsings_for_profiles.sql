DO $$
DECLARE
    last_listing_id UUID;
    profile_network_id UUID;
    profile_category_id UUID;
BEGIN

-- Listing 1: Michael Lopez, Real Estate Loan Finder
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'e7506381-dade-4a95-869a-5c08a2b048a5';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active)
VALUES (
  gen_random_uuid(),
  'e7506381-dade-4a95-869a-5c08a2b048a5',
  profile_network_id,
  profile_category_id,
  'Competitive Home Loan Rates for First-Time Buyers',
  'Lopez Real Estate Loan Finder offers exclusive rates for first-time homebuyers. Get pre-approved quickly and easily.',
  NULL,
  'pending',
  NOW() - interval '2 days',
  NOW(),
  'standard',
  'fixed',
  'United States',
  'California',
  'Los Angeles',
  'Downtown',
  34.0522,
  -118.2437,
  '{"specialization": "First-time homebuyers"}',
  false,
  false,
  NULL,
  false,
  true
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES 
  (gen_random_uuid(), last_listing_id, 'Fees', 'InterestRate', '"3.25"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'LoanType', '"Fixed 30-year"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'Fees', 'MinimumDownPayment', '"3.5"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'CreditScoreRequirement', '"620+"', NOW(), NOW());

-- Listing 2: Lisa Rodriguez, Real Estate Loan Finder
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '2b249afd-592e-427a-9cd7-d137d261b8c3';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active)
VALUES (
  gen_random_uuid(),
  '2b249afd-592e-427a-9cd7-d137d261b8c3',
  profile_network_id,
  profile_category_id,
  'Industrial Property Financing Solutions',
  'Specialized loan packages for industrial real estate investments. Competitive rates and flexible terms available.',
  NULL,
  'pending',
  NOW() - interval '5 days',
  NOW(),
  'standard',
  'variable',
  'United States',
  'New York',
  'New York City',
  'Manhattan',
  40.7128,
  -74.0060,
  '{"specialization": "Industrial properties"}',
  true,
  false,
  NULL,
  false,
  true
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES 
  (gen_random_uuid(), last_listing_id, 'Fees', 'MaxLoanAmount', '"5000000"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'LoanType', '"Commercial"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'LoanTerm', '"20"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'PropertyTypes', '"Warehouses, Manufacturing facilities, Distribution centers"', NOW(), NOW());

-- Listing 3: John Lopez, Business Loan Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '8f531dfb-7537-485a-99bd-d603457af209';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active)
VALUES (
  gen_random_uuid(),
  '8f531dfb-7537-485a-99bd-d603457af209',
  profile_network_id,
  profile_category_id,
  'Fast Business Loans for Equipment Financing',
  'Get quick approval for equipment financing. Loans up to $500,000 with competitive rates.',
  NULL,
  'pending',
  NOW() - interval '1 day',
  NOW(),
  'standard',
  'fixed',
  'United States',
  'Texas',
  'Houston',
  'Downtown',
  29.7604,
  -95.3698,
  '{"specialization": "Equipment financing"}',
  false,
  false,
  NULL,
  false,
  true
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES 
  (gen_random_uuid(), last_listing_id, 'Fees', 'MaxLoanAmount', '"500000"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'LoanPurpose', '"Equipment Financing"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'ApprovalTime', '"48"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'RequiredDocuments', '"Business plan, Financial statements, Equipment quotes"', NOW(), NOW());

-- Listing 4: Thomas Taylor, Business Loan Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'b6d8912b-d65b-4127-90ee-1291da46dbb1';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active)
VALUES (
  gen_random_uuid(),
  'b6d8912b-d65b-4127-90ee-1291da46dbb1',
  profile_network_id,
  profile_category_id,
  'SBA Loans for Small Business Growth',
  'Offering SBA-backed loans to help small businesses expand. Low down payments and long repayment terms available.',
  NULL,
  'pending',
  NOW() - interval '3 days',
  NOW(),
  'standard',
  'variable',
  'United States',
  'California',
  'San Francisco',
  'Financial District',
  37.7749,
  -122.4194,
  '{"specialization": "Small business growth"}',
  true,
  false,
  NULL,
  false,
  true
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES 
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'LoanType', '"SBA 7(a)"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'Fees', 'MaxLoanAmount', '"5000000"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'Fees', 'InterestRate', '"5.5"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'EligibleBusinesses', '"For-profit businesses operating in the US"', NOW(), NOW());

-- Listing 5: William Thomas, Business Loan Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'c6175986-aa61-4d80-a860-d04ebb4d4bb5';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active)
VALUES (
  gen_random_uuid(),
  'c6175986-aa61-4d80-a860-d04ebb4d4bb5',
  profile_network_id,
  profile_category_id,
  'Merchant Cash Advances for Retail Businesses',
  'Quick access to working capital for retail businesses. Flexible repayment based on daily credit card sales.',
  NULL,
  'pending',
  NOW() - interval '4 days',
  NOW(),
  'standard',
  'fixed',
  'United States',
  'Florida',
  'Miami',
  'Downtown',
  25.7617,
  -80.1918,
  '{"specialization": "Retail businesses"}',
  false,
  false,
  NULL,
  false,
  true
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (id, listing_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES 
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'FundingType', '"Merchant Cash Advance"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'Fees', 'AdvanceAmount', '"250000"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'RepaymentMethod', '"Percentage of daily credit card sales"', NOW(), NOW()),
  (gen_random_uuid(), last_listing_id, 'ServiceDetail', 'ApprovalTime', '"24-48 hours"', NOW(), NOW());

-- Listing 6: John Garcia, Acquisition Finance Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '324e8591-9e84-4022-b189-ace2bc36c523';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active)
VALUES (
  gen_random_uuid(),
  '324e8591-9e84-4022-b189-ace2bc36c523',
  profile_network_id,
DO $$
DECLARE
    last_listing_id UUID;
    profile_network_id UUID;
    profile_category_id UUID;
BEGIN

-- Listing 1: Michael Lopez, Real Estate Loan Finder
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'e7506381-dade-4a95-869a-5c08a2b048a5';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  'e7506381-dade-4a95-869a-5c08a2b048a5',
  profile_network_id,
  profile_category_id,
  'Competitive Home Loan Rates for First-Time Buyers',
  'Lopez Real Estate Loan Finder offers exclusive rates for first-time homebuyers. Get pre-approved quickly and easily.',
  NULL,
  'active',
  NOW() - interval '2 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Numeric', 'InterestRate', '3.25'),
  (last_listing_id, 'Text', 'LoanType', 'Fixed 30-year'),
  (last_listing_id, 'Numeric', 'MinimumDownPayment', '3.5'),
  (last_listing_id, 'Text', 'CreditScoreRequirement', '620+');

-- Listing 2: Lisa Rodriguez, Real Estate Loan Finder
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '2b249afd-592e-427a-9cd7-d137d261b8c3';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  '2b249afd-592e-427a-9cd7-d137d261b8c3',
  profile_network_id,
  profile_category_id,
  'Industrial Property Financing Solutions',
  'Specialized loan packages for industrial real estate investments. Competitive rates and flexible terms available.',
  NULL,
  'active',
  NOW() - interval '5 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Numeric', 'MaxLoanAmount', '5000000'),
  (last_listing_id, 'Text', 'LoanType', 'Commercial'),
  (last_listing_id, 'Numeric', 'LoanTerm', '20'),
  (last_listing_id, 'Text', 'PropertyTypes', 'Warehouses, Manufacturing facilities, Distribution centers');

-- Listing 3: John Lopez, Business Loan Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '8f531dfb-7537-485a-99bd-d603457af209';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  '8f531dfb-7537-485a-99bd-d603457af209',
  profile_network_id,
  profile_category_id,
  'Fast Business Loans for Equipment Financing',
  'Get quick approval for equipment financing. Loans up to $500,000 with competitive rates.',
  NULL,
  'active',
  NOW() - interval '1 day',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Numeric', 'MaxLoanAmount', '500000'),
  (last_listing_id, 'Text', 'LoanPurpose', 'Equipment Financing'),
  (last_listing_id, 'Numeric', 'ApprovalTime', '48'),
  (last_listing_id, 'Text', 'RequiredDocuments', 'Business plan, Financial statements, Equipment quotes');

-- Listing 4: Thomas Taylor, Business Loan Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'b6d8912b-d65b-4127-90ee-1291da46dbb1';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  'b6d8912b-d65b-4127-90ee-1291da46dbb1',
  profile_network_id,
  profile_category_id,
  'SBA Loans for Small Business Growth',
  'Offering SBA-backed loans to help small businesses expand. Low down payments and long repayment terms available.',
  NULL,
  'active',
  NOW() - interval '3 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Text', 'LoanType', 'SBA 7(a)'),
  (last_listing_id, 'Numeric', 'MaxLoanAmount', '5000000'),
  (last_listing_id, 'Numeric', 'InterestRate', '5.5'),
  (last_listing_id, 'Text', 'EligibleBusinesses', 'For-profit businesses operating in the US');

-- Listing 5: William Thomas, Business Loan Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'c6175986-aa61-4d80-a860-d04ebb4d4bb5';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  'c6175986-aa61-4d80-a860-d04ebb4d4bb5',
  profile_network_id,
  profile_category_id,
  'Merchant Cash Advances for Retail Businesses',
  'Quick access to working capital for retail businesses. Flexible repayment based on daily credit card sales.',
  NULL,
  'active',
  NOW() - interval '4 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Text', 'FundingType', 'Merchant Cash Advance'),
  (last_listing_id, 'Numeric', 'AdvanceAmount', '250000'),
  (last_listing_id, 'Text', 'RepaymentMethod', 'Percentage of daily credit card sales'),
  (last_listing_id, 'Text', 'ApprovalTime', '24-48 hours');

-- Listing 6: John Garcia, Acquisition Finance Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '324e8591-9e84-4022-b189-ace2bc36c523';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  '324e8591-9e84-4022-b189-ace2bc36c523',
  profile_network_id,
  profile_category_id,
  'Acquisition Financing for Tech Startups',
  'Specialized financing solutions for tech startups looking to acquire other companies. Flexible terms and quick approval process.',
  NULL,
  'active',
  NOW() - interval '6 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Text', 'IndustryFocus', 'Technology'),
  (last_listing_id, 'Numeric', 'MinDealSize', '1000000'),
  (last_listing_id, 'Text', 'FinancingType', 'Debt and Equity'),
  (last_listing_id, 'Text', 'ApprovalTimeframe', '2-4 weeks');

-- Listing 7: Michael Smith, Acquisition Finance Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '1f1f6014-fdf9-4d36-8669-a9c61e7cbd4c';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  '1f1f6014-fdf9-4d36-8669-a9c61e7cbd4c',
  profile_network_id,
  profile_category_id,
  'Mezzanine Financing for Middle Market Acquisitions',
  'Offering mezzanine financing solutions for middle market companies looking to fund acquisitions or recapitalizations.',
  NULL,
  'active',
  NOW() - interval '7 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Text', 'FinancingType', 'Mezzanine'),
  (last_listing_id, 'Numeric', 'TypicalDealSize', '10000000'),
  (last_listing_id, 'Text', 'TargetCompanies', 'Middle market firms with EBITDA $5M-$50M'),
  (last_listing_id, 'Text', 'UseOfFunds', 'Acquisitions, Recapitalizations, Growth capital');

-- Listing 8: Elizabeth Johnson, Salon & Spa Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '5726dbf0-bfdc-41e9-ad7f-d3735764a992';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  '5726dbf0-bfdc-41e9-ad7f-d3735764a992',
  profile_network_id,
  profile_category_id,
  'Luxury Spa Day Package',
  'Indulge in a full day of pampering with our luxury spa package. Includes massage, facial, manicure, and pedicure.',
  299.99,
  'active',
  NOW() - interval '2 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Numeric', 'Duration', '6'),
  (last_listing_id, 'Text', 'Includes', 'Swedish massage, Hydrating facial, Gel manicure, Deluxe pedicure'),
  (last_listing_id, 'Text', 'Availability', 'Monday-Saturday'),
  (last_listing_id, 'Text', 'BookingRequired', 'Yes, 24 hours in advance');

-- Listing 9: Margaret Miller, Salon & Spa Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = '448d426d-1c03-4732-8c8e-8f6dffc83bb5';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  '448d426d-1c03-4732-8c8e-8f6dffc83bb5',
  profile_network_id,
  profile_category_id,
  'Bridal Hair and Makeup Package',
  'Look your best on your special day with our bridal hair and makeup package. Includes trial session and day-of services.',
  399.99,
  'active',
  NOW() - interval '5 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Text', 'Includes', 'Trial hair and makeup session, Day-of hair styling, Day-of makeup application'),
  (last_listing_id, 'Numeric', 'Duration', '4'),
  (last_listing_id, 'Text', 'Location', 'In-salon or on-site'),
  (last_listing_id, 'Text', 'AdditionalServices', 'Bridal party services available');

-- Listing 10: Linda Johnson, Beauty Pro Network
SELECT network_id INTO profile_network_id 
FROM profile WHERE id = 'c4cfe651-2e7d-4e2e-95b5-e1e9c76d8e30';
SELECT id INTO profile_category_id 
FROM category 
LIMIT 1;

INSERT INTO listing (id, profile_id, network_id, category_id, title, description, price, status, created_at, updated_at, listing_type)
VALUES (
  gen_random_uuid(),
  'c4cfe651-2e7d-4e2e-95b5-e1e9c76d8e30',
  profile_network_id,
  profile_category_id,
  'Professional Makeup Artistry Course',
  'Comprehensive 8-week course covering all aspects of professional makeup artistry. Perfect for beginners and intermediate artists.',
  1299.99,
  'active',
  NOW() - interval '3 days',
  NOW(),
  'standard'
) RETURNING id INTO last_listing_id;

INSERT INTO listing_attribute (listing_id, attribute_type, attribute_key, value)
VALUES 
  (last_listing_id, 'Numeric', 'Duration', '8'),
  (last_listing_id, 'Text', 'Schedule', 'Twice weekly, 3 hours per session'),
  (last_listing_id, 'Text', 'Curriculum', 'Color theory, Face shapes, Bridal makeup, Special effects'),
  (last_listing_id, 'Text', 'Materials', 'Professional makeup kit included');

END $$;