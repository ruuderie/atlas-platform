// apps/folio/src/pages/landlord/communications.rs
//
// Communications page — /l/communications
//
// Unified two-pane inbox for all Folio users.
// Left pane: thread list (all rooms sorted by last activity).
// Right pane: message thread for selected room.
// "Atlas Support" thread is always pinned at top.
//
// room_types:
//   "direct"           — Any two Folio parties
//   "group"            — 3+ parties, named
//   "platform_support" — User ↔ Atlas platform operator
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id:           Uuid,
    pub room_type:    String,
    pub entity_type:  String,
    pub entity_id:    Uuid,
    pub is_active:    bool,
    pub created_at:   DateTime<Utc>,
    pub last_message: Option<String>,
    pub last_at:      Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRow {
    pub id:             Uuid,
    pub room_id:        Uuid,
    pub sender_user_id: Option<Uuid>,
    pub message_type:   String,
    pub content:        String,
    pub created_at:     DateTime<Utc>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchRooms, "/api")]
pub async fn fetch_rooms() -> Result<Vec<RoomSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<RoomSummary>>(
        "/api/folio/rooms", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchMessages, "/api")]
pub async fn fetch_messages(room_id: Uuid) -> Result<Vec<MessageRow>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<MessageRow>>(
        &format!("/api/folio/rooms/{room_id}/messages"),
        &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(GetSupportRoom, "/api")]
