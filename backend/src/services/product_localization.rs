//! Product Localization Service
//!
//! Enqueues G-08 `atlas_ai_task` jobs to AI-localize product page variants.
//! The background worker picks up tasks of type `"localize_product_page"`,
//! calls the configured LLM, and writes the results back to the variant's
//! `hero_overrides` and `block_overrides` fields.
//!
//! ## Task lifecycle
//!
//! ```
//! bulk-generate (copy_strategy="ai_localize")
//!   └─► ProductLocalizationService::enqueue_variant_localization()
//!         └─► atlas_ai_task { task_type="localize_product_page", status="queued" }
//!               └─► background worker: LocalizeProductPageWorker::process()
//!                     ├─► LLM call with structured prompt
//!                     ├─► parse structured JSON output
//!                     ├─► write hero_overrides + block_overrides to variant
//!                     ├─► set localization_status = "complete"
//!                     └─► update variant.updated_at
//! ```
//!
//! ## Structured prompt contract
//!
//! Input payload sent to the LLM (via atlas_ai_task.input_payload):
//! ```json
//! {
//!   "variant_id": "uuid",
//!   "product_name": "Folio",
//!   "locale": "pt-BR",
//!   "city": "São Paulo",
//!   "region": "SP",
//!   "country": "Brazil",
//!   "country_code": "BR",
//!   "source_hero": { ... template hero fields ... },
//!   "source_blocks": [ ... template blocks ... ],
//!   "instructions": "Translate and culturally adapt for pt-BR speakers in São Paulo..."
//! }
//! ```
//!
//! Expected output (stored in atlas_ai_task.output_payload, then applied to variant):
//! ```json
//! {
//!   "hero_overrides": { "headline": "...", "subheadline": "..." },
//!   "block_overrides": { "[block_id]": { "body": "..." } },
//!   "meta_title": "Folio — Gestão de imóveis em São Paulo",
//!   "meta_description": "...",
//! }
//! ```

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    entities::{
        atlas_ai_task,
        product_page::{template, variant},
        platform_product,
    },
    services::ai_task_service::AiTaskService,
};

/// Platform sentinel tenant ID — used for platform-level AI tasks
/// that are not scoped to any customer tenant.
pub const PLATFORM_SENTINEL_TENANT_ID: Uuid = Uuid::nil();

pub struct ProductLocalizationService;

impl ProductLocalizationService {
    /// Enqueue an AI localization task for a single variant.
    ///
    /// Sets variant.localization_status = "pending" and creates an atlas_ai_task.
    /// The task is processed by the `localize_product_page` background worker.
    pub async fn enqueue_variant_localization(
        db: &DatabaseConnection,
        variant_id: Uuid,
    ) -> Result<Uuid, String> {
        // Load variant + template + product for context
        let v = variant::Entity::find_by_id(variant_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("variant {variant_id} not found"))?;

        let tmpl = template::Entity::find_by_id(v.template_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "template not found".to_string())?;

        let product = platform_product::Entity::find_by_id(v.product_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "product not found".to_string())?;

        // Build structured prompt input
        let input_payload = json!({
            "variant_id": variant_id,
            "product_name": product.name,
            "product_tagline": product.tagline,
            "locale": v.locale,
            "city": v.city,
            "region": v.region,
            "country_code": v.country_code,
            "source_hero": tmpl.hero_payload,
            "source_blocks": tmpl.blocks_payload,
            "current_hero_overrides": v.hero_overrides,
            "current_block_overrides": v.block_overrides,
            "source_meta_title": tmpl.meta_title,
            "source_meta_description": tmpl.meta_description,
            "instructions": build_localization_instructions(&v, &product),
            "output_schema": {
                "hero_overrides": "object — field-level overrides matching source_hero keys",
                "block_overrides": "object — { block_id: { field: localized_value } }",
                "meta_title": "string <60 chars — localized SEO title",
                "meta_description": "string <160 chars — localized SEO description",
            }
        });

        // Enqueue G-08 task
        let task_id = AiTaskService::enqueue_task(
            db,
            PLATFORM_SENTINEL_TENANT_ID,
            "localize_product_page",
            input_payload,
            Some("product_page_variants"),
            Some(variant_id),
        )
        .await?;

        // Set variant status = pending
        let mut active: variant::ActiveModel = v.into();
        active.localization_status = Set("pending".to_string());
        active.localization_task_id = Set(Some(task_id));
        active.update(db).await.map_err(|e| e.to_string())?;

        Ok(task_id)
    }

