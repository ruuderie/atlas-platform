INSERT INTO site_settings (key, value) VALUES 
    ('meta_title', 'Ruud Salym Erie - Technical Architect'),
    ('meta_description', 'Technical Architect and Software Engineer specializing in Rust, Salesforce, and high-performance enterprise applications.'),
    ('og_image', '')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
