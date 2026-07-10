use leptos::prelude::*;
use crate::api::admin::get_tenant_stats;
use crate::api::models::TenantStatModel;

// ── helpers ───────────────────────────────────────────────────────────────────

fn fmt_mrr(cents: Option<i64>) -> String {
    match cents {
        None | Some(0) => "$0".to_string(),
        Some(c) if c >= 100_000 => format!("${:.1}k", c as f64 / 100_000.0),
        Some(c) if c % 100 == 0 => format!("${}", c / 100),
        Some(c) => format!("${:.2}", c as f64 / 100.0),
    }
}

fn setup_score(t: &TenantStatModel) -> u8 {
    let mut s = 0u8;
    if t.site_status.as_deref().unwrap_or("active").to_lowercase() == "active" { s += 1; }
    if t.mrr_cents.map(|c| c > 0).unwrap_or(false) { s += 1; }
    if t.profile_count > 0 { s += 1; }
    if t.listing_count > 0 { s += 1; }
    s
}

fn score_color(score: u8) -> &'static str {
    if score == 4 { "var(--green)" }
    else if score >= 2 { "var(--amber)" }
    else { "var(--red)" }
}

fn health_color(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "active"       => "var(--green)",
        "suspended"    => "var(--red)",
        "provisioning" => "var(--amber)",
        _              => "var(--text-muted)",
    }
}