pub async fn get_support_room() -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let resp = crate::atlas_client::authenticated_get::<serde_json::Value>(
        "/api/folio/rooms/support", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    resp.get("room_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| server_fn::error::ServerFnError::new("No room_id in response"))
}

#[server(SendMsg, "/api")]
pub async fn send_msg(room_id: Uuid, content: String) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &format!("/api/folio/rooms/{room_id}/messages"),
        &token,
        None,
        &serde_json::json!({ "content": content, "message_type": "text" }),
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

fn room_display_name(room: &RoomSummary) -> String {
    match room.room_type.as_str() {
        "platform_support" => "⚡ Atlas Support".to_string(),
        "direct"           => "Direct Message".to_string(),
        "group"            => "Group Thread".to_string(),
        _                  => "Thread".to_string(),
    }
}

fn room_icon(room_type: &str) -> &str {
    match room_type {
        "platform_support" => "support_agent",
        "direct"           => "chat",
        "group"            => "group",
        _                  => "forum",
    }
}

fn room_type_chip_class(room_type: &str) -> &str {
    match room_type {
        "platform_support" => "comms-chip comms-chip--support",
        "direct"           => "comms-chip comms-chip--direct",
        "group"            => "comms-chip comms-chip--group",
        _                  => "comms-chip",
    }
}

fn fmt_relative(dt: &DateTime<Utc>) -> String {
    let secs = (Utc::now() - *dt).num_seconds();
    if secs < 60       { "just now".to_string() }
    else if secs < 3600 { format!("{}m ago", secs / 60) }
    else if secs < 86400 { format!("{}h ago", secs / 3600) }
    else               { dt.format("%b %d").to_string() }
}

fn fmt_time(dt: &DateTime<Utc>) -> String {
    dt.format("%H:%M").to_string()
}

// ── Thread list item ──────────────────────────────────────────────────────────

#[component]
fn ThreadItem(
    room:       RoomSummary,
    selected:   Uuid,
    on_select:  Callback<Uuid>,
) -> impl IntoView {
    let is_selected = room.id == selected;
    let icon        = room_icon(&room.room_type).to_string();
    let name        = room_display_name(&room);
    let preview     = room.last_message.clone().unwrap_or_else(|| "No messages yet".to_string());
    let time_str    = room.last_at.as_ref().map(fmt_relative).unwrap_or_default();
    let chip_cls    = room_type_chip_class(&room.room_type).to_string();
    let room_type_label = room.room_type.replace('_', " ");
    let room_id     = room.id;
    let is_support  = room.room_type == "platform_support";

    view! {
        <button
            class=move || if is_selected { "comms-thread comms-thread--active" } else { "comms-thread" }
            on:click=move |_| on_select.run(room_id)>
            <div class=move || if is_support { "comms-thread-avatar comms-thread-avatar--support" } else { "comms-thread-avatar" }>
                <span class="material-symbols-outlined">{icon.clone()}</span>
            </div>
            <div class="comms-thread-body">
                <div class="comms-thread-top">
                    <span class="comms-thread-name">{name.clone()}</span>
                    <span class="comms-thread-time">{time_str.clone()}</span>
                </div>
                <div class="comms-thread-preview">{preview.clone()}</div>
                <span class=chip_cls>{room_type_label}</span>
            </div>
        </button>
    }
}

// ── Message bubble ────────────────────────────────────────────────────────────

#[component]
fn MessageBubble(msg: MessageRow, current_user_id: Option<Uuid>) -> impl IntoView {
    let is_mine = msg.sender_user_id == current_user_id;
    let time    = fmt_time(&msg.created_at);

    view! {
        <div class=if is_mine { "comms-msg comms-msg--mine" } else { "comms-msg comms-msg--theirs" }>
            <div class="comms-bubble">
                <span class="comms-bubble-text">{msg.content.clone()}</span>
            </div>
            <span class="comms-msg-time">{time}</span>
        </div>
    }
}

// ── Message panel ─────────────────────────────────────────────────────────────

#[component]
fn MessagePanel(room_id: Uuid, room_name: String) -> impl IntoView {
    let refetch       = RwSignal::new(0u32);
    let draft         = RwSignal::new(String::new());
    let sending       = RwSignal::new(false);
    let messages      = Resource::new(
        move || (room_id, refetch.get()),
        move |(id, _)| fetch_messages(id),
    );

    let handle_send = move |_| {
        let content = draft.get_untracked();
        if content.trim().is_empty() { return; }
        sending.set(true);
        draft.set(String::new());
        leptos::task::spawn_local(async move {
            let _ = send_msg(room_id, content).await;
            refetch.update(|n| *n += 1);
            sending.set(false);
        });
    };

    let handle_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Enter" && !e.shift_key() {
            e.prevent_default();
            let content = draft.get_untracked();
            if content.trim().is_empty() { return; }
            sending.set(true);
            draft.set(String::new());
            leptos::task::spawn_local(async move {
                let _ = send_msg(room_id, content).await;
                refetch.update(|n| *n += 1);
                sending.set(false);
            });
        }
    };

    view! {
        <div class="comms-panel">
            <div class="comms-panel-header">
                <span class="material-symbols-outlined comms-panel-icon">"chat"</span>
                <span class="comms-panel-title">{room_name}</span>
                <button class="comms-panel-refresh" on:click=move |_| refetch.update(|n| *n += 1)>
                    <span class="material-symbols-outlined">"refresh"</span>
                </button>
            </div>

            <div class="comms-messages" id="comms-msg-list">
                <Suspense fallback=|| view! {
                    <div class="comms-msgs-loading">
                        <span class="material-symbols-outlined comms-spin">"autorenew"</span>
                    </div>
                }>
                    {move || messages.get().map(|res| match res {
                        Err(e) => view! {
                            <div class="comms-msg-error">"Failed to load: " {e.to_string()}</div>
                        }.into_any(),
                        Ok(msgs) if msgs.is_empty() => view! {
                            <div class="comms-msg-empty">
                                <span class="material-symbols-outlined">"chat_bubble"</span>
                                <p>"No messages yet. Say hello!"</p>
                            </div>
                        }.into_any(),
                        Ok(msgs) => view! {
                            <div class="comms-msg-list">
                                {msgs.into_iter().map(|m| view! {
                                    <MessageBubble msg=m current_user_id=None />
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                    })}
                </Suspense>
            </div>

            <div class="comms-composer">
                <textarea
                    class="comms-composer-input"
                    placeholder="Type a message… (Enter to send, Shift+Enter for newline)"
                    rows=2
                    prop:value=move || draft.get()
                    on:input=move |e| draft.set(event_target_value(&e))
                    on:keydown=handle_keydown>
                </textarea>
                <button
                    class="comms-send-btn"
                    prop:disabled=move || sending.get() || draft.get().trim().is_empty()
                    on:click=handle_send>
                    <span class="material-symbols-outlined">
                        {move || if sending.get() { "hourglass_empty" } else { "send" }}
                    </span>
                </button>
            </div>
        </div>
    }
}

// ── Empty state (no room selected) ───────────────────────────────────────────

#[component]
fn NoRoomSelected() -> impl IntoView {
    view! {
        <div class="comms-no-room">
            <span class="material-symbols-outlined comms-no-room-icon">"forum"</span>
            <p class="comms-no-room-title">"Select a conversation"</p>
            <p class="comms-no-room-sub">
                "Choose a thread from the left, or open "
                <strong>"⚡ Atlas Support"</strong>
                " to contact the platform team."
            </p>
        </div>
    }
}

// ── New support thread button ─────────────────────────────────────────────────

#[component]
fn SupportThreadButton(
    on_open: Callback<Uuid>,
) -> impl IntoView {
    let loading = RwSignal::new(false);

    view! {
        <button
            class="comms-support-btn"
            prop:disabled=move || loading.get()
            on:click=move |_| {
                loading.set(true);
                on_open.run(Uuid::nil());
                leptos::task::spawn_local(async move {
                    if let Ok(room_id) = get_support_room().await {
                        on_open.run(room_id);
                    }
                    loading.set(false);
                });
            }>
            <span class="material-symbols-outlined">"support_agent"</span>
            {move || if loading.get() { "Opening…" } else { "⚡ Atlas Support" }}
        </button>
    }
}

// ── Main page ─────────────────────────────────────────────────────────────────

#[component]
pub fn Communications() -> impl IntoView {
    let refetch_rooms  = RwSignal::new(0u32);
    let rooms          = Resource::new(
        move || refetch_rooms.get(),
        |_| fetch_rooms(),
    );
    let selected_room  = RwSignal::<Option<Uuid>>::new(None);
    let selected_name  = RwSignal::<String>::new(String::new());



    let on_support_open = Callback::new(move |room_id: Uuid| {
        if room_id.is_nil() { return; }
        selected_room.set(Some(room_id));
        selected_name.set("⚡ Atlas Support".to_string());
        refetch_rooms.update(|n| *n += 1);
    });

    view! {
        <div class="comms-page">
            // ── Left pane — thread list ───────────────────────────────────────
            <div class="comms-sidebar">
                <div class="comms-sidebar-header">
                    <h1 class="comms-sidebar-title">"Messages"</h1>
                    <SupportThreadButton on_open=on_support_open />
                </div>

                <Suspense fallback=|| view! {
                    <div class="comms-thread-skel-list">
                        {(0..5).map(|_| view! { <div class="comms-thread-skel"></div> }).collect::<Vec<_>>()}
                    </div>
                }>
                    {move || rooms.get().map(|res| match res {
                        Err(e) => view! {
                            <div class="comms-load-err">
                                "Failed to load threads: " {e.to_string()}
                            </div>
                        }.into_any(),
                        Ok(mut list) => {
                            // Pin platform_support to top
                            list.sort_by(|a, b| {
                                let a_sup = a.room_type == "platform_support";
                                let b_sup = b.room_type == "platform_support";
                                match (a_sup, b_sup) {
                                    (true, false) => std::cmp::Ordering::Less,
                                    (false, true) => std::cmp::Ordering::Greater,
                                    _             => b.last_at.cmp(&a.last_at),
                                }
                            });

                            if list.is_empty() {
                                view! {
                                    <div class="comms-empty-threads">
                                        <span class="material-symbols-outlined">"chat_bubble_outline"</span>
                                        <p>"No conversations yet."</p>
                                        <p class="comms-empty-sub">"Tap ⚡ Atlas Support to get started."</p>
                                    </div>
                                }.into_any()
                            } else {
                                let sel = selected_room.get().unwrap_or(Uuid::nil());
                                view! {
                                    <div class="comms-thread-list">
                                        {list.into_iter().map(|room| {
                                            let name = room_display_name(&room);
                                            view! {
                                                <ThreadItem
                                                    room=room
                                                    selected=sel
                                                    on_select=Callback::new(move |id: Uuid| {
                                                        selected_room.set(Some(id));
                                                        selected_name.set(name.clone());
                                                    })
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }
                    })}
                </Suspense>
            </div>

            // ── Right pane — message panel ────────────────────────────────────
            <div class="comms-main">
                {move || match selected_room.get() {
                    None => view! { <NoRoomSelected /> }.into_any(),
                    Some(id) => view! {
                        <MessagePanel room_id=id room_name=selected_name.get() />
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
