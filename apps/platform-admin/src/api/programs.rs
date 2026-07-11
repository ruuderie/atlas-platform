//! Platform-admin API client for G-36 growth programs.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::client::{api_get_key, api_request_key, api_url, create_client};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramKind {
    NetworkInvite,
    Referral,
    ReviewRequest,
    WaitlistAccess,
    LeadCapture,
    PartnerShare,
}

impl ProgramKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::NetworkInvite => "Network invite",
            Self::Referral => "Referral",
            Self::ReviewRequest => "Review request",
            Self::WaitlistAccess => "Waitlist access",
            Self::LeadCapture => "Lead capture",
            Self::PartnerShare => "Partner share",
        }
    }
}

impl std::fmt::Display for ProgramKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NetworkInvite => "network_invite",
            Self::Referral => "referral",
            Self::ReviewRequest => "review_request",
            Self::WaitlistAccess => "waitlist_access",
            Self::LeadCapture => "lead_capture",
            Self::PartnerShare => "partner_share",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramOutcomeType {
    Signup,
    WizardComplete,
    FormSubmit,
    ReviewSubmitted,
    FirstJobLogged,
    SubscriptionActivated,
}

impl ProgramOutcomeType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Signup => "Signup",
            Self::WizardComplete => "Wizard complete",
            Self::FormSubmit => "Form submit",
            Self::ReviewSubmitted => "Review submitted",
            Self::FirstJobLogged => "First job logged",
            Self::SubscriptionActivated => "Subscription activated",
        }
    }
}

impl std::fmt::Display for ProgramOutcomeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Signup => "signup",
            Self::WizardComplete => "wizard_complete",
            Self::FormSubmit => "form_submit",
            Self::ReviewSubmitted => "review_submitted",
            Self::FirstJobLogged => "first_job_logged",
            Self::SubscriptionActivated => "subscription_activated",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRewardBeneficiary {
    Actor,
    Target,
}

impl ProgramRewardBeneficiary {
    pub fn label(self) -> &'static str {
        match self {
            Self::Actor => "Actor",
            Self::Target => "Target",
        }
    }
}

impl std::fmt::Display for ProgramRewardBeneficiary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Actor => "actor",
            Self::Target => "target",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRewardType {
    SubscriptionCreditDays,
    FeatureUnlock,
    None,
}

impl ProgramRewardType {
    pub fn label(self) -> &'static str {
        match self {
            Self::SubscriptionCreditDays => "Subscription credit days",
            Self::FeatureUnlock => "Feature unlock",
            Self::None => "None",
        }
    }
}

