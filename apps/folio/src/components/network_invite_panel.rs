//! G-36 NetworkInvite panel — shared invite UI for wizards and dashboards.
//!
//! Loads programs for an actor role, lets the user send one or more email invites,
//! and optionally shows stats + recent actions with status pills.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramDto {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub target_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDto {
    pub id: String,
    pub target_email: Option<String>,
    pub target_role: Option<String>,
    pub status: String,
    pub invite_code: Option<String>,
    pub outcome_type: Option<String>,
    pub outcome_status: Option<String>,
}

#[derive(Clone)]
pub struct AngleCard {
    pub icon: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

#[derive(Clone, PartialEq, Eq)]
struct InviteRow {
    id: u32,
    email: String,
    role: String,
}

#[server(ListNetworkPrograms, "/api")]
pub async fn list_network_programs(actor_role: String) -> Result<Vec<ProgramDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let url = format!(
        "/api/folio/programs?kind=network_invite&actor_role={}",
        urlencoding_encode(&actor_role)
    );
    let raw: serde_json::Value = crate::atlas_client::authenticated_get(&url, &token, None)
        .await
        .map_err(ServerFnError::new)?;
    let programs = raw["programs"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            Some(ProgramDto {
                id: p["id"].as_str()?.to_string(),
                slug: p["slug"].as_str()?.to_string(),
                name: p["name"].as_str()?.to_string(),
                description: p["description"].as_str().map(|s| s.to_string()),
                target_roles: p["target_roles"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();
    Ok(programs)
}

#[server(SendNetworkInvite, "/api")]
pub async fn send_network_invite(
    program_id: String,
    target_email: String,
    target_role: String,
    personal_note: Option<String>,
) -> Result<String, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let payload = serde_json::json!({
        "target_email": target_email,
        "target_role": target_role,
        "personal_note": personal_note,
    });
    let raw: serde_json::Value = crate::atlas_client::authenticated_post(
        &format!("/api/folio/programs/{program_id}/actions"),
        &token,
        None,
        &payload,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(raw["join_url"]
        .as_str()
        .or_else(|| raw["action"]["invite_code"].as_str())
        .unwrap_or_default()
        .to_string())
}

#[server(ListMyNetworkActions, "/api")]
pub async fn list_my_network_actions() -> Result<Vec<ActionDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let raw: serde_json::Value =
        crate::atlas_client::authenticated_get("/api/folio/programs/actions/mine", &token, None)
            .await
            .map_err(ServerFnError::new)?;
    let actions = raw["actions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|a| {
            Some(ActionDto {
                id: a["id"].as_str()?.to_string(),
                target_email: a["target_email"].as_str().map(|s| s.to_string()),
                target_role: a["target_role"].as_str().map(|s| s.to_string()),
                status: a["status"].as_str()?.to_string(),
                invite_code: a["invite_code"].as_str().map(|s| s.to_string()),
                outcome_type: a["outcome_type"].as_str().map(|s| s.to_string()),
                outcome_status: a["outcome_status"].as_str().map(|s| s.to_string()),
            })
        })
        .collect();
    Ok(actions)
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn role_label(role: &str) -> String {
    role.replace('_', " ")
}

fn status_pill(status: &str) -> (&'static str, &'static str) {
    match status {
        "outcome_complete" => ("ni-pill ni-pill-done", "Complete"),
        "accepted" => ("ni-pill ni-pill-joined", "Joined"),
        "opened" => ("ni-pill ni-pill-joined", "Opened"),
        "expired" | "revoked" => ("ni-pill ni-pill-sent", "Expired"),
        _ => ("ni-pill ni-pill-sent", "Sent"),
    }
}

fn outcome_label(outcome_type: &Option<String>, outcome_status: &Option<String>) -> String {
    match (outcome_type.as_deref(), outcome_status.as_deref()) {
        (_, Some("pending") | None) if outcome_type.is_some() => "Pending".into(),
        (Some(t), Some("completed")) => t.replace('_', " "),
        (Some(t), _) => t.replace('_', " "),
        _ => "-".into(),
    }
}

fn pick_program<'a>(
    programs: &'a [ProgramDto],
    preferred_slug: &str,
    role: &str,
) -> Option<&'a ProgramDto> {
    programs
        .iter()
        .find(|p| p.target_roles.iter().any(|r| r == role))
        .or_else(|| programs.iter().find(|p| p.slug == preferred_slug))
        .or_else(|| programs.first())
}

fn union_target_roles(programs: &[ProgramDto]) -> Vec<String> {
    let mut out = Vec::new();
    for p in programs {
        for r in &p.target_roles {
            if !out.iter().any(|x| x == r) {
                out.push(r.clone());
            }
        }
    }
    out
}

/// Shared NetworkInvite panel.
#[component]
pub fn NetworkInvitePanel(
    #[prop(into)] actor_role: String,
    #[prop(into)] preferred_slug: String,
    angles: Vec<AngleCard>,
    #[prop(optional)] show_history: bool,
    #[prop(optional)] show_stats: bool,
    #[prop(optional)] show_note: bool,
    #[prop(default = true)] allow_multi: bool,
    #[prop(optional, into)] section_title: Option<String>,
    #[prop(optional, into)] footnote: Option<String>,
    #[prop(optional, into)] send_label: Option<String>,
) -> impl IntoView {
    let actor_role_c = actor_role.clone();
    let preferred_slug_send = RwSignal::new(preferred_slug);

    let programs = Resource::new(
        move || actor_role_c.clone(),
        |role| async move { list_network_programs(role).await.unwrap_or_default() },
    );

    let next_row_id = RwSignal::new(1u32);
    let rows = RwSignal::new(vec![InviteRow {
        id: 0,
        email: String::new(),
        role: String::new(),
    }]);
    let note = RwSignal::new(String::new());
    let status_msg = RwSignal::new(String::new());
    let sending = RwSignal::new(false);
    let refresh_tick = RwSignal::new(0u32);

    let program_list = RwSignal::new(Vec::<ProgramDto>::new());
    let target_roles = RwSignal::new(Vec::<String>::new());
    let programs_ready = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(list) = programs.get() {
            let targets = union_target_roles(&list);
            program_list.set(list);
            target_roles.set(targets.clone());
            programs_ready.set(true);
            rows.update(|rs| {
                if let Some(first) = rs.first_mut() {
                    if first.role.is_empty() {
                        if let Some(t) = targets.first() {
                            first.role = t.clone();
                        }
                    }
                }
            });
        }
    });

    let need_actions = show_history || show_stats;
    let history = Resource::new(
        move || (need_actions, refresh_tick.get()),
        |(need, _)| async move {
            if need {
                list_my_network_actions().await.unwrap_or_default()
            } else {
                Vec::new()
            }
        },
    );

    let title = section_title.unwrap_or_else(|| "Send an invite".into());
    let send_btn = send_label.unwrap_or_else(|| "Send invites".into());
    let footnote_text = footnote
        .unwrap_or_else(|| "Optional. You can invite people anytime from your dashboard.".into());

    let on_send = move |_| {
        let pending: Vec<(String, String)> = rows
            .get()
            .into_iter()
            .map(|r| (r.email.trim().to_string(), r.role.clone()))
            .filter(|(em, _)| !em.is_empty())
            .collect();
        if pending.is_empty() {
            status_msg.set("Enter at least one email.".into());
            return;
        }
        if pending.iter().any(|(em, _)| !em.contains('@')) {
            status_msg.set("Enter a valid email.".into());
            return;
        }
        sending.set(true);
        status_msg.set(String::new());
        let note_val = if show_note {
            let n = note.get();
            if n.trim().is_empty() {
                None
            } else {
                Some(n)
            }
        } else {
            None
        };
        let pref = preferred_slug_send.get();
        let programs_snap = program_list.get();
        leptos::task::spawn_local(async move {
            let mut ok = 0usize;
            let mut last_link = String::new();
            let mut err: Option<String> = None;
            for (em, rl) in pending {
                let Some(prog) = pick_program(&programs_snap, &pref, &rl) else {
                    err = Some("No matching program for that role.".into());
                    break;
                };
                match send_network_invite(prog.id.clone(), em, rl, note_val.clone()).await {
                    Ok(join) => {
                        ok += 1;
                        if !join.is_empty() {
                            last_link = join;
                        }
                    }
                    Err(e) => {
                        err = Some(format!("Could not send: {e}"));
                        break;
                    }
                }
            }
            if let Some(e) = err {
                status_msg.set(e);
            } else {
                status_msg.set(if last_link.is_empty() {
                    format!("{ok} invite{} sent.", if ok == 1 { "" } else { "s" })
                } else if ok == 1 {
                    format!("Invite sent. Link: {last_link}")
                } else {
                    format!("{ok} invites sent. Last link: {last_link}")
                });
                let default_role = target_roles
                    .get_untracked()
                    .first()
                    .cloned()
                    .unwrap_or_default();
                rows.set(vec![InviteRow {
                    id: 0,
                    email: String::new(),
                    role: default_role,
                }]);
                next_row_id.set(1);
                note.set(String::new());
                refresh_tick.update(|n| *n += 1);
            }
            sending.set(false);
        });
    };

    view! {
        <div class="ni-panel">
            <Show when=move || show_stats>
                <Suspense fallback=|| ()>
                    {move || {
                        let actions = history.get().unwrap_or_default();
                        let sent = actions.len();
                        let joined = actions.iter().filter(|a| {
                            matches!(a.status.as_str(), "accepted" | "outcome_complete" | "opened")
                        }).count();
                        let complete = actions.iter().filter(|a| a.status == "outcome_complete").count();
                        view! {
                            <div class="ni-stats">
                                <div class="ni-stat">
                                    <div class="ni-stat-v">{sent}</div>
                                    <div class="ni-stat-l">"Invites sent"</div>
                                </div>
                                <div class="ni-stat">
                                    <div class="ni-stat-v">{joined}</div>
                                    <div class="ni-stat-l">"Joined"</div>
                                </div>
                                <div class="ni-stat">
                                    <div class="ni-stat-v">{complete}</div>
                                    <div class="ni-stat-l">"Outcomes complete"</div>
                                </div>
                            </div>
                        }
                    }}
                </Suspense>
            </Show>

            <div class="ni-card">
                <div class="ni-ct">{title}</div>
                <div class="ni-angles">
                    {angles.into_iter().map(|a| view! {
                        <div class="ni-angle">
                            <span class="ms msf ni-angle-ico">{a.icon}</span>
                            <div class="ni-angle-h">{a.title}</div>
                            <div class="ni-angle-p">{a.body}</div>
                        </div>
                    }).collect_view()}
                </div>

                <Suspense fallback=move || view! { <p class="ni-muted">"Loading programs…"</p> }>
                    {move || {
                        let _ = programs.get();
                        if !programs_ready.get() {
                            return view! { <p class="ni-muted">"Loading programs…"</p> }.into_any();
                        }
                        if program_list.get().is_empty() {
                            return view! {
                                <p class="ni-muted">"No invite programs available for this role yet."</p>
                            }.into_any();
                        }
                        view! {
                            <div class="ni-form">
                                <For
                                    each=move || rows.get()
                                    key=|r| r.id
                                    children=move |row| {
                                        let row_id = row.id;
                                        view! {
                                            <InviteRowEditor
                                                row_id=row_id
                                                rows=rows
                                                target_roles=target_roles
                                            />
                                        }
                                    }
                                />

                                <Show when=move || allow_multi>
                                    <button type="button" class="ni-add"
                                        on:click=move |_| {
                                            let id = next_row_id.get();
                                            next_row_id.set(id + 1);
                                            let default_role = target_roles.get().first().cloned().unwrap_or_default();
                                            rows.update(|rs| rs.push(InviteRow {
                                                id,
                                                email: String::new(),
                                                role: default_role,
                                            }));
                                        }>
                                        <span class="ms">"add"</span>
                                        "Add another"
                                    </button>
                                </Show>

                                <Show when=move || show_note>
                                    <div class="ni-note">
                                        <label>"Personal message (optional)"</label>
                                        <textarea class="ni-textarea"
                                            placeholder="Hi. We moved portfolio reporting into Folio. This link opens your Owner portal."
                                            prop:value=move || note.get()
                                            on:input=move |e| note.set(event_target_value(&e))
                                        ></textarea>
                                    </div>
                                </Show>

                                <button type="button" class="ni-btn"
                                    disabled=move || sending.get()
                                    on:click=on_send
                                >
                                    <span class="ms">"send"</span>
                                    {send_btn.clone()}
                                </button>
                                <p class="ni-muted" style="margin-top:10px;">{footnote_text.clone()}</p>
                                <Show when=move || !status_msg.get().is_empty()>
                                    <p class="ni-status">{move || status_msg.get()}</p>
                                </Show>
                            </div>
                        }.into_any()
                    }}
                </Suspense>
            </div>

            <Show when=move || show_history>
                <div class="ni-card ni-history">
                    <div class="ni-ct">"Recent invites"</div>
                    <Suspense fallback=move || view! { <p class="ni-muted">"Loading…"</p> }>
                        {move || {
                            let action_rows = history.get().unwrap_or_default();
                            if action_rows.is_empty() {
                                view! { <p class="ni-muted">"No invites sent yet."</p> }.into_any()
                            } else {
                                view! {
                                    <table class="ni-table">
                                        <thead>
                                            <tr>
                                                <th>"Invitee"</th>
                                                <th>"Role"</th>
                                                <th>"Status"</th>
                                                <th>"Outcome"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {action_rows.into_iter().map(|r| {
                                                let (pill_class, pill_label) = status_pill(&r.status);
                                                let outcome = outcome_label(&r.outcome_type, &r.outcome_status);
                                                let role = r.target_role.as_deref().map(role_label).unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td>{r.target_email.unwrap_or_else(|| "-".into())}</td>
                                                        <td>{role}</td>
                                                        <td><span class=pill_class>{pill_label}</span></td>
                                                        <td>{outcome}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </Show>
        </div>
        <style>{NI_CSS}</style>
    }
}

#[component]
fn InviteRowEditor(
    row_id: u32,
    rows: RwSignal<Vec<InviteRow>>,
    target_roles: RwSignal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="ni-row">
            <div class="ni-chip">
                <span class="ms">"mail"</span>
                <input type="email" placeholder="name@email.com"
                    prop:value=move || {
                        rows.get().iter().find(|r| r.id == row_id)
                            .map(|r| r.email.clone()).unwrap_or_default()
                    }
                    on:input=move |e| {
                        let v = event_target_value(&e);
                        rows.update(|rs| {
                            if let Some(r) = rs.iter_mut().find(|r| r.id == row_id) {
                                r.email = v;
                            }
                        });
                    }/>
            </div>
            <select class="ni-select"
                prop:value=move || {
                    rows.get().iter().find(|r| r.id == row_id)
                        .map(|r| r.role.clone()).unwrap_or_default()
                }
                on:change=move |e| {
                    let v = event_target_value(&e);
                    rows.update(|rs| {
                        if let Some(r) = rs.iter_mut().find(|r| r.id == row_id) {
                            r.role = v;
                        }
                    });
                }>
                {move || target_roles.get().into_iter().map(|t| {
                    let label = role_label(&t);
                    view! { <option value=t.clone()>{label}</option> }
                }).collect_view()}
            </select>
            <Show when=move || { rows.get().len() > 1 }>
                <button type="button" class="ni-remove" title="Remove"
                    on:click=move |_| {
                        rows.update(|rs| rs.retain(|r| r.id != row_id));
                    }>
                    <span class="ms">"close"</span>
                </button>
            </Show>
        </div>
    }
}

const NI_CSS: &str = r#"
.ni-panel{margin-top:4px}
.ni-stats{display:grid;grid-template-columns:repeat(3,1fr);gap:12px;margin-bottom:16px}
.ni-stat{background:#fff;border:1px solid #e2e8f0;border-radius:12px;padding:16px;box-shadow:0 1px 3px rgba(0,0,0,.06)}
.ni-stat-v{font-size:22px;font-weight:800}.ni-stat-l{font-size:12px;color:#64748b;margin-top:2px}
.ni-card{background:#fff;border:1px solid #e2e8f0;border-radius:12px;padding:22px;margin-bottom:14px;box-shadow:0 1px 3px rgba(0,0,0,.06)}
.ni-ct{font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.07em;color:#64748b;margin-bottom:14px}
.ni-angles{display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:14px}
.ni-angle{border:1.5px solid #e2e8f0;border-radius:8px;padding:14px;background:#f8fafc}
.ni-angle-ico{font-size:22px;color:#0284c7;display:block;margin-bottom:6px}
.ni-angle-h{font-size:13px;font-weight:700;margin-bottom:4px}
.ni-angle-p{font-size:12px;color:#64748b;line-height:1.45}
.ni-row{display:flex;gap:10px;align-items:center;margin-bottom:10px;flex-wrap:wrap}
.ni-chip{flex:1;min-width:180px;display:flex;align-items:center;gap:8px;background:#f8fafc;border:1.5px solid #cbd5e1;border-radius:8px;padding:9px 12px}
.ni-chip input{border:none;background:none;outline:none;font:inherit;font-size:14px;flex:1;min-width:0}
.ni-select{background:#f8fafc;border:1.5px solid #cbd5e1;border-radius:8px;padding:10px 12px;font:inherit;font-size:14px;min-width:140px}
.ni-remove{background:none;border:none;cursor:pointer;color:#94a3b8;padding:6px;display:inline-flex}
.ni-remove:hover{color:#be123c}
.ni-add{display:flex;align-items:center;gap:6px;font-size:13px;font-weight:600;color:#0284c7;background:none;border:none;cursor:pointer;padding:4px 0;margin-bottom:12px;font-family:inherit}
.ni-note{margin-bottom:14px}
.ni-note label{display:block;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.06em;color:#64748b;margin-bottom:5px}
.ni-textarea{width:100%;min-height:72px;resize:vertical;background:#f8fafc;border:1.5px solid #cbd5e1;border-radius:8px;padding:10px 12px;font:inherit;font-size:14px}
.ni-btn{display:inline-flex;align-items:center;gap:7px;font-size:13px;font-weight:700;padding:10px 16px;border-radius:8px;border:none;cursor:pointer;background:#0f172a;color:#fff;font-family:inherit}
.ni-btn:disabled{opacity:.6;cursor:default}
.ni-muted{font-size:12px;color:#94a3b8}
.ni-status{font-size:13px;color:#047857;margin-top:8px}
.ni-history{margin-top:0}
.ni-table{width:100%;border-collapse:collapse;font-size:13px}
.ni-table th{text-align:left;font-size:11px;text-transform:uppercase;letter-spacing:.06em;color:#64748b;padding:0 0 8px;border-bottom:1px solid #e2e8f0}
.ni-table td{padding:10px 0;border-bottom:1px solid #e2e8f0;vertical-align:middle}
.ni-pill{display:inline-flex;align-items:center;padding:3px 8px;border-radius:20px;font-size:11px;font-weight:700}
.ni-pill-sent{background:rgba(2,132,199,.1);color:#0284c7}
.ni-pill-joined{background:rgba(245,158,11,.12);color:#b45309}
.ni-pill-done{background:rgba(16,185,129,.12);color:#047857}
@media(max-width:600px){.ni-angles,.ni-stats{grid-template-columns:1fr}}
"#;
