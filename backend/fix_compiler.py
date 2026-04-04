import os
import re

directories = ['src']

# Deprecated tenant fields
tenant_deprecated = [
    'domain', 'subdomain', 'custom_domain', 'enabled_modules', 
    'theme', 'custom_settings', 'directory_type_id', 'site_status'
]

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    original = content
    # Remove tenant_type usages (we dropped directory_type_id)
    content = re.sub(r'use crate::models::tenant_type::.*?;', '', content)
    content = re.sub(r'use crate::entities::directory_type.*?;', '', content)

    # 1. Strip deprecated fields from active models
    for field in tenant_deprecated:
        # Match lines like `domain: Set(...)`, or `theme: NotSet`,
        content = re.sub(r'^\s*'+field+r'\s*:(?!\s*(String|i32|Option|u32|HashMap|Value|Vec|bool|DateTime)).*,\n', '', content, flags=re.MULTILINE)
        content = re.sub(r'^\s*'+field+r'\s*:(?!\s*(String|i32|Option|u32|HashMap|Value|Vec|bool|DateTime)).*\n', '', content, flags=re.MULTILINE)

    # 2. Fix the SiteConfig parsing bug
    content = re.sub(r'use crate::config::site_config::ModuleFlags;', '', content)
    # 3. Add service_area_zips to profile::ActiveModel if missing
    # Find all profile::ActiveModel { ... }
    blocks = re.findall(r'profile::ActiveModel\s*\{[^}]*\}', content)
    for block in blocks:
        if 'service_area_zips' not in block:
            new_block = block.replace('profile::ActiveModel {', 'profile::ActiveModel {\n            service_area_zips: sea_orm::NotSet,')
            content = content.replace(block, new_block)
            
    # Also fix UserAccount (if any) or Tenant
    blocks_tenant = re.findall(r'tenant::ActiveModel\s*\{[^}]*\}', content)
    for block in blocks_tenant:
        for field in tenant_deprecated:
            new_block = re.sub(r'\s*'+field+r'\s*:.*?,', '', block, flags=re.DOTALL)
            content = content.replace(block, new_block)

    # Convert generic instances where we passed `state_db` to `tenant` handler but now it expects something else
    # if content != original:
    with open(filepath, 'w') as f:
        f.write(content)

for root, _, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            fix_file(os.path.join(root, file))
