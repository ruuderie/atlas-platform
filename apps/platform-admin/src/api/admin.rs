use super::client::{api_url, create_client, with_credentials};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        res.json::<Vec<UserModel>>()
            .await
            .map_err(|e| e.to_string())
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
    let url = api_url(&format!(
        "api/admin/platform/apps/{}/domains/{}",
        instance_id, domain_name
    ));

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
    pub name: String,        // the custom domain itself
    pub value: String,       // platform CNAME target
    pub note: String,        // human-readable instructions
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

pub async fn update_public_config(
    id: Uuid,
    public_slug: Option<String>,
    custom_domain: Option<String>,
) -> Result<PublicConfigResponse, String> {
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
    let url = crate::api::client::api_url(&format!(
        "api/admin/app-instances/{}/operational-config",
        id
    ));
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
    let url = crate::api::client::api_url(&format!(
        "api/admin/app-instances/{}/operational-config",
        id
    ));
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn resume_instance(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/app-instances/{}/resume", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn upsert_module(
    tenant_id: Uuid,
    module_type: &str,
    is_enabled: bool,
    display_name: Option<String>,
    sort_order: Option<i32>,
) -> Result<(), String> {
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// Fetch per-tenant stats (profile count, listing count, ad purchase count).
/// Calls `GET /api/admin/tenant-stats`.
pub async fn get_tenant_stats() -> Result<Vec<crate::api::models::TenantStatModel>, String> {
    crate::api::client::api_get("api/admin/tenant-stats").await
}

/// Fetch all provisioned app instances with their tenant and domain info.
/// Calls `GET /api/admin/platform/apps`.
pub async fn get_all_platform_apps() -> Result<Vec<crate::api::models::PlatformAppSummary>, String>
{
    crate::api::client::api_get("api/admin/platform/apps").await
}

/// Link a client deployment to a CRM Account (or unlink by passing account_id=None).
/// Calls `PUT /api/admin/platform/apps/{tenant_id}/account`.
pub async fn link_deployment_account(
    tenant_id: &str,
    account_id: Option<&str>,
) -> Result<(), String> {
    use serde_json::json;
    let body = json!({ "account_id": account_id });
    crate::api::client::api_put::<_, serde_json::Value>(
        &format!("api/admin/platform/apps/{}/account", tenant_id),
        &body,
    )
    .await
    .map(|_| ())
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
    )
    .await
    .map(|_| ())
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
// SUPPORT INBOX (platform_support threads via atlas_ws_room)
// ============================================================

/// Fetch all platform_support threads (operator inbox).
/// status: "open" | "closed" | "all"  (default: "open")
/// Calls `GET /api/admin/support/threads?status=open`
pub async fn get_support_threads(
    status: &str,
) -> Result<Vec<crate::api::models::SupportThreadSummary>, String> {
    crate::api::client::api_get(&format!("api/admin/support/threads?status={status}")).await
}

/// Fetch a single support thread with full message history.
/// Calls `GET /api/admin/support/threads/{id}`
pub async fn get_support_thread(
    id: String,
) -> Result<crate::api::models::SupportThreadDetail, String> {
    crate::api::client::api_get(&format!("api/admin/support/threads/{id}")).await
}

/// Send an operator reply into a support thread.
/// Calls `POST /api/admin/support/threads/{id}/reply`
pub async fn send_support_reply(id: String, content: String) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Payload {
        content: String,
    }
    crate::api::client::api_post::<_, serde_json::Value>(
        &format!("api/admin/support/threads/{id}/reply"),
        &Payload { content },
    )
    .await
    .map(|_| ())
}

/// Close / resolve a support thread.
/// Calls `PUT /api/admin/support/threads/{id}/close`
pub async fn close_support_thread(id: String) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Empty {}
    crate::api::client::api_put::<_, serde_json::Value>(
        &format!("api/admin/support/threads/{id}/close"),
        &Empty {},
    )
    .await
    .map(|_| ())
}

