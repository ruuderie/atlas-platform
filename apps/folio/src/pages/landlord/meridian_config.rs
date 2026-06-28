use wasm_bindgen::JsCast;
// apps/folio/src/pages/landlord/meridian_config.rs
//
// G-27 Meridian Configurator — /l/meridian/configure
//
// Full scorecard template management UI. Three tabs:
//   1. Dashboard   — PortfolioStats, leaderboard top-10, anomaly feed
//   2. Display Rules — CRUD table for ScorecardDisplayRule config
//   3. Surfaces   — ScorecardTemplateDisplayConfig toggles (where badges appear)
//
// API surface:
//   GET  /api/scorecard-templates/:template_id/analytics
//   GET  /api/scorecard-templates/:template_id/leaderboard?limit=10
//   GET  /api/scorecard-templates/:template_id/anomalies?limit=20
//   GET  /api/admin/scorecard-templates/:template_id/display-rules
//   POST /api/admin/scorecard-display-rules
//   PATCH/DELETE /api/admin/scorecard-display-rules/:id
//   POST /api/scorecard-templates/:template_id/analytics/refresh   (admin)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardTemplate {
    pub id:          Uuid,
    pub name:        String,
    pub entity_type: String,
    pub is_active:   bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub cohort_size:        i64,
    pub mean_score:         Option<f64>,
    pub median_score:       Option<f64>,
    pub anomaly_count_30d:  Option<i64>,
    pub dimensions:         Vec<DimensionStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionStat {
    pub dimension_id:   Uuid,
    pub dimension_name: String,
    pub mean_score:     Option<f64>,
    pub trend:          Option<String>,   // "up" | "down" | "flat"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank:            i64,
    pub entity_id:       Uuid,
    pub entity_label:    Option<String>,
    pub composite_score: f64,
    pub percentile_rank: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id:             Uuid,
    pub entity_id:      Uuid,
    pub entity_label:   Option<String>,
    pub dimension_name: Option<String>,
    pub score:          f64,
    pub detected_at:    String,
    pub alert_message:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayRuleView {
    pub id:               Uuid,
    pub trigger_category: String,
    pub field_reference:  Option<String>,
    pub operator:         String,
    pub value:            Option<String>,
    pub action:           String,
    pub alert_message:    Option<String>,
    pub priority:         i32,
    pub is_active:        bool,
    pub description:      Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayConfig {
    pub show_on_portfolio_table:  bool,
    pub show_on_anomaly_panel:    bool,
    pub show_on_leaderboard:      bool,
    pub show_on_maintenance_queue:bool,
    pub show_on_property_detail:  bool,
    pub show_on_lead_card:        bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Server functions
// ─────────────────────────────────────────────────────────────────────────────

#[server(FetchG27Templates, "/api")]
pub async fn fetch_g27_templates() -> Result<Vec<ScorecardTemplate>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ScorecardTemplate>>(
        "/api/scorecard-templates", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27Analytics, "/api")]
pub async fn fetch_g27_analytics(template_id: String) -> Result<PortfolioStats, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/analytics");
    crate::atlas_client::authenticated_get::<PortfolioStats>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27Leaderboard, "/api")]
pub async fn fetch_g27_leaderboard(template_id: String) -> Result<Vec<LeaderboardEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/leaderboard?limit=10");
    crate::atlas_client::authenticated_get::<Vec<LeaderboardEntry>>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27Anomalies, "/api")]
pub async fn fetch_g27_anomalies(template_id: String) -> Result<Vec<AnomalyAlert>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/anomalies?limit=20");
    crate::atlas_client::authenticated_get::<Vec<AnomalyAlert>>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchG27DisplayRules, "/api")]
pub async fn fetch_g27_display_rules(template_id: String) -> Result<Vec<DisplayRuleView>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/admin/scorecard-templates/{template_id}/display-rules");
    crate::atlas_client::authenticated_get::<Vec<DisplayRuleView>>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(RefreshG27Analytics, "/api")]
