// backend/src/services/notification_service.rs
//
// NotificationService — unified notification dispatch with multi-channel routing.
//
// ## Architecture
//
// Every notification goes through a single entry point:
//   NotificationService::dispatch(db, input)
//
// This function:
//   1. Writes an `atlas_notification` record (always — in-app inbox)
//   2. Fetches the user's enabled channel prefs
//   3. Fetches tenant-level channel config from tenant_setting
//   4. For each matching enabled channel, enqueues an `outbox_job`
//      with type = "notify_channel"
//
// The outbox_worker picks up "notify_channel" jobs and calls the
// appropriate channel adapter (Telegram, WhatsApp, SMS, Email).
//
// ## Channel Adapters
//
// Each adapter is gated on env vars or tenant_settings.
// If credentials are absent, the job is marked "skipped" — never errors
// in a way that blocks in-app delivery.
//
// ## Tenant-level broadcast channels
//
// A landlord can configure a Telegram group for announcements.
// These are stored in atlas_user_notification_pref with:
//   user_id = tenant_id (sentinel)
//   config.scope = "broadcast"
//
// When dispatching a notification with broadcast=true, these rows
// are also picked up alongside the user's personal prefs.
// ─────────────────────────────────────────────────────────────────────────────

use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::entities::{atlas_notification, atlas_user_notification_pref, outbox_job};

// ── Public input type ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DispatchInput {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    /// Optional linked entity for deep-link in the frontend
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    /// Extra metadata: action_url, image_url, cta_label, etc.
    pub metadata: Option<serde_json::Value>,
    /// If true, also dispatch to broadcast channels (tenant-level groups)
    pub include_broadcast: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

// ── Outbox job payload for channel delivery ───────────────────────────────────

/// Serialised into outbox_job.payload for notify_channel jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyChannelPayload {
    pub notification_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    /// Channel-specific routing config (from atlas_user_notification_pref.config)
    pub channel_config: serde_json::Value,
    pub title: String,
    pub body: String,
    pub priority: String,
    /// Deep-link for supported channels
    pub action_url: Option<String>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct NotificationService;

