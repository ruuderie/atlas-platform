//! LandlordReferrals — `/l/referrals`
//!
//! Trusted invite-out: mint personal `/refer/{code}`, send SMS/email, activity.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyAmbassadorDto {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub refer_url: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReferralJoinedRow {
    pub email_masked: String,
    pub role: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyReferralsDto {
    pub code: String,
    pub refer_url: String,
    pub joined: Vec<ReferralJoinedRow>,
    pub by_role: serde_json::Value,
    pub total: usize,
}

#[server(GetMyAmbassador, "/api")]
pub async fn get_my_ambassador() -> Result<MyAmbassadorDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let raw: serde_json::Value =
        crate::atlas_client::authenticated_get("/api/folio/me/ambassador", &token, None)
            .await
            .map_err(ServerFnError::new)?;
    Ok(MyAmbassadorDto {
        id: raw["id"].as_str().unwrap_or_default().to_string(),
        code: raw["code"].as_str().unwrap_or_default().to_string(),
        display_name: raw["display_name"].as_str().unwrap_or_default().to_string(),
        refer_url: raw["refer_url"].as_str().unwrap_or_default().to_string(),
        status: raw["status"].as_str().unwrap_or_default().to_string(),
    })
}

#[server(ListMyReferrals, "/api")]
pub async fn list_my_referrals() -> Result<MyReferralsDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    crate::atlas_client::authenticated_get("/api/folio/me/referrals", &token, None)
        .await
        .map_err(ServerFnError::new)
}

#[server(SendReferralInvite, "/api")]
pub async fn send_referral_invite(channel: String, to: String) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let payload = serde_json::json!({ "channel": channel, "to": to });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/me/referrals/send",
        &token,
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(ServerFnError::new)
}

#[server(AttributeReferral, "/api")]
pub async fn attribute_referral(referred_by: String, role: String) -> Result<(), ServerFnError> {
    if referred_by.trim().is_empty() {
        return Ok(());
    }
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let payload = serde_json::json!({
        "referred_by": referred_by,
        "role": if role.is_empty() { serde_json::Value::Null } else { serde_json::json!(role) },
    });
    let _ = crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/me/referrals/attribute",
        &token,
        None,
        &payload,
    )
    .await;
    Ok(())
}

/// Call once from a wizard body so `?ref=` / `?referred_by=` credits the ambassador.
pub fn use_referral_attribution(role: &'static str) {
    let query = leptos_router::hooks::use_query_map();
    Effect::new(move |_| {
        let referred_by = query.with(|q| {
            q.get("ref")
                .or_else(|| q.get("referred_by"))
                .map(|s| s.to_string())
                .unwrap_or_default()
        });
        if referred_by.is_empty() {
            return;
        }
        let role = role.to_string();
        leptos::task::spawn_local(async move {
            let _ = attribute_referral(referred_by, role).await;
        });
    });
}

