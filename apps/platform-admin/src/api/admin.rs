use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::client::{api_url, create_client, with_credentials};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub is_admin: bool,
}

pub async fn get_users(network_id: Option<Uuid>) -> Result<Vec<UserModel>, String> {
    let client = create_client();
    // NOTE: backend list_users handler reads `?tenant_id=`, not `?network_id=`.
    // The param was previously mismatched, silently dropping the tenant filter.
    let url = if let Some(net_id) = network_id {
        format!("{}?tenant_id={}", api_url("api/admin/users"), net_id)
    } else {
        api_url("api/admin/users")
    };
    
    let req = client.get(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<Vec<UserModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn toggle_admin(id: Uuid) -> Result<UserModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/users/{}/toggle-admin", id));
    
    let req = client.post(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<UserModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn get_app_domains(instance_id: String) -> Result<Vec<String>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains", instance_id));
    
    let req = client.get(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        res.json::<Vec<String>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn add_app_domain(instance_id: String, domain_name: String) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains", instance_id));
    
    let payload = serde_json::json!({
        "domain_name": domain_name
    });
    
    let req = client.post(&url).json(&payload);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn remove_app_domain(instance_id: String, domain_name: String) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/platform/apps/{}/domains/{}", instance_id, domain_name));
    
    let req = client.delete(&url);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
        
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// DNS record the tenant must set in their registrar when using a custom domain.
/// Returned by `GET /api/admin/app-instances/{id}/public-config` when `custom_domain` is set.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct DnsInstructions {
    pub record_type: String, // "CNAME"
    pub name:        String, // the custom domain itself
    pub value:       String, // platform CNAME target
    pub note:        String, // human-readable instructions
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PublicConfigResponse {
    pub instance_id: Uuid,
    pub tenant_id: Uuid,
    /// Human-readable tenant name (e.g. "buildwithruud").
    /// Populated by the backend from the tenant table — never a raw UUID.
    #[serde(default)]
    pub tenant_name: String,
    pub app_slug: String,
    pub public_slug: Option<String>,
    pub custom_domain: Option<String>,
    pub instance_status: String,
    pub folio_mode: String,
    pub billing_tier: String,
    pub tenant_portal_enabled: bool,
    pub vendor_portal_enabled: bool,
    /// Present when the instance has a custom_domain configured.
    /// Contains the CNAME record the tenant must set in their DNS registrar.
    pub dns_instructions: Option<DnsInstructions>,
}

/// Live per-instance activity counts — returned by
/// `GET /api/admin/app-instances/{id}/stats`.
/// All counts are scoped to the instance's tenant_id and sourced from real DB queries.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct InstanceStatsResponse {
    pub instance_id: Uuid,
    pub tenant_id: Uuid,
    pub app_slug: String,
    /// atlas_assets count (Folio: properties/units)
    pub asset_count: u64,
    /// atlas_contracts with status = 'active' (active leases)
    pub active_contract_count: u64,
    /// atlas_lead total
    pub lead_count: u64,
    /// atlas_cases with status != 'closed'
    pub open_case_count: u64,
    /// atlas_service_providers (Folio: vendors)
    pub vendor_count: u64,
    /// listing count with status = 'approved' (NI: active listings)
    pub active_listing_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdminModuleConfig {
    pub module_type: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_fixed: bool,
    pub category: String,
}

pub async fn get_public_config(id: Uuid) -> Result<PublicConfigResponse, String> {
    crate::api::client::api_get(&format!("api/admin/app-instances/{}/public-config", id)).await
}

/// Fetch live per-instance activity stats.
/// Calls `GET /api/admin/app-instances/{id}/stats`.
pub async fn get_instance_stats(id: Uuid) -> Result<InstanceStatsResponse, String> {
    crate::api::client::api_get(&format!("api/admin/app-instances/{}/stats", id)).await
}

pub async fn update_public_config(id: Uuid, public_slug: Option<String>, custom_domain: Option<String>) -> Result<PublicConfigResponse, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/public-config", id));
    let payload = serde_json::json!({
        "public_slug": public_slug,
        "custom_domain": custom_domain
    });
    let req = client.put(&url).json(&payload);
    crate::api::client::api_request(req).await
}

/// PATCH /api/admin/app-instances/{id}/operational-config
/// Updates folio_mode, billing_tier, portal flags, and/or branding.
pub async fn update_operational_config(
    id: Uuid,
    folio_mode: Option<String>,
    billing_tier: Option<String>,
    tenant_portal_enabled: Option<bool>,
    vendor_portal_enabled: Option<bool>,
) -> Result<PublicConfigResponse, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/operational-config", id));
    let payload = serde_json::json!({
        "folio_mode": folio_mode,
        "billing_tier": billing_tier,
        "tenant_portal_enabled": tenant_portal_enabled,
        "vendor_portal_enabled": vendor_portal_enabled,
    });
    let req = client.patch(&url).json(&payload);
    crate::api::client::api_request(req).await
}

/// PATCH /api/admin/app-instances/{id}/operational-config
/// Saves branding settings only (theme, primary color, font).
/// Stored in config["branding"] JSONB on the backend.
pub async fn update_branding_config(
    id: Uuid,
    branding_theme: Option<String>,
    branding_color: Option<String>,
    branding_font: Option<String>,
) -> Result<PublicConfigResponse, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/operational-config", id));
    let payload = serde_json::json!({
        "branding_theme": branding_theme,
        "branding_color": branding_color,
        "branding_font": branding_font,
    });
    let req = client.patch(&url).json(&payload);
    crate::api::client::api_request(req).await
}

pub async fn suspend_instance(id: Uuid, reason: String) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/suspend", id));
    let payload = serde_json::json!({ "reason": reason });
    let req = client.post(&url).json(&payload);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn resume_instance(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/resume", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn upsert_module(tenant_id: Uuid, module_type: &str, is_enabled: bool, display_name: Option<String>, sort_order: Option<i32>) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/platform/tenants/{}/modules", tenant_id));
    let payload = serde_json::json!({
        "module_type": module_type,
        "is_enabled": is_enabled,
        "display_name": display_name,
        "sort_order": sort_order
    });
    let req = client.post(&url).json(&payload);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn get_admin_modules() -> Result<Vec<AdminModuleConfig>, String> {
    crate::api::client::api_get("api/admin/modules").await
}