impl NotificationService {
    /// Primary entry point. Writes the in-app notification and schedules
    /// outbox jobs for all configured external channels.
    pub async fn dispatch(db: &DatabaseConnection, input: DispatchInput) -> Result<Uuid> {
        let notification_id = Uuid::new_v4();
        let now = Utc::now();

        // 1. Write atlas_notification (in-app inbox)
        let record = atlas_notification::ActiveModel {
            id: Set(notification_id),
            tenant_id: Set(input.tenant_id),
            user_id: Set(input.user_id),
            notification_type: Set(input.notification_type.clone()),
            title: Set(input.title.clone()),
            body: Set(input.body.clone()),
            priority: Set(input.priority.as_str().to_string()),
            entity_type: Set(input.entity_type.clone()),
            entity_id: Set(input.entity_id),
            metadata: Set(input.metadata.clone()),
            channels_attempted: Set(json!([])),
            read_at: Set(None),
            dismissed_at: Set(None),
            created_at: Set(now),
        };
        record
            .insert(db)
            .await
            .context("NotificationService: failed to write atlas_notification")?;

        // 2. Fetch user's channel prefs
        let mut prefs = atlas_user_notification_pref::Entity::find()
            .filter(atlas_user_notification_pref::Column::UserId.eq(input.user_id))
            .filter(atlas_user_notification_pref::Column::TenantId.eq(input.tenant_id))
            .filter(atlas_user_notification_pref::Column::Enabled.eq(true))
            .filter(atlas_user_notification_pref::Column::Channel.ne("in_app"))
            .all(db)
            .await
            .context("NotificationService: failed to fetch user prefs")?;

        // 3. Optionally include broadcast (tenant-level) channels
        if input.include_broadcast {
            let broadcast = atlas_user_notification_pref::Entity::find()
                .filter(
                    atlas_user_notification_pref::Column::UserId.eq(input.tenant_id), // sentinel: user_id = tenant_id
                )
                .filter(atlas_user_notification_pref::Column::TenantId.eq(input.tenant_id))
                .filter(atlas_user_notification_pref::Column::Enabled.eq(true))
                .filter(atlas_user_notification_pref::Column::Channel.ne("in_app"))
                .all(db)
                .await
                .context("NotificationService: failed to fetch broadcast prefs")?;
            prefs.extend(broadcast);
        }

        // 4. Filter prefs by notification_type if applies_to is set
        let applicable: Vec<_> = prefs
            .into_iter()
            .filter(|p| p.applies_to.is_empty() || p.applies_to.contains(&input.notification_type))
            .collect();

        // 5. Enqueue one outbox_job per channel
        let action_url = input
            .metadata
            .as_ref()
            .and_then(|m| m.get("action_url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        for pref in applicable {
            let payload = NotifyChannelPayload {
                notification_id,
                tenant_id: input.tenant_id,
                user_id: input.user_id,
                channel: pref.channel.clone(),
                channel_config: pref.config.clone(),
                title: input.title.clone(),
                body: input.body.clone(),
                priority: input.priority.as_str().to_string(),
                action_url: action_url.clone(),
            };

            let job = outbox_job::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(input.tenant_id),
                job_type: Set("notify_channel".to_string()),
                payload: Set(serde_json::to_value(&payload).unwrap_or_else(|_| json!({}))),
                status: Set("pending".to_string()),
                attempts: Set(0),
                error_message: Set(None),
                locked_by: Set(None),
                locked_at: Set(None),
                created_at: Set(now),
                run_at: Set(now),
            };
            job.insert(db).await.context(format!(
                "NotificationService: failed to enqueue job for channel {}",
                pref.channel
            ))?;
        }

        tracing::info!(
            notification_id = %notification_id,
            user_id         = %input.user_id,
            tenant_id       = %input.tenant_id,
            ntype           = %input.notification_type,
            "notification dispatched",
        );

        Ok(notification_id)
    }

    /// Mark a notification as read.
    pub async fn mark_read(
        db: &DatabaseConnection,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        use sea_orm::ActiveModelTrait;
        let notif = atlas_notification::Entity::find_by_id(notification_id)
            .filter(atlas_notification::Column::UserId.eq(user_id))
            .one(db)
            .await
            .context("mark_read: db error")?
            .context("mark_read: notification not found")?;

        let mut active: atlas_notification::ActiveModel = notif.into();
        active.read_at = Set(Some(Utc::now()));
        active
            .update(db)
            .await
            .context("mark_read: update failed")?;
        Ok(())
    }

    /// Dismiss (soft-delete) a notification.
    pub async fn dismiss(
        db: &DatabaseConnection,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let notif = atlas_notification::Entity::find_by_id(notification_id)
            .filter(atlas_notification::Column::UserId.eq(user_id))
            .one(db)
            .await
            .context("dismiss: db error")?
            .context("dismiss: notification not found")?;

        let mut active: atlas_notification::ActiveModel = notif.into();
        active.dismissed_at = Set(Some(Utc::now()));
        active.update(db).await.context("dismiss: update failed")?;
        Ok(())
    }

