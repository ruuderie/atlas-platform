use crate::api::admin::get_ai_tasks;
use crate::api::verification::get_verification_requests;
use leptos::prelude::*;

/// Right-column intelligence sidebar shown only on the Command Center (/).
///
/// All panels load real data from the backend. No static/hardcoded placeholder
/// values exist in this component — if the backend has no data, each panel
/// shows an explicit empty state rather than fake numbers.
#[component]
pub fn IntelSidebar() -> impl IntoView {
    let tasks_res = LocalResource::new(|| async move { get_ai_tasks().await.unwrap_or_default() });
    let verification_res = LocalResource::new(|| async move {
        get_verification_requests(None, None)
            .await
            .unwrap_or_default()
    });

    view! {
        <aside class="intel-sidebar">

            // ── AI Task Queue (real data from GET /api/admin/ai-tasks) ──
            <div class="intel-panel">
                <div class="intel-panel-header">
                    <span class="intel-panel-title">
                        <span class="live-dot"></span>
                        "AI Tasks"
                    </span>
                    <a href="/admin/aitasks" class="section-action" style="font-size:10px">"Monitor →"</a>
                </div>
                <Suspense fallback=move || view! { <div style="padding:14px 16px;font-size:11px;color:var(--text-muted)">"Loading tasks…"</div> }>
                {move || {
                    let tasks = tasks_res.get().unwrap_or_default();
                    // Show at most 6 most recent tasks
                    let recent: Vec<_> = tasks.into_iter().take(6).collect();
                    if recent.is_empty() {
                        view! {
                            <div style="padding:14px 16px;display:flex;align-items:center;gap:8px;">
                                <span style="font-size:18px">"✅"</span>
                                <div style="font-size:11px;color:var(--text-muted)">"No AI tasks in the queue."</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                {recent.into_iter().map(|task| {
                                    let (status_icon, status_class) = match task.status_class.as_str() {
                                        "running"  => ("↻", "running"),
                                        "done"     => ("✓", "done"),
                                        "failed"   => ("✗", "failed"),
                                        "queued"   => ("·", "queued"),
                                        _          => ("·", "queued"),
                                    };
                                    let task_name = task.task_type.clone();
                                    let entity    = task.entity.clone();
                                    let runtime   = task.runtime.clone();
                                    view! {
                                        <div class="job-row">
                                            <div class={format!("job-status {}", status_class)}>{status_icon}</div>
                                            <div class="flex-col" style="flex:1;gap:1px">
                                                <span class="job-name">{task_name}</span>
                                                <span class="job-tenant">{entity}</span>
                                            </div>
                                            <span class="job-duration">{runtime}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
                </Suspense>
            </div>

            // ── Verification Queue (real data from GET /api/admin/verification-requests) ──
            <div class="intel-panel" style="border-bottom:none">
                <div class="intel-panel-header">
                    <span class="intel-panel-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M8 2l5 2v4c0 3-2 5.5-5 6.5C5 13.5 3 11 3 8V4l5-2z"/>
                        </svg>
                        "Verification Queue"
                        <span class="section-count">
                            {move || {
                                let n = verification_res.get().unwrap_or_default()
                                    .iter()
                                    .filter(|r| r.status == "pending" || r.status == "review")
                                    .count();
                                if n == 0 { "Clear".to_string() } else { format!("{} pending", n) }
                            }}
                        </span>
                    </span>
                    <a href="/verification" class="section-action" style="font-size:10px;text-decoration:none">"Review All →"</a>
                </div>
                <Suspense fallback=move || view! { <div style="padding:14px 16px;font-size:11px;color:var(--text-muted)">"Loading queue…"</div> }>
                {move || {
                    let items = verification_res.get().unwrap_or_default();
                    let pending: Vec<_> = items.into_iter()
                        .filter(|r| r.status == "pending" || r.status == "review")
                        .take(5)
                        .collect();
                    if pending.is_empty() {
                        view! {
                            <div style="padding:14px 16px;display:flex;align-items:center;gap:10px;">
                                <span style="font-size:20px">"✅"</span>
                                <div>
                                    <div style="font-size:12px;font-weight:600;color:var(--text-primary)">"Queue is clear"</div>
                                    <div style="font-size:11px;color:var(--text-muted)">"No pending identity or document verification requests."</div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div style="padding:8px 0;display:flex;flex-direction:column;">
                                {pending.into_iter().map(|item| {
                                    let dot_color = if item.status == "review" {
                                        "var(--amber)"
                                    } else {
                                        "var(--cobalt)"
                                    };
                                    let label     = item.req_type.clone();
                                    let name      = item.entity_name.clone();
                                    let submitted = item.created_at.get(..10)
                                        .map(|d| d.to_string())
                                        .unwrap_or_else(|| "—".to_string());
                                    view! {
                                        <a
                                            href="/verification"
                                            style="display:flex;align-items:center;gap:8px;padding:8px 16px;border-bottom:1px solid var(--border-subtle);text-decoration:none;transition:background 0.1s;"
                                        >
                                            <span style=format!("width:6px;height:6px;border-radius:50%;background:{};flex-shrink:0", dot_color)></span>
                                            <span style="font-size:12px;color:var(--text-primary);font-weight:500;flex:1">{name}</span>
                                            <span style="font-size:10px;color:var(--text-muted);font-family:monospace">{label}</span>
                                            <span style="font-size:10px;color:var(--text-muted)">{submitted}</span>
                                        </a>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
                </Suspense>
            </div>

        </aside>
    }
}
