/// Deal / Opportunity Detail Page — atlas_deals
use leptos::prelude::*;
use crate::api::crm::update_deal;
use crate::api::models::DealModel;

#[component]
pub fn DealDetail(deal: DealModel) -> impl IntoView {
    let d = deal.clone();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    // Stage stepper
    let stages: &[(&str, &str)] = &[
        ("prospecting",  "Prospecting"),
        ("qualifying",   "Qualifying"),
        ("proposal",     "Proposal"),
        ("negotiation",  "Negotiation"),
        ("closed_won",   "Closed Won"),
        ("closed_lost",  "Closed Lost"),
    ];

    let current_stage = d.stage.to_lowercase();
    let current_idx = stages.iter()
        .position(|(k, _)| *k == current_stage.as_str())
        .unwrap_or(0);
    let is_won  = current_stage == "closed_won";
    let is_lost = current_stage == "closed_lost";

    let sc = move |i: usize| -> &'static str {
        if is_won && i == 4  { return "sf-step terminal-won"; }
        if is_lost && i == 5 { return "sf-step terminal-lost"; }
        if is_won || is_lost { return if i < 4 { "sf-step done" } else { "sf-step future" }; }
        if i < current_idx   { "sf-step done" }
        else if i == current_idx { "sf-step current" }
        else { "sf-step future" }
    };

    let handle_stage = StoredValue::new({
        let id2 = d.id.clone();
        let status2 = d.status.clone();
        move |new_stage: String| {
            let id3 = id2.clone();
            let status3 = status2.clone();
            let toast2 = toast.clone();
            leptos::task::spawn_local(async move {
                match update_deal(&id3, &new_stage, &status3).await {
                    Ok(_) => toast2.show_toast("Deal", &format!("Stage updated to {}", new_stage), "success"),
                    Err(e) => toast2.show_toast("Error", &e, "error"),
                }
            });
        }
    });

    // Detail rows
    let detail_rows = StoredValue::new(vec![
        ("Deal ID",      d.id.clone(),         true),
        ("Name",         d.name.clone(),        false),
        ("Customer ID",  d.customer_id.clone(), true),
        ("Amount",       format!("${:.2}", d.amount), false),
        ("Stage",        d.stage.clone(),       false),
        ("Status",       d.status.clone(),      false),
    ]);

    // Pre-extracted values
    let name        = d.name.clone();
    let amount_str  = format!("${:.2}", d.amount);
    let stage_str   = d.stage.clone();
    let status_str  = d.status.clone();
    let account_id  = StoredValue::new(d.customer_id.clone());

    view! {
        <div style="display:flex;flex-direction:column;height:100%;overflow:hidden;">

            // ── Hero ─────────────────────────────────────────────────────────
            <div style="padding:14px 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div style="font-size:11px;color:var(--text-muted);margin-bottom:8px;display:flex;align-items:center;gap:5px;">
                    <a href="/pipeline" style="color:var(--text-link);text-decoration:none;">"Pipeline"</a>
                    " › "
                    {name.clone()}
                </div>
                <div style="display:flex;align-items:flex-start;justify-content:space-between;">
                    <div style="display:flex;gap:14px;align-items:center;">
                        <div style="width:48px;height:48px;border-radius:9px;background:var(--violet-dim,rgba(139,92,246,0.12));border:1px solid var(--violet,#8b5cf6);display:flex;align-items:center;justify-content:center;font-size:20px;flex-shrink:0;">
                            "💼"
                        </div>
                        <div>
                            <div style="font-size:20px;font-weight:700;letter-spacing:-0.3px;display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:4px;">
                                {name.clone()}
                                <span style="font-size:9px;font-weight:700;padding:2px 7px;border-radius:3px;border:1px solid var(--violet,#8b5cf6);color:var(--violet,#8b5cf6);">{stage_str.clone()}</span>
                                <span style=format!(
                                    "font-size:9px;font-weight:700;padding:2px 7px;border-radius:3px;border:1px solid {};color:{};",
                                    if is_won { "var(--green)" } else if is_lost { "var(--red)" } else { "var(--text-muted)" },
                                    if is_won { "var(--green)" } else if is_lost { "var(--red)" } else { "var(--text-muted)" },
                                )>{status_str.clone()}</span>
                            </div>
                            <div style="font-size:13px;color:var(--text-secondary);">
                                {(!account_id.get_value().is_empty()).then(|| {
                                    let aid = account_id.get_value();
                                    let aid2 = aid.clone();
                                    view! {
                                        <span>"Account: "
                                            <a href={format!("/accounts/{}", aid)} style="color:var(--text-link);text-decoration:none;">{aid2}</a>
                                        </span>
                                    }
                                })}
                            </div>
                        </div>
                    </div>
                    <div style="text-align:right;">
                        <div style="font-size:22px;font-weight:700;color:var(--cobalt,#3b82f6);">{amount_str.clone()}</div>
                        <div style="font-size:10px;color:var(--text-muted);font-family:monospace;margin-top:2px;">{d.id.clone()}</div>
                    </div>
                </div>
            </div>

            // ── Stage Stepper ─────────────────────────────────────────────────
            <div style="padding:10px 24px;border-bottom:1px solid var(--border-default);flex-shrink:0;display:flex;align-items:center;gap:4px;flex-wrap:wrap;overflow-x:auto;">
                {stages.iter().enumerate().filter(|(i, _)| *i < 4).map(|(i, (key, label))| {
                    let k = key.to_string();
                    view! {
                        <>
                            <div class={sc(i)}>
                                <div class="sf-pill"
                                    style="cursor:pointer;"
                                    on:click=move |_| handle_stage.get_value()(k.clone())
                                >{*label}</div>
                            </div>
                            <div class="sf-arrow">"→"</div>
                        </>
                    }
                }).collect::<Vec<_>>()}
                // Closed Won
                <div class={sc(4)}>
                    <div class="sf-pill"
                        style="cursor:pointer;"
                        on:click=move |_| handle_stage.get_value()("closed_won".into())
                    >"Closed Won"</div>
                </div>
                <div class="sf-arrow">"|"</div>
                // Closed Lost
                <div class={sc(5)}>
                    <div class="sf-pill"
                        style="cursor:pointer;"
                        on:click=move |_| handle_stage.get_value()("closed_lost".into())
                    >"Closed Lost"</div>
                </div>
            </div>

            // ── KPI Strip ─────────────────────────────────────────────────────
            <div style="display:flex;border-bottom:1px solid var(--border-default);flex-shrink:0;">
                <div class="kpi">
                    <div class="kpi-label">"Deal Value"</div>
                    <div class="kpi-value mono" style="color:var(--cobalt,#3b82f6);">{amount_str}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Stage"</div>
                    <div class="kpi-value" style="color:var(--violet,#8b5cf6);">{stage_str}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Status"</div>
                    <div class="kpi-value" style=format!("color:{};", if is_won { "var(--green)" } else if is_lost { "var(--red)" } else { "var(--text-secondary)" })>{status_str}</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Win Probability"</div>
                    <div class="kpi-value" style="color:var(--text-muted);">"—"</div>
                </div>
                <div class="kpi">
                    <div class="kpi-label">"Close Date"</div>
                    <div class="kpi-value" style="color:var(--text-muted);">"—"</div>
                </div>
            </div>

            // ── Content ───────────────────────────────────────────────────────
            <div class="content-body" style="flex:1;overflow-y:auto;padding:20px 24px;">
                <div class="col-7-5">
                    // Left — deal details card
                    <div>
                        <div class="card">
                            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"All Fields · atlas_deals"</span>
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
                    </div>

                    // Right rail
                    <div>
                        <crate::pages::billing::scorecard_panel::ScorecardPanel
                            entity_type="atlas_opportunity".to_string()
                            entity_id=d.id.clone()
                            subject_label=name.clone()
                        />
                        // Account link
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
                                        <div style="width:30px;height:30px;border-radius:5px;background:var(--cobalt-dim);border:1px solid var(--cobalt);display:flex;align-items:center;justify-content:center;font-size:14px;flex-shrink:0;">"🏢"</div>
                                        <div>
                                            <div style="font-size:12.5px;font-weight:600;color:var(--text-link);">{aid2}</div>
                                            <div style="font-size:11px;color:var(--text-muted);">"View account →"</div>
                                        </div>
                                    </a>
                                </div>
                            }
                        })}

                        // Stage change hint
                        <div class="card">
                            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);">
                                <span class="card-title" style="font-size:11.5px;font-weight:600;">"Quick Actions"</span>
                            </div>
                            <div style="padding:12px 14px;display:flex;flex-direction:column;gap:6px;">
                                <button class="btn btn-primary btn-sm"
                                    on:click=move |_| handle_stage.get_value()("closed_won".into())
                                    style="background:var(--green);border-color:var(--green);"
                                >"✓ Mark Closed Won"</button>
                                <button class="btn btn-ghost btn-sm"
                                    on:click=move |_| handle_stage.get_value()("closed_lost".into())
                                    style="color:var(--red);border-color:var(--red);"
                                >"✕ Mark Closed Lost"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
