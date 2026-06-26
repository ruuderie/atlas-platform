use crate::api::client::{api_get, api_url, create_client, api_request};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub provider: String,
    pub amount: i64,
    pub currency: String,
    pub provider_tx_id: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
}

/// Matches `backend/src/entities/billing_plan::Model`
/// Returned by `GET /api/admin/billing/plans`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingPlanModel {
    pub id: Uuid,
    pub name: String,
    /// Price in cents (e.g. 90000 = $900.00)
    pub price: i64,
    pub currency: String,
    /// "month" or "year"
    pub interval: String,
    pub created_at: Option<String>,
}

pub async fn get_tenant_ledger(tenant_id: &str) -> Result<Vec<TransactionModel>, String> {
    api_get(&format!("api/admin/billing/tenant/{}", tenant_id)).await
}

pub async fn list_billing_plans() -> Result<Vec<BillingPlanModel>, String> {
    api_get("api/admin/billing/plans").await
}

pub async fn suspend_subscription(tenant_id: &str, subscription_id: &str) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/tenant/{}/subscription/{}/suspend", tenant_id, subscription_id));
    let req = client.post(&url);
    api_request(req).await
}

pub async fn reactivate_subscription(tenant_id: &str, subscription_id: &str) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/tenant/{}/subscription/{}/reactivate", tenant_id, subscription_id));
    let req = client.post(&url);
    api_request(req).await
}

#[derive(Serialize)]
pub struct IssueCreditInput {
    pub amount_cents: i64,
    pub reason: String,
}

pub async fn issue_credit(tenant_id: &str, amount_cents: i64, reason: String) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/tenant/{}/credits", tenant_id));
    let req = client.post(&url).json(&IssueCreditInput { amount_cents, reason });
    api_request(req).await
}

#[derive(Serialize)]
pub struct GenerateInvoiceInput {
    pub amount_cents: i64,
    pub period: String,
}

pub async fn generate_invoice(tenant_id: &str, amount_cents: i64, period: String) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/tenant/{}/invoices", tenant_id));
    let req = client.post(&url).json(&GenerateInvoiceInput { amount_cents, period });
    api_request(req).await
}

#[derive(Serialize)]
pub struct ChangePlanInput {
    pub plan_id: String,
}

pub async fn change_plan(tenant_id: &str, plan_id: String) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/tenant/{}/plan", tenant_id));
    let req = client.put(&url).json(&ChangePlanInput { plan_id });
    api_request(req).await
}

// ── Billing Plan CRUD ─────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct BillingPlanInput {
    pub name: String,
    /// Price in cents
    pub price: i64,
    pub currency: Option<String>,
    /// "month" | "year"
    pub interval: String,
}

/// `POST /api/admin/billing/plans`
pub async fn create_billing_plan(input: BillingPlanInput) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url("api/admin/billing/plans");
    let req = client.post(&url).json(&input);
    api_request(req).await
}

/// `PUT /api/admin/billing/plans/{id}`
pub async fn update_billing_plan(plan_id: &str, input: BillingPlanInput) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/plans/{}", plan_id));
    let req = client.put(&url).json(&input);
    api_request(req).await
}

/// `DELETE /api/admin/billing/plans/{id}`
pub async fn delete_billing_plan(plan_id: &str) -> Result<serde_json::Value, String> {
    let client = create_client();
    let url = api_url(&format!("api/admin/billing/plans/{}", plan_id));
    let req = client.delete(&url);
    api_request(req).await
}

// ── AI Task Logs ──────────────────────────────────────────────────────────────

/// `GET /api/admin/ai-tasks/{id}/logs`
///
/// Returns a snapshot of log lines for a task from the backend.
/// Frontend polls this every 2s while status = "running".
pub async fn get_ai_task_logs(task_id: &str) -> Result<Vec<String>, String> {
    api_get(&format!("api/admin/ai-tasks/{}/logs", task_id)).await
}