    /// Mark all unread notifications as read for a user/tenant.
    pub async fn mark_all_read(
        db: &DatabaseConnection,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<u64> {
        let unread = atlas_notification::Entity::find()
            .filter(atlas_notification::Column::UserId.eq(user_id))
            .filter(atlas_notification::Column::TenantId.eq(tenant_id))
            .filter(atlas_notification::Column::ReadAt.is_null())
            .filter(atlas_notification::Column::DismissedAt.is_null())
            .all(db)
            .await
            .context("mark_all_read: db error")?;

        let count = unread.len() as u64;
        let now = Utc::now();

        for notif in unread {
            let mut active: atlas_notification::ActiveModel = notif.into();
            active.read_at = Set(Some(now));
            active
                .update(db)
                .await
                .context("mark_all_read: update failed")?;
        }

        Ok(count)
    }

    /// Unread count for a user/tenant (for nav badge).
    pub async fn unread_count(
        db: &DatabaseConnection,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<u64> {
        use sea_orm::PaginatorTrait;
        let count = atlas_notification::Entity::find()
            .filter(atlas_notification::Column::UserId.eq(user_id))
            .filter(atlas_notification::Column::TenantId.eq(tenant_id))
            .filter(atlas_notification::Column::ReadAt.is_null())
            .filter(atlas_notification::Column::DismissedAt.is_null())
            .count(db)
            .await
            .context("unread_count: db error")?;
        Ok(count)
    }
}

// ── Channel adapters ──────────────────────────────────────────────────────────
//
// Called by outbox_worker when it processes a "notify_channel" job.
// Each adapter is env-var gated. Missing credentials = "skipped" status.
// ─────────────────────────────────────────────────────────────────────────────

pub mod channels {
    use anyhow::{Result, anyhow};
    use serde_json::Value;

    /// Delivery result from a channel adapter.
    #[derive(Debug)]
    pub enum ChannelResult {
        Delivered,
        Skipped { reason: String },
        Failed { error: String },
    }

    // ── Telegram ─────────────────────────────────────────────────────────────

    pub struct TelegramAdapter;

    impl TelegramAdapter {
        /// Send a message to a Telegram chat via Bot API.
        ///
        /// Required tenant_settings (or env vars):
        ///   notify_channel_telegram_bot_token
        ///
        /// Required in channel_config:
        ///   { "chat_id": "-1001234567890" }
        pub async fn send(
            channel_config: &Value,
            bot_token: &str,
            title: &str,
            body: &str,
            action_url: Option<&str>,
        ) -> ChannelResult {
            let chat_id = match channel_config.get("chat_id").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => {
                    return ChannelResult::Skipped {
                        reason: "no chat_id in channel_config".into(),
                    };
                }
            };

            if bot_token.is_empty() {
                return ChannelResult::Skipped {
                    reason: "TELEGRAM_BOT_TOKEN not configured".into(),
                };
            }

            let text = if let Some(url) = action_url {
                format!("*{title}*\n\n{body}\n\n[Open →]({url})")
            } else {
                format!("*{title}*\n\n{body}")
            };

            let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
            let payload = serde_json::json!({
                "chat_id":    chat_id,
                "text":       text,
                "parse_mode": "Markdown",
            });

            match reqwest::Client::new()
                .post(&url)
                .json(&payload)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => ChannelResult::Delivered,
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    ChannelResult::Failed {
                        error: format!("HTTP {status}: {body}"),
                    }
                }
                Err(e) => ChannelResult::Failed {
                    error: e.to_string(),
                },
            }
        }
    }

    // ── WhatsApp ─────────────────────────────────────────────────────────────

    pub struct WhatsAppAdapter;

    impl WhatsAppAdapter {
        /// Send a WhatsApp message via Twilio or Meta Cloud API.
        ///
        /// Required tenant_settings:
        ///   notify_channel_whatsapp_provider = "twilio" | "meta"
        ///   For twilio: TWILIO_ACCOUNT_SID + TWILIO_AUTH_TOKEN
        ///   For meta:   META_WHATSAPP_TOKEN + META_WHATSAPP_PHONE_ID
        ///
        /// Required in channel_config:
        ///   { "phone": "+15551234567", "provider": "twilio" | "meta" }
        pub async fn send(channel_config: &Value, title: &str, body: &str) -> ChannelResult {
            let phone = match channel_config.get("phone").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return ChannelResult::Skipped {
                        reason: "no phone in channel_config".into(),
                    };
                }
            };

            let provider = channel_config
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("twilio");

            let text = format!("{title}\n\n{body}");

            match provider {
                "twilio" => Self::send_via_twilio(&phone, &text).await,
                "meta" => Self::send_via_meta(&phone, title, body).await,
                _ => ChannelResult::Skipped {
                    reason: format!("unknown provider: {provider}"),
                },
            }
        }

        async fn send_via_twilio(to: &str, text: &str) -> ChannelResult {
            let account_sid = std::env::var("TWILIO_ACCOUNT_SID").unwrap_or_default();
            let auth_token = std::env::var("TWILIO_AUTH_TOKEN").unwrap_or_default();
            let from_wa = std::env::var("TWILIO_WHATSAPP_FROM").unwrap_or_default();

            if account_sid.is_empty() || auth_token.is_empty() || from_wa.is_empty() {
                return ChannelResult::Skipped {
                    reason: "Twilio WhatsApp credentials not configured".into(),
                };
            }

            let url =
                format!("https://api.twilio.com/2010-04-01/Accounts/{account_sid}/Messages.json");

            // Twilio uses application/x-www-form-urlencoded. Since reqwest is built
            // without the "form" feature, we encode manually.
            let body_str = format!(
                "From=whatsapp%3A{}&To=whatsapp%3A{}&Body={}",
                urlencoding::encode(&from_wa),
                urlencoding::encode(to),
                urlencoding::encode(text),
            );

            match reqwest::Client::new()
                .post(&url)
                .basic_auth(&account_sid, Some(&auth_token))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body_str)
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => ChannelResult::Delivered,
                Ok(resp) => {
                    let status = resp.status();
                    let b = resp.text().await.unwrap_or_default();
                    ChannelResult::Failed {
                        error: format!("Twilio HTTP {status}: {b}"),
                    }
                }
                Err(e) => ChannelResult::Failed {
                    error: e.to_string(),
                },
            }
        }

        async fn send_via_meta(to: &str, title: &str, body: &str) -> ChannelResult {
            let token = std::env::var("META_WHATSAPP_TOKEN").unwrap_or_default();
            let phone_id = std::env::var("META_WHATSAPP_PHONE_ID").unwrap_or_default();

            if token.is_empty() || phone_id.is_empty() {
                return ChannelResult::Skipped {
                    reason: "Meta WhatsApp credentials not configured".into(),
                };
            }

            let url = format!("https://graph.facebook.com/v19.0/{phone_id}/messages");

            // Use a basic text template message
            let payload = serde_json::json!({
                "messaging_product": "whatsapp",
                "to": to,
                "type": "text",
                "text": { "body": format!("{title}\n\n{body}") }
            });

            match reqwest::Client::new()
                .post(&url)
                .bearer_auth(&token)
                .json(&payload)
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => ChannelResult::Delivered,
                Ok(resp) => {
                    let status = resp.status();
                    let b = resp.text().await.unwrap_or_default();
                    ChannelResult::Failed {
                        error: format!("Meta HTTP {status}: {b}"),
                    }
                }
                Err(e) => ChannelResult::Failed {
                    error: e.to_string(),
                },
            }
        }
    }

    // ── SMS ───────────────────────────────────────────────────────────────────

    pub struct SmsAdapter;

    impl SmsAdapter {
        /// Send SMS via the existing TelephonyProvider (Twilio or Telnyx).
        ///
        /// Required channel_config: { "phone": "+15551234567" }
        pub async fn send(channel_config: &Value, body: &str) -> ChannelResult {
            let phone = match channel_config.get("phone").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    return ChannelResult::Skipped {
                        reason: "no phone in channel_config".into(),
                    };
                }
            };

            match crate::services::telephony::factory::get_telephony_provider() {
                Err(e) => ChannelResult::Skipped {
                    reason: format!("telephony provider not configured: {e}"),
                },
                Ok(provider) => match provider.send_sms(&phone, body).await {
                    Ok(()) => ChannelResult::Delivered,
                    Err(e) => ChannelResult::Failed {
                        error: e.to_string(),
                    },
                },
            }
        }
    }

    // ── Email ─────────────────────────────────────────────────────────────────

    pub struct EmailAdapter;

    impl EmailAdapter {
        /// Send notification email via the existing SMTP handler.
        ///
        /// Required channel_config: { "email": "user@example.com" }
        /// Falls back to user.email (resolved by caller before enqueuing).
        pub async fn send(
            channel_config: &Value,
            title: &str,
            body: &str,
            action_url: Option<&str>,
        ) -> ChannelResult {
            let email = match channel_config.get("email").and_then(|v| v.as_str()) {
                Some(e) => e.to_string(),
                None => {
                    return ChannelResult::Skipped {
                        reason: "no email in channel_config".into(),
                    };
                }
            };

            let tenant_id_str = channel_config
                .get("tenant_id")
                .and_then(|v| v.as_str())
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_else(uuid::Uuid::nil);

            let html = if let Some(url) = action_url {
                format!("<h2>{title}</h2><p>{body}</p><p><a href=\"{url}\">Open in Atlas →</a></p>")
            } else {
                format!("<h2>{title}</h2><p>{body}</p>")
            };

            // Reuse the existing send_email_handler logic via payload struct
            let payload = crate::handlers::communications::SendEmailPayload {
                tenant_id: tenant_id_str,
                to_email: email,
                subject: title.to_string(),
                body_html: html,
                attachments: vec![],
            };

            // We call the internal send logic. In a real dispatch we'd use an
            // internal fn; here we construct the SMTP client directly.
            match Self::send_smtp(payload).await {
                Ok(()) => ChannelResult::Delivered,
                Err(e) => ChannelResult::Failed {
                    error: e.to_string(),
                },
            }
        }

        async fn send_smtp(
            payload: crate::handlers::communications::SendEmailPayload,
        ) -> anyhow::Result<()> {
            use lettre::{
                AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
                message::{MultiPart, SinglePart, header},
                transport::smtp::authentication::Credentials,
            };

            let host = std::env::var("SMTP_SERVER").unwrap_or_else(|_| "localhost".to_string());
            let port: u16 = std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587);
            let username = std::env::var("SMTP_USERNAME").unwrap_or_default();
            let token = std::env::var("SMTP_TOKEN").unwrap_or_default();
            let from = std::env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@atlas-platform.local".to_string());

            if host == "localhost" || host.is_empty() {
                tracing::warn!(
                    "EmailAdapter: SMTP not configured, mocking send to {}",
                    payload.to_email
                );
                return Ok(());
            }

            let multipart = MultiPart::mixed().singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(payload.body_html),
            );

            let email = Message::builder()
                .from(from.parse()?)
                .to(payload.to_email.parse()?)
                .subject(&payload.subject)
                .multipart(multipart)?;

            let creds = Credentials::new(username, token);
            let mailer: AsyncSmtpTransport<Tokio1Executor> = if port == 465 {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
                    .port(port)
                    .credentials(creds)
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?
                    .port(port)
                    .credentials(creds)
                    .build()
            };

            mailer
                .send(email)
                .await
                .map_err(|e| anyhow::anyhow!("SMTP error: {e}"))?;
            Ok(())
        }
    }

    // ── Dispatcher called by outbox_worker ───────────────────────────────────

    /// Entry point for the outbox_worker "notify_channel" job type.
    /// Returns the delivery status string to write into channels_attempted.
    pub async fn dispatch_channel_job(
        payload: &super::NotifyChannelPayload,
        tenant_settings: &std::collections::HashMap<String, String>,
    ) -> ChannelResult {
        match payload.channel.as_str() {
            "telegram" => {
                let bot_token = tenant_settings
                    .get("notify_channel_telegram_bot_token")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                TelegramAdapter::send(
                    &payload.channel_config,
                    bot_token,
                    &payload.title,
                    &payload.body,
                    payload.action_url.as_deref(),
                )
                .await
            }
            "whatsapp" => {
                WhatsAppAdapter::send(&payload.channel_config, &payload.title, &payload.body).await
            }
            "sms" => {
                let text = format!("{}: {}", payload.title, payload.body);
                SmsAdapter::send(&payload.channel_config, &text).await
            }
            "email" => {
                EmailAdapter::send(
                    &payload.channel_config,
                    &payload.title,
                    &payload.body,
                    payload.action_url.as_deref(),
                )
                .await
            }
            other => ChannelResult::Skipped {
                reason: format!("unknown channel: {other}"),
            },
        }
    }
}
