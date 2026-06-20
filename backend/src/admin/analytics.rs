use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait,
    QueryOrder, QuerySelect, sea_query::Expr, PaginatorTrait,
};
use serde::{Deserialize, Serialize};
use crate::entities::{
    platform_metrics_daily,
    atlas_subscription::{self, SubscriptionStatus},
    user_account,
    listing,
    atlas_ledger_entry,
    tenant,
    atlas_app_deployment_config,
};
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
    pub previous_value: f32,
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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Sum a `platform_metrics_daily` metric key for a specific date (used for
/// time-series KPIs that are pre-aggregated by the metrics pipeline).
async fn fetch_daily_metric_sum(
    db: &DatabaseConnection,
    metric_key: &str,
    date: NaiveDate,
) -> f32 {
    platform_metrics_daily::Entity::find()
        .filter(platform_metrics_daily::Column::MetricKey.eq(metric_key))
        .filter(platform_metrics_daily::Column::Date.eq(date))
        .select_only()
        .column_as(Expr::col(platform_metrics_daily::Column::MetricValue).sum(), "sum")
        .into_tuple::<Option<f32>>()
        .one(db)
        .await
        .ok()
        .flatten()
        .flatten()
        .unwrap_or(0.0)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn get_business_kpis(
    State(db): State<DatabaseConnection>,
) -> Result<Json<BusinessKpisResponse>, (StatusCode, String)> {
    if let Some(cached_str) = ANALYTICS_CACHE.get(&"business_kpis".to_string()).await {
        if let Ok(res) = serde_json::from_str::<BusinessKpisResponse>(&cached_str) {
            return Ok(Json(res));
        }
    }

    // --- MRR: sum of price_cents for all active subscriptions ---
    let all_subs = atlas_subscription::Entity::find()
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let today = Utc::now().date_naive();
    let yesterday = today.pred_opt().unwrap_or(today);

    // Current active subscription set
    let active_subs: Vec<_> = all_subs.iter().filter(|s| s.status == SubscriptionStatus::Active).collect();
    let active_count = active_subs.len() as f32;

    // MRR = sum of monthly price in dollars for all active subscriptions
    let mrr_cents: i64 = active_subs.iter().map(|s| s.price_cents).sum();
    let mrr_today = mrr_cents as f32 / 100.0;

    // MRR yesterday from metrics table (pre-aggregated by ingest pipeline) or fall back to today
    let mrr_yesterday = {
        let v = fetch_daily_metric_sum(&db, "mrr", yesterday).await;
        if v > 0.0 { v } else { mrr_today }
    };

    // --- Network Liquidity Index: net settled ledger volume per active sub ---
    let net_cents: Option<i64> = atlas_ledger_entry::Entity::find()
        .filter(atlas_ledger_entry::Column::Status.eq("settled"))
        .select_only()
        .column_as(Expr::col(atlas_ledger_entry::Column::NetAmountCents).sum(), "sum")
        .into_tuple()
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let net_settled = net_cents.unwrap_or(0) as f32 / 100.0;
    let liquidity_index = if active_count > 0.0 {
        net_settled / active_count
    } else {
        // Fall back to metrics pipeline value if we have no subscription data yet
        fetch_daily_metric_sum(&db, "network_liquidity_index", today).await
    };

    let response = BusinessKpisResponse {
        mrr: KpiData {
            value: mrr_today,
            previous_value: mrr_yesterday,
            name: "MRR".to_string(),
        },
        active_subscriptions: KpiData {
            value: active_count,
            previous_value: active_count,
            name: "Active Subscriptions".to_string(),
        },
        network_liquidity_index: KpiData {
            value: liquidity_index,
            previous_value: liquidity_index,
            name: "Network Liquidity Index".to_string(),
        },
    };

    if let Ok(json_str) = serde_json::to_string(&response) {
        ANALYTICS_CACHE.insert("business_kpis".to_string(), json_str).await;
    }

    Ok(Json(response))
}

pub async fn get_engagement(
    State(db): State<DatabaseConnection>,
) -> Result<Json<EngagementResponse>, (StatusCode, String)> {
    // Total active users
    let total_users = user_account::Entity::find()
        .filter(user_account::Column::IsActive.eq(true))
        .count(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? as f32;

    // Active listings
    let active_listings = listing::Entity::find()
        .filter(listing::Column::IsActive.eq(true))
        .count(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? as f32;

    Ok(Json(EngagementResponse {
        total_users: KpiData {
            value: total_users,
            previous_value: total_users,
            name: "Total Active Users".to_string(),
        },
        active_listings: KpiData {
            value: active_listings,
            previous_value: active_listings,
            name: "Active Listings".to_string(),
        },
    }))
}

pub async fn get_trends(
    State(db): State<DatabaseConnection>,
    Query(query): Query<TrendsQuery>,
) -> Result<Json<Vec<TrendPoint>>, (StatusCode, String)> {
    let days_limit = query.days.unwrap_or(30) as i64;
    let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(days_limit);

    let results: Vec<(NaiveDate, f32)> = platform_metrics_daily::Entity::find()
        .filter(platform_metrics_daily::Column::MetricKey.eq(&query.metric_key))
        .filter(platform_metrics_daily::Column::Date.gte(cutoff_date))
        .select_only()
        .column(platform_metrics_daily::Column::Date)
        .column_as(Expr::col(platform_metrics_daily::Column::MetricValue).sum(), "sum")
        .group_by(platform_metrics_daily::Column::Date)
        .order_by_asc(platform_metrics_daily::Column::Date)
        .into_tuple()
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let trends = results
        .into_iter()
        .map(|(date, val)| TrendPoint {
            date: date.to_string(),
            value: val,
        })
        .collect();

    Ok(Json(trends))
}

// ---------------------------------------------------------------------------
// Billing summary
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct ExemptionSummary {
    pub tenant_name: String,
    pub app_slug: String,
    pub lost_revenue: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BillingSummaryResponse {
    pub active_subscriptions: usize,
    pub in_trial: usize,
    pub in_grace_period: usize,
    pub suspended: usize,
    pub canceled: usize,
    pub gross_churn_rate: f32,
    pub collection_success_rate: f32,
    pub failed_invoices_count: usize,
    pub failed_invoices_value: f32,
    pub exemptions: Vec<ExemptionSummary>,
}

pub async fn get_billing_summary(
    State(db): State<DatabaseConnection>,
) -> Result<Json<BillingSummaryResponse>, (StatusCode, String)> {
    // --- 1. Subscription lifecycle counts ---
    let all_subs = atlas_subscription::Entity::find()
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut active = 0usize;
    let mut trial = 0usize;
    let mut grace = 0usize;
    let mut suspended = 0usize;
    let mut canceled = 0usize;

    for sub in &all_subs {
        match sub.status {
            SubscriptionStatus::Active    => active    += 1,
            SubscriptionStatus::Trial     => trial     += 1,
            SubscriptionStatus::PastDue   => grace     += 1,
            SubscriptionStatus::Suspended => suspended += 1,
            SubscriptionStatus::Canceled  => canceled  += 1,
        }
    }

    let total = active + trial + grace + suspended + canceled;
    let gross_churn_rate = if total > 0 {
        (canceled as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    // --- 2. Collection success rate from atlas_ledger_entry ---
    let all_ledger = atlas_ledger_entry::Entity::find()
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_ledger = all_ledger.len();
    let settled_count = all_ledger.iter().filter(|e| e.status == "settled" || e.status == "paid").count();
    let collection_success_rate = if total_ledger > 0 {
        (settled_count as f32 / total_ledger as f32) * 100.0
    } else {
        100.0 // no invoices yet = 100% (vacuously true)
    };

    // --- 3. Failed invoices ---
    let failed_entries: Vec<_> = all_ledger.iter().filter(|e| e.status == "failed").collect();
    let failed_invoices_count = failed_entries.len();
    let failed_invoices_value: f32 = failed_entries
        .iter()
        .map(|e| e.gross_amount_cents as f32 / 100.0)
        .sum();

    // --- 4. Billing exemptions ---
    let exempt_subs: Vec<_> = all_subs.iter().filter(|s| s.is_billing_exempt).collect();
    let mut exemptions: Vec<ExemptionSummary> = Vec::new();

    for sub in exempt_subs {
        let tenant_name = tenant::Entity::find_by_id(sub.tenant_id)
            .one(&db)
            .await
            .ok()
            .flatten()
            .map(|t| t.name)
            .unwrap_or_else(|| "Unknown Tenant".to_string());

        let app_slug = atlas_app_deployment_config::Entity::find()
            .filter(atlas_app_deployment_config::Column::TenantId.eq(sub.tenant_id))
            .one(&db)
            .await
            .ok()
            .flatten()
            .map(|c| c.app_slug)
            .unwrap_or_else(|| "folio".to_string());

        let lost_revenue = format!(
            "${}.{:02}/mo",
            sub.price_cents / 100,
            sub.price_cents % 100,
        );
        let reason = sub
            .billing_exemption_reason
            .clone()
            .unwrap_or_else(|| "No reason provided".to_string());

        exemptions.push(ExemptionSummary {
            tenant_name,
            app_slug,
            lost_revenue,
            reason,
        });
    }

    Ok(Json(BillingSummaryResponse {
        active_subscriptions: active,
        in_trial: trial,
        in_grace_period: grace,
        suspended,
        canceled,
        gross_churn_rate,
        collection_success_rate,
        failed_invoices_count,
        failed_invoices_value,
        exemptions,
    }))
}
