use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkInstanceItem {
    pub name: String,
    pub domain: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub tenants: i32,
    pub listings: i32,
    pub created: String,
}

#[component]
pub fn NetworkRegistry() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let instances = RwSignal::new(vec![
        NetworkInstanceItem {
            name: "leira Rentals".to_string(),
            domain: "leira-rentals.app".to_string(),
            status: "Live".to_string(),
            capabilities: vec!["ltr".to_string(), "map".to_string()],
            tenants: 8,
            listings: 242,
            created: "Feb 12, 2024".to_string(),
        },
        NetworkInstanceItem {
            name: "leira Stays".to_string(),
            domain: "leira-stays.app".to_string(),
            status: "Live".to_string(),
            capabilities: vec!["str".to_string(), "tax".to_string()],
            tenants: 12,
            listings: 156,
            created: "Mar 01, 2024".to_string(),
        },
        NetworkInstanceItem {
            name: "leira Pros".to_string(),
            domain: "leira-pros.app".to_string(),
            status: "Beta".to_string(),
            capabilities: vec!["b2b".to_string(), "leads".to_string()],
            tenants: 3,
            listings: 48,
            created: "May 20, 2026".to_string(),
        },
        NetworkInstanceItem {
            name: "Nesta Directory".to_string(),
            domain: "nesta-directory.app".to_string(),
            status: "Draft".to_string(),
            capabilities: vec!["ltr".to_string()],
            tenants: 0,
            listings: 0,
            created: "June 08, 2026".to_string(),
        },
    ]);

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
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"7"</span>
                    <span class="text-[10px] text-emerald-400 font-semibold mt-1">"↑ 2 this quarter"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Live"</span>
                    <span class="text-3xl font-extrabold text-emerald-400 tracking-tight mt-2">"5"</span>
                    <span class="text-[10px] text-on-surface-variant/50 mt-1">"Edge networks active"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Beta"</span>
                    <span class="text-3xl font-extrabold text-amber-400 tracking-tight mt-2">"1"</span>
                    <span class="text-[10px] text-on-surface-variant/50 mt-1">"Undergoing calibrations"</span>
                </div>
                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                    <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Active Syndications"</span>
                    <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"9"</span>
                    <span class="text-[10px] text-emerald-400 font-semibold mt-1">"↑ 3 this month"</span>
                </div>
            </div>

            // ── Table Container ──
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                <div class="flex justify-between items-center p-5 border-b border-outline-variant/20 bg-surface-container-high/20">
                    <span class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Active Network Directory"</span>
                    <input 
                        type="text" 
                        placeholder="Filter instances..." 
                        class="bg-surface-container border border-outline-variant/30 text-xs rounded-lg px-3 py-1.5 focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all text-on-surface w-48"
                    />
                </div>

                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-surface-container-high/40 border-b border-outline-variant/20 text-[10px] tracking-wider uppercase text-on-surface-variant">
                                <th class="px-6 py-4 font-semibold">"Instance"</th>
                                <th class="px-6 py-4 font-semibold">"Status"</th>
                                <th class="px-6 py-4 font-semibold">"Capabilities"</th>
                                <th class="px-6 py-4 font-semibold">"Tenants"</th>
                                <th class="px-6 py-4 font-semibold">"Listings"</th>
                                <th class="px-6 py-4 font-semibold">"Created"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-outline-variant/10 text-xs text-on-surface">
                            {move || instances.get().into_iter().map(|item| {
                                view! {
                                    <tr class="hover:bg-surface-bright/5 transition-colors cursor-pointer">
                                        <td class="px-6 py-4">
                                            <div class="font-bold text-on-surface">{item.name.clone()}</div>
                                            <div class="text-[10px] text-on-surface-variant/60 font-mono mt-0.5">{item.domain.clone()}</div>
                                        </td>
                                        <td class="px-6 py-4">
                                            {
                                                let status = item.status.clone();
                                                view! {
                                                    <span class=move || match status.as_str() {
                                                        "Live" => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider",
                                                        "Beta" => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 uppercase tracking-wider",
                                                        _ => "inline-flex items-center px-2 py-0.5 rounded text-[8px] font-bold bg-muted text-on-surface-variant/60 border border-outline-variant/30 uppercase tracking-wider",
                                                    }>
                                                        {item.status.clone()}
                                                    </span>
                                                }
                                            }
                                        </td>
                                        <td class="px-6 py-4">
                                            <div class="flex gap-1">
                                                {
                                                    item.capabilities.clone().into_iter().map(|cap| {
                                                        view! {
                                                            <span class="px-1.5 py-0.5 rounded bg-surface-container border border-outline-variant/20 text-[9px] font-mono text-on-surface-variant/80 uppercase">
                                                                {cap.clone()}
                                                            </span>
                                                        }
                                                    }).collect_view()
                                                }
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 font-bold font-mono">{item.tenants}</td>
                                        <td class="px-6 py-4 font-bold font-mono text-primary">{item.listings}</td>
                                        <td class="px-6 py-4 text-on-surface-variant/80">{item.created.clone()}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