// ── component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantList() -> impl IntoView {
    let tenants_res = LocalResource::new(|| async move {
        get_tenant_stats().await.unwrap_or_default()
    });

    // Search and filter state
    let search = RwSignal::new(String::new());
    let filter_status = RwSignal::new("all".to_string());

    view! {
        <div class="main-canvas w-full">
            // ── Page Header ──────────────────────────────────────────────────
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Tenants"</h1>
                    <p class="page-subtitle">
                        {move || tenants_res.get().map(|ts: Vec<TenantStatModel>| {
                            format!("{} registered · Real-time", ts.len())
                        }).unwrap_or_else(|| "Loading…".to_string())}
                    </p>
                </div>
                <div class="page-actions" style="display:flex;align-items:center;gap:10px;">
                    // Search
                    <input
                        type="search"
                        placeholder="Search tenants…"
                        class="input"
                        style="width:220px;font-size:12px;"
                        prop:value=move || search.get()
                        on:input=move |ev| search.set(event_target_value(&ev))
                    />
                    // Status filter
                    <select
                        class="input"
                        style="width:130px;font-size:12px;"
                        on:change=move |ev| filter_status.set(event_target_value(&ev))
                    >
                        <option value="all">"All Status"</option>
                        <option value="active">"Active"</option>
                        <option value="suspended">"Suspended"</option>
                        <option value="provisioning">"Provisioning"</option>
                    </select>
                    <a
                        href="/apps/new"
                        class="btn btn-primary"
                        style="text-decoration:none;display:inline-flex;align-items:center;gap:6px;white-space:nowrap;"
                    >
                        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                            <path d="M8 3a1 1 0 0 1 1 1v3h3a1 1 0 1 1 0 2H9v3a1 1 0 1 1-2 0V9H4a1 1 0 1 1 0-2h3V4a1 1 0 0 1 1-1z"/>
                        </svg>
                        "New Tenant"
                    </a>
                </div>
            </div>

            // ── Content ──────────────────────────────────────────────────────
            <Suspense fallback=move || view! {
                <div style="display:flex;align-items:center;justify-content:center;height:200px;gap:10px;color:var(--text-muted);">
                    <div style="width:18px;height:18px;border:2px solid var(--primary);border-top-color:transparent;border-radius:50%;animation:spin 0.7s linear infinite;"></div>
                    "Loading tenants…"
                </div>
            }>
            {move || {
                let all_tenants = tenants_res.get().unwrap_or_default();
                let q = search.get().to_lowercase();
                let status_filter = filter_status.get();

                let tenants: Vec<TenantStatModel> = all_tenants.into_iter().filter(|t| {
                    // search filter
                    let name_match = t.name.to_lowercase().contains(&q)
                        || t.tenant_id.to_lowercase().contains(&q)
                        || t.slug.to_lowercase().contains(&q);
                    if !q.is_empty() && !name_match { return false; }

                    // status filter
                    if status_filter != "all" {
                        let tenant_status = t.site_status.as_deref().unwrap_or("active");
                        if !tenant_status.to_lowercase().starts_with(&status_filter) { return false; }
                    }
                    true
                }).collect();

                if tenants.is_empty() && search.get().is_empty() && filter_status.get() == "all" {
                    return view! {
                        <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;padding:80px 24px;gap:16px;text-align:center;">
                            <div style="font-size:48px;">"🏗"</div>
                            <h2 style="font-size:18px;font-weight:700;color:var(--text-primary);margin:0;">"No tenants provisioned yet"</h2>
                            <p style="color:var(--text-muted);font-size:13px;max-width:420px;margin:0;">
                                "Your platform has no tenants. Provision your first tenant to get started — it creates the app instance, domain, CMS, and admin user in one step."
                            </p>
                            <a href="/apps/new" class="btn btn-primary" style="text-decoration:none;margin-top:8px;">
                                "Provision First Tenant"
                            </a>
                        </div>
                    }.into_any();
                }

                if tenants.is_empty() {
                    return view! {
                        <div style="padding:48px 24px;text-align:center;color:var(--text-muted);font-size:13px;">
                            "No tenants match your search."
                        </div>
                    }.into_any();
                }

                view! {
                    <div class="w-full" style="padding:0 0 24px;">
                        // ── Summary stats bar ─────────────────────────────────
                        {
                            let all = tenants_res.get().unwrap_or_default();
                            let total = all.len();
                            let active = all.iter().filter(|t| t.site_status.as_deref().unwrap_or("active") == "active").count();
                            let total_mrr: i64 = all.iter().filter_map(|t| t.mrr_cents).sum();
                            view! {
                                <div style="display:flex;gap:24px;padding:16px 0 20px;border-bottom:1px solid var(--border,rgba(255,255,255,0.06));margin-bottom:4px;">
                                    <div>
                                        <div style="font-size:10px;font-weight:600;letter-spacing:0.07em;color:var(--text-muted);text-transform:uppercase;margin-bottom:2px;">"Total Tenants"</div>
                                        <div style="font-size:22px;font-weight:800;color:var(--text-primary);font-family:monospace;">{total.to_string()}</div>
                                    </div>
                                    <div>
                                        <div style="font-size:10px;font-weight:600;letter-spacing:0.07em;color:var(--text-muted);text-transform:uppercase;margin-bottom:2px;">"Active"</div>
                                        <div style="font-size:22px;font-weight:800;color:var(--green);font-family:monospace;">{active.to_string()}</div>
                                    </div>
                                    <div>
                                        <div style="font-size:10px;font-weight:600;letter-spacing:0.07em;color:var(--text-muted);text-transform:uppercase;margin-bottom:2px;">"Platform MRR"</div>
                                        <div style="font-size:22px;font-weight:800;color:var(--cobalt);font-family:monospace;">{fmt_mrr(Some(total_mrr))}</div>
                                    </div>
                                </div>
                            }
                        }

                        // ── Table ─────────────────────────────────────────────
                        // Setup x/4 and Health are client heuristics (not backend fields).
                        <div class="table-container" style="background:var(--bg-surface,#111520);border:1px solid var(--border-default,rgba(255,255,255,0.08));border-radius:6px;overflow:hidden;">
                        <table style="width:100%;border-collapse:collapse;">
                            <thead>
                                <tr>
                                    <th style="text-align:left;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));">"Tenant"</th>
                                    <th style="text-align:left;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));">"Plan"</th>
                                    <th style="text-align:right;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));">"MRR"</th>
                                    <th style="text-align:center;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));">"Users"</th>
                                    <th style="text-align:center;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));">"Setup"</th>
                                    <th style="text-align:center;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));">"Health"</th>
                                    <th style="text-align:right;padding:10px 14px;font-size:10px;font-weight:700;letter-spacing:0.06em;text-transform:uppercase;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.06));"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {tenants.into_iter().map(|t| {
                                    let status = t.site_status.clone().unwrap_or_else(|| "active".to_string());
                                    let hcolor = health_color(&status);
                                    let score = setup_score(&t);
                                    let scolor = score_color(score);
                                    let mrr = fmt_mrr(t.mrr_cents);
                                    let plan = t.plan.clone().unwrap_or_else(|| "—".to_string());
                                    let joined = t.joined_at.as_ref()
                                        .and_then(|d| d.get(..7))
                                        .unwrap_or("—")
                                        .to_string();

                                    // Always route to /tenants/:tenant_id — the detail page
                                    // is keyed by path param, never by the dropdown signal.
                                    let href = format!("/tenants/{}", t.tenant_id);
                                    let href_click = href.clone();

                                    let initial = t.name.chars().next().unwrap_or('?').to_string();
                                    let tenant_name = t.name.clone();
                                    let tenant_id_short = t.tenant_id.chars().take(8).collect::<String>();

                                    view! {
                                        <tr
                                            style="cursor:pointer;transition:background 0.12s;"
                                            on:click=move |_| {
                                                let _ = web_sys::window()
                                                    .and_then(|w| w.location().assign(&href_click).ok());
                                            }
                                        >
                                            // Tenant identity
                                            <td style="padding:12px 14px;border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                <div style="display:flex;align-items:center;gap:10px;">
                                                    <div style="width:34px;height:34px;border-radius:8px;background:rgba(59,130,246,0.14);color:#60a5fa;font-size:13px;font-weight:800;display:flex;align-items:center;justify-content:center;flex-shrink:0;">
                                                        {initial}
                                                    </div>
                                                    <div>
                                                        <div style="font-size:13px;font-weight:600;color:var(--text-primary);">{tenant_name}</div>
                                                        <div style="font-size:10px;font-family:monospace;color:var(--text-muted);margin-top:1px;">
                                                            {tenant_id_short}"… · "{joined}
                                                        </div>
                                                    </div>
                                                </div>
                                            </td>
                                            // Plan
                                            <td style="padding:12px 14px;border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                <span style="font-size:11px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.05);color:var(--text-muted);white-space:nowrap;">
                                                    {plan}
                                                </span>
                                            </td>
                                            // MRR
                                            <td style="padding:12px 14px;text-align:right;font-family:monospace;font-size:12px;font-weight:700;color:var(--text-primary);border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                {mrr}
                                            </td>
                                            // Users
                                            <td style="padding:12px 14px;text-align:center;font-size:12px;color:var(--text-muted);border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                {t.profile_count.to_string()}
                                            </td>
                                            // Setup score
                                            <td style="padding:12px 14px;text-align:center;border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                <span style=format!("font-size:11px;font-weight:700;color:{}", scolor)>
                                                    {format!("{}/4", score)}
                                                </span>
                                            </td>
                                            // Health dot
                                            <td style="padding:12px 14px;text-align:center;border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                <span style=format!("display:inline-block;width:8px;height:8px;border-radius:50%;background:{}", hcolor)></span>
                                            </td>
                                            // Open link
                                            <td style="padding:12px 14px;text-align:right;border-bottom:1px solid var(--border,rgba(255,255,255,0.04));">
                                                <a
                                                    href=href
                                                    style="font-size:11px;color:var(--cobalt);text-decoration:none;white-space:nowrap;font-weight:500;"
                                                    on:click=move |e| e.stop_propagation()
                                                >
                                                    "Open →"
                                                </a>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                        </div>
                    </div>
                }.into_any()
            }}
            </Suspense>
        </div>
    }
}
