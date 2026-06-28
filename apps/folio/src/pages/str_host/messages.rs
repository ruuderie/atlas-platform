// apps/folio/src/pages/str_host/messages.rs
//
// STR Guest Messaging — /s/messages
//
// Threaded messaging interface for host-guest communications.
// Uses /api/folio/comms (ws_room messages filtered to STR context).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos::task::spawn_local;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrMessage {
    pub id:          Uuid,
    pub room_id:     Uuid,
    pub sender_id:   Uuid,
    pub sender_name: Option<String>,
    pub body:        String,
    pub created_at:  String,
    pub is_host:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrConversation {
    pub room_id:       Uuid,
    pub guest_name:    Option<String>,
    pub reservation_id:Option<Uuid>,
    pub last_message:  Option<String>,
    pub last_at:       Option<String>,
    pub unread_count:  usize,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchStrConversations, "/api")]
pub async fn fetch_str_conversations() -> Result<Vec<StrConversation>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<StrConversation>>(
        "/api/folio/comms/str/conversations", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchStrMessages, "/api")]
pub async fn fetch_str_messages(room_id: String) -> Result<Vec<StrMessage>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/folio/comms/str/rooms/{room_id}/messages");
    crate::atlas_client::authenticated_get::<Vec<StrMessage>>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

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

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrGuestMessaging() -> impl IntoView {
    let active_room = RwSignal::new(None::<Uuid>);
    let compose     = RwSignal::new(String::new());
    let sending     = RwSignal::new(false);

    let convs_res = Resource::new(|| (), |_| fetch_str_conversations());
    let msgs_res  = Resource::new(
        move || active_room.get().map(|id| id.to_string()),
        |id_opt| async move {
            match id_opt {
                Some(id) => fetch_str_messages(id).await,
                None     => Ok(vec![]),
            }
        },
    );

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Guest Messaging"</h1>
                    <p class="page-subtitle">"Host-to-guest communications for all active bookings"</p>
                </div>
            </div>

            <div class="str-msgs-layout">

                // ── Conversation list ──
                <div class="str-msgs-sidebar">
                    <div class="str-msgs-sidebar-title">"Conversations"</div>
                    <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                        {move || convs_res.get().map(|res| {
                            match res {
                                Ok(convs) if !convs.is_empty() => view! {
                                    <div class="str-conv-list">
                                        <For
                                            each=move || convs.clone()
                                            key=|c| c.room_id
                                            children=move |conv| {
                                                let rid    = conv.room_id;
                                                let name   = conv.guest_name.clone().unwrap_or_else(|| "Guest".to_string());
                                                let last   = conv.last_message.clone().unwrap_or_default();
                                                let unread = conv.unread_count;
                                                view! {
                                                    <div
                                                        class=move || format!("str-conv-row {}", if active_room.get() == Some(rid) { "str-conv-row--active" } else { "" })
                                                        on:click=move |_| active_room.set(Some(rid))
                                                    >
                                                        <div class="str-conv-avatar">{name.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "G".to_string())}</div>
                                                        <div class="str-conv-info">
                                                            <div class="str-conv-name">{name}</div>
                                                            <div class="str-conv-last">{if last.len() > 35 { format!("{}…", &last[..35]) } else { last }}</div>
                                                        </div>
                                                        {if unread > 0 {
                                                            view! { <span class="str-conv-unread">{unread.to_string()}</span> }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any(),
                                Ok(_) => view! {
                                    <div class="str-msgs-empty">
                                        <div class="str-msgs-empty-icon">"✉"</div>
                                        <div>"No conversations yet"</div>
                                        <div class="text-xs text-on-surface-variant">"Messages from guests will appear here once bookings begin."</div>
                                    </div>
                                }.into_any(),
                                Err(_) => view! {
                                    <div class="str-msgs-empty">
                                        <div>"No conversation data."</div>
                                        <div class="text-xs text-on-surface-variant">"This data is available once the comms channel is active."</div>
                                    </div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>

                // ── Message thread ──
                <div class="str-msgs-thread">
                    {move || if active_room.get().is_none() {
                        view! {
                            <div class="str-msgs-empty" style="height:100%;align-items:center;justify-content:center;">
                                <div class="str-msgs-empty-icon">"💬"</div>
                                <div>"Select a conversation"</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="str-thread-messages">
                                <Suspense fallback=|| view! { <div class="doc-empty">"Loading messages…"</div> }>
                                    {move || msgs_res.get().map(|res| {
                                        match res {
                                            Ok(msgs) if !msgs.is_empty() => view! {
                                                <For
                                                    each=move || msgs.clone()
                                                    key=|m| m.id
                                                    children=move |msg| {
                                                        let is_host = msg.is_host;
                                                        let name = msg.sender_name.clone().unwrap_or_else(|| "Guest".to_string());
                                                        let date = msg.created_at.chars().take(16).collect::<String>().replace('T', " ");
                                                        view! {
                                                            <div class=format!("str-msg-bubble {}", if is_host { "str-msg-bubble--host" } else { "str-msg-bubble--guest" })>
                                                                <div class="str-msg-meta">{name} " · " {date}</div>
                                                                <div class="str-msg-body">{msg.body}</div>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            }.into_any(),
                                            _ => view! {
                                                <div class="str-msgs-empty">"No messages yet."</div>
                                            }.into_any(),
                                        }
                                    })}
                                </Suspense>
                            </div>

                            // Compose
                            <div class="str-thread-compose">
                                <textarea
                                    class="str-thread-input"
                                    placeholder="Write a message to your guest…"
                                    prop:value=move || compose.get()
                                    on:input=move |ev| compose.set(event_target_value(&ev))
                                ></textarea>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || sending.get() || compose.get().trim().is_empty()
                                    on:click=move |_| {
                                        if compose.get().trim().is_empty() { return; }
                                        sending.set(true);
                                        // Phase 7: POST /api/folio/comms/str/rooms/:id/messages
                                        compose.set(String::new());
                                        sending.set(false);
                                    }
                                >
                                    {move || if sending.get() { "Sending…" } else { "Send" }}
                                </button>
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
