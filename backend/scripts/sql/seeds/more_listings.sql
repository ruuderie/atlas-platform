-- Commercial Property Management Services
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '61fc549b-0d31-4551-b3d7-1507f211844e',
  '34719ace-bc2b-427b-b17c-4e7e88d8475a',
  '7c60f6ab-34ae-4b2f-a236-e5dfda7e0060',
  'Commercial Property Management Services',
  'Comprehensive commercial property management services for office buildings, retail spaces, and industrial properties.',
  'service',
  5000,
  'monthly',
  'USA',
  'Arizona',
  'Phoenix',
  'Downtown',
  33.4484,
  -112.0740,
  '{"services": ["Tenant relations", "Maintenance coordination", "Financial reporting", "Lease administration"], "property_types": ["Office", "Retail", "Industrial"], "minimum_square_footage": 10000}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- Eco-Friendly Home Renovation
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '448d426d-1c03-4732-8c8e-8f6dffc83bb5',
  'e6c78333-35d1-45ea-bfaf-878ee1520295',
  '10c5e4eb-ff71-4133-a4a1-f4e3b993254d',
  'Eco-Friendly Home Renovation',
  'Sustainable home renovation services using eco-friendly materials and energy-efficient solutions.',
  'service',
  75000,
  'project-based',
  'USA',
  'Oregon',
  'Portland',
  'Pearl District',
  45.5231,
  -122.6765,
  '{"services": ["Energy audits", "Solar panel installation", "Sustainable material sourcing", "Water conservation systems"], "certification": "LEED Accredited", "average_project_duration": "3-6 months"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- Mobile Auto Detailing Service
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '340d0ec8-998d-4cfd-8e90-272f6ff13697',
  '77f4b52b-8222-4a5c-8cda-a77505e15454',
  '830ffb02-c53b-47b7-9d27-f63c06ba8e85',
  'Mobile Auto Detailing Service',
  'Professional auto detailing service that comes to you. We offer interior and exterior detailing for all vehicle types.',
  'service',
  150,
  'starting-at',
  'USA',
  'Florida',
  'Miami',
  'Brickell',
  25.7617,
  -80.1918,
  '{"services": ["Exterior wash and wax", "Interior deep cleaning", "Engine bay detailing", "Headlight restoration"], "service_area": "Within 30 miles of Miami", "appointment_availability": "7 days a week"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- 2021 Airstream Globetrotter 27FB
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '12547edf-3b4a-4cad-87dc-ecceeb9800c1',
  '77f4b52b-8222-4a5c-8cda-a77505e15454',
  '8e7bd86b-6af0-46ae-b570-a46c7c367f1e',
  '2021 Airstream Globetrotter 27FB',
  'Luxurious 2021 Airstream Globetrotter 27FB. Perfect for comfortable long-distance travel and camping.',
  'product',
  95000,
  'fixed',
  'USA',
  'Colorado',
  'Denver',
  'LoDo',
  39.7392,
  -104.9903,
  '{"length": "27 feet", "sleeps": "Up to 6", "features": ["Solar panels", "Smart control technology", "Ducted air conditioning"], "included": "1 year of maintenance"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- Business Startup Consulting
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  'b6d8912b-d65b-4127-90ee-1291da46dbb1',
  '0a529c09-3a95-4e30-9875-e8544928cd9c',
  '6788c8f0-71b8-4193-893b-79ab8489fd16',
  'Business Startup Consulting',
  'Expert consulting services for startups. From business plan development to funding acquisition, we guide you through every step.',
  'service',
  2000,
  'monthly',
  'USA',
  'Texas',
  'Austin',
  'Downtown',
  30.2672,
  -97.7431,
  '{"services": ["Business plan development", "Market analysis", "Financial projections", "Pitch deck creation"], "industries": ["Tech", "E-commerce", "Green energy"], "consultation_method": "In-person and virtual options available"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- Advanced CNC Machining Services
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '5706e747-8a16-472c-b4b5-667ff0ca4c11',
  '152f995e-3109-45a8-86d8-924697e207f2',
  '7c60f6ab-34ae-4b2f-a236-e5dfda7e0060',
  'Advanced CNC Machining Services',
  'Precision CNC machining services for various industries. We specialize in complex parts and tight tolerances.',
  'service',
  0,
  'quote',
  'USA',
  'Michigan',
  'Detroit',
  'Midtown',
  42.3314,
  -83.0458,
  '{"capabilities": ["5-axis machining", "Micro-machining", "Large part machining"], "materials": ["Aluminum", "Steel", "Titanium", "Plastics"], "industries_served": ["Aerospace", "Automotive", "Medical"], "certifications": ["ISO 9001:2015", "AS9100D"]}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- Luxury Yacht Charter
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '830622b2-2be8-4d4f-a916-1557c8d8e497',
  'e6c78333-35d1-45ea-bfaf-878ee1520295',
  '8e7bd86b-6af0-46ae-b570-a46c7c367f1e',
  'Luxury Yacht Charter',
  'Experience the ultimate in luxury with our yacht charter service. Cruise the Caribbean in style.',
  'service',
  15000,
  'per-day',
  'USA',
  'Florida',
  'Miami',
  'South Beach',
  25.7617,
  -80.1918,
  '{"yacht_details": {"name": "Sea Breeze", "length": "120 feet", "capacity": "12 guests"}, "amenities": ["5 luxurious cabins", "Jacuzzi", "Jet skis", "Gourmet chef"], "destinations": ["Bahamas", "Virgin Islands", "St. Barts"], "minimum_charter": "3 days"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- AI-Powered Marketing Analytics Platform
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '17319eb2-6430-4679-8c4c-d2337e6217ac',
  '34719ace-bc2b-427b-b17c-4e7e88d8475a',
  'e2161009-0baa-4535-8baf-f791055614c5',
  'AI-Powered Marketing Analytics Platform',
  'Harness the power of AI for your marketing efforts. Our platform provides deep insights and predictive analytics to optimize your campaigns.',
  'product',
  1000,
  'monthly',
  'USA',
  'California',
  'San Francisco',
  'SoMa',
  37.7749,
  -122.4194,
  '{"features": ["Real-time data analysis", "Predictive modeling", "Multi-channel attribution", "Custom reporting"], "integrations": ["Google Analytics", "Facebook Ads", "Salesforce"], "onboarding": "Included", "support": "24/7 priority support"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);

-- Gourmet Food Truck for Sale
INSERT INTO listing (id, profile_id, network_id, category_id, title, description, listing_type, price, price_type, country, state, city, neighborhood, latitude, longitude, additional_info, status, is_featured, is_based_on_template, based_on_template_id, is_ad_placement, is_active, created_at, updated_at)
VALUES (
  gen_random_uuid(),
  '324e8591-9e84-4022-b189-ace2bc36c523',
  '152f995e-3109-45a8-86d8-924697e207f2',
  '70f54a8c-4770-4c8b-9bcc-609188cdc264',
  'Gourmet Food Truck for Sale',
  'Fully equipped gourmet food truck ready for business. Perfect for aspiring restaurateurs or established brands looking to expand.',
  'product',
  85000,
  'fixed',
  'USA',
  'California',
  'Los Angeles',
  'Downtown',
  34.0522,
  -118.2437,
  '{"vehicle": "2019 Ford F-59 Step Van", "length": "22 feet", "equipment": ["Commercial grade kitchen", "Refrigeration", "POS system"], "permits": "All current LA County permits included", "reason_for_selling": "Owner relocating"}',
  'active',
  true,
  false,
  NULL,
  false,
  true,
  NOW(),
  NOW()
);