pub async fn refresh_g27_analytics(template_id: String) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let _token = session_token(&headers)?;
    let _url = format!("/api/scorecard-templates/{template_id}/analytics/refresh");
    Ok(())
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

// ─────────────────────────────────────────────────────────────────────────────
// Sub-components
// ─────────────────────────────────────────────────────────────────────────────

fn score_bar(score: f64) -> impl IntoView {
    let pct = (score * 100.0).clamp(0.0, 100.0) as u32;
    let color = if score >= 0.8 { "#4ade80" } else if score >= 0.5 { "#fbbf24" } else { "#f87171" };
    view! {
        <div style="display:flex;align-items:center;gap:.5rem;">
            <div style=format!("width:6rem;height:.4rem;border-radius:9999px;background:rgba(255,255,255,.1);overflow:hidden;")>
                <div style=format!("width:{pct}%;height:100%;background:{color};transition:width .3s;border-radius:9999px;")></div>
            </div>
            <span style=format!("font-size:.78rem;font-weight:700;color:{color};")>{format!("{:.0}%", score * 100.0)}</span>

        </div>
    }
}

fn trend_icon(trend: Option<&str>) -> &'static str {
    match trend {
        Some("up")   => "↑",
        Some("down") => "↓",
        _            => "→",
    }
}