pub async fn impersonate_user(user_id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/users/{}/impersonate", user_id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

/// Fetch per-tenant stats (profile count, listing count, ad purchase count).
/// Calls `GET /api/admin/tenant-stats`.
pub async fn get_tenant_stats() -> Result<Vec<crate::api::models::TenantStatModel>, String> {
    crate::api::client::api_get("api/admin/tenant-stats").await
}

/// Fetch all provisioned app instances with their tenant and domain info.
/// Calls `GET /api/admin/platform/apps`.
pub async fn get_all_platform_apps() -> Result<Vec<crate::api::models::PlatformAppSummary>, String> {
    crate::api::client::api_get("api/admin/platform/apps").await
}

/// Link a client deployment to a CRM Account (or unlink by passing account_id=None).
/// Calls `PUT /api/admin/platform/apps/{tenant_id}/account`.
pub async fn link_deployment_account(tenant_id: &str, account_id: Option<&str>) -> Result<(), String> {
    use serde_json::json;
    let body = json!({ "account_id": account_id });
    crate::api::client::api_put::<_, serde_json::Value>(
        &format!("api/admin/platform/apps/{}/account", tenant_id),
        &body,
    ).await.map(|_| ())
}

/// Set the operational purpose label for an internal deployment.
/// Calls `PUT /api/admin/platform/apps/{tenant_id}/purpose`.
/// Valid values: "demo" | "test" | "staging" | "managed_service" | null (to clear).
pub async fn set_deployment_purpose(tenant_id: &str, purpose: Option<&str>) -> Result<(), String> {
    use serde_json::json;
    let body = json!({ "purpose": purpose });
    crate::api::client::api_put::<_, serde_json::Value>(
        &format!("api/admin/platform/apps/{}/purpose", tenant_id),
        &body,
    ).await.map(|_| ())
}

/// Fetch CRM accounts for the account search picker.
/// Calls `GET /api/admin/accounts`.
pub async fn get_crm_accounts() -> Result<Vec<crate::api::models::AccountSummary>, String> {
    crate::api::client::api_get("api/admin/accounts").await
}

/// Fetch all billing plans from the platform.
/// Calls `GET /api/admin/billing/plans`.
pub async fn get_billing_plans() -> Result<Vec<crate::api::models::BillingPlanModel>, String> {
    crate::api::client::api_get("api/admin/billing/plans").await
}

/// Fetch all platform transactions from the ledger.
/// Calls `GET /api/admin/billing/transactions`.
pub async fn get_all_transactions() -> Result<Vec<crate::api::models::TransactionModel>, String> {
    crate::api::client::api_get("api/admin/billing/transactions").await
}

// ============================================================
// SUPPORT CASES
// ============================================================

/// Fetch all support cases.
/// Calls `GET /api/admin/cases`
pub async fn get_admin_cases() -> Result<Vec<crate::api::models::CaseModel>, String> {
    crate::api::client::api_get("api/admin/cases").await
}

/// Fetch a single support case with notes and activities.
/// Calls `GET /api/admin/cases/{id}`
pub async fn get_admin_case(id: String) -> Result<crate::api::models::CaseModel, String> {
    crate::api::client::api_get(&format!("api/admin/cases/{}", id)).await
}

/// Update a case status (e.g. "Resolved", "In Progress", "Escalated").
/// Calls `PUT /api/admin/cases/{id}`
pub async fn update_case_status(id: String, status: String) -> Result<crate::api::models::CaseModel, String> {
    #[derive(serde::Serialize)]
    struct Payload { status: String }
    crate::api::client::api_put(&format!("api/admin/cases/{}", id), &Payload { status }).await
}

// ============================================================
// FEATURE FLAGS
// ============================================================

/// Fetch all feature flags with overrides and audit logs.
/// Calls `GET /api/admin/flags`
pub async fn get_admin_flags() -> Result<Vec<crate::api::models::FeatureFlagModel>, String> {
    crate::api::client::api_get("api/admin/flags").await
}

/// Create a new feature flag.
/// Calls `POST /api/admin/flags`
pub async fn create_flag(input: crate::api::models::CreateFlagInput) -> Result<crate::api::models::FeatureFlagModel, String> {
    crate::api::client::api_post("api/admin/flags", &input).await
}

/// Update a feature flag (enabled state, rollout %, etc).
/// Calls `PUT /api/admin/flags/{key}`
pub async fn update_flag(key: String, input: crate::api::models::UpdateFlagInput) -> Result<crate::api::models::FeatureFlagModel, String> {
    crate::api::client::api_put(&format!("api/admin/flags/{}", key), &input).await
}

/// Add or replace a per-tenant NI override for a flag.
/// Calls `POST /api/admin/flags/{key}/overrides`
pub async fn add_flag_override(key: String, input: crate::api::models::CreateFlagOverrideInput) -> Result<crate::api::models::FlagOverrideModel, String> {
    crate::api::client::api_post(&format!("api/admin/flags/{}/overrides", key), &input).await
}

/// Remove a per-tenant NI override from a flag.
/// Calls `DELETE /api/admin/flags/{key}/overrides/{tenant_id}`
pub async fn remove_flag_override(key: String, tenant_id: String) -> Result<(), String> {
    crate::api::client::api_delete(&format!("api/admin/flags/{}/overrides/{}", key, tenant_id)).await
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PermitModel {
    pub id: Uuid,
    pub name: String,
    pub holder: String,
    pub license: String,
    pub permit_type: String,
    pub status: String,
    pub status_class: String,
    pub last_checked: String,
    pub date_renewed: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GeoZoneModel {
    pub key: String,
    pub name: String,
    pub region: String,
    pub listings: String,
    pub status: String,
    pub status_class: String,
    pub coverage: String,
    pub points: String,
}

pub async fn get_permits() -> Result<Vec<PermitModel>, String> {
    crate::api::client::api_get("api/admin/compliance/permits").await
}

pub async fn create_permit(name: String, license: String) -> Result<PermitModel, String> {
    #[derive(Serialize)]
    struct Payload { name: String, license: String }
    crate::api::client::api_post("api/admin/compliance/permits", &Payload { name, license }).await
}

pub async fn verify_permit(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/compliance/permits/{}/verify", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn get_geo_zones() -> Result<Vec<GeoZoneModel>, String> {
    crate::api::client::api_get("api/admin/compliance/geo-zones").await
}

pub async fn create_geo_zone(name: String, region: String, points: String) -> Result<GeoZoneModel, String> {
    #[derive(Serialize)]
    struct Payload { name: String, region: String, points: String }
    crate::api::client::api_post("api/admin/compliance/geo-zones", &Payload { name, region, points }).await
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AdminAiTaskModel {
    pub id: String,
    pub task_type: String,
    pub entity: String,
    pub status: String,
    pub status_class: String,
    pub runtime: String,
    pub tokens: String,
    pub completed: String,
    pub model: String,
    pub params: serde_json::Value,
    pub initial_logs: Vec<String>,
    pub streamable: bool,
}

pub async fn get_ai_tasks() -> Result<Vec<AdminAiTaskModel>, String> {
    crate::api::client::api_get("api/admin/ai-tasks").await
}

pub async fn abort_ai_task(id: String) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/ai-tasks/{}/abort", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

pub async fn rerun_ai_task(id: String) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/ai-tasks/{}/rerun", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InviteModel {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub tenant: String,
    pub invited_by: String,
    pub sent: String,
    pub expires: String,
}

pub async fn get_invites() -> Result<Vec<InviteModel>, String> {
    crate::api::client::api_get("api/admin/users/invites").await
}

pub async fn create_invite(email: String, role: String, tenant: String) -> Result<InviteModel, String> {
    #[derive(Serialize)]
    struct Payload { email: String, role: String, tenant: String }
    crate::api::client::api_post("api/admin/users/invite", &Payload { email, role, tenant }).await
}

pub async fn revoke_invite(id: Uuid) -> Result<(), String> {
    crate::api::client::api_delete(&format!("api/admin/users/invites/{}", id)).await
}

pub async fn resend_invite(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/users/invites/{}/resend", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

// ============================================================
// CONTRACTS (G-11)
// ============================================================

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ContractModel {
    pub id: String,
    pub name: String,
    pub signee: String,
    pub contract_type: String,
    pub status: String,
    pub status_class: String,
    pub date_executed: String,
    pub expiry_date: String,
    pub vault_file: Option<String>,
}

/// Fetch all contracts from atlas_contracts.
/// Calls `GET /api/admin/compliance/contracts`.
pub async fn get_contracts() -> Result<Vec<ContractModel>, String> {
    crate::api::client::api_get("api/admin/compliance/contracts").await
}

/// Create a new contract.
/// Calls `POST /api/admin/compliance/contracts`.
pub async fn create_contract(
    contract_type: String,
    start_date: String,
    end_date: Option<String>,
    vault_file: Option<String>,
) -> Result<ContractModel, String> {
    #[derive(Serialize)]
    struct Payload {
        contract_type: String,
        signee_tenant_id: Option<Uuid>,
        start_date: String,
        end_date: Option<String>,
        vault_file: Option<String>,
    }
    crate::api::client::api_post(
        "api/admin/compliance/contracts",
        &Payload { contract_type, signee_tenant_id: None, start_date, end_date, vault_file },
    ).await
}

// ============================================================
// PASSKEYS ADMIN
// ============================================================

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PasskeyAdminModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub name: String,
    pub sign_count: i32,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

/// List all registered passkeys (super-admin).
/// Calls `GET /api/admin/passkeys`.
pub async fn get_all_passkeys(user_id: Option<Uuid>) -> Result<Vec<PasskeyAdminModel>, String> {
    let url = if let Some(uid) = user_id {
        format!("api/admin/passkeys?user_id={}", uid)
    } else {
        "api/admin/passkeys".to_string()
    };
    crate::api::client::api_get(&url).await
}

/// Revoke a passkey by ID (super-admin).
/// Calls `DELETE /api/admin/passkeys/{id}`.
pub async fn revoke_passkey_admin(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/passkeys/{}", id));
    let req = crate::api::client::with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

// ============================================================
// A/B TEST — END TEST
// ============================================================

/// End an A/B test (set status -> "Ended").
/// Calls `POST /api/admin/ab-tests/{id}/end`.
pub async fn end_ab_test(test_id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/ab-tests/{}/end", test_id));
    let req = crate::api::client::with_credentials(client.post(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() { Ok(()) } else { Err(res.text().await.unwrap_or_default()) }
}

// ============================================================
// AUDIT LOGS
// ============================================================

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AuditLogModel {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub created_at: String,
}

/// Fetch platform audit logs (super-admin sees all).
/// Calls `GET /api/admin/audit-logs`.
pub async fn get_audit_logs() -> Result<Vec<AuditLogModel>, String> {
    crate::api::client::api_get("api/admin/audit-logs").await
}

// ── Campaign API (G-19) ──────────────────────────────────────────────────────

/// Campaign summary model returned by the backend admin campaigns handler.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(default)]
pub struct CampaignModel {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: String,
    pub campaign_type: String,
    pub status: String,
    pub goal_type: Option<String>,
    pub budget_cents: Option<i64>,
    pub spent_cents: i64,
    pub total_contacts: i32,
    pub total_opens: i32,
    pub total_clicks: i32,
    pub total_replies: i32,
    pub total_conversions: i32,
    pub attribution_window_days: i32,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Campaign enrollment model for the members list.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(default)]
pub struct CampaignEnrollmentModel {
    pub id: uuid::Uuid,
    pub campaign_id: uuid::Uuid,
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub status: String,
    pub current_step: i32,
    pub exit_reason: Option<String>,
    pub converted_at: Option<String>,
    pub enrolled_at: String,
    pub contact_metadata: Option<serde_json::Value>,
}

/// Payload for creating a new campaign.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateCampaignInput {
    pub name: String,
    pub campaign_type: String,
    pub tenant_id: uuid::Uuid,
    pub goal_type: Option<String>,
    pub budget_cents: Option<i64>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

/// GET /api/admin/campaigns — list all campaigns.
pub async fn list_campaigns() -> Result<Vec<CampaignModel>, String> {
    let client = create_client();
    let url = api_url("api/admin/campaigns");
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<Vec<CampaignModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// GET /api/admin/campaigns/:id — single campaign detail.
pub async fn get_campaign(id: uuid::Uuid) -> Result<CampaignModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<CampaignModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// POST /api/admin/campaigns — create a new campaign.
pub async fn create_campaign(input: CreateCampaignInput) -> Result<CampaignModel, String> {
    let client = create_client();
    let url = api_url("api/admin/campaigns");
    let req = with_credentials(client.post(&url)).json(&input);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<CampaignModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// GET /api/admin/campaigns/:id/enrollments — list members enrolled in a campaign.
pub async fn list_campaign_members(campaign_id: uuid::Uuid) -> Result<Vec<CampaignEnrollmentModel>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/enrollments", campaign_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<Vec<CampaignEnrollmentModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// PUT /api/admin/campaigns/:id/status — transition campaign status.
pub async fn update_campaign_status(id: uuid::Uuid, status: String) -> Result<CampaignModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/status", id));
    let req = with_credentials(client.put(&url))
        .json(&serde_json::json!({ "status": status }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<CampaignModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// Enroll a batch of leads into a campaign by lead ID.
pub async fn enroll_leads(campaign_id: uuid::Uuid, lead_ids: Vec<uuid::Uuid>) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/enroll-leads", campaign_id));
    let req = with_credentials(client.post(&url)).json(&serde_json::json!({ "lead_ids": lead_ids }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<serde_json::Value>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// Returns the download URL for the campaign member CSV export.
pub fn campaign_export_url(campaign_id: uuid::Uuid) -> String {
    api_url(&format!("api/admin/campaigns/{}/enrollments/export.csv", campaign_id))
}


// ============================================================
// SESSION MANAGEMENT
// ============================================================

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SessionSummaryModel {
    pub id: uuid::Uuid,
    pub created_at: String,
    pub last_accessed_at: String,
    pub is_active: bool,
    pub is_current: bool,
}

/// List all active sessions for the current user.
/// Calls `GET /api/me/sessions`.
pub async fn list_my_sessions() -> Result<Vec<SessionSummaryModel>, String> {
    crate::api::client::api_get("api/me/sessions").await
}

/// Revoke a specific session by ID.
/// Calls `DELETE /api/me/sessions/{id}`.
pub async fn revoke_session_by_id(session_id: uuid::Uuid) -> Result<(), String> {
    crate::api::client::api_delete(&format!("api/me/sessions/{}", session_id)).await
}

/// Revoke all sessions except the current one.
/// Calls `DELETE /api/me/sessions`.
pub async fn revoke_all_other_sessions() -> Result<serde_json::Value, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url("api/me/sessions");
    let req = crate::api::client::with_credentials(client.delete(&url));
    crate::api::client::api_request(req).await
}
