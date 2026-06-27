// apps/folio/src/pages/landlord/notifications.rs
//
// Route: /l/notifications
// Role:  All authenticated Folio users (shared)
//
// Two-tab layout:
//   Tab 1 — "Inbox"        : paginated notification list, mark-read, dismiss
//   Tab 2 — "Channels"     : per-user channel opt-in prefs + tenant channel creds
//
// Notification types rendered with icons and priority badges.
// Unread notifications show a bold left-border indicator.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRow {
    pub id:                 String,
    pub notification_type:  String,
    pub title:              String,
    pub body:               String,
    pub priority:           String,
    pub entity_type:        Option<String>,
    pub metadata:           Option<serde_json::Value>,
    pub channels_attempted: serde_json::Value,
    pub read_at:            Option<String>,
    pub created_at:         String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefRow {
    pub id:         String,
    pub channel:    String,
    pub config:     serde_json::Value,
    pub enabled:    bool,
    pub applies_to: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSettingRow {
    pub key:       String,
    pub value:     String,
    pub is_set:    bool,
    pub is_masked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnreadCountResponse {
    count: u64,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

#[server(FetchNotifications, "/api")]
pub async fn fetch_notifications(
    page: u64,
    unread_only: bool,
) -> Result<Vec<NotificationRow>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let limit  = 30u64;
    let offset = page * limit;
    let url = format!(
        "/api/folio/notifications?limit={limit}&offset={offset}&unread={unread_only}"
    );
    crate::atlas_client::authenticated_get::<Vec<NotificationRow>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e))
}

#[server(FetchUnreadCount, "/api")]
pub async fn fetch_unread_count() -> Result<u64, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let resp = crate::atlas_client::authenticated_get::<UnreadCountResponse>(
        "/api/folio/notifications/unread-count", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))?;
    Ok(resp.count)
}

#[server(MarkRead, "/api")]
pub async fn mark_read(id: String) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &format!("/api/folio/notifications/{id}/read"),
        &token, None,
        &serde_json::json!({}),
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))?;
    Ok(())
}

#[server(MarkAllRead, "/api")]
pub async fn mark_all_read() -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        "/api/folio/notifications/read-all",
        &token, None,
        &serde_json::json!({}),
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))?;
    Ok(())
}

#[server(DismissNotification, "/api")]
pub async fn dismiss_notification(id: String) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_delete(
        &format!("/api/folio/notifications/{id}"),
        &token,
        None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))?;
    Ok(())
}

#[server(FetchPrefs, "/api")]
pub async fn fetch_prefs() -> Result<Vec<PrefRow>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<PrefRow>>(
        "/api/folio/notification-prefs", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))
}

#[server(UpsertPref, "/api")]
pub async fn upsert_pref(
    channel:    String,
    config:     String,
    enabled:    bool,
    applies_to: Vec<String>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let config_val: serde_json::Value = serde_json::from_str(&config)
        .unwrap_or_else(|_| serde_json::json!({}));
    crate::atlas_client::authenticated_put::<serde_json::Value, ()>(
        &format!("/api/folio/notification-prefs/{channel}"),
        &token, None,
        &serde_json::json!({ "config": config_val, "enabled": enabled, "applies_to": applies_to }),
    ).await.ok();
    Ok(())
}

#[server(DeletePref, "/api")]
pub async fn delete_pref(channel: String) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_delete(
        &format!("/api/folio/notification-prefs/{channel}"),
        &token,
        None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))?;
    Ok(())
}

#[server(FetchChannelSettings, "/api")]
pub async fn fetch_channel_settings() -> Result<Vec<ChannelSettingRow>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ChannelSettingRow>>(
        "/api/folio/notification-channel-settings", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e))
}

#[server(UpsertChannelSetting, "/api")]
pub async fn upsert_channel_setting(
    key:   String,
    value: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_put::<serde_json::Value, ()>(
        "/api/folio/notification-channel-settings",
        &token, None,
        &serde_json::json!({ "key": key, "value": value }),
    ).await.ok();
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn notif_icon(ntype: &str) -> &'static str {
    match ntype {
        "rent_due"             => "payments",
        "lease_expiring"       => "calendar_today",
        "maintenance_request"  => "build",
        "message_received"     => "chat_bubble",
        "violation_filed"      => "gavel",
        "inspection_scheduled" => "fact_check",
        "payment_received"     => "price_check",
        "lead_assigned"        => "person_add",
        "scorecard_nudge"      => "insights",
        "announcement"         => "campaign",
        "system"               => "notifications",
        _                      => "circle_notifications",
    }
}

