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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingPlanModel {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub base_price_cents: i64,
    pub include_seats: i32,
    pub overage_seat_price_cents: i64,
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
