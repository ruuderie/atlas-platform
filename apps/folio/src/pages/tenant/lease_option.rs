//! Tenant-buyer lease-option portal — `/t/option`

use leptos::prelude::*;

#[component]
pub fn TenantLeaseOption() -> impl IntoView {
    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"My Option"</h1>
                    <p class="page-subtitle">"Lease-option path to purchase · rent + Down Payment Assistance"</p>
                </div>
            </div>

            <div class="card p-5 mb-4" style="background:#191c1e;color:#fff;border-radius:1rem;">
                <p class="text-xs uppercase opacity-60 mb-1">"Path to purchase"</p>
                <div class="grid gap-4" style="grid-template-columns:1fr 1fr;">
                    <div>
                        <p class="text-sm opacity-70">"Option price"</p>
                        <p class="text-2xl font-bold">"$234,900"</p>
                    </div>
                    <div>
                        <p class="text-sm opacity-70">"After DPAP credits"</p>
                        <p class="text-2xl font-bold" style="color:#85f8c4;">"$233,250"</p>
                    </div>
                    <div>
                        <p class="text-sm opacity-70">"Option deposit"</p>
                        <p class="text-xl font-bold">"$10,000"</p>
                    </div>
                    <div>
                        <p class="text-sm opacity-70">"DP applied"</p>
                        <p class="text-xl font-bold">"$2,400"</p>
                    </div>
                </div>
                <p class="text-xs opacity-70 mt-3">
                    "Wire live lease_option contract terms from G-11 when your household has an installed Deal Ops disposition."
                </p>
            </div>

            <div class="card p-4 mb-4">
                <h3 class="font-bold mb-2">"This month"</h3>
                <div class="flex justify-between py-2 border-b text-sm">
                    <span>"Rent (required)"</span>
                    <strong>"$1,750"</strong>
                </div>
                <div class="flex justify-between py-2 text-sm">
                    <span>"DPAP extra (optional · non-refundable)"</span>
                    <strong>"$300 → $450 price credit"</strong>
                </div>
                <a class="btn btn-primary btn-sm mt-3" href="/t/payments">"Pay"</a>
            </div>

            <div class="card p-4 text-sm text-on-surface-variant">
                <p>"Repairs after day 30 are your responsibility. Late fees apply to rent only. Up to 2 DPAP skips without voiding the option."</p>
                <a class="underline mt-2 inline-block" href="/t/documents">"View lease-option & DPAP addendum"</a>
            </div>
        </div>
    }
}