fn priority_badge(p: &str) -> (&'static str, &'static str) {
    match p {
        "urgent" => ("notif-badge notif-badge--urgent", "Urgent"),
        "high"   => ("notif-badge notif-badge--high",   "High"),
        "low"    => ("notif-badge notif-badge--low",    "Low"),
        _        => ("", ""),
    }
}

fn channel_icon(ch: &str) -> &'static str {
    match ch {
        "telegram"  => "send",
        "whatsapp"  => "chat",
        "sms"       => "sms",
        "email"     => "mail",
        "in_app"    => "notifications",
        _           => "device_unknown",
    }
}

fn channel_label(ch: &str) -> &'static str {
    match ch {
        "telegram"  => "Telegram",
        "whatsapp"  => "WhatsApp",
        "sms"       => "SMS",
        "email"     => "Email",
        "in_app"    => "In-App",
        _           => "Unknown",
    }
}

// ── Main page component ───────────────────────────────────────────────────────

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let active_tab    = RwSignal::new(0usize); // 0 = Inbox, 1 = Channels
    let unread_only   = RwSignal::new(false);
    let refetch_inbox = RwSignal::new(0u32);

    let notifications = Resource::new(
        move || (refetch_inbox.get(), unread_only.get()),
        move |(_, unread)| fetch_notifications(0, unread),
    );

    let unread_count = Resource::new(
        move || refetch_inbox.get(),
        |_| fetch_unread_count(),
    );

    let handle_mark_all = {
        let refetch_inbox = refetch_inbox;
        move |_| {
            spawn_local(async move {
                let _ = mark_all_read().await;
                refetch_inbox.update(|n| *n += 1);
            });
        }
    };

    view! {
        <div class="notif-page">
            // ── Page header ──────────────────────────────────────────────────
            <div class="notif-header">
                <div class="notif-header-left">
                    <h1 class="notif-title">"Notifications"</h1>
                    <p class="notif-subtitle">
                        "Stay updated — all alerts, announcements and channel preferences."
                    </p>
                </div>
                <div class="notif-header-actions">
                    // Unread badge
                    <Suspense fallback=|| ()>
                        {move || unread_count.get().map(|r| r.unwrap_or(0)).map(|cnt| {
                            if cnt > 0 {
                                view! {
                                    <span class="notif-unread-badge">
                                        {cnt.to_string()} " unread"
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        })}
                    </Suspense>
                    <button class="notif-action-btn" on:click=handle_mark_all>
                        <span class="material-symbols-outlined">"done_all"</span>
                        "Mark all read"
                    </button>
                </div>
            </div>

            // ── Tabs ─────────────────────────────────────────────────────────
            <div class="notif-tabs">
                <button
                    class=move || if active_tab.get() == 0 { "notif-tab notif-tab--active" } else { "notif-tab" }
                    on:click=move |_| active_tab.set(0)>
                    <span class="material-symbols-outlined">"inbox"</span>
                    "Inbox"
                </button>
                <button
                    class=move || if active_tab.get() == 1 { "notif-tab notif-tab--active" } else { "notif-tab" }
                    on:click=move |_| active_tab.set(1)>
                    <span class="material-symbols-outlined">"tune"</span>
                    "Channel Preferences"
                </button>
            </div>

            // ── Tab content ───────────────────────────────────────────────────
            {move || match active_tab.get() {
                0 => view! {
                    <InboxTab
                        notifications=notifications
                        unread_only=unread_only
                        refetch=refetch_inbox
                    />
                }.into_any(),
                _ => view! {
                    <ChannelsTab />
                }.into_any(),
            }}
        </div>
    }
}

// ── Inbox Tab ─────────────────────────────────────────────────────────────────

#[component]
fn InboxTab(
    notifications: Resource<Result<Vec<NotificationRow>, server_fn::error::ServerFnError>>,
    unread_only:   RwSignal<bool>,
    refetch:       RwSignal<u32>,
) -> impl IntoView {
    view! {
        <div class="notif-inbox">
            // Filter strip
            <div class="notif-filter-strip">
                <label class="notif-toggle">
                    <input
                        type="checkbox"
                        prop:checked=move || unread_only.get()
                        on:change=move |e| {
                            let checked = event_target_checked(&e);
                            unread_only.set(checked);
                        }
                    />
                    <span>"Unread only"</span>
                </label>
                <span class="notif-filter-hint">"Showing last 30 notifications"</span>
            </div>

            // Notification list
            <Suspense fallback=|| view! {
                <div class="notif-skel-list">
                    {(0..6).map(|_| view! { <div class="notif-skel"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || notifications.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="notif-empty">
                            <span class="material-symbols-outlined">"error_outline"</span>
                            <p>"Failed to load notifications: " {e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <div class="notif-empty">
                            <span class="material-symbols-outlined notif-empty-icon">"notifications_none"</span>
                            <p class="notif-empty-title">"All clear!"</p>
                            <p class="notif-empty-sub">"You have no notifications right now."</p>
                        </div>
                    }.into_any(),
                    Ok(list) => view! {
                        <div class="notif-list">
                            {list.into_iter().map(|notif| {
                                view! { <NotifCard notif=notif refetch=refetch /> }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// ── Notification Card ─────────────────────────────────────────────────────────

#[component]
fn NotifCard(notif: NotificationRow, refetch: RwSignal<u32>) -> impl IntoView {
    let id         = notif.id.clone();
    let id2        = id.clone();
    let is_unread  = notif.read_at.is_none();
    let icon       = notif_icon(&notif.notification_type).to_string();
    let (badge_cls, badge_label) = priority_badge(&notif.priority);
    let badge_cls  = badge_cls.to_string();
    let badge_label = badge_label.to_string();

    // Delivery channel chips
    let channels: Vec<String> = notif.channels_attempted
        .as_array()
        .map(|arr| arr.iter()
            .filter_map(|v| v.get("channel").and_then(|c| c.as_str()).map(|s| s.to_string()))
            .collect())
        .unwrap_or_default();

    let handle_read = move |_| {
        let id = id.clone();
        spawn_local(async move {
            let _ = mark_read(id).await;
            refetch.update(|n| *n += 1);
        });
    };

    let handle_dismiss = move |_| {
        let id = id2.clone();
        spawn_local(async move {
            let _ = dismiss_notification(id).await;
            refetch.update(|n| *n += 1);
        });
    };

    view! {
        <div class=if is_unread { "notif-card notif-card--unread" } else { "notif-card" }>
            <div class="notif-card-icon-col">
                <span class="material-symbols-outlined notif-card-icon">{icon}</span>
                {if is_unread { view! { <div class="notif-unread-dot"></div> }.into_any() } else { view! {<span></span>}.into_any() }}
            </div>
            <div class="notif-card-body">
                <div class="notif-card-top">
                    <span class="notif-card-title">{notif.title.clone()}</span>
                    {if !badge_cls.is_empty() {
                        view! { <span class=badge_cls>{badge_label}</span> }.into_any()
                    } else { view! {<span></span>}.into_any() }}
                </div>
                <p class="notif-card-body-text">{notif.body.clone()}</p>
                <div class="notif-card-footer">
                    <span class="notif-card-time">{notif.created_at.clone()}</span>
                    // Delivery chips
                    {channels.into_iter().map(|ch| {
                        let icon = channel_icon(&ch).to_string();
                        view! {
                            <span class="notif-delivery-chip">
                                <span class="material-symbols-outlined">{icon}</span>
                                {channel_label(&ch)}
                            </span>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            <div class="notif-card-actions">
                {if is_unread {
                    view! {
                        <button class="notif-icon-btn" title="Mark read" on:click=handle_read>
                            <span class="material-symbols-outlined">"check"</span>
                        </button>
                    }.into_any()
                } else { view! {<span></span>}.into_any() }}
                <button class="notif-icon-btn notif-icon-btn--dismiss" title="Dismiss" on:click=handle_dismiss>
                    <span class="material-symbols-outlined">"close"</span>
                </button>
            </div>
        </div>
    }
}

// ── Channels Tab ──────────────────────────────────────────────────────────────

#[component]
fn ChannelsTab() -> impl IntoView {
    let refetch_prefs    = RwSignal::new(0u32);
    let refetch_settings = RwSignal::new(0u32);

    let prefs = Resource::new(
        move || refetch_prefs.get(),
        |_| fetch_prefs(),
    );
    let settings = Resource::new(
        move || refetch_settings.get(),
        |_| fetch_channel_settings(),
    );

    // New pref form state
    let new_channel  = RwSignal::new("telegram".to_string());
    let new_config   = RwSignal::new("{}".to_string());
    let new_enabled  = RwSignal::new(true);
    let saving_pref  = RwSignal::new(false);
    let pref_err     = RwSignal::<Option<String>>::new(None);

    // Setting form state
    let setting_key   = RwSignal::new("notify_channel_telegram_bot_token".to_string());
    let setting_value = RwSignal::new(String::new());
    let saving_setting = RwSignal::new(false);

    let save_pref = move |_| {
        let channel   = new_channel.get_untracked();
        let config    = new_config.get_untracked();
        let enabled   = new_enabled.get_untracked();
        pref_err.set(None);
        saving_pref.set(true);
        spawn_local(async move {
            match upsert_pref(channel, config, enabled, vec![]).await {
                Ok(()) => { refetch_prefs.update(|n| *n += 1); }
                Err(e) => { pref_err.set(Some(e.to_string())); }
            }
            saving_pref.set(false);
        });
    };

    let save_setting = move |_| {
        let key   = setting_key.get_untracked();
        let value = setting_value.get_untracked();
        saving_setting.set(true);
        spawn_local(async move {
            let _ = upsert_channel_setting(key, value).await;
            refetch_settings.update(|n| *n += 1);
            saving_setting.set(false);
        });
    };

    view! {
        <div class="notif-channels">

            // ── Section 1: My channel opt-ins ─────────────────────────────────
            <section class="notif-section">
                <div class="notif-section-header">
                    <h2 class="notif-section-title">"My Channel Opt-ins"</h2>
                    <p class="notif-section-sub">
                        "Choose which channels receive your personal notifications. \
                         Landlords can also configure group broadcast channels below."
                    </p>
                </div>

                <Suspense fallback=|| view! {
                    <div class="notif-pref-skel-list">
                        {(0..3).map(|_| view! { <div class="notif-pref-skel"></div> }).collect::<Vec<_>>()}
                    </div>
                }>
                    {move || prefs.get().map(|res| match res {
                        Err(_) => view! { <p class="notif-err">"Failed to load preferences"</p> }.into_any(),
                        Ok(list) if list.is_empty() => view! {
                            <p class="notif-no-prefs">"No channels configured yet. Add one below."</p>
                        }.into_any(),
                        Ok(list) => view! {
                            <div class="notif-pref-list">
                                {list.into_iter().map(|pref| {
                                    view! { <PrefCard pref=pref refetch=refetch_prefs /> }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                    })}
                </Suspense>

                // Add / edit pref form
                <div class="notif-form-card">
                    <h3 class="notif-form-title">"Add / Update Channel"</h3>
                    <div class="notif-form-grid">
                        <div class="notif-form-field">
                            <label class="notif-label">"Channel"</label>
                            <select
                                class="notif-select"
                                on:change=move |e| new_channel.set(event_target_value(&e))>
                                <option value="telegram">"Telegram"</option>
                                <option value="whatsapp">"WhatsApp"</option>
                                <option value="sms">"SMS"</option>
                                <option value="email">"Email"</option>
                            </select>
                        </div>
                        <div class="notif-form-field notif-form-field--wide">
                            <label class="notif-label">
                                "Config (JSON) "
                                <span class="notif-label-hint">
                                    "e.g. {\"chat_id\": \"...\"}  or  {\"phone\": \"+1...\"}  or  {\"email\": \"...\"}"
                                </span>
                            </label>
                            <textarea
                                class="notif-textarea"
                                rows="3"
                                prop:value=move || new_config.get()
                                on:input=move |e| new_config.set(event_target_value(&e))>
                            </textarea>
                        </div>
                        <div class="notif-form-field notif-form-field--inline">
                            <label class="notif-toggle">
                                <input
                                    type="checkbox"
                                    prop:checked=move || new_enabled.get()
                                    on:change=move |e| new_enabled.set(event_target_checked(&e))
                                />
                                <span>"Enabled"</span>
                            </label>
                        </div>
                    </div>
                    {move || pref_err.get().map(|e| view! {
                        <p class="notif-err">{e}</p>
                    })}
                    <button
                        class="notif-save-btn"
                        prop:disabled=move || saving_pref.get()
                        on:click=save_pref>
                        {move || if saving_pref.get() { "Saving…" } else { "Save Channel" }}
                    </button>
                </div>
            </section>

            // ── Section 2: Tenant channel credentials ─────────────────────────
            <section class="notif-section">
                <div class="notif-section-header">
                    <h2 class="notif-section-title">"Platform Channel Credentials"</h2>
                    <p class="notif-section-sub">
                        "Set API keys and credentials used by Atlas to send notifications. \
                         Sensitive fields are masked once saved. These apply to all users in your workspace."
                    </p>
                </div>

                <Suspense fallback=|| view! {
                    <div class="notif-pref-skel-list">
                        {(0..4).map(|_| view! { <div class="notif-pref-skel"></div> }).collect::<Vec<_>>()}
                    </div>
                }>
                    {move || settings.get().map(|res| match res {
                        Err(_) => view! { <p class="notif-err">"Failed to load settings"</p> }.into_any(),
                        Ok(list) => view! {
                            <div class="notif-settings-grid">
                                {list.into_iter().map(|s| {
                                    view! { <SettingRow row=s refetch=refetch_settings /> }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                    })}
                </Suspense>

                // Quick-set form
                <div class="notif-form-card">
                    <h3 class="notif-form-title">"Set Credential"</h3>
                    <div class="notif-form-grid">
                        <div class="notif-form-field">
                            <label class="notif-label">"Setting Key"</label>
                            <select
                                class="notif-select"
                                on:change=move |e| setting_key.set(event_target_value(&e))>
                                <option value="notify_channel_telegram_bot_token">"Telegram Bot Token"</option>
                                <option value="notify_channel_telegram_default_chat_id">"Telegram Default Chat ID"</option>
                                <option value="notify_channel_telegram_enabled">"Telegram Enabled (true/false)"</option>
                                <option value="notify_channel_whatsapp_enabled">"WhatsApp Enabled"</option>
                                <option value="notify_channel_whatsapp_provider">"WhatsApp Provider (twilio/meta)"</option>
                                <option value="notify_channel_whatsapp_from">"WhatsApp From Number"</option>
                                <option value="notify_channel_sms_enabled">"SMS Enabled"</option>
                                <option value="notify_channel_sms_from">"SMS From Number"</option>
                                <option value="notify_channel_email_enabled">"Email Enabled"</option>
                            </select>
                        </div>
                        <div class="notif-form-field notif-form-field--wide">
                            <label class="notif-label">"Value"</label>
                            <input
                                type="text"
                                class="notif-input"
                                placeholder="Enter value…"
                                prop:value=move || setting_value.get()
                                on:input=move |e| setting_value.set(event_target_value(&e))
                            />
                        </div>
                    </div>
                    <button
                        class="notif-save-btn"
                        prop:disabled=move || saving_setting.get()
                        on:click=save_setting>
                        {move || if saving_setting.get() { "Saving…" } else { "Save Setting" }}
                    </button>
                </div>
            </section>
        </div>
    }
}

// ── Pref Card ──────────────────────────────────────────────────────────────────

#[component]
fn PrefCard(pref: PrefRow, refetch: RwSignal<u32>) -> impl IntoView {
    let channel = pref.channel.clone();
    let channel2 = channel.clone();
    let icon    = channel_icon(&channel).to_string();
    let label   = channel_label(&channel).to_string();

    let config_str = serde_json::to_string_pretty(&pref.config)
        .unwrap_or_else(|_| "{}".to_string());

    let handle_delete = move |_| {
        let ch = channel2.clone();
        spawn_local(async move {
            let _ = delete_pref(ch).await;
            refetch.update(|n| *n += 1);
        });
    };

    view! {
        <div class="notif-pref-card">
            <div class="notif-pref-icon">
                <span class="material-symbols-outlined">{icon}</span>
            </div>
            <div class="notif-pref-body">
                <div class="notif-pref-top">
                    <span class="notif-pref-label">{label}</span>
                    {if pref.enabled {
                        view! { <span class="notif-pref-chip notif-pref-chip--on">"Enabled"</span> }.into_any()
                    } else {
                        view! { <span class="notif-pref-chip notif-pref-chip--off">"Disabled"</span> }.into_any()
                    }}
                </div>
                <pre class="notif-pref-config">{config_str}</pre>
                {if !pref.applies_to.is_empty() {
                    view! {
                        <p class="notif-pref-applies">
                            "Types: " {pref.applies_to.join(", ")}
                        </p>
                    }.into_any()
                } else {
                    view! { <p class="notif-pref-applies">"All notification types"</p> }.into_any()
                }}
            </div>
            <button class="notif-icon-btn notif-icon-btn--dismiss" on:click=handle_delete title="Remove">
                <span class="material-symbols-outlined">"delete"</span>
            </button>
        </div>
    }
}

// ── Setting Row ────────────────────────────────────────────────────────────────

#[component]
fn SettingRow(row: ChannelSettingRow, refetch: RwSignal<u32>) -> impl IntoView {
    // Pretty-print the key
    let label = row.key
        .trim_start_matches("notify_channel_")
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div class="notif-setting-row">
            <span class="notif-setting-key">{label}</span>
            <span class=if row.is_set { "notif-setting-value notif-setting-value--set" } else { "notif-setting-value notif-setting-value--empty" }>
                {if row.is_set { row.value.clone() } else { "— not set —".to_string() }}
            </span>
            {if row.is_set {
                view! { <span class="notif-setting-badge">"configured"</span> }.into_any()
            } else {
                view! { <span class="notif-setting-badge notif-setting-badge--missing">"missing"</span> }.into_any()
            }}
        </div>
    }
}
