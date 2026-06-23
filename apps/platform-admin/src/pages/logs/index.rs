use leptos::prelude::*;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::api::audit_logs::{get_audit_logs, AuditLogModel};
use crate::app::GlobalToast;

#[allow(dead_code)]
pub fn format_datetime_diff(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_json_diff(val: &Option<Value>) -> String {
    match val {
        Some(v) => serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string()),
        None => "None".to_string(),
    }
}

#[component]
pub fn AuditLogs() -> impl IntoView {
    let (logs, set_logs) = signal(Vec::<AuditLogModel>::new());
    let (loading, set_loading) = signal(true);
    
    let (selected_log, set_selected_log) = signal(None::<AuditLogModel>);
    let (show_modal, set_show_modal) = signal(false);

    let (filter_cat, set_filter_cat) = signal("all".to_string());
    let (search_query, set_search_query) = signal("".to_string());

    let active_network = use_context::<ReadSignal<Option<uuid::Uuid>>>().expect("active network missing");
    let toast = use_context::<GlobalToast>().expect("toast context missing");

    let fetch_logs = move || {
        set_loading.set(true);
        leptos::task::spawn_local(async move {
            let tenant_id = active_network.get();
            match get_audit_logs(tenant_id, None, None).await {
                Ok(data) => {
                    set_logs.set(data);
                }
                Err(e) => {
                    toast.show_toast("Error", &e, "error");
                    leptos::logging::error!("Failed to fetch logs: {}", e);
                }
            }
            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        fetch_logs();
    });

    let filtered_logs = Signal::derive(move || {
        let raw_logs = logs.get();
        let cat = filter_cat.get();
        let q = search_query.get().to_lowercase();
        
        raw_logs.into_iter().filter(|log| {
            let action_lower = log.action_type.to_lowercase();
            let matches_cat = match cat.as_str() {
                "all" => true,
                "security" => action_lower.contains("security") || action_lower.contains("auth") || action_lower.contains("login") || action_lower.contains("user"),
                "ni" => action_lower.contains("ni") || action_lower.contains("tenant") || action_lower.contains("provision"),
                "billing" => action_lower.contains("bill") || action_lower.contains("sub") || action_lower.contains("mrr") || action_lower.contains("invoice"),
                "flags" => action_lower.contains("flag") || action_lower.contains("rollout"),
                _ => true,
            };
            
            let matches_query = q.is_empty() 
                || action_lower.contains(&q) 
                || log.entity_type.to_lowercase().contains(&q)
                || log.entity_id.to_string().contains(&q)
                || log.actor_id.map(|id| id.to_string().contains(&q)).unwrap_or(false);
                
            matches_cat && matches_query
        }).collect::<Vec<AuditLogModel>>()
    });

    let grouped_logs = Signal::derive(move || {
        let raw_logs = filtered_logs.get();
        let mut groups: Vec<(String, Vec<AuditLogModel>)> = Vec::new();
        for log in raw_logs {
            let date_str = log.created_at.format("%A, %B %d, %Y").to_string();
            if let Some(group) = groups.iter_mut().find(|(d, _)| d == &date_str) {
                group.1.push(log);
            } else {
                groups.push((date_str, vec![log]));
            }
        }
        groups
    });

    view! {
        <div class="main-area">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Security Audit Ledger"</h1>
                    <p class="page-subtitle">"Immutable log of critical system operations and state changes."</p>
                </div>
                <button 
                    class="btn btn-ghost btn-sm"
                    on:click=move |_| fetch_logs()
                >
                    <span class="material-symbols-outlined text-sm" style="margin-right:4px;vertical-align:middle;display:inline-block;">"refresh"</span>
                    "Refresh Logs"
                </button>
            </div>

            // ── Stat Strip ──
            <div class="stat-strip">
                <div class="stat">
                    <span class="stat-val">{move || logs.get().len().to_string()}</span>
                    <span class="stat-lbl">"Total Logs"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--red)">
                        {move || logs.get().iter().filter(|l| {
                            let a = l.action_type.to_lowercase();
                            a.contains("security") || a.contains("auth") || a.contains("login") || a.contains("user")
                        }).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Security Alerts"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--violet)">
                        {move || logs.get().iter().filter(|l| {
                            let a = l.action_type.to_lowercase();
                            a.contains("ni") || a.contains("tenant") || a.contains("provision")
                        }).count().to_string()}
                    </span>
                    <span class="stat-lbl">"NI Configs"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--green)">
                        {move || logs.get().iter().filter(|l| {
                            let a = l.action_type.to_lowercase();
                            a.contains("bill") || a.contains("sub") || a.contains("mrr") || a.contains("invoice")
                        }).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Billing Ops"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--cobalt)">
                        {move || logs.get().iter().filter(|l| {
                            let a = l.action_type.to_lowercase();
                            a.contains("flag") || a.contains("rollout")
                        }).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Flag Rollouts"</span>
                </div>
            </div>

            // ── Filter Bar ──
            <div class="filter-bar">
                <select 
                    class="filter-select" 
                    on:change=move |ev| set_filter_cat.set(event_target_value(&ev))
                >
                    <option value="all">"All Categories"</option>
                    <option value="security">"Security"</option>
                    <option value="ni">"NI / Tenant"</option>
                    <option value="billing">"Billing"</option>
                    <option value="flags">"Flags"</option>
                </select>
                <div style="flex:1"></div>
                <div class="flag-search-wrap" style="width:260px">
                    <input 
                        type="text" 
                        class="date-input" 
                        style="width:100%"
                        placeholder="Search logs..."
                        prop:value=search_query
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // ── Logs Grouped List ──
            <div class="log-body">
                <Show when=move || !loading.get() fallback=move || view! {
                    <div style="padding:48px; text-align:center; color:var(--text-muted)">
                        "Loading ledger entries..."
                    </div>
                }>
                    <Show when=move || !grouped_logs.get().is_empty() fallback=move || view! {
                        <div style="padding:48px; text-align:center; color:var(--text-muted)">
                            "No audit logs found matching criteria."
                        </div>
                    }>
                        {move || grouped_logs.get().into_iter().map(|(date_str, group_items)| {
                            view! {
                                <div class="log-date-group">{date_str} " · " {group_items.len()} " entries"</div>
                                {group_items.into_iter().map(|log| {
                                    let log_clone = log.clone();
                                    
                                    let action_lower = log.action_type.to_lowercase();
                                    let (dot_class, cat_class, cat_label) = if action_lower.contains("flag") || action_lower.contains("rollout") {
                                        ("dot-flag", "cat-flag", "Flags")
                                    } else if action_lower.contains("bill") || action_lower.contains("sub") || action_lower.contains("mrr") || action_lower.contains("invoice") {
                                        ("dot-billing", "cat-billing", "Billing")
                                    } else if action_lower.contains("ni") || action_lower.contains("tenant") || action_lower.contains("provision") {
                                        ("dot-ni", "cat-ni", "NI / Tenant")
                                    } else if action_lower.contains("security") || action_lower.contains("auth") || action_lower.contains("login") || action_lower.contains("user") {
                                        ("dot-security", "cat-security", "Security")
                                    } else if action_lower.contains("support") || action_lower.contains("ticket") {
                                        ("dot-support", "cat-support", "Support")
                                    } else {
                                        ("dot-system", "cat-system", "System")
                                    };

                                    let time_str = log.created_at.format("%H:%M UTC").to_string();

                                    view! {
                                        <div class="log-entry" on:click=move |_| {
                                            set_selected_log.set(Some(log_clone.clone()));
                                            set_show_modal.set(true);
                                        }>
                                            <div class=format!("log-dot {}", dot_class)></div>
                                            <div class="log-content">
                                                <div class="log-action">
                                                    {log.action_type.clone()}
                                                    " — "
                                                    <code>{log.entity_type.clone()}</code>
                                                </div>
                                                <div class="log-meta">
                                                    <span class="log-actor">{log.actor_id.map(|id| id.to_string()).unwrap_or_else(|| "System".to_string())}</span>
                                                    <span>"·"</span>
                                                    <span class="log-resource">"id: " {log.entity_id.to_string()}</span>
                                                    {log.ip_address.as_ref().map(|ip| {
                                                        view! {
                                                            <>
                                                                <span>"·"</span>
                                                                <span>"IP: " {ip.clone()}</span>
                                                            </>
                                                        }.into_any()
                                                    }).unwrap_or_else(|| view! {}.into_any())}
                                                </div>
                                            </div>
                                            <div class="log-right">
                                                <span class="log-time">{time_str}</span>
                                                <span class=format!("log-cat {}", cat_class)>{cat_label}</span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            }
                        }).collect_view()}
                    </Show>
                </Show>
            </div>

            // ── Detail Slide-Out Panel ──
            <div class=move || format!("detail-backdrop {}", if show_modal.get() { "open" } else { "" }) on:click=move |_| set_show_modal.set(false)></div>
            <div class=move || format!("detail-panel {}", if show_modal.get() { "open" } else { "" })>
                <div class="detail-hdr">
                    <div class="detail-title">"Audit Entry Detail"</div>
                    <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_modal.set(false)>"✕ Close"</button>
                </div>
                <div class="detail-body">
                    {move || selected_log.get().map(|log| {
                        view! {
                            <div class="detail-row">
                                <span class="dr-label">"Action"</span>
                                <span class="dr-value" style="font-weight:600;">{log.action_type.clone()}</span>
                            </div>
                            <div class="detail-row">
                                <span class="dr-label">"Entity Type"</span>
                                <span class="dr-value">{log.entity_type.clone()}</span>
                            </div>
                            <div class="detail-row">
                                <span class="dr-label">"Entity ID"</span>
                                <span class="dr-value" style="font-family:monospace;font-size:11px;">{log.entity_id.to_string()}</span>
                            </div>
                            <div class="detail-row">
                                <span class="dr-label">"Actor"</span>
                                <span class="dr-value" style="font-family:monospace;">{log.actor_id.map(|id| id.to_string()).unwrap_or_else(|| "System".to_string())}</span>
                            </div>
                            <div class="detail-row">
                                <span class="dr-label">"Timestamp"</span>
                                <span class="dr-value" style="font-variant-numeric:tabular-nums;">{log.created_at.to_rfc3339()}</span>
                            </div>
                            {log.ip_address.as_ref().map(|ip| {
                                view! {
                                    <div class="detail-row">
                                        <span class="dr-label">"IP Address"</span>
                                        <span class="dr-value">{ip.clone()}</span>
                                    </div>
                                }.into_any()
                            }).unwrap_or_else(|| view! {}.into_any())}
                            
                            <div style="margin-top:20px; font-weight:600; font-size:11px; color:var(--text-muted); text-transform:uppercase; letter-spacing:0.08em;">"State Differential"</div>
                            
                            <div style="display:flex; flex-direction:column; gap:12px; margin-top:10px;">
                                <div class="diff-block" style="background:#060e20;">
                                    <div class="diff-rem" style="font-weight:bold;margin-bottom:4px;">"BEFORE (old_state):"</div>
                                    <pre style="overflow-x:auto;font-size:11px;white-space:pre-wrap;word-break:break-all;">{format_json_diff(&log.old_state)}</pre>
                                </div>
                                <div class="diff-block" style="background:#060e20;">
                                    <div class="diff-add" style="font-weight:bold;margin-bottom:4px;">"AFTER (new_state):"</div>
                                    <pre style="overflow-x:auto;font-size:11px;white-space:pre-wrap;word-break:break-all;">{format_json_diff(&log.new_state)}</pre>
                                </div>
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}
