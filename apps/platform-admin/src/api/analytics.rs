use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct KpiData {
    pub value: f32,
    pub previous_value: f32, 
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BusinessKpisResponse {
    pub mrr: KpiData,
    pub active_subscriptions: KpiData,
    pub network_liquidity_index: KpiData,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EngagementResponse {
    pub total_users: KpiData,
    pub active_listings: KpiData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrendPoint {
    pub date: String,
    pub value: f32,
}

pub async fn get_business_kpis() -> Result<BusinessKpisResponse, String> {
    api_get("/api/admin/analytics/business_kpis").await
}

pub async fn get_engagement() -> Result<EngagementResponse, String> {
    api_get("/api/admin/analytics/engagement").await
}

pub async fn get_trends(metric_key: &str, days: u32) -> Result<Vec<TrendPoint>, String> {
    let path = format!("/api/admin/analytics/trends?metric_key={}&days={}", metric_key, days);
    api_get(&path).await
}
