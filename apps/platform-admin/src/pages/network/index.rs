use leptos::prelude::*;
use crate::api::admin::{get_all_platform_apps, get_tenant_stats};

#[component]
pub fn NetworkRegistry() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let apps_res = LocalResource::new(|| async move { get_all_platform_apps().await.unwrap_or_default() });
    let tenants_res = LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    let handle_export = move |_| {
        toast.show_toast("Export", "Network registry database exported to JSON.", "success");
    };

    view! {
        <div class="space-y-6">
            // ── Page Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">"Network Instances"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">"Public-facing search portals and marketplaces served by the Atlas Platform"</p>
                </div>
                <div class="flex items-center gap-3">
                    <button
                        class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 hover:text-on-surface transition-all active:scale-95"
                        on:click=handle_export
                    >
                        "Export Database"
                    </button>
                    <button class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all">
                        "+ New Instance"
                    </button>
                </div>
            </div>

            // ── KPI Row ──
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Total Instances"</span>
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">
                        {move || apps_res.get().unwrap_or_default().len().to_string()}
                    </span>
                    <span class="text-[10px] text-on-surface-variant/60 font-semibold mt-1">"registered app instances"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Live"</span>
                    <span class="text-3xl font-extrabold text-emerald-400 tracking-tight mt-2">
                        {move || apps_res.get().unwrap_or_default().iter().filter(|a| a.site_status == "active").count().to_string()}
                    </span>
                    <span class="text-[10px] text-on-surface-variant/50 mt-1">"Edge networks active"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Tenants"</span>
                    <span class="text-3xl font-extrabold text-amber-400 tracking-tight mt-2">
                        {move || tenants_res.get().unwrap_or_default().len().to_string()}
                    </span>
                    <span class="text-[10px] text-on-surface-variant/50 mt-1">"registered tenants"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Provisioning"</span>
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">
                        {move || apps_res.get().unwrap_or_default().iter().filter(|a| a.site_status == "provisioning").count().to_string()}
                    </span>
                    <span class="text-[10px] text-on-surface-variant/60 font-semibold mt-1">"in provisioning state"</span>
                </div>
            </div>

            // ── Table Container ──
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                <div class="flex justify-between items-center p-5 border-b border-outline-variant/20 bg-surface-container-high/20">
                    <span class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                        {move || format!("Active Network Directory · {} instances", apps_res.get().unwrap_or_default().len())}
                    </span>
                    <input
                        type="text"
                        placeholder="Filter instances..."
                        class="bg-surface-container border border-outline-variant/30 text-xs rounded-lg px-3 py-1.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all text-on-surface w-48"
                    />
                </div>

                <div class="overflow-x-auto">
                    <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant/50 text-xs">"Loading network instances..."</div> }>
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-surface-container-high/40 border-b border-outline-variant/20 text-[10px] tracking-wider uppercase text-on-surface-variant">
                                <th class="px-6 py-4 font-semibold">"Instance"</th>
                                <th class="px-6 py-4 font-semibold">"Type"</th>
                                <th class="px-6 py-4 font-semibold">"Status"</th>
                                <th class="px-6 py-4 font-semibold">"Tenant"</th>
                                <th class="px-6 py-4 font-semibold">"Domain"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-outline-variant/10 text-xs text-on-surface">
                            {move || apps_res.get().unwrap_or_default().into_iter().map(|item| {
                                let status = item.site_status.clone();
                                let status_class = match status.as_str() {
                                    "active" => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider",
                                    "provisioning" => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-blue-500/10 text-blue-400 border border-blue-500/20 uppercase tracking-wider",
                                    "beta" => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 uppercase tracking-wider",
                                    "suspended" => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-red-500/10 text-red-400 border border-red-500/20 uppercase tracking-wider",
                                    _ => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-muted text-on-surface-variant/60 border border-outline-variant/30 uppercase tracking-wider",
                                };
                                view! {
                                    <tr class="hover:bg-surface-bright/5 transition-colors cursor-pointer">
                                        <td class="px-6 py-4">
                                            <div class="font-bold text-on-surface">{item.name.clone()}</div>
                                            <div class="text-[10px] text-on-surface-variant/60 font-mono mt-0.5">{item.instance_id.clone()}</div>
                                        </td>
                                        <td class="px-6 py-4">
                                            <span class="px-1.5 py-0.5 rounded bg-surface-container border border-outline-variant/20 text-[9px] font-mono text-on-surface-variant/80 uppercase">
                                                {item.app_type.clone()}
                                            </span>
                                        </td>
                                        <td class="px-6 py-4">
                                            <span class=status_class>
                                                {item.site_status.clone()}
                                            </span>
                                        </td>
                                        <td class="px-6 py-4 font-mono text-xs text-on-surface-variant/80">{item.tenant_id.clone()}</td>
                                        <td class="px-6 py-4 text-on-surface-variant/80 font-mono">{item.domain.clone()}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
