use leptos::prelude::*;

#[component]
pub fn BillingDashboard() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6 animate-fade-in fade-in-up">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-on-surface tracking-tight mb-1">"Financial Dashboard"</h1>
                    <p class="text-on-surface-variant text-sm">"Global MRR, Gateway Toggles, and Network Ledger"</p>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <!-- MRR Card -->
                <div class="glass-panel p-6 rounded-2xl border border-outline-variant/30 flex flex-col gap-4">
                    <h3 class="text-sm font-semibold text-on-surface-variant uppercase tracking-widest">"Global MRR"</h3>
                    <div class="text-4xl font-bold text-primary">"$24,500.00"</div>
                    <div class="text-xs text-primary-fixed">"+12% from last month"</div>
                </div>

                <!-- Crypto vs Fiat Card -->
                <div class="glass-panel p-6 rounded-2xl border border-outline-variant/30 flex flex-col gap-4">
                    <h3 class="text-sm font-semibold text-on-surface-variant uppercase tracking-widest">"Volume Breakdown"</h3>
                    <div class="flex gap-4">
                        <div class="flex-1">
                            <div class="text-2xl font-bold text-on-surface">"82%"</div>
                            <div class="text-xs text-on-surface-variant">"Fiat / Stripe"</div>
                        </div>
                        <div class="flex-1 border-l border-outline-variant/20 pl-4">
                            <div class="text-2xl font-bold text-[#f7931a]">"18%"</div>
                            <div class="text-xs text-on-surface-variant">"BTC / Zaprite"</div>
                        </div>
                    </div>
                </div>

                <!-- Churn Metrics -->
                <div class="glass-panel p-6 rounded-2xl border border-outline-variant/30 flex flex-col gap-4">
                    <h3 class="text-sm font-semibold text-on-surface-variant uppercase tracking-widest">"Network Churn"</h3>
                    <div class="text-4xl font-bold text-error">"1.2%"</div>
                    <div class="text-xs text-on-surface-variant">"Within industry average"</div>
                </div>
            </div>

            <!-- Gateway Toggles -->
            <div class="glass-panel rounded-2xl border border-outline-variant/30 overflow-hidden mt-4">
                <div class="bg-surface-container-high px-6 py-4 border-b border-outline-variant/30 flex justify-between items-center">
                    <h3 class="font-bold text-on-surface">"Payment Gateways"</h3>
                </div>
                <div class="p-6 grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div class="p-4 rounded-xl border border-outline-variant/50 flex justify-between items-center bg-[#05183c]">
                        <div>
                            <div class="font-bold text-on-surface">"Stripe"</div>
                            <div class="text-xs text-on-surface-variant">"CC & ACH"</div>
                        </div>
                        <div class="w-10 h-5 bg-primary rounded-full relative">
                            <div class="absolute right-1 top-1 w-3 h-3 bg-white rounded-full"></div>
                        </div>
                    </div>
                    <div class="p-4 rounded-xl border border-outline-variant/50 flex justify-between items-center bg-[#05183c]">
                        <div>
                            <div class="font-bold text-on-surface">"Paddle"</div>
                            <div class="text-xs text-on-surface-variant">"Global MOR"</div>
                        </div>
                        <div class="w-10 h-5 bg-surface-container-highest rounded-full relative border border-outline-variant/40">
                            <div class="absolute left-1 top-1 w-3 h-3 bg-on-surface-variant rounded-full"></div>
                        </div>
                    </div>
                    <div class="p-4 rounded-xl border border-outline-variant/50 flex justify-between items-center bg-[#05183c]">
                        <div>
                            <div class="font-bold text-on-surface">"Zaprite"</div>
                            <div class="text-xs text-on-surface-variant">"Bitcoin & LN"</div>
                        </div>
                        <div class="w-10 h-5 bg-primary rounded-full relative">
                            <div class="absolute right-1 top-1 w-3 h-3 bg-white rounded-full"></div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Global Transaction Ledger (Stub) -->
            <div class="glass-panel rounded-2xl border border-outline-variant/30 overflow-hidden mt-4">
                <div class="bg-surface-container-high px-6 py-4 border-b border-outline-variant/30">
                    <h3 class="font-bold text-on-surface">"Recent Transactions"</h3>
                </div>
                <div class="p-6 flex flex-col items-center justify-center py-12 text-on-surface-variant">
                    <span class="material-symbols-outlined text-4xl mb-2 opacity-50">"receipt_long"</span>
                    <p>"Transaction history will populate as networks process payments."</p>
                </div>
            </div>
        </div>
    }
}
