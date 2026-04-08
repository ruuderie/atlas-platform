WITH business_data AS (
  SELECT 
      COALESCE(prefix, 'Default') || ' ' || COALESCE(suffix, 'Business') AS name,
      COALESCE(category, 'Miscellaneous') AS category,
      COALESCE(street, 'Main') AS street,
      COALESCE(city, 'Anytown') AS city,
      COALESCE(state, 'ST') AS state
  FROM (
      SELECT 
          prefix, suffix, category, street, city, state,
          ROW_NUMBER() OVER (ORDER BY RANDOM()) as rn
      FROM (
          VALUES 
              ('Sunset', 'Cafe'), ('Riverside', 'Bistro'), ('Oakwood', 'Grill'),
              ('Metro', 'Diner'), ('Green Valley', 'Restaurant'), ('Blue Ocean', 'Eatery'),
              ('Golden Gate', 'Market'), ('Silver Lake', 'Shop'), ('Mountain View', 'Store'),
              ('Cypress', 'Boutique'), ('Redwood', 'Emporium'), ('Evergreen', 'Tech'),
              ('Seaside', 'Solutions'), ('Hillcrest', 'Systems'), ('Moonlight', 'Innovations'),
              ('Starlight', 'Clinic'), ('Sunflower', 'Care'), ('Bluebird', 'Wellness'),
              ('Rosewood', 'Health'), ('Willow Creek', 'Academy')
      ) AS prefixes_suffixes(prefix, suffix),
      (
          VALUES 
              ('Restaurant'), ('Cafe'), ('Retail'), ('Grocery'), ('Technology'),
              ('Software'), ('Healthcare'), ('Fitness'), ('Education'), ('Automotive'),
              ('Real Estate'), ('Financial Services'), ('Legal Services'), ('Home Services'),
              ('Entertainment'), ('Art Gallery'), ('Bookstore'), ('Pet Services'),
              ('Travel Agency'), ('Photography')
      ) AS categories(category),
      (
          VALUES 
              ('Main'), ('Oak'), ('Pine'), ('Maple'), ('Cedar'), ('Elm'), ('Washington'),
              ('Park'), ('Lake'), ('Hill'), ('River'), ('Spring'), ('Market'), ('Church'),
              ('Bridge'), ('Walnut'), ('Highland'), ('Union'), ('Mill'), ('Willow')
      ) AS streets(street),
      (
          VALUES 
              ('New York', 'NY'), ('Los Angeles', 'CA'), ('Chicago', 'IL'), ('Houston', 'TX'),
              ('Phoenix', 'AZ'), ('Philadelphia', 'PA'), ('San Antonio', 'TX'), ('San Diego', 'CA'),
              ('Dallas', 'TX'), ('San Jose', 'CA'), ('Austin', 'TX'), ('Jacksonville', 'FL'),
              ('San Francisco', 'CA'), ('Columbus', 'OH'), ('Fort Worth', 'TX'), ('Indianapolis', 'IN'),
              ('Charlotte', 'NC'), ('Seattle', 'WA'), ('Denver', 'CO'), ('Washington', 'DC')
      ) AS cities_states(city, state)
  ) AS data
  WHERE rn <= 100
)
INSERT INTO public.business (name, category, address, phone, website)
SELECT 
  name,
  category,
  (RANDOM() * 9998 + 1)::INT || ' ' || street || ' ' ||
  CASE (RANDOM() * 3)::INT
      WHEN 0 THEN 'St'
      WHEN 1 THEN 'Ave'
      WHEN 2 THEN 'Blvd'
      ELSE 'Rd'
  END || ', ' || city || ', ' || state || ' ' || 
  (RANDOM() * 89999 + 10000)::INT AS address,
  '+1 (' || (RANDOM() * 900 + 100)::INT || ') ' || 
  (RANDOM() * 900 + 100)::INT || '-' || 
  (RANDOM() * 9000 + 1000)::INT AS phone,
  'www.' || LOWER(REGEXP_REPLACE(name, '[^a-zA-Z0-9]', '')) || '.com' AS website
FROM business_data;