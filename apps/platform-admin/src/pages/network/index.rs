use crate::api::admin::{get_all_platform_apps, get_tenant_stats};
use leptos::prelude::*;

#[component]
pub fn NetworkRegistry() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let apps_res =
        LocalResource::new(|| async move { get_all_platform_apps().await.unwrap_or_default() });
    let tenants_res =
        LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    let handle_export = move |_| {
        toast.show_toast(
            "Export",
            "Network registry database exported to JSON.",
            "success",
        );
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Network Instances"</h1>
                    <p class="page-subtitle">"Public-facing search portals and marketplaces served by the Atlas Platform"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost" on:click=handle_export>"Export Database"</button>
                    <button class="btn btn-primary">"+ New Instance"</button>
                </div>
            </div>

            // ── KPI Row ──
            <div class="kpi-row">
                <div class="kpi-card">
                    <div class="kpi-label">"Total Instances"</div>
                    <div class="kpi-value mono">
                        {move || apps_res.get().unwrap_or_default().len().to_string()}
                    </div>
                    <div class="kpi-delta neutral">"Registered app instances"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Live"</div>
                    <div class="kpi-value mono" style="color:var(--green)">
                        {move || apps_res.get().unwrap_or_default().iter().filter(|a| a.site_status == "active").count().to_string()}
                    </div>
                    <div class="kpi-delta neutral">"Edge networks active"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Tenants"</div>
                    <div class="kpi-value mono" style="color:var(--amber)">
                        {move || tenants_res.get().unwrap_or_default().len().to_string()}
                    </div>
                    <div class="kpi-delta neutral">"Registered tenants"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Provisioning"</div>
                    <div class="kpi-value mono">
                        {move || apps_res.get().unwrap_or_default().iter().filter(|a| a.site_status == "provisioning").count().to_string()}
                    </div>
                    <div class="kpi-delta neutral">"In provisioning state"</div>
                </div>
            </div>

            // ── Instances Table ──
            <div class="section">
                <div class="section-header">
                    <div class="section-title">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4"><rect x="1" y="2" width="12" height="10" rx="1"/><path d="M1 6h12"/></svg>
                        "Active Network Directory"
                        <span class="section-count">{move || format!("{} instances", apps_res.get().unwrap_or_default().len())}</span>
                    </div>
                    <input type="text" placeholder="Filter instances..." style="font-size:12px;"/>
                </div>
                <Suspense fallback=move || view! { <div class="p-8 muted">"Loading network instances…"</div> }>
                <table>
                    <thead><tr>
                        <th>"Instance"</th>
                        <th>"Type"</th>
                        <th>"Status"</th>
                        <th>"Tenant"</th>
                        <th>"Domain"</th>
                    </tr></thead>
                    <tbody>
                        {move || apps_res.get().unwrap_or_default().into_iter().map(|item| {
                            let status = item.site_status.clone();
                            let (status_color, status_bg) = match status.as_str() {
                                "active"       => ("var(--green)",  "var(--green-dim)"),
                                "provisioning" => ("var(--cobalt)", "var(--cobalt-dim)"),
                                "beta"         => ("var(--amber)",  "var(--amber-dim)"),
                                "suspended"    => ("var(--red)",    "var(--red-dim)"),
                                _              => ("var(--text-muted)", "var(--bg-elevated)"),
                            };
                            view! {
                                <tr>
                                    <td>
                                        <div style="font-weight:600">{item.name.clone()}</div>
                                        <div class="mono muted" style="font-size:10px">{item.instance_id.clone()}</div>
                                    </td>
                                    <td>
                                        <span class="plan-badge">{item.app_type.clone()}</span>
                                    </td>
                                    <td>
                                        <span class="plan-badge" style=format!("color:{c};border-color:{c};background:{b}", c=status_color, b=status_bg)>
                                            {item.site_status.clone()}
                                        </span>
                                    </td>
                                    <td class="mono secondary">{item.tenant_id.clone()}</td>
                                    <td class="mono secondary">{item.domain.clone()}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
                </Suspense>
            </div>
        </div>
    }
}