#[component]
pub fn LandlordReferrals() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let sms_to = RwSignal::new(String::new());
    let email_to = RwSignal::new(String::new());
    let status_msg = RwSignal::new(String::new());
    let sending = RwSignal::new(false);

    let ambassador = LocalResource::new(move || {
        let _ = refresh.get();
        async move { get_my_ambassador().await }
    });
    let activity = LocalResource::new(move || {
        let _ = refresh.get();
        async move { list_my_referrals().await }
    });

    view! {
        <div class="p-6 max-w-2xl mx-auto space-y-6">
            <div>
                <p class="text-[0.65rem] font-bold uppercase tracking-widest text-on-surface-variant">"Invite friends"</p>
                <h1 class="text-2xl font-extrabold tracking-tight mt-1">"Share Folio with someone you trust."</h1>
                <p class="text-sm text-on-surface-variant mt-1">
                    "One link. They create an account, pick how they’ll use Folio, and you get credit. Send by SMS or email — or copy the link yourself."
                </p>
            </div>

            <Suspense fallback=move || view! { <p class="text-sm text-on-surface-variant">"Loading…"</p> }>
                {move || match ambassador.get() {
                    Some(Ok(a)) if !a.code.is_empty() => {
                        let url = a.refer_url.clone();
                        let code = a.code.clone();
                        view! {
                            <div class="bg-surface-container-lowest rounded-xl border border-outline-variant/25 p-5 space-y-4">
                                <div class="flex items-center justify-between gap-2 flex-wrap">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Your link"</h3>
                                    <span class="text-xs font-semibold text-emerald-700 bg-emerald-50 border border-emerald-200 rounded-full px-2 py-0.5">
                                        {format!("Credited as {code}")}
                                    </span>
                                </div>
                                <code class="block text-sm text-emerald-700 break-all bg-surface-container-low rounded-lg px-3 py-2.5 border border-outline-variant/20">{url.clone()}</code>
                                <div class="flex flex-wrap gap-2">
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--primary folio-btn--sm"
                                        on:click={
                                            let url = url.clone();
                                            move |_| {
                                                #[cfg(target_arch = "wasm32")]
                                                {
                                                    if let Some(c) = web_sys::window().map(|w| w.navigator().clipboard()) {
                                                        let _ = c.write_text(&url);
                                                        status_msg.set("Link copied".into());
                                                    }
                                                }
                                            }
                                        }
                                    >"Copy link"</button>
                                    <a class="folio-btn folio-btn--ghost folio-btn--sm" href=url.clone() target="_blank" rel="noopener">"Open link"</a>
                                </div>
                            </div>

                            <div class="bg-surface-container-lowest rounded-xl border border-outline-variant/25 p-5 space-y-4">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Send invite"</h3>
                                <p class="text-xs text-on-surface-variant">"From you — SMS or email with your link."</p>
                                <div>
                                    <label class="block text-[0.65rem] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Text (SMS)"</label>
                                    <div class="flex gap-2 flex-wrap">
                                        <input
                                            type="tel"
                                            class="flex-1 min-w-[10rem] rounded-lg border border-outline-variant/40 bg-surface-container-low px-3 py-2 text-sm"
                                            placeholder="+1 (555) 000-0000"
                                            prop:value=move || sms_to.get()
                                            on:input=move |ev| sms_to.set(event_target_value(&ev))
                                        />
                                        <button
                                            type="button"
                                            class="folio-btn folio-btn--primary folio-btn--sm"
                                            disabled=move || sending.get()
                                            on:click=move |_| {
                                                sending.set(true);
                                                let to = sms_to.get();
                                                leptos::task::spawn_local(async move {
                                                    match send_referral_invite("sms".into(), to).await {
                                                        Ok(()) => status_msg.set("SMS sent".into()),
                                                        Err(e) => status_msg.set(e.to_string()),
                                                    }
                                                    sending.set(false);
                                                    refresh.update(|n| *n += 1);
                                                });
                                            }
                                        >"Send SMS"</button>
                                    </div>
                                </div>
                                <div>
                                    <label class="block text-[0.65rem] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Email magic link"</label>
                                    <div class="flex gap-2 flex-wrap">
                                        <input
                                            type="email"
                                            class="flex-1 min-w-[10rem] rounded-lg border border-outline-variant/40 bg-surface-container-low px-3 py-2 text-sm"
                                            placeholder="friend@email.com"
                                            prop:value=move || email_to.get()
                                            on:input=move |ev| email_to.set(event_target_value(&ev))
                                        />
                                        <button
                                            type="button"
                                            class="folio-btn folio-btn--ghost folio-btn--sm"
                                            disabled=move || sending.get()
                                            on:click=move |_| {
                                                sending.set(true);
                                                let to = email_to.get();
                                                leptos::task::spawn_local(async move {
                                                    match send_referral_invite("email".into(), to).await {
                                                        Ok(()) => status_msg.set("Email sent".into()),
                                                        Err(e) => status_msg.set(e.to_string()),
                                                    }
                                                    sending.set(false);
                                                    refresh.update(|n| *n += 1);
                                                });
                                            }
                                        >"Send email"</button>
                                    </div>
                                </div>
                                <p class="text-xs text-on-surface-variant">{move || status_msg.get()}</p>
                            </div>
                        }.into_any()
                    }
                    Some(Ok(_)) | None => view! {
                        <div class="bg-surface-container-lowest rounded-xl border border-outline-variant/25 p-8 text-center">
                            <h3 class="text-base font-bold">"Get your personal share link"</h3>
                            <p class="text-xs text-on-surface-variant mt-2 max-w-sm mx-auto">
                                "Trusted members only. Once you have a link, text or email it from here."
                            </p>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                on:click=move |_| {
                                    leptos::task::spawn_local(async move {
                                        let _ = get_my_ambassador().await;
                                        refresh.update(|n| *n += 1);
                                    });
                                }
                            >"Generate my link"</button>
                        </div>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <p class="text-sm text-red-600">{e.to_string()}</p>
                    }.into_any(),
                }}
            </Suspense>

            <Suspense fallback=|| ()>
                {move || match activity.get() {
                    Some(Ok(a)) if a.total > 0 => view! {
                        <div class="bg-surface-container-lowest rounded-xl border border-outline-variant/25 overflow-hidden">
                            <div class="px-5 py-3 border-b border-outline-variant/20 flex justify-between">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Activity"</h3>
                                <span class="text-xs text-on-surface-variant">{format!("{} joined", a.total)}</span>
                            </div>
                            <ul class="divide-y divide-outline-variant/15 text-sm">
                                {a.joined.into_iter().map(|row| {
                                    let label = match &row.role {
                                        Some(r) => format!("{} · {}", row.email_masked, r),
                                        None => row.email_masked.clone(),
                                    };
                                    view! {
                                        <li class="px-5 py-3 flex justify-between gap-3">
                                            <span>{label}</span>
                                            <span class="text-xs text-on-surface-variant font-mono shrink-0">{row.joined_at.chars().take(10).collect::<String>()}</span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </div>
                    }.into_any(),
                    Some(Ok(_)) => view! {
                        <div class="bg-surface-container-lowest rounded-xl border border-outline-variant/25 p-5">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-2">"Activity"</h3>
                            <p class="text-xs text-on-surface-variant">"No one has joined yet. Send an invite to get started."</p>
                        </div>
                    }.into_any(),
                    _ => ().into_any(),
                }}
            </Suspense>
        </div>
    }
}