    /// Process the output of a completed AI localization task and apply it to the variant.
    ///
    /// Called by the background worker after the LLM responds.
    pub async fn apply_localization_result(
        db: &DatabaseConnection,
        task_id: Uuid,
    ) -> Result<(), String> {
        let task = atlas_ai_task::Entity::find_by_id(task_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("task {task_id} not found"))?;

        let output = task.output_payload
            .ok_or_else(|| "task has no output_payload".to_string())?;

        let variant_id = task.callback_entity_id
            .or(task.source_entity_id)
            .ok_or_else(|| "task has no variant_id reference".to_string())?;

        let v = variant::Entity::find_by_id(variant_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("variant {variant_id} not found"))?;

        let mut active: variant::ActiveModel = v.into();

        // Apply output fields — each is optional in the LLM response
        if let Some(hero_ov) = output.get("hero_overrides") {
            active.hero_overrides = Set(hero_ov.clone());
        }
        if let Some(block_ov) = output.get("block_overrides") {
            active.block_overrides = Set(block_ov.clone());
        }
        if let Some(title) = output.get("meta_title").and_then(|v| v.as_str()) {
            active.meta_title = Set(Some(title.to_string()));
        }
        if let Some(desc) = output.get("meta_description").and_then(|v| v.as_str()) {
            active.meta_description = Set(Some(desc.to_string()));
        }

        active.localization_status = Set("complete".to_string());
        active.update(db).await.map_err(|e| e.to_string())?;

        tracing::info!(
            task_id = %task_id,
            variant_id = %variant_id,
            "AI localization applied to variant"
        );

        Ok(())
    }

    /// Mark a variant as localization-failed (called by worker on error).
    pub async fn mark_localization_failed(
        db: &DatabaseConnection,
        variant_id: Uuid,
        error: &str,
    ) -> Result<(), String> {
        let v = variant::Entity::find_by_id(variant_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("variant {variant_id} not found"))?;

        let mut active: variant::ActiveModel = v.into();
        active.localization_status = Set("failed".to_string());
        active.update(db).await.map_err(|e| e.to_string())?;

        tracing::error!(variant_id = %variant_id, error, "AI localization failed");
        Ok(())
    }

    /// Bulk: enqueue localization for all variants of a product that use ai_localize strategy.
    pub async fn enqueue_all_pending_for_product(
        db: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Vec<Uuid>, String> {
        let variants = variant::Entity::find()
            .filter(variant::Column::ProductId.eq(product_id))
            .filter(variant::Column::CopyStrategy.eq("ai_localize"))
            .filter(variant::Column::LocalizationStatus.is_in(["not_started", "failed"]))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

        let mut task_ids = Vec::new();
        for v in variants {
            match Self::enqueue_variant_localization(db, v.id).await {
                Ok(tid) => task_ids.push(tid),
                Err(e) => tracing::warn!(variant_id = %v.id, error = %e, "failed to enqueue localization"),
            }
        }

        Ok(task_ids)
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_localization_instructions(v: &variant::Model, product: &platform_product::Model) -> String {
    let city = v.city.as_deref().unwrap_or("this market");
    let locale = &v.locale;
    let country = v.country_code.as_deref().unwrap_or("");

    format!(
        "You are a professional marketing copywriter specializing in SaaS for international markets. \
         Translate and culturally adapt the following content for the {locale} market (country: {country}, city: {city}). \
         \n\nProduct: {name}. {tagline}. \
         \n\nRules:\
         \n1. Translate to {locale} — do not use English unless the target market commonly uses English terms.\
         \n2. Culturally adapt, not just translate. Use local real estate terminology for {country}.\
         \n3. Reference {city} specifically in headlines and subheadlines where natural.\
         \n4. Keep CTAs short and action-oriented in the target language.\
         \n5. meta_title must be under 60 characters. meta_description under 160 characters.\
         \n6. Return ONLY valid JSON matching the output_schema. No markdown, no explanation.",
        locale = locale,
        country = country,
        city = city,
        name = product.name,
        tagline = product.tagline.as_deref().unwrap_or(""),
    )
}
