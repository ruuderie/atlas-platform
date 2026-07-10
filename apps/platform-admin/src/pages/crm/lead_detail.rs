/// Lead Detail Page — full stitch-aligned implementation for atlas_leads
use leptos::prelude::*;
use crate::api::crm::{
    get_contact_notes, add_contact_note, get_contact_activities,
    convert_lead, update_lead, log_contact_activity,
};
use crate::api::models::{CrmNote, CrmActivity, LeadModel};

fn fmt_opt(v: &Option<String>) -> String {
    v.as_deref().unwrap_or("—").to_string()
}

fn ini2(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .map(|c| c.to_uppercase().to_string())
        .collect()
}

#[component]
pub fn LeadDetail(
    lead: LeadModel,
    #[prop(into)] on_email: Callback<()>,
    #[prop(into)] on_convert_done: Callback<()>,
) -> impl IntoView {
    let l = lead.clone();
    let id = l.id.clone();

    let toast        = use_context::<crate::app::GlobalToast>().expect("toast");
    let active_tab   = RwSignal::new("overview");
    let note_content = RwSignal::new(String::new());
    let call_notes   = RwSignal::new(String::new());
    let (trigger, set_trigger) = signal(0_u32);

    // ── Async resources ──────────────────────────────────────────────────────
    let notes_res: LocalResource<Vec<CrmNote>> = {
        let id2 = id.clone();
        LocalResource::new(move || {
            trigger.get();
            let i = id2.clone();
            async move { get_contact_notes(&i).await.unwrap_or_default() }
        })
    };
    let activities_res: LocalResource<Vec<CrmActivity>> = {
        let id2 = id.clone();
        LocalResource::new(move || {
            trigger.get();
            let i = id2.clone();
            async move { get_contact_activities(&i).await.unwrap_or_default() }
        })
    };

    // ── Handlers ─────────────────────────────────────────────────────────────
    let handle_note = StoredValue::new({
        let id2 = id.clone();
        let toast2 = toast.clone();
        move |_| {
            let content = note_content.get();
            if content.trim().is_empty() { return; }
            let id3 = id2.clone();
            let toast3 = toast2.clone();
            leptos::task::spawn_local(async move {
                match add_contact_note(&id3, &content).await {
                    Ok(_) => {
                        toast3.show_toast("Note", "Note saved.", "success");
                        note_content.set(String::new());
                        set_trigger.update(|v| *v += 1);
                    }
                    Err(e) => toast3.show_toast("Error", &e, "error"),
                }
            });
        }
    });

    let handle_call = StoredValue::new({
        let id2 = id.clone();
        let toast2 = toast.clone();
        move |_| {
            let notes = call_notes.get();
            let id3 = id2.clone();
            let toast3 = toast2.clone();
            leptos::task::spawn_local(async move {
                match log_contact_activity(&id3, "call", &notes).await {
                    Ok(_) => {
                        toast3.show_toast("Call", "Call logged.", "success");
                        call_notes.set(String::new());
                        set_trigger.update(|v| *v += 1);
                    }
                    Err(e) => toast3.show_toast("Error", &e, "error"),
                }
            });
        }
    });

    let handle_convert = StoredValue::new({
        let id2 = id.clone();
        let toast2 = toast.clone();
        move |_| {
            let id3 = id2.clone();
            let toast3 = toast2.clone();
            leptos::task::spawn_local(async move {
                match convert_lead(&id3).await {
                    Ok(_) => {
                        toast3.show_toast("Converted", "Lead converted to Contact + Account.", "success");
                        on_convert_done.run(());
                    }
                    Err(e) => toast3.show_toast("Error", &e, "error"),
                }
            });
        }
    });

    let handle_status = StoredValue::new({
        let id2 = id.clone();
        let toast2 = toast.clone();
        move |new_status: String| {
            let id3 = id2.clone();
            let toast3 = toast2.clone();
            leptos::task::spawn_local(async move {
                match update_lead(&id3, &new_status).await {
                    Ok(_) => {
                        toast3.show_toast("Status", &format!("Lead moved to {}", new_status), "success");
                        set_trigger.update(|v| *v += 1);
                    }
                    Err(e) => toast3.show_toast("Error", &e, "error"),
                }
            });
        }
    });

    // ── Pre-extracted display values ─────────────────────────────────────────
    let name         = l.name.clone();
    let initials     = ini2(&name);
    let email_val    = fmt_opt(&l.email);
    let phone_val    = fmt_opt(&l.phone);
    let company_val  = fmt_opt(&l.company);
    let title_val    = fmt_opt(&l.title);
    let source_val   = fmt_opt(&l.source);
    let status_val   = l.lead_status.clone().unwrap_or_else(|| "new".into());
    let is_converted = l.is_converted;

    let subtitle = {
        let mut p = Vec::new();
        if title_val != "—"   { p.push(title_val.clone()); }
        if company_val != "—" { p.push(company_val.clone()); }
        p.join(" · ")
    };

    // Stage stepper data
    let stages: &[&str] = &["New", "Contacted", "Qualified", "Proposal"];
    let current_idx = stages.iter()
        .position(|&s| s.to_lowercase() == status_val.to_lowercase())
        .unwrap_or(0);
    let is_disqualified  = status_val.to_lowercase() == "disqualified";
    let is_conv          = is_converted || status_val.to_lowercase() == "converted";
    let terminal_class   = if is_disqualified { "sf-step terminal-lost" }
                           else if is_conv    { "sf-step terminal-won"  }
                           else               { "sf-step future"        };
    let terminal_label   = if is_disqualified { "Disqualified" } else { "Converted" };

    let sc = move |i: usize| -> &'static str {
        if is_conv { return "sf-step done"; }
        if i < current_idx { "sf-step done" }
        else if i == current_idx { "sf-step current" }
        else { "sf-step future" }
    };

    // Detail rows
    let detail_rows = StoredValue::new(vec![
        ("Lead ID",    l.id.clone(),                    true),
        ("Name",       l.name.clone(),                  false),
        ("First Name", fmt_opt(&l.first_name),          false),
        ("Last Name",  fmt_opt(&l.last_name),           false),
        ("Email",      fmt_opt(&l.email),               false),
        ("Phone",      fmt_opt(&l.phone),               false),
        ("WhatsApp",   fmt_opt(&l.whatsapp),            false),
        ("Telegram",   fmt_opt(&l.telegram),            false),
        ("Company",    fmt_opt(&l.company),             false),
        ("Title",      fmt_opt(&l.title),               false),
        ("Source",     fmt_opt(&l.source),              false),
        ("Status",     fmt_opt(&l.lead_status),        false),
        ("Converted",  if l.is_converted { "Yes".into() } else { "No".into() }, false),
        ("Created At", fmt_opt(&l.created_at),          true),
        ("Updated At", fmt_opt(&l.updated_at),          true),
    ]);

    // Right-rail info rows
    let info_rows = StoredValue::new(vec![
        ("Status",  status_val.clone()),
        ("Source",  source_val.clone()),
        ("Company", company_val.clone()),
        ("Title",   title_val.clone()),
        ("Email",   email_val.clone()),
        ("Phone",   phone_val.clone()),
    ]);

    view! {
        <div style="display:flex;flex-direction:column;height:100%;overflow:hidden;">

            // ── Hero ─────────────────────────────────────────────────────────
            <div style="padding:14px 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div style="font-size:11px;color:var(--text-muted);margin-bottom:8px;display:flex;align-items:center;gap:5px;">
                    <a href="/leads" style="color:var(--text-link);text-decoration:none;">"Leads"</a>
                    " › "
                    {name.clone()}
                </div>
                <div style="display:flex;align-items:flex-start;justify-content:space-between;gap:12px;">
                    <div style="display:flex;gap:14px;align-items:flex-start;">
                        <div style="width:52px;height:52px;border-radius:50%;background:var(--amber-dim,rgba(245,158,11,0.12));border:1px solid var(--amber,#f59e0b);display:flex;align-items:center;justify-content:center;font-size:18px;font-weight:700;color:var(--amber,#f59e0b);flex-shrink:0;">
                            {initials}
                        </div>
                        <div>
                            <div style="font-size:20px;font-weight:700;letter-spacing:-0.3px;display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:4px;">
                                {name.clone()}
                                {is_converted.then(|| view! {
                                    <span class="tag" style="color:var(--green);border-color:var(--green);">"Converted"</span>
                                })}
                            </div>
                            {(!subtitle.is_empty()).then(|| view! {
                                <div style="font-size:12px;color:var(--text-secondary);margin-bottom:8px;">{subtitle.clone()}</div>
                            })}
                            <div style="display:flex;gap:6px;flex-wrap:wrap;">
                                <button class="btn btn-ghost btn-sm" on:click=move |_| on_email.run(())>"✉ Email"</button>
                                <button class="btn btn-ghost btn-sm" on:click=move |e| handle_call.get_value()(e)>"📞 Log Call"</button>
                                {(!is_converted).then(|| view! {
                                    <button class="btn btn-convert btn-sm" on:click=move |e| handle_convert.get_value()(e)>"⇉ Convert Lead"</button>
                                })}
                            </div>
                        </div>
                    </div>
                    <div style="text-align:right;font-size:10px;color:var(--text-muted);font-family:monospace;flex-shrink:0;">
                        {l.id.clone()}
                    </div>
                </div>
            </div>

            // ── Stage Stepper ─────────────────────────────────────────────────
            <div class="status-flow" style="padding:10px 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;display:flex;align-items:center;gap:6px;flex-wrap:wrap;">
                <div class={sc(0)}><div class="sf-pill"
                    on:click=move |_| handle_status.get_value()("new".into())
                    style="cursor:pointer;">"New"</div></div>
                <div class="sf-arrow">"→"</div>
                <div class={sc(1)}><div class="sf-pill"
                    on:click=move |_| handle_status.get_value()("contacted".into())
                    style="cursor:pointer;">"Contacted"</div></div>
                <div class="sf-arrow">"→"</div>
                <div class={sc(2)}><div class="sf-pill"
                    on:click=move |_| handle_status.get_value()("qualified".into())
                    style="cursor:pointer;">"Qualified"</div></div>
                <div class="sf-arrow">"→"</div>
                <div class={sc(3)}><div class="sf-pill"
                    on:click=move |_| handle_status.get_value()("proposal".into())
                    style="cursor:pointer;">"Proposal"</div></div>
                <div class="sf-arrow">"→"</div>
                <div class={terminal_class}><div class="sf-pill">{terminal_label}</div></div>
            </div>

            // ── KPI Strip ─────────────────────────────────────────────────────
            <div style="display:flex;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div class="kpi">
                    <div class="kpi-label">"Stage"</div>
                    <div class="kpi-value" style="color:var(--amber,#f59e0b);">{status_val.clone()}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Source"</div>
                    <div class="kpi-value">{source_val.clone()}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Activities"</div>
                    <div class="kpi-value mono">{move || activities_res.get().map(|a| a.len().to_string()).unwrap_or_else(|| "—".into())}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Notes"</div>
                    <div class="kpi-value mono">{move || notes_res.get().map(|n| n.len().to_string()).unwrap_or_else(|| "—".into())}</div>
                </div>
            </div>

            // ── Tab Bar ───────────────────────────────────────────────────────
            <div class="tab-bar" style="display:flex;padding:0 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <button class=move || format!("tab {}", if active_tab.get() == "overview" { "active" } else { "" }) on:click=move |_| active_tab.set("overview")>"Overview"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "activity" { "active" } else { "" }) on:click=move |_| active_tab.set("activity")>"Activity"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "details"  { "active" } else { "" }) on:click=move |_| active_tab.set("details") >"Details · atlas_leads"</button>
            </div>

            // ── Tab Content ───────────────────────────────────────────────────
            <div class="content-body" style="flex:1;overflow-y:auto;padding:20px 24px;">
                {move || match active_tab.get() {

                    // ── Overview ──────────────────────────────────────────────
                    "overview" => view! {
                        <div class="col-7-5">
                            // Left column
                            <div>
                                // Convert panel (shown if not yet converted)
                                {(!is_converted).then(|| view! {
                                    <div class="convert-panel" style="margin-bottom:14px;">
                                        <div class="convert-panel-hdr">"⇉ Convert this Lead"</div>
                                        <div class="convert-panel-body">
                                            "Converting will create a Contact, Account, and Opportunity in one atomic operation."
                                        </div>
                                        <button class="btn btn-convert btn-sm" on:click=move |e| handle_convert.get_value()(e)>"Qualification Conversion →"</button>
                                    </div>
                                })}

                                // Note composer
                                <div class="card" style="margin-bottom:14px;">
                                    <div class="composer" style="padding:12px 14px;">
                                        <div style="display:flex;gap:4px;margin-bottom:8px;">
                                            <button class="c-tab active">"Note"</button>
                                        </div>
                                        <textarea
                                            style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:5px;padding:8px 10px;font-size:12px;color:var(--text-primary);font-family:inherit;resize:none;outline:none;min-height:60px;"
                                            placeholder="Add a note…"
                                            prop:value=move || note_content.get()
                                            on:input=move |e| note_content.set(event_target_value(&e))
                                        ></textarea>
                                        <div style="display:flex;justify-content:flex-end;margin-top:6px;">
                                            <button class="btn btn-primary btn-sm" on:click=move |e| handle_note.get_value()(e)>"Save Note"</button>
                                        </div>
                                    </div>
                                </div>

                                // Activity timeline
                                <div class="card">
                                    <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Activity Timeline · G-29"</span>
                                    </div>
                                    <Suspense fallback=move || view! { <div style="padding:16px;color:var(--text-muted);font-size:12px;">"Loading…"</div> }>
                                        {move || {
                                            let notes = notes_res.get().unwrap_or_default();
                                            let acts  = activities_res.get().unwrap_or_default();
                                            if notes.is_empty() && acts.is_empty() {
                                                return view! {
                                                    <div style="padding:24px;text-align:center;color:var(--text-muted);font-size:12px;">"No activity yet. Add a note above."</div>
                                                }.into_any();
                                            }
                                            view! {
                                                <div>
                                                    {notes.into_iter().map(|n| view! {
                                                        <div style="display:flex;gap:12px;padding:10px 14px;border-bottom:1px solid var(--border-subtle);">
                                                            <div style="width:28px;height:28px;border-radius:50%;background:var(--cobalt-dim);color:var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:11px;flex-shrink:0;">"📝"</div>
                                                            <div style="flex:1;">
                                                                <div style="font-size:12px;font-weight:500;">"Note added"</div>
                                                                <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px;">{n.created_at}</div>
                                                                <div style="font-size:11.5px;color:var(--text-secondary);margin-top:4px;line-height:1.5;">{n.content}</div>
                                                            </div>
                                                        </div>
                                                    }).collect::<Vec<_>>()}
                                                    {acts.into_iter().map(|a| {
                                                        let icon = match a.activity_type.as_str() {
                                                            "call"  => "📞",
                                                            "email" => "📧",
                                                            _       => "⚙",
                                                        };
                                                        view! {
                                                            <div style="display:flex;gap:12px;padding:10px 14px;border-bottom:1px solid var(--border-subtle);">
                                                                <div style="width:28px;height:28px;border-radius:50%;background:var(--green-dim);color:var(--green);display:flex;align-items:center;justify-content:center;font-size:11px;flex-shrink:0;">{icon}</div>
                                                                <div style="flex:1;">
                                                                    <div style="font-size:12px;font-weight:500;">{a.activity_type}</div>
                                                                    <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px;">{a.created_at}</div>
                                                                    <div style="font-size:11.5px;color:var(--text-secondary);margin-top:4px;line-height:1.5;">{a.description}</div>
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        }}
                                    </Suspense>
                                </div>
                            </div>

                            // Right rail
                            <div>
                                <crate::pages::billing::scorecard_panel::ScorecardPanel
                                    entity_type="atlas_lead".to_string()
                                    entity_id=id.clone()
                                    subject_label=name.clone()
                                />
                                <div class="card">
                                    <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Lead Info"</span>
                                    </div>
                                    {info_rows.get_value().into_iter().map(|(label, value)| {
                                        let is_set = value != "—";
                                        let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                        view! {
                                            <div style="display:flex;justify-content:space-between;align-items:center;padding:7px 14px;border-bottom:1px solid var(--border-subtle);">
                                                <span style="font-size:12px;color:var(--text-secondary);">{label}</span>
                                                <span style=format!("font-size:12px;font-weight:500;color:{};", color)>{value}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    // ── Activity ──────────────────────────────────────────────
                    "activity" => view! {
                        <div>
                            // Call logger
                            <div class="card" style="margin-bottom:14px;">
                                <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                    <span class="card-title" style="font-size:11.5px;font-weight:600;">"Log a Call"</span>
                                </div>
                                <div style="padding:12px 14px;">
                                    <textarea
                                        style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:5px;padding:8px 10px;font-size:12px;color:var(--text-primary);font-family:inherit;resize:none;outline:none;min-height:60px;"
                                        placeholder="Call notes…"
                                        prop:value=move || call_notes.get()
                                        on:input=move |e| call_notes.set(event_target_value(&e))
                                    ></textarea>
                                    <div style="display:flex;justify-content:flex-end;margin-top:6px;">
                                        <button class="btn btn-primary btn-sm" on:click=move |e| handle_call.get_value()(e)>"Log Call"</button>
                                    </div>
                                </div>
                            </div>
                            // Full timeline
                            <div class="card">
                                <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                    <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Activity"</span>
                                </div>
                                <Suspense>
                                    {move || activities_res.get().unwrap_or_default().into_iter().map(|a| {
                                        let icon = match a.activity_type.as_str() {
                                            "call"  => "📞",
                                            "email" => "📧",
                                            _       => "⚙",
                                        };
                                        view! {
                                            <div style="display:flex;gap:12px;padding:10px 14px;border-bottom:1px solid var(--border-subtle);">
                                                <div style="width:28px;height:28px;border-radius:50%;background:var(--green-dim);color:var(--green);display:flex;align-items:center;justify-content:center;font-size:11px;flex-shrink:0;">{icon}</div>
                                                <div style="flex:1;">
                                                    <div style="font-size:12px;font-weight:500;">{a.activity_type}</div>
                                                    <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px;">{a.created_at}</div>
                                                    <div style="font-size:11.5px;color:var(--text-secondary);margin-top:4px;line-height:1.5;">{a.description}</div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </Suspense>
                            </div>
                        </div>
                    }.into_any(),

                    // ── Details: full atlas_leads field grid ──────────────────
                    "details" => view! {
                        <div class="card">
                            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Fields · atlas_leads"</span>
                            </div>
                            <div style="display:grid;grid-template-columns:1fr 1fr;">
                                {detail_rows.get_value().into_iter().enumerate().map(|(i, (label, value, mono))| {
                                    let border_right = if i % 2 == 0 { "1px solid var(--border-subtle)" } else { "none" };
                                    let is_set = value != "—";
                                    let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                    let font_family = if mono { "font-family:monospace;" } else { "" };
                                    view! {
                                        <div style=format!("display:flex;flex-direction:column;padding:8px 14px;border-bottom:1px solid var(--border-subtle);border-right:{};", border_right)>
                                            <span style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);margin-bottom:2px;">{label}</span>
                                            <span style=format!("font-size:12px;color:{};{}", color, font_family)>{value}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any(),

                    _ => view! { <div></div> }.into_any(),
                }}
            </div>
        </div>
    }
}