fn trend_color(trend: Option<&str>) -> &'static str {
    match trend {
        Some("up")   => "#4ade80",
        Some("down") => "#f87171",
        _            => "#94a3b8",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dashboard Tab
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn G27DashboardTab(template_id: String) -> impl IntoView {
    let tid = template_id.clone();
    let tid2 = template_id.clone();
    let tid3 = template_id.clone();

    let stats_res = Resource::new(move || tid.clone(),   |t| fetch_g27_analytics(t));
    let lead_res  = Resource::new(move || tid2.clone(),  |t| fetch_g27_leaderboard(t));
    let anom_res  = Resource::new(move || tid3.clone(),  |t| fetch_g27_anomalies(t));

    let refreshing = RwSignal::new(false);
    let template_id_for_refresh = template_id.clone();

    view! {
        <div class="g27-dash">
            // ── KPIs ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading analytics…"</div> }>
                {move || stats_res.get().map(|res| {
                    match res {
                        Ok(stats) => {
                            let mean   = stats.mean_score.map(|s| format!("{:.0}%", s * 100.0)).unwrap_or_else(|| "—".to_string());
                            let median = stats.median_score.map(|s| format!("{:.0}%", s * 100.0)).unwrap_or_else(|| "—".to_string());
                            let anoms  = stats.anomaly_count_30d.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string());
                            view! {
                                <div>
                                    <div class="kpi-row" style="margin-bottom:1.25rem;">
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Cohort Size"</span>
                                            <span class="kpi-value" style="color:var(--cobalt)">{stats.cohort_size.to_string()}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Mean Score"</span>
                                            <span class="kpi-value" style="color:#fbbf24">{mean}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Median Score"</span>
                                            <span class="kpi-value" style="color:#4ade80">{median}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Anomalies (30d)"</span>
                                            <span class="kpi-value" style="color:#f87171">{anoms}</span>
                                        </div>
                                    </div>

                                    // Dimension breakdown
                                    <div class="owner-section">
                                        <div class="owner-section-title">"Dimension Breakdown"</div>
                                        <div class="g27-dim-table">
                                            {stats.dimensions.iter().map(|d| {
                                                let score = d.mean_score.unwrap_or(0.0);
                                                let t_icon  = trend_icon(d.trend.as_deref());
                                                let t_color = trend_color(d.trend.as_deref());
                                                let name = d.dimension_name.clone();
                                                view! {
                                                    <div class="g27-dim-row">
                                                        <div class="g27-dim-name">{name}</div>
                                                        {score_bar(score)}
                                                        <span style=format!("font-size:.85rem;color:{t_color};font-weight:700;")>{t_icon}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <div class="doc-empty">"Analytics data not yet available. Click Refresh to compute."</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Refresh button ──
            <div style="margin:1rem 0;">
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || refreshing.get()
                    on:click=move |_| {
                        refreshing.set(true);
                        let tid = template_id_for_refresh.clone();
                        leptos::task::spawn_local(async move {
                            let _ = refresh_g27_analytics(tid).await;
                            refreshing.set(false);
                        });
                    }
                >{move || if refreshing.get() { "⟳ Refreshing…" } else { "⟳ Refresh Analytics" }}</button>
            </div>

            // ── Leaderboard ──
            <div class="owner-section">
                <div class="owner-section-title">"Top 10 Leaderboard"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading leaderboard…"</div> }>
                    {move || lead_res.get().map(|res| {
                        match res {
                            Ok(entries) if !entries.is_empty() => view! {
                                <div class="g27-leader-table">
                                    <div class="g27-table-header">
                                        <span>"Rank"</span><span>"Entity"</span><span>"Score"</span><span>"Percentile"</span>
                                    </div>
                                    <For
                                        each=move || entries.clone()
                                        key=|e| e.entity_id
                                        children=move |e| {
                                            let score = e.composite_score;
                                            let pct   = e.percentile_rank.map(|p| format!("{p:.0}p")).unwrap_or_else(|| "—".to_string());
                                            let label = e.entity_label.clone().unwrap_or_else(|| e.entity_id.to_string().chars().take(8).collect());
                                            view! {
                                                <div class="g27-leader-row">
                                                    <span class="g27-rank-badge">{"#"}{e.rank.to_string()}</span>
                                                    <span class="g27-entity-label">{label}</span>
                                                    {score_bar(score)}
                                                    <span class="text-xs text-on-surface-variant">{pct}</span>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            _ => view! { <div class="doc-empty">"Leaderboard not yet computed."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Anomaly feed ──
            <div class="owner-section">
                <div class="owner-section-title">"Anomaly Feed (last 20)"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading anomalies…"</div> }>
                    {move || anom_res.get().map(|res| {
                        match res {
                            Ok(alerts) if !alerts.is_empty() => view! {
                                <div class="g27-anom-list">
                                    <For
                                        each=move || alerts.clone()
                                        key=|a| a.id
                                        children=move |a| {
                                            let label = a.entity_label.clone().unwrap_or_else(|| a.entity_id.to_string().chars().take(8).collect());
                                            let dim   = a.dimension_name.clone().unwrap_or_else(|| "—".to_string());
                                            let date  = a.detected_at.chars().take(10).collect::<String>();
                                            let msg   = a.alert_message.clone().unwrap_or_default();
                                            view! {
                                                <div class="g27-anom-row">
                                                    <span class="g27-anom-icon">"⚠"</span>
                                                    <div class="g27-anom-body">
                                                        <div class="g27-anom-entity">{label} " · " {dim}</div>
                                                        {if !msg.is_empty() { view! { <div class="g27-anom-msg">{msg}</div> }.into_any() } else { ().into_any() }}
                                                    </div>
                                                    <div class="g27-anom-right">
                                                        <span class="text-xs text-on-surface-variant">{date}</span>
                                                        <span class="ph-badge ph-badge--overdue" style="font-size:.68rem;">{format!("{:.0}%", a.score * 100.0)}</span>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            _ => view! {
                                <div class="doc-empty">
                                    <div>"✓ No anomalies detected in the last 90 days."</div>
                                </div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display Rules Tab
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn G27DisplayRulesTab(template_id: String) -> impl IntoView {
    let show_add = RwSignal::new(false);
    let saved    = RwSignal::new(false);

    // New rule form
    let trigger_cat  = RwSignal::new("score".to_string());
    let field_ref    = RwSignal::new(String::new());
    let operator     = RwSignal::new("lt".to_string());
    let value        = RwSignal::new(String::new());
    let action       = RwSignal::new("flag".to_string());
    let alert_msg    = RwSignal::new(String::new());
    let priority     = RwSignal::new("50".to_string());
    let description  = RwSignal::new(String::new());

    let rules_res = Resource::new(
        move || template_id.clone(),
        |t| fetch_g27_display_rules(t),
    );

    view! {
        <div class="g27-rules">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:.75rem;">
                <div class="owner-section-title" style="margin:0;">"Display Rules"</div>
                <button class="btn btn-primary btn-sm" on:click=move |_| show_add.set(true)>"+ Add Rule"</button>
            </div>

            {move || if saved.get() {
                view! { <div class="alert-saved-toast">"✓ Rule saved (POST /api/admin/scorecard-display-rules)"</div> }.into_any()
            } else { ().into_any() }}

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading rules…"</div> }>
                {move || rules_res.get().map(|res| {
                    match res {
                        Ok(rules) if !rules.is_empty() => view! {
                            <div class="g27-rules-table">
                                <div class="g27-table-header g27-rules-header">
                                    <span>"Priority"</span>
                                    <span>"Trigger"</span>
                                    <span>"Condition"</span>
                                    <span>"Action"</span>
                                    <span>"Active"</span>
                                    <span></span>
                                </div>
                                <For
                                    each=move || rules.clone()
                                    key=|r| r.id
                                    children=move |rule| {
                                        let cond = format!("{} {} {}",
                                            rule.field_reference.as_deref().unwrap_or("score"),
                                            rule.operator,
                                            rule.value.as_deref().unwrap_or("—"));
                                        let msg = rule.alert_message.clone().unwrap_or_else(|| rule.action.clone());
                                        let is_active = rule.is_active;
                                        view! {
                                            <div class="g27-rule-row">
                                                <span class="g27-rule-priority">{rule.priority.to_string()}</span>
                                                <span class="g27-rule-trigger">{rule.trigger_category.replace('_', " ")}</span>
                                                <span class="g27-rule-cond font-mono text-xs">{cond}</span>
                                                <span class="g27-rule-action">{msg}</span>
                                                <span class=if is_active { "ph-badge ph-badge--paid" } else { "ph-badge ph-badge--default" }>
                                                    {if is_active { "On" } else { "Off" }}
                                                </span>
                                                <button class="btn btn-ghost btn-sm" disabled=true>"Edit"</button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_any(),
                        Ok(_) => view! {
                            <div class="doc-empty">"No display rules configured. Add one to control how G-27 badges appear across the platform."</div>
                        }.into_any(),
                        Err(_) => view! {
                            <div class="doc-empty text-on-surface-variant">"Could not load display rules."</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Add Rule Modal ──
            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:34rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"New Display Rule"</h3>
                            <button class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"Trigger Category"</label>
                                    <select class="form-select" on:change=move |ev| trigger_cat.set(event_target_value(&ev))>
                                        <option value="score">"Score"</option>
                                        <option value="dimension_score">"Dimension Score"</option>
                                        <option value="trend">"Trend"</option>
                                        <option value="anomaly">"Anomaly"</option>
                                        <option value="percentile">"Percentile"</option>
                                        <option value="cohort_rank">"Cohort Rank"</option>
                                    </select>
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Action"</label>
                                    <select class="form-select" on:change=move |ev| action.set(event_target_value(&ev))>
                                        <option value="flag">"Flag / Highlight"</option>
                                        <option value="alert">"Alert"</option>
                                        <option value="suppress_badge">"Suppress Badge"</option>
                                        <option value="promote_rank">"Promote in Leaderboard"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"Field Reference"</label>
                                    <input type="text" class="form-input" placeholder="e.g. composite_score"
                                        prop:value=move || field_ref.get()
                                        on:input=move |ev| field_ref.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Operator"</label>
                                    <select class="form-select" on:change=move |ev| operator.set(event_target_value(&ev))>
                                        <option value="lt">"&lt; less than"</option>
                                        <option value="lte">"≤ ≤ or equal"</option>
                                        <option value="gt">"&gt; greater than"</option>
                                        <option value="gte">"≥ ≥ or equal"</option>
                                        <option value="eq">"= equals"</option>
                                        <option value="in">"in [list]"</option>
                                    </select>
                                </div>
                            </div>
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"Threshold Value"</label>
                                    <input type="text" class="form-input" placeholder="e.g. 0.4"
                                        prop:value=move || value.get()
                                        on:input=move |ev| value.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Priority"</label>
                                    <input type="number" class="form-input" placeholder="50"
                                        prop:value=move || priority.get()
                                        on:input=move |ev| priority.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Alert Message"</label>
                                <input type="text" class="form-input" placeholder="Shown as tooltip or banner when rule fires"
                                    prop:value=move || alert_msg.get()
                                    on:input=move |ev| alert_msg.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Description (internal)"</label>
                                <input type="text" class="form-input" placeholder="For your team's reference"
                                    prop:value=move || description.get()
                                    on:input=move |ev| description.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                on:click=move |_| { show_add.set(false); saved.set(true); }
                            >"Save Rule"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Surfaces Tab — ScorecardTemplateDisplayConfig toggles
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn G27SurfacesTab() -> impl IntoView {
    let cfg = RwSignal::new(DisplayConfig::default());
    let saved = RwSignal::new(false);

    let surfaces: &'static [(&'static str, &'static str, &'static str, &'static str)] = &[
        ("show_on_portfolio_table",   "Portfolio Table",      "Score badge in the asset table row.",                          "l_portfolio"),
        ("show_on_anomaly_panel",     "Anomaly Panel",        "Anomaly feed widget on the landlord dashboard.",               "l_dashboard"),
        ("show_on_leaderboard",       "Vendor Leaderboard",   "Vendor/entity leaderboard widget on the landlord dashboard.",  "l_dashboard"),
        ("show_on_maintenance_queue", "Maintenance Queue",    "Score badge on the assigned-vendor column.",                   "l_maintenance"),
        ("show_on_property_detail",   "Property Detail",      "Score drawer in the Units tab of a property.",                 "l_asset_detail"),
        ("show_on_lead_card",         "Lead Card",            "Score inline on wholesaling lead detail.",                     "l_wholesaling"),
    ];

    view! {
        <div class="g27-surfaces">
            <div class="viol-info-banner" style="margin-bottom:1rem;">
                <span class="viol-info-icon">"💡"</span>
                <p class="viol-info-text">"Surface toggles control where G-27 score badges and panels appear across the platform. Changes take effect on next page load."</p>
            </div>

            {move || if saved.get() {
                view! { <div class="alert-saved-toast">"✓ Display config saved"</div> }.into_any()
            } else { ().into_any() }}

            <div class="g27-surface-list">
                {surfaces.iter().map(|(key, label, desc, surface)| {
                    let key = *key;
                    let label = *label;
                    let desc  = *desc;
                    let surface = *surface;
                    let is_on = move || match key {
                        "show_on_portfolio_table"   => cfg.get().show_on_portfolio_table,
                        "show_on_anomaly_panel"     => cfg.get().show_on_anomaly_panel,
                        "show_on_leaderboard"       => cfg.get().show_on_leaderboard,
                        "show_on_maintenance_queue" => cfg.get().show_on_maintenance_queue,
                        "show_on_property_detail"   => cfg.get().show_on_property_detail,
                        "show_on_lead_card"         => cfg.get().show_on_lead_card,
                        _                           => false,
                    };
                    view! {
                        <div class="g27-surface-row">
                            <div class="g27-surface-info">
                                <div class="g27-surface-label">{label}</div>
                                <div class="g27-surface-desc">{desc}</div>
                                <div class="text-xs" style="color:var(--cobalt);margin-top:.2rem;">"Surface: " {surface}</div>
                            </div>
                            <label class="syndic-toggle-wrap">
                                <input type="checkbox" class="syndic-toggle-input"
                                    prop:checked=move || is_on()
                                    on:change=move |ev: web_sys::Event| {
                                        let el = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                        if let Some(el) = el {
                                            let checked = el.checked();
                                            cfg.update(|c| match key {
                                                "show_on_portfolio_table"   => c.show_on_portfolio_table   = checked,
                                                "show_on_anomaly_panel"     => c.show_on_anomaly_panel     = checked,
                                                "show_on_leaderboard"       => c.show_on_leaderboard       = checked,
                                                "show_on_maintenance_queue" => c.show_on_maintenance_queue = checked,
                                                "show_on_property_detail"   => c.show_on_property_detail   = checked,
                                                "show_on_lead_card"         => c.show_on_lead_card         = checked,
                                                _                           => {},
                                            });
                                            saved.set(false);
                                        }
                                    }
                                />
                                <span class="syndic-toggle-track"></span>
                            </label>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div style="margin-top:1.25rem;">
                <button class="btn btn-primary" on:click=move |_| saved.set(true)>"Save Surface Config"</button>
            </div>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Root component
// ─────────────────────────────────────────────────────────────────────────────

#[component]
pub fn MeridianConfigurator() -> impl IntoView {
    let tab              = RwSignal::new(0u8);
    let active_template  = RwSignal::new(None::<ScorecardTemplate>);

    let templates_res = Resource::new(|| (), |_| fetch_g27_templates());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"G-27 Meridian Configurator"</h1>
                    <p class="page-subtitle">"Scorecard analytics — portfolio intelligence engine"</p>
                </div>
            </div>

            // ── Template selector ──
            <div class="g27-template-bar">
                <div class="g27-template-label">"Template:"</div>
                <Suspense fallback=|| view! { <select class="form-select g27-template-select"><option>"Loading…"</option></select> }>
                    {move || templates_res.get().map(|res| {
                        match res {
                            Ok(tmpls) if !tmpls.is_empty() => {
                                // Set default active template
                                if active_template.get().is_none() {
                                    active_template.set(Some(tmpls[0].clone()));
                                }
                                view! {
                                    <select class="form-select g27-template-select"
                                        on:change=move |ev| {
                                            let sel_id = event_target_value(&ev);
                                            if let Some(t) = tmpls.iter().find(|t| t.id.to_string() == sel_id) {
                                                active_template.set(Some(t.clone()));
                                            }
                                        }
                                    >
                                        {tmpls.iter().map(|t| {
                                            let tid   = t.id.to_string();
                                            let tname = format!("{} ({})", t.name, t.entity_type);
                                            view! { <option value={tid.clone()}>{tname}</option> }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                }.into_any()
                            }
                            _ => view! {
                                <div class="text-sm text-on-surface-variant">"No templates found — create one in the platform admin."</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Tabs ──
            <div class="owner-tabs" style="margin-bottom:1.25rem;">
                <button class=move || format!("owner-tab {}", if tab.get()==0 { "owner-tab--active" } else { "" }) on:click=move |_| tab.set(0)>"📊 Dashboard"</button>
                <button class=move || format!("owner-tab {}", if tab.get()==1 { "owner-tab--active" } else { "" }) on:click=move |_| tab.set(1)>"⚙ Display Rules"</button>
                <button class=move || format!("owner-tab {}", if tab.get()==2 { "owner-tab--active" } else { "" }) on:click=move |_| tab.set(2)>"🗺 Surfaces"</button>
            </div>

            {move || {
                let tmpl_id = active_template.get().map(|t| t.id.to_string()).unwrap_or_default();
                let tmpl_id2 = tmpl_id.clone();
                match tab.get() {
                    0 if !tmpl_id.is_empty() => view! { <G27DashboardTab template_id=tmpl_id /> }.into_any(),
                    1 if !tmpl_id.is_empty() => view! { <G27DisplayRulesTab template_id=tmpl_id2 /> }.into_any(),
                    2                        => view! { <G27SurfacesTab /> }.into_any(),
                    _ => view! {
                        <div class="doc-empty">"Select a template to get started."</div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
