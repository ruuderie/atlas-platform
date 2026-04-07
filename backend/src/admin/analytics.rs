use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, QuerySelect, sea_query::Expr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::entities::platform_metrics_daily;
use chrono::{NaiveDate, Utc};
use moka::future::Cache;
use once_cell::sync::Lazy;
use std::time::Duration;

static ANALYTICS_CACHE: Lazy<Cache<String, String>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(900))
        .build()
});

#[derive(Serialize, Deserialize, Clone)]
pub struct KpiData {
    pub value: f32,
    pub previous_value: f32, // for computing e.g. % growth
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BusinessKpisResponse {
    pub mrr: KpiData,
    pub active_subscriptions: KpiData,
    pub network_liquidity_index: KpiData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TrendPoint {
    pub date: String,
    pub value: f32,
}

#[derive(Deserialize)]
pub struct TrendsQuery {
    pub metric_key: String,
    pub days: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EngagementResponse {
    pub total_users: KpiData,
    pub active_listings: KpiData,
}

async fn fetch_latest_metric_sum(
    db: &DatabaseConnection,
    metric_key: &str,
    date: NaiveDate,
) -> Result<f32, sea_orm::DbErr> {
    let result: Option<f32> = platform_metrics_daily::Entity::find()
        .filter(platform_metrics_daily::Column::MetricKey.eq(metric_key))
        .filter(platform_metrics_daily::Column::Date.eq(date))
        .select_only()
        .column_as(Expr::col(platform_metrics_daily::Column::MetricValue).sum(), "sum")
        .into_tuple()
        .one(db)
        .await?;
        
    Ok(result.unwrap_or(0.0))
}

pub async fn get_business_kpis(
    State(db): State<DatabaseConnection>,
) -> Result<Json<BusinessKpisResponse>, (StatusCode, String)> {
    if let Some(cached_str) = ANALYTICS_CACHE.get(&"business_kpis".to_string()).await {
        if let Ok(res) = serde_json::from_str::<BusinessKpisResponse>(&cached_str) {
            return Ok(Json(res));
        }
    }

    let today = Utc::now().date_naive();
    let yesterday = today.pred_opt().unwrap_or(today);

    let mrr_today = fetch_latest_metric_sum(&db, "mrr", today)
        .await.unwrap_or(0.0);
    let mrr_yesterday = fetch_latest_metric_sum(&db, "mrr", yesterday)
        .await.unwrap_or(0.0);

    let subs_today = fetch_latest_metric_sum(&db, "subscription_created", today)
        .await.unwrap_or(0.0);

    let liq_today = fetch_latest_metric_sum(&db, "network_liquidity_index", today)
        .await.unwrap_or(0.0);

    let response = BusinessKpisResponse {
        mrr: KpiData { value: mrr_today, previous_value: mrr_yesterday, name: "MRR".to_string() },
        active_subscriptions: KpiData { value: subs_today, previous_value: subs_today, name: "Active Subs".to_string() },
        network_liquidity_index: KpiData { value: liq_today, previous_value: liq_today, name: "Liquidity Index".to_string() },
    };

    if let Ok(json_str) = serde_json::to_string(&response) {
        ANALYTICS_CACHE.insert("business_kpis".to_string(), json_str).await;
    }

    Ok(Json(response))
}

pub async fn get_engagement(
    State(db): State<DatabaseConnection>,
) -> Result<Json<EngagementResponse>, (StatusCode, String)> {
    let today = Utc::now().date_naive();
    
    let users = fetch_latest_metric_sum(&db, "user_signed_up", today)
        .await.unwrap_or(0.0);
    let listings = fetch_latest_metric_sum(&db, "listing_published", today)
        .await.unwrap_or(0.0);

    Ok(Json(EngagementResponse {
        total_users: KpiData { value: users, previous_value: users, name: "Total Users".to_string() },
        active_listings: KpiData { value: listings, previous_value: listings, name: "Active Listings".to_string() },
    }))
}

pub async fn get_trends(
    State(db): State<DatabaseConnection>,
    Query(query): Query<TrendsQuery>,
) -> Result<Json<Vec<TrendPoint>>, (StatusCode, String)> {
    let days_limit = query.days.unwrap_or(30) as i64;
    let _cutoff_date = Utc::now().date_naive() - chrono::Duration::days(days_limit);

    // Group by Date for the specific metric
    let results: Vec<(NaiveDate, f32)> = platform_metrics_daily::Entity::find()
        .filter(platform_metrics_daily::Column::MetricKey.eq(&query.metric_key))
        //.filter(platform_metrics_daily::Column::Date.gte(cutoff_date)) // Note: Requires sea_orm::QueryFilter trait, uncomment when needed or if filtering bounds
        .select_only()
        .column(platform_metrics_daily::Column::Date)
        .column_as(Expr::col(platform_metrics_daily::Column::MetricValue).sum(), "sum")
        .group_by(platform_metrics_daily::Column::Date)
        .order_by_asc(platform_metrics_daily::Column::Date)
        .into_tuple()
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let trends = results.into_iter().map(|(date, val)| TrendPoint {
        date: date.to_string(),
        value: val,
    }).collect();

    Ok(Json(trends))
}
