//! # Attribution Conversion Hooks
//!
//! Helper functions to record paid conversions in the attribution system.
//! These are called from various payment success handlers throughout the platform.

use anyhow::Result;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::services::pm::attribution::{AttributionService, ConversionPayload};
use crate::types::pm::AttributionModel;

/// Record a paid conversion against existing attribution touchpoints.
/// 
/// Call this from Stripe webhook handlers, payment success callbacks, or any
/// place where a monetary conversion is confirmed. This will find all 
/// attribution touchpoints for the user/email within the attribution window
/// and distribute conversion credit according to the specified model.
///
/// ## Usage
/// ```ignore
/// // From a Stripe webhook handler after confirming payment:
/// if let Err(e) = record_paid_conversion(
///     &db,
///     tenant_id,
///     user_id,
///     email,
///     "atlas_reservations", // or "stripe_payment_intents"
///     booking_id,           // or payment_intent_id
///     total_cents,
///     AttributionModel::LastTouch,
///     None, // use default 30-day window
/// ).await {
///     // Log but don't fail the payment flow
///     tracing::warn!(error = %e, "attribution conversion recording failed");
/// }
/// ```
pub async fn record_paid_conversion(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    contact_email: Option<String>,
    conversion_entity_type: &str,
    conversion_entity_id: Uuid,
    conversion_value_cents: i64,
    model: AttributionModel,
    attribution_window_days: Option<i64>,
) -> Result<Vec<Uuid>> {
    let payload = ConversionPayload {
        user_id,
        contact_email,
        conversion_entity_type: conversion_entity_type.to_string(),
        conversion_entity_id,
        conversion_value_cents,
        model,
        attribution_window_days,
    };

    let credited_touchpoint_ids = AttributionService::record_conversion(db, tenant_id, payload).await?;

    tracing::info!(
        %tenant_id,
        ?user_id,
        %conversion_entity_type,
        %conversion_entity_id,
        value_cents = conversion_value_cents,
        touchpoints_credited = credited_touchpoint_ids.len(),
        "record_paid_conversion: conversion recorded"
    );

    Ok(credited_touchpoint_ids)
}