use crate::api::client::api_get;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct KpiData {
    #[serde(default)]
    pub value: f32,
    #[serde(default)]
    pub previous_value: f32,
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BusinessKpisResponse {
    #[serde(default)]
    pub mrr: KpiData,
    #[serde(default)]
    pub active_subscriptions: KpiData,
    #[serde(default)]
    pub network_liquidity_index: KpiData,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EngagementResponse {
    #[serde(default)]
    pub total_users: KpiData,
    #[serde(default)]
    pub active_listings: KpiData,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TrendPoint {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub value: f32,
}

pub async fn get_business_kpis() -> Result<BusinessKpisResponse, String> {
    api_get("/api/admin/analytics/business_kpis").await
}

pub async fn get_engagement() -> Result<EngagementResponse, String> {
    api_get("/api/admin/analytics/engagement").await
}

pub async fn get_trends(metric_key: &str, days: u32) -> Result<Vec<TrendPoint>, String> {
    let path = format!(
        "/api/admin/analytics/trends?metric_key={}&days={}",
        metric_key, days
    );
    api_get(&path).await
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExemptionSummary {
    #[serde(default)]
    pub tenant_name: String,
    #[serde(default)]
    pub app_slug: String,
    #[serde(default)]
    pub lost_revenue: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BillingSummaryResponse {
    #[serde(default)]
    pub active_subscriptions: usize,
    #[serde(default)]
    pub in_trial: usize,
    #[serde(default)]
    pub in_grace_period: usize,
    #[serde(default)]
    pub suspended: usize,
    #[serde(default)]
    pub canceled: usize,
    #[serde(default)]
    pub gross_churn_rate: f32,
    #[serde(default)]
    pub collection_success_rate: f32,
    #[serde(default)]
    pub failed_invoices_count: usize,
    #[serde(default)]
    pub failed_invoices_value: f32,
    #[serde(default)]
    pub exemptions: Vec<ExemptionSummary>,
}

pub async fn get_billing_summary() -> Result<BillingSummaryResponse, String> {
    api_get("/api/admin/analytics/billing_summary").await
}
