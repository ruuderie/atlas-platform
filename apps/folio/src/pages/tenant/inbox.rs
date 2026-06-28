// apps/folio/src/pages/tenant/inbox.rs
//
// Tenant Inbox — /t/inbox
//
// Tenant-facing messaging interface. Uses the same atlas_ws_room / atlas_ws_message
// system as the landlord communications page but scoped to the tenant's identity.
//
// Features:
//   - Thread list (direct, group, platform_support) sorted by last activity
//   - "⚡ Atlas Support" thread always pinned / auto-created on first visit
//   - Message thread view with send compose area
//   - Visual distinction: tenant messages (right/blue), others (left/slate)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos::task::spawn_local;
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

#[server(TenantFetchRooms, "/api")]
pub async fn tenant_fetch_rooms() -> Result<Vec<RoomSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<RoomSummary>>(
        "/api/folio/rooms", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(TenantFetchMessages, "/api")]
pub async fn tenant_fetch_messages(room_id: Uuid) -> Result<Vec<MessageRow>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<MessageRow>>(
        &format!("/api/folio/rooms/{room_id}/messages"),
        &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(TenantGetSupportRoom, "/api")]
pub async fn tenant_get_support_room() -> Result<Uuid, server_fn::error::ServerFnError> {
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

#[server(TenantSendMsg, "/api")]
pub async fn tenant_send_msg(room_id: Uuid, content: String) -> Result<(), server_fn::error::ServerFnError> {
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

fn room_label(room: &RoomSummary) -> String {
    match room.room_type.as_str() {
        "platform_support" => "⚡ Atlas Support".to_string(),
        "group"            => "Group Chat".to_string(),
        _                  => "Message Thread".to_string(),
    }
}

fn fmt_time(dt: &DateTime<Utc>) -> String {
    dt.format("%b %d, %H:%M").to_string()
}

fn msg_preview(s: &str) -> String {
    if s.len() > 60 { format!("{}…", &s[..60]) } else { s.to_string() }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantInbox() -> impl IntoView {
    // ── Data ──────────────────────────────────────────────────────────────────
    let refresh       = RwSignal::new(0u32);
    let selected_room = RwSignal::new(None::<Uuid>);
    let compose_text  = RwSignal::new(String::new());
    let sending       = RwSignal::new(false);

    let rooms_res = Resource::new(
        move || refresh.get(),
        |_| tenant_fetch_rooms(),
    );

    // Auto-create / load the support room on mount so it's always available
    let support_room_res = Resource::new(
        || (),
        |_| tenant_get_support_room(),
    );

    let messages_res = Resource::new(
        move || selected_room.get(),
        |rid| async move {
            match rid {
                Some(id) => tenant_fetch_messages(id).await.ok(),
                None     => None,
            }
        },
    );

    // ── Send handler ──────────────────────────────────────────────────────────
    let do_send = move || {
        let txt = compose_text.get();
        let Some(rid) = selected_room.get() else { return; };
        if txt.trim().is_empty() { return; }
        sending.set(true);
        spawn_local(async move {
            if tenant_send_msg(rid, txt).await.is_ok() {
                compose_text.set(String::new());
                messages_res.refetch();
            }
            sending.set(false);
        });
    };
    let handle_send_click = move |_: leptos::ev::MouseEvent| do_send();
    let handle_send = do_send;  // alias for keydown path

    view! {
        <div class="main-area">

            // ── Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Inbox"</h1>
                    <p class="page-subtitle">"Your messages with management, vendors, and Atlas support"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        <svg class="w-3 h-3 inline mr-1" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                        </svg>
                        "Refresh"
                    </button>
                </div>
            </div>

            // ── Atlas Support quick-open banner ──
            <Suspense fallback=|| ()>
                {move || support_room_res.get().map(|res| {
                    match res {
                        Ok(rid) => view! {
                            <button
                                class="inbox-support-banner"
                                on:click=move |_| selected_room.set(Some(rid))
                            >
                                <span class="inbox-support-icon">"⚡"</span>
                                <div class="inbox-support-text">
                                    <span class="inbox-support-title">"Atlas Support"</span>
                                    <span class="inbox-support-sub">"Have a question or issue? Message us directly."</span>
                                </div>
                                <span class="inbox-support-cta">"Open Thread →"</span>
                            </button>
                        }.into_any(),
                        Err(_) => ().into_any(),
                    }
                })}
            </Suspense>

            // ── 2-Panel Layout ──
            <div class="inbox-panel">

                // Left: thread list
                <div class="inbox-thread-list">
                    <div class="inbox-thread-list-header">
                        <span class="font-bold text-sm">"Conversations"</span>
                    </div>
                    <Suspense fallback=|| view! {
                        <div class="inbox-empty">"Loading…"</div>
                    }>
                        {move || rooms_res.get().map(|res| {
                            match res {
                                Ok(rooms) => {
                                    if rooms.is_empty() {
                                        return view! {
                                            <div class="inbox-empty">
                                                "No conversations yet."
                                            </div>
                                        }.into_any();
                                    }
                                    view! {
                                        <For
                                            each=move || rooms.clone()
                                            key=|r| r.id
                                            children=move |room| {
                                                let rid = room.id;
                                                let is_sel = Signal::derive(move || selected_room.get() == Some(rid));
                                                let label   = room_label(&room);
                                                let preview = room.last_message.as_deref().map(msg_preview).unwrap_or_else(|| "No messages yet".to_string());
                                                let time    = room.last_at.as_ref().map(fmt_time).unwrap_or_default();
                                                let is_support = room.room_type == "platform_support";

                                                view! {
                                                    <div
                                                        class=move || format!("inbox-thread-item {}",
                                                            if is_sel.get() { "inbox-thread-item--selected" } else { "" }
                                                        )
                                                        on:click=move |_| selected_room.set(Some(rid))
                                                    >
                                                        <div class=format!("inbox-thread-avatar {}",
                                                            if is_support { "inbox-thread-avatar--support" } else { "inbox-thread-avatar--default" }
                                                        )>
                                                            {if is_support { "⚡" } else { "💬" }}
                                                        </div>
                                                        <div class="inbox-thread-meta">
                                                            <div class="inbox-thread-name">{label}</div>
                                                            <div class="inbox-thread-preview">{preview}</div>
                                                        </div>
                                                        <div class="inbox-thread-time">{time}</div>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <div class="inbox-empty text-red-400">"Error: " {e.to_string()}</div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>

                // Right: message thread
                <div class="inbox-thread-view">
                    {move || match selected_room.get() {
                        None => view! {
                            <div class="inbox-thread-empty">
                                <svg class="w-12 h-12 mx-auto mb-3 opacity-25" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
                                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                                </svg>
                                <p class="font-semibold text-sm">"Select a conversation"</p>
                                <p class="text-xs mt-1">"Choose a thread from the list to read messages"</p>
                            </div>
                        }.into_any(),
                        Some(_) => view! {
                            // Messages scroll area
                            <div class="inbox-messages">
                                <Suspense fallback=|| view! {
                                    <div class="inbox-empty">"Loading messages…"</div>
                                }>
                                    {move || messages_res.get().map(|opt| {
                                        match opt {
                                            Some(msgs) if !msgs.is_empty() => view! {
                                                <For
                                                    each=move || msgs.clone()
                                                    key=|m| m.id
                                                    children=move |msg| {
                                                        let is_system = msg.message_type == "system";
                                                        let is_op     = msg.message_type == "operator_reply";
                                                        let time      = fmt_time(&msg.created_at);
                                                        let content   = msg.content.clone();

                                                        if is_system {
                                                            view! {
                                                                <div class="inbox-msg-system">
                                                                    <div class="inbox-msg-system-line"/>
                                                                    <span class="inbox-msg-system-text">{content}</span>
                                                                    <div class="inbox-msg-system-line"/>
                                                                </div>
                                                            }.into_any()
                                                        } else if is_op {
                                                            // Platform support reply — left aligned, special color
                                                            view! {
                                                                <div class="inbox-msg-row inbox-msg-row--left">
                                                                    <div class="inbox-msg-avatar inbox-msg-avatar--support">"⚡"</div>
                                                                    <div class="inbox-msg-bubble-wrap">
                                                                        <div class="inbox-msg-meta">"Atlas Support · " {time}</div>
                                                                        <div class="inbox-msg-bubble inbox-msg-bubble--support">{content}</div>
                                                                    </div>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            // Tenant's own message — right aligned
                                                            view! {
                                                                <div class="inbox-msg-row inbox-msg-row--right">
                                                                    <div class="inbox-msg-bubble-wrap inbox-msg-bubble-wrap--right">
                                                                        <div class="inbox-msg-meta inbox-msg-meta--right">"You · " {time}</div>
                                                                        <div class="inbox-msg-bubble inbox-msg-bubble--self">{content}</div>
                                                                    </div>
                                                                    <div class="inbox-msg-avatar inbox-msg-avatar--self">"T"</div>
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }
                                                />
                                            }.into_any(),
                                            _ => view! {
                                                <div class="inbox-empty">"No messages yet. Say hello! 👋"</div>
                                            }.into_any(),
                                        }
                                    })}
                                </Suspense>
                            </div>

                            // Compose
                            <div class="inbox-compose">
                                <textarea
                                    rows="2"
                                    placeholder="Type a message…"
                                    class="inbox-compose-input"
                                    prop:value=compose_text
                                    on:input=move |ev| compose_text.set(event_target_value(&ev))
                                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                                        if ev.key() == "Enter" && !ev.shift_key() {
                                            ev.prevent_default();
                                            handle_send();
                                        }
                                    }
                                ></textarea>
                                <button
                                    class="inbox-compose-send"
                                    disabled=move || sending.get() || compose_text.get().trim().is_empty()
                                    on:click=handle_send_click
                                >
                                    {move || if sending.get() { "…" } else { "Send" }}
                                    <svg class="w-3.5 h-3.5 ml-1" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M2.01 21 23 12 2.01 3 2 10l15 2-15 2z"/>
                                    </svg>
                                </button>
                            </div>
                        }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}
