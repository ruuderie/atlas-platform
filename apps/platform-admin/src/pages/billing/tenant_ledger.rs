use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn TenantLedger() -> impl IntoView {
    let params = use_params_map();
    let tenant_id = move || params.with(|p| p.get("id").unwrap_or_default());

    view! {
        <div class="flex flex-col gap-6 animate-fade-in fade-in-up">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-on-surface tracking-tight mb-1">"Tenant Ledger"</h1>
                    <p class="text-on-surface-variant text-sm text-mono">"ID: " {tenant_id}</p>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <!-- Current Subscription -->
                <div class="glass-panel p-6 rounded-2xl border border-outline-variant/30 flex flex-col gap-4 relative overflow-hidden">
                    <div class="absolute top-0 right-0 w-32 h-32 bg-primary/10 rounded-bl-full mix-blend-screen opacity-50"></div>
                    <h3 class="text-sm font-semibold text-on-surface-variant uppercase tracking-widest">"Active Plan"</h3>
                    <div class="text-2xl font-bold text-on-surface">"Enterprise Anchor"</div>
                    <div class="flex gap-4 items-center">
                        <span class="px-2.5 py-0.5 rounded-full bg-[#1b5e20] text-[#a5d6a7] text-[10px] font-bold uppercase tracking-widest">"ACTIVE"</span>
                        <div class="text-xs text-on-surface-variant">"Renews: Oct 15, 2026"</div>
                    </div>
                </div>

                <!-- MRR for this tenant -->
                <div class="glass-panel p-6 rounded-2xl border border-outline-variant/30 flex flex-col gap-4">
                    <h3 class="text-sm font-semibold text-on-surface-variant uppercase tracking-widest">"Tenant MRR"</h3>
                    <div class="text-4xl font-bold text-primary">"$199.00"</div>
                    <div class="text-xs text-on-surface-variant">"Paid via Stripe"</div>
                </div>
            </div>

            <!-- Transaction History -->
            <div class="glass-panel rounded-2xl border border-outline-variant/30 overflow-hidden mt-4">
                <div class="bg-surface-container-high px-6 py-4 border-b border-outline-variant/30">
                    <h3 class="font-bold text-on-surface">"Transaction History"</h3>
                </div>
                <div class="p-0">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-[#05183c] border-b border-outline-variant/20">
                                <th class="p-4 text-xs font-semibold text-on-surface-variant uppercase tracking-widest">"Date"</th>
                                <th class="p-4 text-xs font-semibold text-on-surface-variant uppercase tracking-widest">"Amount"</th>
                                <th class="p-4 text-xs font-semibold text-on-surface-variant uppercase tracking-widest">"Provider"</th>
                                <th class="p-4 text-xs font-semibold text-on-surface-variant uppercase tracking-widest">"Status"</th>
                                <th class="p-4 text-xs font-semibold text-on-surface-variant uppercase tracking-widest">"Tx ID"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <!-- Mock Data -->
                            <tr class="border-b border-outline-variant/10 hover:bg-surface-bright/20">
                                <td class="p-4 text-sm text-on-surface">"Sep 15, 2026"</td>
                                <td class="p-4 text-sm text-on-surface font-mono">"$199.00"</td>
                                <td class="p-4 text-sm text-on-surface">
                                    <div class="flex items-center gap-2">
                                        <div class="w-2 h-2 rounded-full bg-primary"></div>
                                        "Stripe"
                                    </div>
                                </td>
                                <td class="p-4 text-sm">
                                    <span class="text-[#a5d6a7]">"Completed"</span>
                                </td>
                                <td class="p-4 text-xs text-on-surface-variant font-mono">"pi_3Pj...Kq9T"</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
