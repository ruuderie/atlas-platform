-- Small Business Loan - Quick Approval
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '4241d04c-7543-44c2-a834-1380e562212c', NULL, 'ServiceDetail', 'Specialization', '"Small Business Loans"', NOW(), NOW()),
  (gen_random_uuid(), '4241d04c-7543-44c2-a834-1380e562212c', NULL, 'ServiceDetail', 'LoanAmount', '{"min": 10000, "max": 500000}', NOW(), NOW()),
  (gen_random_uuid(), '4241d04c-7543-44c2-a834-1380e562212c', NULL, 'ServiceDetail', 'InterestRate', '{"min": 4.5, "max": 8.5}', NOW(), NOW()),
  (gen_random_uuid(), '4241d04c-7543-44c2-a834-1380e562212c', NULL, 'ServiceDetail', 'LoanTerm', '["1 year", "3 years", "5 years"]', NOW(), NOW());

-- Advanced Inventory Management Solutions
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), 'bcadc675-2e04-480d-87eb-e9a917a64394', NULL, 'ProductDetail', 'MaxSKUCapacity', '1000000', NOW(), NOW()),
  (gen_random_uuid(), 'bcadc675-2e04-480d-87eb-e9a917a64394', NULL, 'ProductDetail', 'SupportedIntegrations', '["Shopify", "WooCommerce", "Amazon", "eBay"]', NOW(), NOW()),
  (gen_random_uuid(), 'bcadc675-2e04-480d-87eb-e9a917a64394', NULL, 'ProductDetail', 'MultiWarehouseSupport', 'true', NOW(), NOW()),
  (gen_random_uuid(), 'bcadc675-2e04-480d-87eb-e9a917a64394', NULL, 'ProductDetail', 'MobileApp', '{"ios": true, "android": true}', NOW(), NOW());

-- Nationwide LTL Freight Services
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '178c2e8f-4c50-4b37-8d81-a8871e1616c0', NULL, 'ServiceDetail', 'ServiceArea', '["Continental US", "Alaska", "Hawaii"]', NOW(), NOW()),
  (gen_random_uuid(), '178c2e8f-4c50-4b37-8d81-a8871e1616c0', NULL, 'ServiceDetail', 'TransitTime', '{"min": 1, "max": 5, "unit": "business days"}', NOW(), NOW()),
  (gen_random_uuid(), '178c2e8f-4c50-4b37-8d81-a8871e1616c0', NULL, 'ServiceDetail', 'HazmatShipping', 'true', NOW(), NOW()),
  (gen_random_uuid(), '178c2e8f-4c50-4b37-8d81-a8871e1616c0', NULL, 'ServiceDetail', 'TrackingMethod', '["Online", "Mobile App", "SMS"]', NOW(), NOW());

-- International Air Freight Services
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), 'f9baf871-1da5-423b-bff1-55c92465b3db', NULL, 'ServiceDetail', 'Destinations', '["Europe", "Asia", "South America", "Africa", "Australia"]', NOW(), NOW()),
  (gen_random_uuid(), 'f9baf871-1da5-423b-bff1-55c92465b3db', NULL, 'ServiceDetail', 'CargoTypes', '["General", "Perishable", "Dangerous Goods", "Oversized"]', NOW(), NOW()),
  (gen_random_uuid(), 'f9baf871-1da5-423b-bff1-55c92465b3db', NULL, 'ServiceDetail', 'CustomsClearance', 'true', NOW(), NOW()),
  (gen_random_uuid(), 'f9baf871-1da5-423b-bff1-55c92465b3db', NULL, 'ServiceDetail', 'MinimumWeight', '{"value": 10, "unit": "kg"}', NOW(), NOW());

-- 2022 Ford Transit 250 Cargo Van
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '18cda2d2-e5e5-4cdd-a119-3d5e2814566b', NULL, 'ProductDetail', 'Mileage', '15200', NOW(), NOW()),
  (gen_random_uuid(), '18cda2d2-e5e5-4cdd-a119-3d5e2814566b', NULL, 'ProductDetail', 'FuelType', '"Gasoline"', NOW(), NOW()),
  (gen_random_uuid(), '18cda2d2-e5e5-4cdd-a119-3d5e2814566b', NULL, 'ProductDetail', 'Transmission', '"Automatic"', NOW(), NOW()),
  (gen_random_uuid(), '18cda2d2-e5e5-4cdd-a119-3d5e2814566b', NULL, 'ProductDetail', 'CargoVolume', '{"value": 277.7, "unit": "cubic feet"}', NOW(), NOW());

