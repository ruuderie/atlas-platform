import re

with open('src/handlers/admin.rs', 'r') as f:
    content = f.read()

# Replace handler calls
content = content.replace('tenant::get_directories(', 'tenant::list_tenants(')
content = content.replace('tenant::get_directory(', 'tenant::get_tenant_by_id(')
content = content.replace('tenant::update_directory(', 'tenant::update_tenant(')
content = content.replace('tenant::delete_directory(', 'tenant::delete_tenant(')
content = content.replace('tenant::create_directory(', 'tenant::create_tenant(')
content = content.replace('tenant::get_directory_by_id(', 'tenant::get_tenant_by_id(')

content = content.replace('use crate::models::tenant_type::{TenantTypeModel, CreateTenantType, UpdateTenantType};', '')
content = content.replace('use crate::entities::{tenant', 'use crate::entities::{tenant, user, user_account, listing, ad_purchase, profile, account, session, request_log, template, category}')
content = content.replace('Ok(directories)', 'Ok((StatusCode::OK, directories.1))') # fix return types if any? Actually tenant::list_tenants returns `Result<(StatusCode, Json<Vec<TenantModel>>), StatusCode>`

# Just to be safe, I'll delete the entire `admin.rs` since it's just proxying things from `tenant.rs`.
# Wait, I can't delete it, because `admin_routes` is exported! 
