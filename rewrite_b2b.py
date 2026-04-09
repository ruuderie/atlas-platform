import re

with open("apps/anchor/src/b2b.rs", "r") as f:
    content = f.read()

# Replace Service logic
content = content.replace(
    'SELECT id, title, description, deliverables, price_range, is_visible, display_order FROM services WHERE is_visible = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC',
    "SELECT id, title, payload->>'description' as description, payload->'deliverables' as deliverables, payload->>'price_range' as price_range, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'service' AND status = 'published' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
)
content = content.replace(
    'SELECT id, title, description, deliverables, price_range, is_visible, display_order FROM services WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC',
    "SELECT id, title, payload->>'description' as description, payload->'deliverables' as deliverables, payload->>'price_range' as price_range, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'service' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
)

content = content.replace(
    '"INSERT INTO services (tenant_id, title, description, deliverables, price_range, is_visible, display_order) VALUES ($$7, $$1, $$2, $$3, $$4, $$5, $$6)")\n        .bind(title).bind(description).bind(deliv_json).bind(price_range).bind(is_visible).bind(display_order)\n        .bind(tenant.0)',
    '"INSERT INTO app_content (tenant_id, collection_type, title, payload, status, display_order) VALUES ($$6, \\'service\\', $$1, $$2, $$3, $$4)")\n        .bind(title).bind(serde_json::json!({"description": description, "deliverables": deliverables, "price_range": price_range})).bind(if is_visible { "published" } else { "hidden" }).bind(display_order)\n        .bind(tenant.0)'
)

content = content.replace(
    '"UPDATE services SET title = $1, description = $2, deliverables = $3, price_range = $4, is_visible = $5, display_order = $$6 WHERE id = $$7 AND tenant_id IS NOT DISTINCT FROM $$8")\n        .bind(title).bind(description).bind(deliv_json).bind(price_range).bind(is_visible).bind(display_order).bind(id)\n        .bind(tenant.0)',
    '"UPDATE app_content SET title = $1, payload = $2, status = $3, display_order = $$4 WHERE id = $$5 AND tenant_id IS NOT DISTINCT FROM $$6")\n        .bind(title).bind(serde_json::json!({"description": description, "deliverables": deliverables, "price_range": price_range})).bind(if is_visible { "published" } else { "hidden" }).bind(display_order).bind(id)\n        .bind(tenant.0)'
)

content = content.replace(
    '"DELETE FROM services WHERE id = $$1 AND tenant_id IS NOT DISTINCT FROM $$2"',
    '"DELETE FROM app_content WHERE id = $$1 AND collection_type = \\'service\\' AND tenant_id IS NOT DISTINCT FROM $$2"'
)

# Case studies
content = content.replace(
    'SELECT id, client_name, problem, solution, roi_impact, is_visible, display_order FROM case_studies WHERE is_visible = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC',
    "SELECT id, payload->>'client_name' as client_name, payload->>'problem' as problem, payload->>'solution' as solution, payload->>'roi_impact' as roi_impact, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'case_study' AND status = 'published' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
)
content = content.replace(
    'SELECT id, client_name, problem, solution, roi_impact, is_visible, display_order FROM case_studies WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC',
    "SELECT id, payload->>'client_name' as client_name, payload->>'problem' as problem, payload->>'solution' as solution, payload->>'roi_impact' as roi_impact, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'case_study' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
)

content = content.replace(
    '"INSERT INTO case_studies (tenant_id, client_name, problem, solution, roi_impact, is_visible, display_order) VALUES ($$7, $$1, $$2, $$3, $$4, $$5, $$6)")\n        .bind(client_name).bind(problem).bind(solution).bind(roi_impact).bind(is_visible).bind(display_order)',
    '"INSERT INTO app_content (tenant_id, collection_type, title, payload, status, display_order) VALUES ($$5, \\'case_study\\', \\'Case Study\\', $$1, $$2, $$3)")\n        .bind(serde_json::json!({"client_name": client_name, "problem": problem, "solution": solution, "roi_impact": roi_impact})).bind(if is_visible { "published" } else { "hidden" }).bind(display_order)\n        .bind(tenant.0)'
)