-- 2023 Tesla Model 3 Long Range
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '9631dc2c-e6f3-4b95-bfd5-1780d5ef48ad', NULL, 'ProductDetail', 'Range', '{"value": 358, "unit": "miles"}', NOW(), NOW()),
  (gen_random_uuid(), '9631dc2c-e6f3-4b95-bfd5-1780d5ef48ad', NULL, 'ProductDetail', 'Acceleration', '{"value": 4.2, "unit": "seconds", "description": "0-60 mph"}', NOW(), NOW()),
  (gen_random_uuid(), '9631dc2c-e6f3-4b95-bfd5-1780d5ef48ad', NULL, 'ProductDetail', 'Autopilot', '"Enhanced Autopilot"', NOW(), NOW()),
  (gen_random_uuid(), '9631dc2c-e6f3-4b95-bfd5-1780d5ef48ad', NULL, 'ProductDetail', 'InteriorColor', '"Black"', NOW(), NOW());

-- 2020 BMW 5 Series 530i xDrive
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '4066163f-b1a6-4741-a111-ee0230eeabb4', NULL, 'ProductDetail', 'Mileage', '28500', NOW(), NOW()),
  (gen_random_uuid(), '4066163f-b1a6-4741-a111-ee0230eeabb4', NULL, 'ProductDetail', 'Drivetrain', '"All-Wheel Drive"', NOW(), NOW()),
  (gen_random_uuid(), '4066163f-b1a6-4741-a111-ee0230eeabb4', NULL, 'ProductDetail', 'Engine', '"2.0L Turbo Inline-4"', NOW(), NOW()),
  (gen_random_uuid(), '4066163f-b1a6-4741-a111-ee0230eeabb4', NULL, 'ProductDetail', 'HeatedSeats', 'true', NOW(), NOW()),
  (gen_random_uuid(), '4066163f-b1a6-4741-a111-ee0230eeabb4', NULL, 'ProductDetail', 'Warranty', '"CPO warranty until 2026 or 100,000 miles"', NOW(), NOW());

-- Office Building Construction
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '92975af1-2beb-4ecf-ae9c-9125da0ddbd7', NULL, 'ServiceDetail', 'SquareFootage', '{"min": 50000, "max": 200000}', NOW(), NOW()),
  (gen_random_uuid(), '92975af1-2beb-4ecf-ae9c-9125da0ddbd7', NULL, 'ServiceDetail', 'ConstructionType', '"Steel Frame"', NOW(), NOW()),
  (gen_random_uuid(), '92975af1-2beb-4ecf-ae9c-9125da0ddbd7', NULL, 'ServiceDetail', 'GreenCertification', '"LEED Gold"', NOW(), NOW()),
  (gen_random_uuid(), '92975af1-2beb-4ecf-ae9c-9125da0ddbd7', NULL, 'ServiceDetail', 'EstimatedCompletionTime', '{"value": 18, "unit": "months"}', NOW(), NOW());

-- Kitchen Remodeling Services
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '4ed611f7-b557-46c8-b8d2-b91bf67476a5', NULL, 'ServiceDetail', 'ServicesOffered', '["Cabinet Installation", "Countertop Replacement", "Flooring", "Lighting", "Plumbing"]', NOW(), NOW()),
  (gen_random_uuid(), '4ed611f7-b557-46c8-b8d2-b91bf67476a5', NULL, 'ServiceDetail', 'AverageProjectDuration', '{"value": 4, "unit": "weeks"}', NOW(), NOW()),
  (gen_random_uuid(), '4ed611f7-b557-46c8-b8d2-b91bf67476a5', NULL, 'ServiceDetail', '3DDesignService', 'true', NOW(), NOW()),
  (gen_random_uuid(), '4ed611f7-b557-46c8-b8d2-b91bf67476a5', NULL, 'ServiceDetail', 'Warranty', '"5-year workmanship warranty"', NOW(), NOW());

-- Custom Home Building Services
INSERT INTO listing_attribute (id, listing_id, template_id, attribute_type, attribute_key, value, created_at, updated_at)
VALUES
  (gen_random_uuid(), '6ace4a4d-e696-408a-bbff-805105570926', NULL, 'ServiceDetail', 'MinimumProjectSize', '{"value": 2500, "unit": "square feet"}', NOW(), NOW()),
  (gen_random_uuid(), '6ace4a4d-e696-408a-bbff-805105570926', NULL, 'ServiceDetail', 'ArchitecturalStyles', '["Modern", "Contemporary", "Traditional", "Mediterranean"]', NOW(), NOW()),
  (gen_random_uuid(), '6ace4a4d-e696-408a-bbff-805105570926', NULL, 'ServiceDetail', 'EnergyEfficientConstruction', 'true', NOW(), NOW()),
  (gen_random_uuid(), '6ace4a4d-e696-408a-bbff-805105570926', NULL, 'ServiceDetail', 'BuildAreas', '["San Antonio", "Austin", "Houston"]', NOW(), NOW());