impl std::fmt::Display for ProgramRewardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SubscriptionCreditDays => "subscription_credit_days",
            Self::FeatureUnlock => "feature_unlock",
            Self::None => "none",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramActionStatus {
    Created,
    Sent,
    Opened,
    Accepted,
    OutcomeComplete,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramOutcomeStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRewardGrantStatus {
    Pending,
    Granted,
    Applied,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Program {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub program_kind: ProgramKind,
    pub campaign_id: Option<Uuid>,
    pub actor_roles: Value,
    pub target_roles: Value,
    pub config: Value,
    pub default_outcome_type: ProgramOutcomeType,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Program {
    pub fn actor_roles_display(&self) -> String {
        value_string_list(&self.actor_roles).join(", ")
    }

    pub fn target_roles_display(&self) -> String {
        value_string_list(&self.target_roles).join(", ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstanceProgram {
    #[serde(flatten)]
    pub program: Program,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProgramInput {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub program_kind: ProgramKind,
    pub campaign_id: Option<Uuid>,
    pub actor_roles: Option<Value>,
    pub target_roles: Option<Value>,
    pub config: Option<Value>,
    pub default_outcome_type: Option<ProgramOutcomeType>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgramUpdatePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_id: Option<Option<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_roles: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_roles: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRuleInput {
    pub beneficiary: ProgramRewardBeneficiary,
    pub reward_type: ProgramRewardType,
    pub amount: String,
    pub trigger_outcome_type: ProgramOutcomeType,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardRule {
    pub id: Uuid,
    pub program_id: Uuid,
    pub beneficiary: ProgramRewardBeneficiary,
    pub reward_type: ProgramRewardType,
    pub amount: Value,
    pub trigger_outcome_type: ProgramOutcomeType,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramAction {
    pub id: Uuid,
    pub program_id: Uuid,
    pub program_slug: Option<String>,
    pub actor_user_id: Uuid,
    pub target_email: Option<String>,
    pub target_role: Option<String>,
    pub delivery_entity_type: Option<String>,
    pub delivery_entity_id: Option<Uuid>,
    pub status: ProgramActionStatus,
    pub invite_code: Option<String>,
    pub outcome_type: Option<ProgramOutcomeType>,
    pub outcome_status: Option<ProgramOutcomeStatus>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramGrant {
    pub id: Uuid,
    pub program_action_id: Uuid,
    pub rule_id: Uuid,
    pub beneficiary_user_id: Uuid,
    pub status: ProgramRewardGrantStatus,
    pub reward_type: Option<ProgramRewardType>,
    pub amount: Option<Value>,
    pub granted_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramAnalytics {
    pub total_actions: i64,
    pub total_grants: i64,
    pub actions_by_status: Vec<StatusCount>,
    pub outcomes_by_status: Vec<StatusCount>,
    pub grants_by_status: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramInstanceEnablement {
    pub id: Uuid,
    pub program_id: Uuid,
    pub app_instance_id: Uuid,
    pub is_enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetProgramEnablement {
    pub app_instance_id: Uuid,
    pub is_enabled: bool,
}

pub async fn list_programs(include_inactive: bool) -> Result<Vec<Program>, String> {
    api_get_key(
        &format!("/api/admin/programs?include_inactive={include_inactive}"),
        "programs",
    )
    .await
}

pub async fn create_program(input: CreateProgramInput) -> Result<Program, String> {
    let client = create_client();
    let req = client.post(&api_url("/api/admin/programs")).json(&input);
    api_request_key(req, "program").await
}

pub async fn get_program(id: Uuid) -> Result<Program, String> {
    api_get_key(&format!("/api/admin/programs/{id}"), "program").await
}

pub async fn update_program(id: Uuid, patch: ProgramUpdatePatch) -> Result<Program, String> {
    let client = create_client();
    let req = client
        .patch(&api_url(&format!("/api/admin/programs/{id}")))
        .json(&patch);
    api_request_key(req, "program").await
}

pub async fn list_reward_rules(program_id: Uuid) -> Result<Vec<RewardRule>, String> {
    api_get_key(
        &format!("/api/admin/programs/{program_id}/reward-rules"),
        "reward_rules",
    )
    .await
}

pub async fn replace_reward_rules(
    program_id: Uuid,
    rules: Vec<RewardRuleInput>,
) -> Result<Vec<RewardRule>, String> {
    let client = create_client();
    let req = client
        .put(&api_url(&format!(
            "/api/admin/programs/{program_id}/reward-rules"
        )))
        .json(&rules);
    api_request_key(req, "reward_rules").await
}

pub async fn list_program_actions(program_id: Uuid) -> Result<Vec<ProgramAction>, String> {
    api_get_key(
        &format!("/api/admin/programs/{program_id}/actions"),
        "actions",
    )
    .await
}

pub async fn list_program_grants(program_id: Uuid) -> Result<Vec<ProgramGrant>, String> {
    api_get_key(
        &format!("/api/admin/programs/{program_id}/grants"),
        "grants",
    )
    .await
}

pub async fn get_program_analytics(program_id: Uuid) -> Result<ProgramAnalytics, String> {
    api_get_key(
        &format!("/api/admin/programs/{program_id}/analytics"),
        "analytics",
    )
    .await
}

pub async fn list_instance_enablements(
    program_id: Uuid,
) -> Result<Vec<ProgramInstanceEnablement>, String> {
    api_get_key(
        &format!("/api/admin/programs/{program_id}/instance-enablements"),
        "instance_enablements",
    )
    .await
}

pub async fn set_instance_enablements(
    program_id: Uuid,
    items: Vec<SetProgramEnablement>,
) -> Result<Vec<ProgramInstanceEnablement>, String> {
    let client = create_client();
    let req = client
        .put(&api_url(&format!(
            "/api/admin/programs/{program_id}/instance-enablements"
        )))
        .json(&items);
    api_request_key(req, "instance_enablements").await
}

pub async fn list_programs_for_instance(
    app_instance_id: Uuid,
) -> Result<Vec<InstanceProgram>, String> {
    api_get_key(
        &format!("/api/admin/app-instances/{app_instance_id}/programs"),
        "programs",
    )
    .await
}

pub async fn set_program_enabled_for_instance(
    program_id: Uuid,
    app_instance_id: Uuid,
    is_enabled: bool,
) -> Result<Vec<ProgramInstanceEnablement>, String> {
    set_instance_enablements(
        program_id,
        vec![SetProgramEnablement {
            app_instance_id,
            is_enabled,
        }],
    )
    .await
}

pub fn value_string_list(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn amount_display(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| value.as_i64().map(|n| n.to_string()))
        .or_else(|| value.as_f64().map(|n| n.to_string()))
        .unwrap_or_else(|| "0".to_string())
}