content = content.replace(
    '"UPDATE case_studies SET client_name = $1, problem = $2, solution = $3, roi_impact = $4, is_visible = $5, display_order = $$6 WHERE id = $$7 AND tenant_id IS NOT DISTINCT FROM $$8")\n        .bind(client_name).bind(problem).bind(solution).bind(roi_impact).bind(is_visible).bind(display_order).bind(id)',
    '"UPDATE app_content SET payload = $1, status = $2, display_order = $$3 WHERE id = $$4 AND tenant_id IS NOT DISTINCT FROM $$5")\n        .bind(serde_json::json!({"client_name": client_name, "problem": problem, "solution": solution, "roi_impact": roi_impact})).bind(if is_visible { "published" } else { "hidden" }).bind(display_order).bind(id)\n        .bind(tenant.0)'
)

content = content.replace(
    '"DELETE FROM case_studies WHERE id = $$1 AND tenant_id IS NOT DISTINCT FROM $$2"',
    '"DELETE FROM app_content WHERE id = $$1 AND collection_type = \\'case_study\\' AND tenant_id IS NOT DISTINCT FROM $$2"'
)

# Highlights
content = content.replace(
    'SELECT id, title, url, image_url, description, is_visible, display_order FROM highlights WHERE is_visible = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC',
    "SELECT id, title, payload->>'url' as url, payload->>'image_url' as image_url, payload->>'description' as description, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'highlight' AND status = 'published' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
)
content = content.replace(
    'SELECT id, title, url, image_url, description, is_visible, display_order FROM highlights WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC',
    "SELECT id, title, payload->>'url' as url, payload->>'image_url' as image_url, payload->>'description' as description, (status = 'published') as is_visible, display_order FROM app_content WHERE collection_type = 'highlight' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC"
)

content = content.replace(
    '"INSERT INTO highlights (tenant_id, title, url, image_url, description, is_visible, display_order) VALUES ($$7, $$1, $$2, $$3, $$4, $$5, $$6)")\n        .bind(title).bind(url).bind(image_url).bind(description).bind(is_visible).bind(display_order)',
    '"INSERT INTO app_content (tenant_id, collection_type, title, payload, status, display_order) VALUES ($$5, \\'highlight\\', $$1, $$2, $$3, $$4)")\n        .bind(title).bind(serde_json::json!({"url": url, "image_url": image_url, "description": description})).bind(if is_visible { "published" } else { "hidden" }).bind(display_order)\n        .bind(tenant.0)'
)

content = content.replace(
    '"UPDATE highlights SET title = $1, url = $2, image_url = $3, description = $4, is_visible = $5, display_order = $$6 WHERE id = $$7 AND tenant_id IS NOT DISTINCT FROM $$8")\n        .bind(title).bind(url).bind(image_url).bind(description).bind(is_visible).bind(display_order).bind(id)',
    '"UPDATE app_content SET title = $1, payload = $2, status = $3, display_order = $$4 WHERE id = $$5 AND tenant_id IS NOT DISTINCT FROM $$6")\n        .bind(title).bind(serde_json::json!({"url": url, "image_url": image_url, "description": description})).bind(if is_visible { "published" } else { "hidden" }).bind(display_order).bind(id)\n        .bind(tenant.0)'
)

content = content.replace(
    '"DELETE FROM highlights WHERE id = $$1 AND tenant_id IS NOT DISTINCT FROM $$2"',
    '"DELETE FROM app_content WHERE id = $$1 AND collection_type = \\'highlight\\' AND tenant_id IS NOT DISTINCT FROM $$2"'
)

# Update id args
content = re.sub(r'id: i32', 'id: uuid::Uuid', content)

with open("apps/anchor/src/b2b.rs", "w") as f:
    f.write(content)

