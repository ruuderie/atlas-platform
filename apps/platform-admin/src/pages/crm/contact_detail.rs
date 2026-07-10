/// Contact Detail Page — full stitch-aligned implementation
use leptos::prelude::*;
use crate::api::crm::{add_contact_note, get_contact_notes, get_contact_activities};
use crate::api::models::{ContactModel, CrmNote, CrmActivity};

fn ini2(s: &str) -> String {
    s.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .map(|c| c.to_uppercase().to_string())
        .collect()
}

fn fmt_opt(v: &Option<String>) -> String {
    v.as_deref().unwrap_or("—").to_string()
}

#[component]
pub fn ContactDetail(
    contact: ContactModel,
    #[prop(into)] on_email: Callback<()>,
    #[prop(into)] on_call: Callback<()>,
) -> impl IntoView {
    let c = contact.clone();
    let id = c.id.clone();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    let active_tab   = RwSignal::new("overview");
    let note_content = RwSignal::new(String::new());
    let (trigger, set_trigger) = signal(0_u32);

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

    let handle_note = StoredValue::new({
        let id2 = id.clone();
        move |_| {
            let content = note_content.get();
            if content.trim().is_empty() { return; }
            let id3 = id2.clone();
            let toast2 = toast.clone();
            leptos::task::spawn_local(async move {
                match add_contact_note(&id3, &content).await {
                    Ok(_) => {
                        toast2.show_toast("Note", "Note saved.", "success");
                        note_content.set(String::new());
                        set_trigger.update(|v| *v += 1);
                    }
                    Err(e) => toast2.show_toast("Error", &e, "error"),
                }
            });
        }
    });

    // ── Pre-computed display values (no closures over c needed in view) ─────────
    let display      = c.display_name().to_string();
    let initials     = ini2(&display);
    let subtitle     = {
        let mut p = Vec::new();
        if let Some(ref t) = c.title      { p.push(t.clone()); }
        if let Some(ref d) = c.department { p.push(d.clone()); }
        p.join(" · ")
    };
    let email_val      = c.email.clone().unwrap_or_default();
    let email_verified = c.email_verified;
    let phone_val      = c.phone.clone().unwrap_or_default();
    let phone_verified = c.phone_verified;
    let is_primary     = c.is_primary;
    let account_id     = StoredValue::new(c.account_id.clone());
    let whatsapp_val   = c.whatsapp.clone().unwrap_or_default();
    let linkedin_val   = c.linkedin_url.clone().unwrap_or_default();
    let contact_id_display = c.id.clone();

    // ── Rows stored reactively to avoid FnOnce captures ─────────────────────
    let channel_rows = StoredValue::new(vec![
        ("Email",    fmt_opt(&c.email),        c.email_verified),
        ("Phone",    fmt_opt(&c.phone),        c.phone_verified),
        ("WhatsApp", fmt_opt(&c.whatsapp),     false),
        ("LinkedIn", fmt_opt(&c.linkedin_url), false),
        ("Telegram", fmt_opt(&c.telegram),     false),
    ]);

    let identity_rows = StoredValue::new(vec![
        ("First Name",  fmt_opt(&c.first_name)),
        ("Last Name",   fmt_opt(&c.last_name)),
        ("Full Name",   fmt_opt(&c.full_name)),
        ("Title",       fmt_opt(&c.title)),
        ("Department",  fmt_opt(&c.department)),
        ("Data Source", fmt_opt(&c.data_source)),
    ]);

    let detail_rows = StoredValue::new(vec![
        ("Contact ID",      c.id.clone(),                         true),
        ("Account ID",      c.account_id.clone(),                 true),
        ("First Name",      fmt_opt(&c.first_name),               false),
        ("Last Name",       fmt_opt(&c.last_name),                false),
        ("Full Name",       fmt_opt(&c.full_name),                false),
        ("Preferred Name",  fmt_opt(&c.preferred_name),           false),
        ("Title",           fmt_opt(&c.title),                    false),
        ("Department",      fmt_opt(&c.department),               false),
        ("Is Primary",      if c.is_primary { "Yes".into() } else { "No".into() }, false),
        ("Email",           fmt_opt(&c.email),                    false),
        ("Email Verified",  if c.email_verified { "Yes · MillionVerifier".into() } else { "No".into() }, false),
        ("Phone",           fmt_opt(&c.phone),                    false),
        ("Phone Verified",  if c.phone_verified { "Yes".into() } else { "No".into() }, false),
        ("WhatsApp",        fmt_opt(&c.whatsapp),                 false),
        ("Telegram",        fmt_opt(&c.telegram),                 false),
        ("LinkedIn URL",    fmt_opt(&c.linkedin_url),             false),
        ("Avatar URL",      fmt_opt(&c.avatar_url),               false),
        ("Data Source",     fmt_opt(&c.data_source),              false),
        ("Created At",      fmt_opt(&c.created_at),               true),
        ("Updated At",      fmt_opt(&c.updated_at),               true),
    ]);

    view! {
        <div style="display:flex;flex-direction:column;height:100%;overflow:hidden;">

            // ── Hero Header ──────────────────────────────────────────────────
            <div class="rec-hdr" style="padding:14px 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div class="breadcrumb" style="font-size:11px;color:var(--text-muted);margin-bottom:8px;display:flex;align-items:center;gap:5px;">
                    <a href="/contacts" style="color:var(--text-link);text-decoration:none;">"Contacts"</a>
                    " › "
                    {display.clone()}
                </div>
                <div style="display:flex;align-items:flex-start;justify-content:space-between;">
                    <div style="display:flex;gap:14px;align-items:flex-start;">
                        <div style="width:52px;height:52px;border-radius:50%;background:var(--violet-dim);border:1px solid var(--violet);display:flex;align-items:center;justify-content:center;font-size:18px;font-weight:700;color:var(--violet);flex-shrink:0;">
                            {initials.clone()}
                        </div>
                        <div>
                            <div style="font-size:20px;font-weight:700;letter-spacing:-0.3px;display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:3px;">
                                {display.clone()}
                                {is_primary.then(|| view! {
                                    <span class="tag" style="color:var(--amber);border-color:var(--amber);background:var(--amber-dim);font-size:9px;">"Primary"</span>
                                })}
                                {email_verified.then(|| view! {
                                    <span class="tag" style="color:var(--green);border-color:var(--green);background:var(--green-dim);font-size:9px;">"✓ Verified"</span>
                                })}
                            </div>
                            {(!subtitle.is_empty()).then(|| view! {
                                <div style="font-size:12.5px;color:var(--text-secondary);margin-bottom:8px;">{subtitle.clone()}</div>
                            })}
                            // Channel row
                            <div style="display:flex;gap:12px;flex-wrap:wrap;margin-bottom:10px;">
                                {(!email_val.is_empty()).then(|| {
                                    let ev = email_val.clone();
                                    view! {
                                        <span style="display:flex;align-items:center;gap:5px;font-size:12px;color:var(--text-link);">
                                            <span style="width:20px;height:20px;border-radius:4px;background:var(--cobalt-dim);color:var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:10px;">"✉"</span>
                                            {ev}
                                            {email_verified.then(|| view! { <span style="color:var(--green);font-size:10px;font-weight:600;">"✓"</span> })}
                                        </span>
                                    }
                                })}
                                {(!phone_val.is_empty()).then(|| {
                                    let pv = phone_val.clone();
                                    view! {
                                        <span style="display:flex;align-items:center;gap:5px;font-size:12px;color:var(--text-secondary);">
                                            <span style="width:20px;height:20px;border-radius:4px;background:var(--green-dim);color:var(--green);display:flex;align-items:center;justify-content:center;font-size:10px;">"☎"</span>
                                            {pv}
                                            {phone_verified.then(|| view! { <span style="color:var(--green);font-size:10px;font-weight:600;">"✓"</span> })}
                                        </span>
                                    }
                                })}
                                {(!whatsapp_val.is_empty()).then(|| {
                                    let wv = whatsapp_val.clone();
                                    view! {
                                        <span style="display:flex;align-items:center;gap:5px;font-size:12px;color:#25D366;">
                                            <span style="width:20px;height:20px;border-radius:4px;background:rgba(37,211,102,0.12);color:#25D366;display:flex;align-items:center;justify-content:center;font-size:10px;font-weight:700;">"W"</span>
                                            {wv}
                                        </span>
                                    }
                                })}
                                {(!linkedin_val.is_empty()).then(|| {
                                    let lv = linkedin_val.clone();
                                    let lv2 = lv.clone();
                                    view! {
                                        <a href={lv} target="_blank" style="display:flex;align-items:center;gap:5px;font-size:12px;color:#0A66C2;text-decoration:none;">
                                            <span style="width:20px;height:20px;border-radius:4px;background:rgba(10,102,194,0.12);color:#0A66C2;display:flex;align-items:center;justify-content:center;font-size:10px;font-weight:700;">"in"</span>
                                            {lv2}
                                        </a>
                                    }
                                })}
                            </div>
                            // Action buttons
                            <div style="display:flex;gap:6px;flex-wrap:wrap;">
                                <button class="btn btn-ghost btn-sm" on:click=move |_| on_email.run(())>"✉ Email"</button>
                                <button class="btn btn-ghost btn-sm" on:click=move |_| on_call.run(())>"📞 Log Call"</button>
                                {(!phone_val.is_empty()).then(|| {
                                    let wa_num = phone_val.chars().filter(|c| c.is_numeric()).collect::<String>();
                                    view! {
                                        <a class="btn btn-ghost btn-sm"
                                            href={format!("https://wa.me/{}", wa_num)}
                                            target="_blank"
                                        >"💬 WhatsApp"</a>
                                    }
                                })}
                            </div>
                        </div>
                    </div>
                    // Top-right: account link
                    <div style="text-align:right;font-size:11px;color:var(--text-muted);flex-shrink:0;">
                        {(!account_id.get_value().is_empty()).then(|| {
                            let aid = account_id.get_value();
                            let aid2 = aid.clone();
                            view! {
                                <div>
                                    "Account: "
                                    <a href={format!("/accounts/{}", aid)} style="color:var(--text-link);text-decoration:none;">{aid2}</a>
                                </div>
                            }
                        })}
                        <div style="margin-top:3px;font-family:monospace;font-size:10px;">{contact_id_display.clone()}</div>
                    </div>
                </div>
            </div>

            // ── KPI Strip ────────────────────────────────────────────────────
            <div style="display:flex;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div class="kpi">
                    <div class="kpi-label">"Activities"</div>
                    <div class="kpi-value mono">{move || activities_res.get().map(|a| a.len().to_string()).unwrap_or_else(|| "—".into())}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Notes"</div>
                    <div class="kpi-value mono">{move || notes_res.get().map(|n| n.len().to_string()).unwrap_or_else(|| "—".into())}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Email"</div>
                    <div class="kpi-value" style={if email_verified { "color:var(--green)" } else { "color:var(--text-muted)" }}>
                        {if email_verified { "Verified" } else { "Unverified" }}
                    </div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Phone"</div>
                    <div class="kpi-value" style={if phone_verified { "color:var(--green)" } else { "color:var(--text-muted)" }}>
                        {if phone_verified { "Verified" } else if !phone_val.is_empty() { "Unverified" } else { "—" }}
                    </div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Open Opps"</div>
                    <div class="kpi-value" style="color:var(--text-muted);">"—"</div>
                </div>
            </div>

            // ── Tab Bar ───────────────────────────────────────────────────────
            <div class="tab-bar" style="display:flex;padding:0 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <button class=move || format!("tab {}", if active_tab.get() == "overview" { "active" } else { "" }) on:click=move |_| active_tab.set("overview")>"Overview"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "details"  { "active" } else { "" }) on:click=move |_| active_tab.set("details") >"Details · atlas_contacts"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "activity" { "active" } else { "" }) on:click=move |_| active_tab.set("activity")>"Activity"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "notes"    { "active" } else { "" }) on:click=move |_| active_tab.set("notes")   >"Notes"</button>
            </div>

            // ── Tab Content ───────────────────────────────────────────────────
            <div class="content-body" style="flex:1;overflow-y:auto;padding:20px 24px;">
                {move || match active_tab.get() {

                    // ── Overview ──────────────────────────────────────────────
                    "overview" => view! {
                        <div class="col-7-5">
                            // Left column
                            <div>
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
                                    <div class="card-hdr" style="display:flex;align-items:center;padding:9px 14px;border-bottom:1px solid var(--border-default);">
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
                                                        let (bg, color) = match a.activity_type.as_str() {
                                                            "call"  => ("var(--cobalt-dim)", "var(--cobalt)"),
                                                            "email" => ("var(--violet-dim)", "var(--violet)"),
                                                            _       => ("var(--green-dim)",  "var(--green)"),
                                                        };
                                                        let icon = match a.activity_type.as_str() {
                                                            "call"  => "📞",
                                                            "email" => "📧",
                                                            _       => "⚙",
                                                        };
                                                        view! {
                                                            <div style="display:flex;gap:12px;padding:10px 14px;border-bottom:1px solid var(--border-subtle);">
                                                                <div style=format!("width:28px;height:28px;border-radius:50%;background:{};color:{};display:flex;align-items:center;justify-content:center;font-size:11px;flex-shrink:0;", bg, color)>{icon}</div>
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
                                // Contact Channels card
                                <div class="card" style="margin-bottom:14px;">
                                    <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Contact Channels"</span>
                                    </div>
                                    {channel_rows.get_value().into_iter().map(|(label, value, verified)| {
                                        let is_set = value != "—";
                                        let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                        view! {
                                            <div style="display:flex;align-items:center;justify-content:space-between;padding:7px 14px;border-bottom:1px solid var(--border-subtle);">
                                                <span style="font-size:12px;color:var(--text-secondary);">{label}</span>
                                                <div style="text-align:right;">
                                                    <div style=format!("font-size:11.5px;color:{};", color)>{value.clone()}</div>
                                                    {(verified && is_set).then(|| view! {
                                                        <div style="font-size:10px;color:var(--green);">"✓ verified"</div>
                                                    })}
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                // Account card
                                {(!account_id.get_value().is_empty()).then(|| {
                                    let aid = account_id.get_value();
                                    let aid2 = aid.clone();
                                    view! {
                                        <div class="card" style="margin-bottom:14px;">
                                            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"Account"</span>
                                            </div>
                                            <a href={format!("/accounts/{}", aid)}
                                                style="display:flex;align-items:center;gap:10px;padding:12px 14px;text-decoration:none;">
                                                <div style="width:30px;height:30px;border-radius:5px;background:var(--cobalt-dim);border:1px solid var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:700;color:var(--cobalt);flex-shrink:0;">"·"</div>
                                                <div>
                                                    <div style="font-size:12.5px;font-weight:600;color:var(--text-link);">{aid2}</div>
                                                    <div style="font-size:11px;color:var(--text-muted);">"View account →"</div>
                                                </div>
                                            </a>
                                        </div>
                                    }
                                })}

                                // Identity & Professional card
                                <div class="card">
                                    <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                        <span class="card-title" style="font-size:11.5px;font-weight:600;">"Identity & Professional"</span>
                                    </div>
                                    {identity_rows.get_value().into_iter().map(|(label, value)| {
                                        let is_set = value != "—";
                                        let color = if is_set { "var(--text-primary)" } else { "var(--text-muted)" };
                                        view! {
                                            <div style="display:flex;align-items:center;justify-content:space-between;padding:7px 14px;border-bottom:1px solid var(--border-subtle);">
                                                <span style="font-size:12px;color:var(--text-secondary);">{label}</span>
                                                <span style=format!("font-size:12px;font-weight:500;color:{};", color)>{value}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                    <div style="display:flex;align-items:center;justify-content:space-between;padding:7px 14px;">
                                        <span style="font-size:12px;color:var(--text-secondary);">"Primary Contact?"</span>
                                        <span style=format!("font-size:12px;font-weight:500;color:{};", if is_primary { "var(--green)" } else { "var(--text-muted)" })>
                                            {if is_primary { "Yes" } else { "No" }}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    // ── Details: full atlas_contacts field grid ───────────────
                    "details" => view! {
                        <div class="card">
                            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Fields · atlas_contacts"</span>
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

                    // ── Activity tab ──────────────────────────────────────────
                    "activity" => view! {
                        <div class="card">
                            <div class="card-hdr" style="display:flex;align-items:center;justify-content:space-between;padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Activities · G-29"</span>
                            </div>
                            <table style="width:100%;border-collapse:collapse;">
                                <thead>
                                    <tr>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Type"</th>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Description"</th>
                                        <th style="font-size:9.5px;font-weight:600;text-transform:uppercase;letter-spacing:0.06em;color:var(--text-muted);padding:6px 12px;text-align:left;border-bottom:1px solid var(--border-default);">"Date"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <Suspense>
                                        {move || activities_res.get().unwrap_or_default().into_iter().map(|a| view! {
                                            <tr style="border-bottom:1px solid var(--border-subtle);">
                                                <td style="padding:7px 12px;font-size:12px;">
                                                    <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt);">{a.activity_type}</span>
                                                </td>
                                                <td style="padding:7px 12px;font-size:12px;color:var(--text-secondary);">{a.description}</td>
                                                <td style="padding:7px 12px;font-size:11px;color:var(--text-muted);">{a.created_at}</td>
                                            </tr>
                                        }).collect::<Vec<_>>()}
                                    </Suspense>
                                </tbody>
                            </table>
                        </div>
                    }.into_any(),

                    // ── Notes tab ─────────────────────────────────────────────
                    "notes" => view! {
                        <div>
                            <div class="card" style="margin-bottom:14px;">
                                <div style="padding:12px 14px;">
                                    <textarea
                                        style="width:100%;background:var(--bg-elevated);border:1px solid var(--border-default);border-radius:5px;padding:8px 10px;font-size:12px;color:var(--text-primary);font-family:inherit;resize:none;outline:none;min-height:80px;"
                                        placeholder="Write a note…"
                                        prop:value=move || note_content.get()
                                        on:input=move |e| note_content.set(event_target_value(&e))
                                    ></textarea>
                                    <div style="display:flex;justify-content:flex-end;margin-top:6px;">
                                        <button class="btn btn-primary btn-sm" on:click=move |e| handle_note.get_value()(e)>"Save Note"</button>
                                    </div>
                                </div>
                            </div>
                            <div class="card">
                                <Suspense>
                                    {move || {
                                        let notes = notes_res.get().unwrap_or_default();
                                        if notes.is_empty() {
                                            return view! {
                                                <div style="padding:24px;text-align:center;color:var(--text-muted);font-size:12px;">"No notes yet."</div>
                                            }.into_any();
                                        }
                                        view! {
                                            <div>
                                                {notes.into_iter().map(|n| view! {
                                                    <div style="padding:12px 14px;border-bottom:1px solid var(--border-subtle);">
                                                        <div style="font-size:10.5px;color:var(--text-muted);margin-bottom:4px;">{n.created_at}</div>
                                                        <div style="font-size:12.5px;color:var(--text-primary);line-height:1.6;">{n.content}</div>
                                                    </div>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }}
                                </Suspense>
                            </div>
                        </div>
                    }.into_any(),

                    _ => view! { <div></div> }.into_any(),
                }}
            </div>
        </div>
    }
}