/// Add an operator-only internal note to a support thread.
/// Calls `POST /api/admin/support/threads/{id}/notes`
pub async fn send_support_note(id: String, content: String) -> Result<(), String> {
    #[derive(serde::Serialize)]
    struct Payload {
        content: String,
    }
    crate::api::client::api_post::<_, serde_json::Value>(
        &format!("api/admin/support/threads/{id}/notes"),
        &Payload { content },
    )
    .await
    .map(|_| ())
}

/// Legacy CRM case stubs — kept for backward compatibility with older pages.
/// Calls `GET /api/admin/cases`
#[allow(dead_code)]
pub async fn get_admin_cases() -> Result<Vec<crate::api::models::CaseModel>, String> {
    crate::api::client::api_get("api/admin/cases").await
}

#[allow(dead_code)]
pub async fn get_admin_case(id: String) -> Result<crate::api::models::CaseModel, String> {
    crate::api::client::api_get(&format!("api/admin/cases/{}", id)).await
}

#[allow(dead_code)]
pub async fn update_case_status(
    id: String,
    status: String,
) -> Result<crate::api::models::CaseModel, String> {
    #[derive(serde::Serialize)]
    struct Payload {
        status: String,
    }
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
pub async fn create_flag(
    input: crate::api::models::CreateFlagInput,
) -> Result<crate::api::models::FeatureFlagModel, String> {
    crate::api::client::api_post("api/admin/flags", &input).await
}

/// Update a feature flag (enabled state, rollout %, etc).
/// Calls `PUT /api/admin/flags/{key}`
pub async fn update_flag(
    key: String,
    input: crate::api::models::UpdateFlagInput,
) -> Result<crate::api::models::FeatureFlagModel, String> {
    crate::api::client::api_put(&format!("api/admin/flags/{}", key), &input).await
}

/// Add or replace a per-tenant NI override for a flag.
/// Calls `POST /api/admin/flags/{key}/overrides`
pub async fn add_flag_override(
    key: String,
    input: crate::api::models::CreateFlagOverrideInput,
) -> Result<crate::api::models::FlagOverrideModel, String> {
    crate::api::client::api_post(&format!("api/admin/flags/{}/overrides", key), &input).await
}

/// Remove a per-tenant NI override from a flag.
/// Calls `DELETE /api/admin/flags/{key}/overrides/{tenant_id}`
pub async fn remove_flag_override(key: String, tenant_id: String) -> Result<(), String> {
    crate::api::client::api_delete(&format!("api/admin/flags/{}/overrides/{}", key, tenant_id))
        .await
}

// ============================================================
// PER-INSTANCE FEATURE FLAGS
// ============================================================

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlagEffect {
    Grant,
    Deny,
}

impl FlagEffect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Deny => "deny",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Grant => "Grant",
            Self::Deny => "Deny",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InstanceFlagRow {
    pub flag_key: String,
    pub description: String,
    pub catalog_enabled: bool,
    pub has_global: bool,
    pub global_rollout_pct: i32,
    pub effect: Option<FlagEffect>,
    pub rollout_pct: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InstanceFlagsResponse {
    pub flags: Vec<InstanceFlagRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct InstanceFlagUpdateItem {
    pub flag_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<FlagEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollout_pct: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateInstanceFlagsInput {
    pub updates: Vec<InstanceFlagUpdateItem>,
}

/// List catalog flags with this instance's enablement (or inherit).
/// Calls `GET /api/admin/app-instances/{id}/feature-flags`
pub async fn get_instance_feature_flags(
    app_instance_id: &str,
) -> Result<InstanceFlagsResponse, String> {
    crate::api::client::api_get(&format!(
        "api/admin/app-instances/{}/feature-flags",
        app_instance_id
    ))
    .await
}

/// Upsert or clear instance feature-flag enablements.
/// Calls `PUT /api/admin/app-instances/{id}/feature-flags`
/// Pass `effect: None` (serialized as null) to clear / inherit.
pub async fn update_instance_feature_flags(
    app_instance_id: &str,
    updates: Vec<InstanceFlagUpdateItem>,
) -> Result<InstanceFlagsResponse, String> {
    // Explicit null for clear: serialize effect as Option so null clears inherit.
    #[derive(Serialize)]
    struct WireItem<'a> {
        flag_key: &'a str,
        effect: Option<&'a FlagEffect>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rollout_pct: Option<i32>,
    }
    #[derive(Serialize)]
    struct WireBody<'a> {
        updates: Vec<WireItem<'a>>,
    }
    let body = WireBody {
        updates: updates
            .iter()
            .map(|u| WireItem {
                flag_key: &u.flag_key,
                effect: u.effect.as_ref(),
                rollout_pct: u.rollout_pct,
            })
            .collect(),
    };
    crate::api::client::api_put(
        &format!("api/admin/app-instances/{}/feature-flags", app_instance_id),
        &body,
    )
    .await
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
    struct Payload {
        name: String,
        license: String,
    }
    crate::api::client::api_post("api/admin/compliance/permits", &Payload { name, license }).await
}

pub async fn verify_permit(id: Uuid) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/compliance/permits/{}/verify", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn get_geo_zones() -> Result<Vec<GeoZoneModel>, String> {
    crate::api::client::api_get("api/admin/compliance/geo-zones").await
}

pub async fn create_geo_zone(
    name: String,
    region: String,
    points: String,
) -> Result<GeoZoneModel, String> {
    #[derive(Serialize)]
    struct Payload {
        name: String,
        region: String,
        points: String,
    }
    crate::api::client::api_post(
        "api/admin/compliance/geo-zones",
        &Payload {
            name,
            region,
            points,
        },
    )
    .await
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn rerun_ai_task(id: String) -> Result<(), String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url(&format!("api/admin/ai-tasks/{}/rerun", id));
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AiQueueStatus {
    pub paused: bool,
}

pub async fn get_ai_queue_status() -> Result<AiQueueStatus, String> {
    crate::api::client::api_get("api/admin/ai-tasks/queue/status").await
}

pub async fn pause_ai_queue() -> Result<AiQueueStatus, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url("api/admin/ai-tasks/queue/pause");
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn resume_ai_queue() -> Result<AiQueueStatus, String> {
    let client = crate::api::client::create_client();
    let url = crate::api::client::api_url("api/admin/ai-tasks/queue/resume");
    let req = client.post(&url);
    let req = crate::api::client::with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InviteModel {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    /// Role within the target app instance — interpreted by that app, not the platform
    pub app_role: Option<String>,
    pub tenant: String,
    pub app_instance_id: Option<Uuid>,
    pub invited_by: String,
    pub sent: String,
    pub expires: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateInviteInput {
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub app_role: Option<String>,
    pub tenant: String,
    pub app_instance_id: Option<Uuid>,
    pub target_app_url: Option<String>,
    pub personal_message: Option<String>,
    pub expires_days: Option<i64>,
}

pub async fn get_invites() -> Result<Vec<InviteModel>, String> {
    crate::api::client::api_get("api/admin/users/invites").await
}

pub async fn create_invite(input: CreateInviteInput) -> Result<InviteModel, String> {
    crate::api::client::api_post("api/admin/users/invite", &input).await
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
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
        &Payload {
            contract_type,
            signee_tenant_id: None,
            start_date,
            end_date,
            vault_file,
        },
    )
    .await
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
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
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
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
    pub global_name: String,
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
        res.json::<Vec<CampaignModel>>()
            .await
            .map_err(|e| e.to_string())
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
pub async fn list_campaign_members(
    campaign_id: uuid::Uuid,
) -> Result<Vec<CampaignEnrollmentModel>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/enrollments", campaign_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<Vec<CampaignEnrollmentModel>>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerLeaderboardRow {
    pub referred_by: String,
    pub signup_count: i64,
    pub latest_signup_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerLeaderboardResponse {
    pub campaign_id: uuid::Uuid,
    pub utm_campaign: Option<String>,
    pub total_attributed: i64,
    pub referrers: Vec<ReferrerLeaderboardRow>,
}

/// GET /api/admin/campaigns/:id/referrers — who referred the most.
pub async fn list_campaign_referrers(
    campaign_id: uuid::Uuid,
) -> Result<ReferrerLeaderboardResponse, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/referrers", campaign_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<ReferrerLeaderboardResponse>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// PUT /api/admin/campaigns/:id/status — transition campaign status.
pub async fn update_campaign_status(
    id: uuid::Uuid,
    status: String,
) -> Result<CampaignModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/status", id));
    let req = with_credentials(client.put(&url)).json(&serde_json::json!({ "status": status }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<CampaignModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// POST /api/admin/campaigns/:id/spend — record manual spend (cents).
pub async fn record_campaign_spend(
    campaign_id: uuid::Uuid,
    cents: i64,
    source: &str,
    external_ref: Option<&str>,
) -> Result<CampaignModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/spend", campaign_id));
    let req = with_credentials(client.post(&url)).json(&serde_json::json!({
        "cents": cents,
        "source": source,
        "external_ref": external_ref,
    }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<CampaignModel>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailDropModel {
    pub id: uuid::Uuid,
    pub campaign_id: uuid::Uuid,
    pub drop_name: String,
    pub creative_variant: Option<String>,
    pub utm_content: Option<String>,
    pub piece_count: i32,
    pub unit_cost_cents: Option<i64>,
    pub status: String,
    pub mailed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferCodeModel {
    pub id: uuid::Uuid,
    pub campaign_id: uuid::Uuid,
    pub mail_drop_id: Option<uuid::Uuid>,
    pub code: String,
    pub is_active: bool,
    pub redemption_count: i32,
}

pub async fn list_mail_drops(campaign_id: uuid::Uuid) -> Result<Vec<MailDropModel>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/mail-drops", campaign_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn create_mail_drop(
    campaign_id: uuid::Uuid,
    drop_name: &str,
    utm_content: Option<&str>,
    piece_count: i32,
    unit_cost_cents: Option<i64>,
) -> Result<MailDropModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/mail-drops", campaign_id));
    let req = with_credentials(client.post(&url)).json(&serde_json::json!({
        "drop_name": drop_name,
        "utm_content": utm_content,
        "piece_count": piece_count,
        "unit_cost_cents": unit_cost_cents,
    }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn list_offer_codes(campaign_id: uuid::Uuid) -> Result<Vec<OfferCodeModel>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/offer-codes", campaign_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub async fn create_offer_code(
    campaign_id: uuid::Uuid,
    code: &str,
    mail_drop_id: Option<uuid::Uuid>,
) -> Result<OfferCodeModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/offer-codes", campaign_id));
    let req = with_credentials(client.post(&url)).json(&serde_json::json!({
        "code": code,
        "mail_drop_id": mail_drop_id,
    }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionTouchpointModel {
    pub id: uuid::Uuid,
    pub channel: String,
    pub contact_email: Option<String>,
    pub anonymous_id: Option<String>,
    pub utm_content: Option<String>,
    pub utm_campaign: Option<String>,
    pub conversion_entity_type: Option<String>,
    pub conversion_value_cents: Option<i64>,
    pub occurred_at: String,
}

pub async fn get_campaign_attribution(
    campaign_id: uuid::Uuid,
) -> Result<Vec<AttributionTouchpointModel>, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/attribution", campaign_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

pub fn campaign_qr_url(campaign_id: uuid::Uuid) -> String {
    api_url(&format!("api/admin/campaigns/{}/qr", campaign_id))
}

/// Enroll a batch of leads into a campaign by lead ID.
pub async fn enroll_leads(
    campaign_id: uuid::Uuid,
    lead_ids: Vec<uuid::Uuid>,
) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/campaigns/{}/enroll-leads", campaign_id));
    let req =
        with_credentials(client.post(&url)).json(&serde_json::json!({ "lead_ids": lead_ids }));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<serde_json::Value>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// Returns the download URL for the campaign member CSV export.
pub fn campaign_export_url(campaign_id: uuid::Uuid) -> String {
    api_url(&format!(
        "api/admin/campaigns/{}/enrollments/export.csv",
        campaign_id
    ))
}

// ── Ambassadors API (G-37) ───────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(default)]
pub struct AmbassadorModel {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub code: String,
    pub display_name: String,
    pub partner_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub campaign_ids: Vec<uuid::Uuid>,
    pub fulfillment_requests: serde_json::Value,
    pub landlord_url: String,
    pub vendor_url: String,
    #[serde(default)]
    pub refer_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateAmbassadorInput {
    pub code: String,
    pub display_name: String,
    pub partner_type: String,
    pub notes: Option<String>,
    pub campaign_ids: Vec<uuid::Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateFulfillmentInput {
    pub kind: String,
    pub landlord_qty: i32,
    pub vendor_qty: i32,
}

/// GET /api/admin/ambassadors
pub async fn list_ambassadors() -> Result<Vec<AmbassadorModel>, String> {
    let client = create_client();
    let url = api_url("api/admin/ambassadors");
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<Vec<AmbassadorModel>>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// POST /api/admin/ambassadors
pub async fn create_ambassador(input: CreateAmbassadorInput) -> Result<AmbassadorModel, String> {
    let client = create_client();
    let url = api_url("api/admin/ambassadors");
    let req = with_credentials(client.post(&url)).json(&input);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<AmbassadorModel>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// POST /api/admin/ambassadors/:id/fulfillments
pub async fn create_ambassador_fulfillment(
    id: uuid::Uuid,
    input: CreateFulfillmentInput,
) -> Result<AmbassadorModel, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/ambassadors/{}/fulfillments", id));
    let req = with_credentials(client.post(&url)).json(&input);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<AmbassadorModel>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// Download QR PNG for an ambassador by audience (`landlord` | `vendor`).
/// POST /api/admin/ambassadors/:id/send
pub async fn send_ambassador_invite(
    id: uuid::Uuid,
    channel: &str,
    to: &str,
    app_slug: &str,
) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/ambassadors/{}/send", id));
    let payload = serde_json::json!({
        "channel": channel,
        "to": to,
        "app_slug": app_slug,
    });
    let req = with_credentials(client.post(&url)).json(&payload);
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}

/// GET /api/admin/ambassadors/:id/qr?audience= — download PNG (unified /refer URL).
pub async fn download_ambassador_qr(id: uuid::Uuid, audience: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!(
        "api/admin/ambassadors/{}/qr?audience={}",
        id, audience
    ));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(res.text().await.unwrap_or_default());
    }
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let uint8 = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        uint8.copy_from(&bytes);
        let parts = js_sys::Array::new();
        parts.push(&uint8);
        let mut opts = web_sys::BlobPropertyBag::new();
        opts.type_("image/png");
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts)
            .map_err(|_| "blob create failed".to_string())?;
        let object_url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "object url failed".to_string())?;
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| "no document".to_string())?;
        let anchor = document
            .create_element("a")
            .map_err(|_| "create a failed".to_string())?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "cast a failed".to_string())?;
        anchor.set_href(&object_url);
        anchor.set_download(&format!("ambassador-qr-{audience}.png"));
        anchor.click();
        let _ = web_sys::Url::revoke_object_url(&object_url);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (bytes, audience);
    }
    Ok(())
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
