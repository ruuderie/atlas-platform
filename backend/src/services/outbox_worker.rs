use crate::entities::outbox_job;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set
};
use chrono::Utc;
use std::time::Duration;
use tracing::{info, error};

pub struct OutboxWorker;

impl OutboxWorker {
    pub async fn start_worker(db: DatabaseConnection) {
        tokio::spawn(async move {
            info!("Starting OutboxWorker background worker.");
            let mut interval = tokio::time::interval(Duration::from_millis(1500)); // Tick every 1.5s
            
            loop {
                interval.tick().await;
                if let Err(e) = Self::process_next_job(&db).await {
                    error!("OutboxWorker encountered an error: {}", e);
                }
            }
        });
    }

    pub async fn process_next_job(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".to_string());
        // Stale locks: older than 5 minutes can be retried/recovered
        let stale_lock_threshold = Utc::now() - chrono::Duration::minutes(5);

        let query = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE outbox_job
            SET 
                status = 'processing',
                locked_by = $1,
                locked_at = NOW(),
                attempts = attempts + 1
            WHERE id = (
                SELECT id 
                FROM outbox_job 
                WHERE (status = 'pending' 
                       OR (status = 'processing' AND locked_at < $2) 
                       OR (status = 'failed' AND attempts < 5))
                  AND run_at <= NOW()
                ORDER BY run_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, tenant_id, job_type, payload, status, attempts, error_message, locked_by, locked_at, created_at, run_at
            "#,
            vec![hostname.clone().into(), stale_lock_threshold.into()],
        );

        let opt_job = outbox_job::Entity::find()
            .from_raw_sql(query)
            .one(db)
            .await?;

        if let Some(job) = opt_job {
            info!("OutboxWorker: checked out job {} (type: {})", job.id, job.job_type);

            let start_time = std::time::Instant::now();
            let tenant_str = job.tenant_id.to_string();

            // Parse job type at the read boundary — unknown type is an immediate
            // logged error rather than a silent _ arm match.
            let job_type_enum = match crate::types::outbox::OutboxJobType::try_from(job.job_type.as_str()) {
                Ok(jt) => jt,
                Err(e) => {
                    error!("OutboxWorker: unregistered job type '{}': {}", job.job_type, e);
                    // Mark as failed so it is not retried forever
                    let mut active: outbox_job::ActiveModel = job.into();
                    active.status = Set(crate::types::outbox::OutboxJobStatus::Failed.to_string());
                    active.error_message = Set(Some(format!("unregistered job type: {e}")));
                    active.locked_by = Set(None);
                    active.locked_at = Set(None);
                    active.update(db).await?;
                    return Ok(());
                }
            };

            let result = match job_type_enum {
                crate::types::outbox::OutboxJobType::SendMagicLinkEmail => {
                    match serde_json::from_value::<crate::handlers::communications::SendEmailPayload>(job.payload.clone()) {
                        Ok(email_payload) => {
                            match crate::handlers::communications::send_email_handler(
                                axum::extract::State(db.clone()),
                                axum::extract::Json(email_payload),
                            ).await {
                                Ok(_) => Ok(()),
                                Err((status, msg)) => Err(format!("Email send failed with status {:?}: {}", status, msg)),
                            }
                        }
                        Err(e) => Err(format!("Failed to deserialize email payload: {:?}", e)),
                    }
                }

                // ── Waitlist confirmation ─────────────────────────────────────────────
                // Sent to every new lead immediately after atlas_lead is inserted.
                // Payload: { to_email, name, product_slug, variant_slug? }
                crate::types::outbox::OutboxJobType::SendWaitlistConfirmation => {
                    let to_email = job.payload.get("to_email").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let name     = job.payload.get("name").and_then(|v| v.as_str()).unwrap_or("there").to_string();
                    let product  = job.payload.get("product_slug").and_then(|v| v.as_str()).unwrap_or("folio").to_string();

                    if to_email.is_empty() {
                        return Err(sea_orm::DbErr::Custom("send_waitlist_confirmation: missing to_email in payload".to_string()));
                    }

                    let first_name = name.split_whitespace().next().unwrap_or(&name).to_string();

                    let body_html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width,initial-scale=1"/>
  <title>You're on the Folio waitlist</title>
</head>
<body style="margin:0;padding:0;background:#0d0f14;font-family:'Segoe UI',Helvetica,Arial,sans-serif;color:#e2e8f0;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background:#0d0f14;padding:40px 0;">
    <tr><td align="center">
      <table width="560" cellpadding="0" cellspacing="0"
             style="background:#141720;border:1px solid rgba(255,255,255,.08);border-radius:16px;overflow:hidden;max-width:560px;width:100%;">

        <!-- Header -->
        <tr>
          <td style="background:linear-gradient(135deg,#1a1f2e 0%,#141720 100%);padding:32px 40px 24px;border-bottom:1px solid rgba(255,255,255,.07);">
            <table width="100%" cellpadding="0" cellspacing="0">
              <tr>
                <td>
                  <span style="display:inline-block;background:#ff6b35;color:#fff;font-size:18px;font-weight:800;
                               width:36px;height:36px;line-height:36px;text-align:center;border-radius:9px;
                               margin-right:10px;vertical-align:middle;">F</span>
                  <span style="font-size:20px;font-weight:700;color:#fff;vertical-align:middle;">Folio</span>
                </td>
              </tr>
            </table>
          </td>
        </tr>

        <!-- Body -->
        <tr>
          <td style="padding:36px 40px 32px;">
            <h1 style="margin:0 0 8px;font-size:24px;font-weight:700;color:#fff;">
              You're on the list, {first_name}! 🎉
            </h1>
            <p style="margin:0 0 24px;font-size:15px;line-height:1.6;color:#94a3b8;">
              Thanks for signing up for early access to Folio — the modern landlord OS built for independent
              property owners who manage LTR, STR, and everything in between.
            </p>
            <table width="100%" cellpadding="0" cellspacing="0"
                   style="background:rgba(255,107,53,.08);border:1px solid rgba(255,107,53,.25);
                          border-radius:12px;padding:0;margin-bottom:24px;">
              <tr>
                <td style="padding:20px 24px;">
                  <p style="margin:0 0 6px;font-size:12px;font-weight:600;color:#ff6b35;text-transform:uppercase;letter-spacing:.06em;">
                    What happens next
                  </p>
                  <ul style="margin:8px 0 0;padding-left:18px;font-size:14px;line-height:1.8;color:#cbd5e1;">
                    <li>We review every application personally</li>
                    <li>Approved users receive a private invite link</li>
                    <li>You'll be one of the first to experience the full platform</li>
                  </ul>
                </td>
              </tr>
            </table>
            <p style="margin:0 0 28px;font-size:14px;line-height:1.6;color:#64748b;">
              We're being selective about early access to make sure every user gets a great experience from day one.
              We'll email you the moment your spot is confirmed.
            </p>
            <p style="margin:0;font-size:15px;color:#94a3b8;">
              — The Folio team
            </p>
          </td>
        </tr>

        <!-- Footer -->
        <tr>
          <td style="padding:20px 40px;border-top:1px solid rgba(255,255,255,.06);">
            <p style="margin:0;font-size:12px;color:#475569;text-align:center;">
              You're receiving this because you signed up for the Folio waitlist.<br/>
              If this was a mistake, you can safely ignore this email.
            </p>
          </td>
        </tr>

      </table>
    </td></tr>
  </table>
</body>
</html>"#, first_name = first_name);

                    let email_payload = crate::handlers::communications::SendEmailPayload {
                        tenant_id:   job.tenant_id,
                        to_email,
                        subject:     format!("You're on the Folio waitlist, {}!", first_name),
                        body_html,
                        attachments: vec![],
                    };

                    match crate::handlers::communications::send_email_handler(
                        axum::extract::State(db.clone()),
                        axum::extract::Json(email_payload),
                    ).await {
                        Ok(_) => {
                            tracing::info!(product = %product, "waitlist confirmation email sent");
                            Ok(())
                        }
                        Err((status, msg)) => Err(format!("waitlist confirmation email failed ({:?}): {}", status, msg)),
                    }
                }

                // G-27: Recompute dimension aggregates, composite score, confidence level,
                // and dimension_vector for all scorecards that have new verified entries
                // since the last run. Runs every 5 minutes (interval_seconds = 300).
                //
                // The payload may contain {"scorecard_id": "<uuid>"} to target a single
                // scorecard, or be absent to scan all stale scorecards for the tenant.
                crate::types::outbox::OutboxJobType::RecomputeScorecardAggregates => {
                    let maybe_id = job.payload
                        .get("scorecard_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok());

                    if let Some(scorecard_id) = maybe_id {
                        crate::services::scorecard_service::ScorecardService::recompute_aggregates(
                            db,
                            scorecard_id,
                        )
                        .await
                        .map_err(|e| format!("recompute_aggregates({scorecard_id}) failed: {e}"))
                    } else {
                        // Tenant-wide sweep: recompute all stale scorecards for this tenant.
                        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                        use crate::entities::atlas_scorecard;

                        let stale = match atlas_scorecard::Entity::find()
                            .filter(atlas_scorecard::Column::TenantId.eq(job.tenant_id))
                            .all(db)
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => return Err(sea_orm::DbErr::Custom(
                                format!("recompute_scorecard_aggregates: failed to fetch scorecards: {e}")
                            )),
                        };

                        let mut errors: Vec<String> = Vec::new();
                        for sc in stale {
                            if let Err(e) = crate::services::scorecard_service::ScorecardService::recompute_aggregates(db, sc.id).await {
                                errors.push(format!("{}: {}", sc.id, e));
                            }
                        }

                        if errors.is_empty() {
                            Ok(())
                        } else {
                            Err(format!("recompute_aggregates sweep had {} failures: {}", errors.len(), errors.join("; ")))
                        }
                    }
                }

                // G-27: Rebuild monthly + quarterly time-series trend buckets for all
                // scorecard dimensions for this tenant. Runs hourly (interval_seconds = 3600).
                //
                // refresh_time_series_for_dimension internally handles both period types
                // (monthly and quarterly) — no period arg needed.
                crate::types::outbox::OutboxJobType::RefreshScorecardTimeSeries => {
                    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                    use crate::entities::{atlas_scorecard, atlas_scorecard_dimension};

                    let scorecards = match atlas_scorecard::Entity::find()
                        .filter(atlas_scorecard::Column::TenantId.eq(job.tenant_id))
                        .all(db)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(sea_orm::DbErr::Custom(
                            format!("refresh_scorecard_time_series: failed to fetch scorecards: {e}")
                        )),
                    };

                    let mut errors: Vec<String> = Vec::new();

                    for sc in &scorecards {
                        let dimensions = match atlas_scorecard_dimension::Entity::find()
                            .filter(atlas_scorecard_dimension::Column::TemplateId.eq(sc.template_id))
                            .filter(atlas_scorecard_dimension::Column::IsActive.eq(true))
                            .all(db)
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                errors.push(format!("{}: failed to fetch dimensions: {e}", sc.id));
                                continue;
                            }
                        };

                        for dim in &dimensions {
                            if let Err(e) = crate::services::scorecard_service::ScorecardService::refresh_time_series_for_dimension(
                                db,
                                sc.id,
                                dim.id,
                            )
                            .await
                            {
                                errors.push(format!("{}:{}: {}", sc.id, dim.id, e));
                            }
                        }
                    }

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(format!("refresh_scorecard_time_series had {} failures: {}", errors.len(), errors.join("; ")))
                    }
                }



                // G-27: Evaluate display rules after an activity is logged and dispatch
                // the resulting nudge dimensions to the rater via WebSocket (G-07).
                //
                // Payload shape:
                // {
                //   "template_id":          "<uuid>",
                //   "subject_entity_type":  "lead" | "deal" | ...,
                //   "subject_entity_id":    "<uuid>",
                //   "activity_type":        "call" | "demo" | "meeting" | ...,
                //   "rater_user_id":        "<uuid>"
                // }
                //
                // Steps:
                //   1. Gate: scorecard_display_rules_enabled must be true for the tenant.
                //   2. Call ScorecardService::get_nudge_dimensions_for_activity.
                //   3. If non-empty: find-or-create the scorecard_nudge WS room for the entity.
                //   4. Write an atlas_ws_message with the nudge payload to that room.
                //
                // The frontend ScorecardWidget subscribes to "scorecard_nudge" rooms on
                // entity page load and renders NudgePrompt on receipt.
                crate::types::outbox::OutboxJobType::EvaluateScorecardNudge => {
                    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                    use crate::entities::{tenant_setting, atlas_ws_room, atlas_ws_message};

                    // ── Parse payload ────────────────────────────────────────
                    let template_id = job.payload.get("template_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok());
                    let subject_entity_type = job.payload.get("subject_entity_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_owned());
                    let subject_entity_id = job.payload.get("subject_entity_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok());
                    let activity_type = job.payload.get("activity_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_owned());
                    let rater_user_id = job.payload.get("rater_user_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok());

                    let (template_id, entity_type, entity_id, act_type) =
                        match (template_id, subject_entity_type, subject_entity_id, activity_type) {
                            (Some(t), Some(et), Some(ei), Some(at)) => (t, et, ei, at),
                            _ => {
                                error!("evaluate_scorecard_nudge: missing required payload fields, skipping");
                                return Ok(());   // early exit from process_next_job — treat as success
                            }
                        };

                    // ── Gate: scorecard_display_rules_enabled ────────────────
                    let enabled = tenant_setting::Entity::find()
                        .filter(tenant_setting::Column::TenantId.eq(job.tenant_id))
                        .filter(tenant_setting::Column::Key.eq("scorecard_display_rules_enabled"))
                        .filter(tenant_setting::Column::Value.eq("true"))
                        .one(db)
                        .await
                        .map_err(|e| format!("evaluate_scorecard_nudge: tenant_setting query: {e}"))
                        .map_err(|e| sea_orm::DbErr::Custom(e))?;

                    if enabled.is_none() {
                        info!("evaluate_scorecard_nudge: feature disabled for tenant {}, skipping", job.tenant_id);
                        return Ok(());
                    }

                    // ── Get nudge dimensions ─────────────────────────────────
                    let mut nudges = crate::services::scorecard_service::ScorecardService::get_nudge_dimensions_for_activity(
                        db,
                        job.tenant_id,
                        template_id,
                        &entity_type,
                        entity_id,
                        &act_type,
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(format!("evaluate_scorecard_nudge: get_nudge_dimensions: {e}")))?;

                    // Trigger path fallback: no display-rule match → surface all
                    // active template dimensions so post_checkout / case_resolved
                    // still deliver a WS nudge when a session was opened.
                    if nudges.is_empty() {
                        if let Some(sid) = job.payload.get("session_id").and_then(|v| v.as_str()).and_then(|s| uuid::Uuid::parse_str(s).ok()) {
                            let scorecard_id = job.payload.get("scorecard_id")
                                .and_then(|v| v.as_str())
                                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                                .unwrap_or(entity_id);
                            use crate::entities::atlas_scorecard_dimension as dims;
                            let dim_rows = dims::Entity::find()
                                .filter(dims::Column::TemplateId.eq(template_id))
                                .filter(dims::Column::TenantId.eq(job.tenant_id))
                                .filter(dims::Column::IsActive.eq(true))
                                .all(db)
                                .await
                                .map_err(|e| sea_orm::DbErr::Custom(format!("evaluate_scorecard_nudge: dim fallback: {e}")))?;
                            nudges = dim_rows
                                .into_iter()
                                .map(|d| crate::services::scorecard_service::NudgeDimension {
                                    dimension_id: d.id,
                                    dimension_slug: d.slug,
                                    dimension_name: d.name,
                                    action: "surface_as_nudge".to_owned(),
                                    scale_type: d.scale_type,
                                    scorecard_id,
                                    session_type_hint: act_type.clone(),
                                })
                                .collect();
                            if !nudges.is_empty() {
                                info!(
                                    "evaluate_scorecard_nudge: fallback dims for session {} (trigger {})",
                                    sid, act_type
                                );
                            }
                        }
                    }

                    if nudges.is_empty() {
                        info!("evaluate_scorecard_nudge: no nudge dimensions for '{}', skipping", act_type);
                        return Ok(());
                    }

                    // ── Find or create the scorecard_nudge WS room ───────────
                    let room = atlas_ws_room::Entity::find()
                        .filter(atlas_ws_room::Column::TenantId.eq(job.tenant_id))
                        .filter(atlas_ws_room::Column::RoomType.eq("scorecard_nudge"))
                        .filter(atlas_ws_room::Column::EntityType.eq(entity_type.clone()))
                        .filter(atlas_ws_room::Column::EntityId.eq(entity_id))
                        .filter(atlas_ws_room::Column::IsActive.eq(true))
                        .one(db)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(format!("evaluate_scorecard_nudge: ws_room query: {e}")))?;

                    let room_id = if let Some(r) = room {
                        r.id
                    } else {
                        let new_room = atlas_ws_room::ActiveModel {
                            id: Set(uuid::Uuid::new_v4()),
                            tenant_id: Set(job.tenant_id),
                            room_type: Set("scorecard_nudge".to_owned()),
                            entity_type: Set(entity_type.clone()),
                            entity_id: Set(entity_id),
                            is_active: Set(true),
                            created_at: Set(Utc::now()),
                        };
                        new_room.insert(db)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(format!("evaluate_scorecard_nudge: ws_room insert: {e}")))?
                            .id
                    };

                    // ── Write the nudge message ──────────────────────────────
                    // Content is the full NudgeDimension list as JSON.
                    // The frontend ScorecardWidget deserializes and renders NudgePrompt.
                    let content = serde_json::to_string(&nudges)
                        .map_err(|e| sea_orm::DbErr::Custom(format!("evaluate_scorecard_nudge: serialize: {e}")))?;

                    let msg = atlas_ws_message::ActiveModel {
                        id: Set(uuid::Uuid::new_v4()),
                        room_id: Set(room_id),
                        sender_user_id: Set(None), // system-generated
                        message_type: Set("scorecard_nudge".to_owned()),
                        content: Set(content),
                        translated_content: Set(None),
                        attachment_id: Set(None),
                        created_at: Set(Utc::now()),
                    };
                    msg.insert(db)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(format!("evaluate_scorecard_nudge: ws_message insert: {e}")))?;

                    info!(
                        "evaluate_scorecard_nudge: dispatched {} nudge dimensions to room {} ({}/{})",
                        nudges.len(), room_id, entity_type, entity_id
                    );

                    if let Some(uid) = rater_user_id {
                        info!("evaluate_scorecard_nudge: nudge targeted at rater_user_id={uid}");
                    }

                    Ok(())
                }


                // G-27 Phase 3: Refresh mv_scorecard_portfolio_analytics + batch-update
                // percentile ranks for all scorecards in every template for this tenant.
                // Runs every 4 hours (interval_seconds = 14400).
                //
                // Uses REFRESH MATERIALIZED VIEW CONCURRENTLY — readers are never blocked.
                // After the MV refresh, writes updated percentile_rank, percentile_band,
                // and percentile_cohort_size to atlas_scorecard_dimension_aggregates for
                // every dimension of every scorecard in the template's pool.
                //
                // Also serves as the source of the BYOC peer_pool snapshot (Phase 5):
                // the freshly-refreshed MV data is used by peer_pool_snapshot() in
                // G27SC_ByocComputeCalloutController requests.
                crate::types::outbox::OutboxJobType::RefreshScorecardPortfolio => {
                    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                    use crate::entities::atlas_scorecard_template;

                    let templates = match atlas_scorecard_template::Entity::find()
                        .filter(atlas_scorecard_template::Column::TenantId.eq(job.tenant_id))
                        .filter(atlas_scorecard_template::Column::IsPublished.eq(true))
                        .all(db)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(sea_orm::DbErr::Custom(
                            format!("refresh_scorecard_portfolio: failed to fetch templates: {e}")
                        )),
                    };

                    if templates.is_empty() {
                        info!(
                            "refresh_scorecard_portfolio: no published templates for tenant {}, skipping",
                            job.tenant_id
                        );
                        return Ok(());
                    }

                    let mut errors: Vec<String> = Vec::new();

                    for template in &templates {
                        if let Err(e) = crate::services::scorecard_analytics_service::ScorecardAnalyticsService::refresh_and_rerank(
                            db,
                            template.id,
                            job.tenant_id,
                        )
                        .await
                        {
                            errors.push(format!("template {}: {e}", template.id));
                        } else {
                            info!(
                                "refresh_scorecard_portfolio: template {} refreshed successfully",
                                template.id
                            );
                        }
                    }

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(format!(
                            "refresh_scorecard_portfolio had {} failures: {}",
                            errors.len(),
                            errors.join("; ")
                        ))
                    }
                }


                // G-27 Phase 4: Compute per-contributor bias_offset + scale_factor for all
                // published templates for this tenant. Runs weekly (interval_seconds = 604800).
                //
                // Reads all verified entries for the template, groups by (contributor, dimension),
                // and writes calibration rows to atlas_scorecard_contributor_calibration.
                //
                // Gate: only contributors with entry_count >= template.calibration_minimum_entries
                // (default 100) receive calibration rows. Below threshold, raw scores are used.
                //
                // Applied automatically on the next recompute_scorecard_aggregates run — the
                // calibration map is loaded once per scorecard recompute (keyed by
                // (contributor_user_id, dimension_id)) and applied per-entry in
                // compute_numeric_aggregate.
                crate::types::outbox::OutboxJobType::CalibrateScorecardContributors => {
                    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                    use crate::entities::atlas_scorecard_template;

                    let templates = match atlas_scorecard_template::Entity::find()
                        .filter(atlas_scorecard_template::Column::TenantId.eq(job.tenant_id))
                        .filter(atlas_scorecard_template::Column::IsPublished.eq(true))
                        .all(db)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => return Err(sea_orm::DbErr::Custom(
                            format!("calibrate_scorecard_contributors: failed to fetch templates: {e}")
                        )),
                    };

                    if templates.is_empty() {
                        info!(
                            "calibrate_scorecard_contributors: no published templates for tenant {}, skipping",
                            job.tenant_id
                        );
                        return Ok(());
                    }

                    let mut errors: Vec<String> = Vec::new();
                    let mut total_upserted: usize = 0;

                    for template in &templates {
                        match crate::services::scorecard_service::ScorecardService::calibrate_contributor_bias(
                            db,
                            template.id,
                        )
                        .await
                        {
                            Ok(upserted) => {
                                total_upserted += upserted;
                                info!(
                                    "calibrate_scorecard_contributors: template {} → {} calibration rows upserted",
                                    template.id, upserted
                                );
                            }
                            Err(e) => {
                                errors.push(format!("template {}: {e}", template.id));
                            }
                        }
                    }

                    info!(
                        "calibrate_scorecard_contributors: tenant {} → {} total rows upserted across {} templates",
                        job.tenant_id, total_upserted, templates.len()
                    );

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(format!(
                            "calibrate_scorecard_contributors had {} failures: {}",
                            errors.len(),
                            errors.join("; ")
                        ))
                    }
                }

                // G-19: release hold — handled by core_platform background_jobs
                // This variant is registered here so it is part of the enum and does
                // not fall through to an unknown-type error. The actual execution runs
                // in the tenant_background_job loop, not the outbox worker.
                crate::types::outbox::OutboxJobType::ReleaseExpiredReservationHolds => {
                    Ok(()) // delegated — outbox worker is not the executor for this type
                }

                // G-07 ext: Notification channel delivery
                // Dispatches telegram / whatsapp / sms / email to one channel per job.
                // Records delivery status in atlas_notification.channels_attempted.
                crate::types::outbox::OutboxJobType::NotifyChannel => {
                    use crate::services::notification_service::{NotifyChannelPayload, channels};
                    use crate::entities::atlas_notification;
                    use sea_orm::{EntityTrait, ActiveModelTrait, Set};
                    use chrono::Utc;

                    let payload = match serde_json::from_value::<NotifyChannelPayload>(job.payload.clone()) {
                        Ok(p)  => p,
                        Err(e) => return Err(sea_orm::DbErr::Custom(format!("NotifyChannel: failed to deserialise payload: {e}"))),
                    };

                    // Fetch tenant settings for channel credentials
                    let settings: std::collections::HashMap<String, String> = {
                        use crate::entities::tenant_setting;
                        use sea_orm::{QueryFilter, ColumnTrait};
                        tenant_setting::Entity::find()
                            .filter(tenant_setting::Column::TenantId.eq(payload.tenant_id))
                            .all(db)
                            .await
                            .unwrap_or_default()
                            .into_iter()
                            .map(|s| (s.key, s.value))
                            .collect()
                    };

                    let result = channels::dispatch_channel_job(&payload, &settings).await;

                    // Write delivery receipt into atlas_notification.channels_attempted
                    let receipt = match &result {
                        channels::ChannelResult::Delivered      => serde_json::json!({ "channel": payload.channel, "status": "delivered",  "attempted_at": Utc::now() }),
                        channels::ChannelResult::Skipped { reason } => serde_json::json!({ "channel": payload.channel, "status": "skipped",   "attempted_at": Utc::now(), "reason": reason }),
                        channels::ChannelResult::Failed { error }   => serde_json::json!({ "channel": payload.channel, "status": "failed",    "attempted_at": Utc::now(), "error": error }),
                    };

                    // Append receipt to channels_attempted via read-modify-write
                    if let Ok(Some(notif)) = atlas_notification::Entity::find_by_id(payload.notification_id)
                        .one(db)
                        .await
                    {
                        use sea_orm::{ActiveModelTrait, Set};
                        let mut existing: Vec<serde_json::Value> = notif.channels_attempted
                            .as_array()
                            .cloned()
                            .unwrap_or_default();
                        existing.push(receipt);
                        let mut active: atlas_notification::ActiveModel = notif.into();
                        active.channels_attempted = Set(serde_json::Value::Array(existing));
                        let _ = active.update(db).await;
                    }

                    match result {
                        channels::ChannelResult::Delivered                => Ok(()),
                        channels::ChannelResult::Skipped { reason }       => {
                            info!("NotifyChannel: skipped channel={} reason={}", payload.channel, reason);
                            Ok(()) // Skipped is not an error
                        }
                        channels::ChannelResult::Failed { error }         => {
                            Err(format!("NotifyChannel: channel={} error={}", payload.channel, error))
                        }
                    }
                }
            };

            let duration = start_time.elapsed().as_secs_f64();
            crate::metrics::OUTBOX_JOB_LATENCY
                .with_label_values(&[&tenant_str, &job.job_type])
                .observe(duration);

            let job_type = job.job_type.clone();
            let mut active: outbox_job::ActiveModel = job.into();
            let job_id = active.id.as_ref().clone();
            match result {
                Ok(_) => {
                    active.status = Set(crate::types::outbox::OutboxJobStatus::Completed.to_string());
                    active.error_message = Set(None);
                    active.locked_by = Set(None);
                    active.locked_at = Set(None);
                    active.update(db).await?;
                    info!("OutboxWorker: successfully processed job: {}", job_id);

                    crate::metrics::OUTBOX_JOBS_PROCESSED
                        .with_label_values(&[&tenant_str, &job_type])
                        .inc();
                }
                Err(err_msg) => {
                    error!("OutboxWorker: job execution failed: {}", err_msg);

                    crate::metrics::OUTBOX_JOB_FAILURES
                        .with_label_values(&[&tenant_str, &job_type, &err_msg])
                        .inc();

                    active.status = Set(crate::types::outbox::OutboxJobStatus::Failed.to_string());
                    active.error_message = Set(Some(err_msg));
                    active.locked_by = Set(None);
                    active.locked_at = Set(None);
                    
                    // Simple exponential backoff for retries: retry after 2^attempts * 10 seconds
                    let attempts = active.attempts.as_ref();
                    let backoff_secs = 2i64.pow(*attempts as u32) * 10;
                    let next_run = Utc::now() + chrono::Duration::seconds(backoff_secs);
                    active.run_at = Set(next_run);
                    
                    active.update(db).await?;
                }
            }
        }

        Ok(())
    }
}
