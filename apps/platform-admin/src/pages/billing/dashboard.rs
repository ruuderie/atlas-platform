use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::button::{Button, ButtonVariant, ButtonSize};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::switch::Switch;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use crate::api::admin::{get_billing_plans, get_all_transactions};
use crate::api::analytics::{get_billing_summary, get_business_kpis};

#[component]
pub fn BillingDashboard() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Active workspace tab state
    let active_tab = RwSignal::new("products_plans".to_string());

    // Modals state
    let show_new_sub_modal = RwSignal::new(false);
    let show_log_payment_modal = RwSignal::new(false);
    let show_exemption_modal = RwSignal::new(false);

    // Form inputs state
    let new_sub_tenant = RwSignal::new("Nexus PM Group".to_string());
    let new_sub_product = RwSignal::new("Folio".to_string());
    let new_sub_plan = RwSignal::new("Starter - $400/mo".to_string());

    let log_payment_tenant = RwSignal::new("South Beach Nets".to_string());
    let log_payment_amount = RwSignal::new("".to_string());
    let log_payment_ref = RwSignal::new("".to_string());

    let exempt_tenant = RwSignal::new("".to_string());
    let exempt_reason = RwSignal::new("".to_string());

    // Filter selectors state
    let filter_tenant = RwSignal::new("All Tenants".to_string());
    let filter_rail = RwSignal::new("All Rails".to_string());
    let filter_status = RwSignal::new("All Statuses".to_string());

    // Derived style helpers
    let tab_class = move |tab_id: &str| {
        let active = active_tab.get() == tab_id;
        if active {
            "px-4 py-2.5 text-xs font-bold text-primary border-b-2 border-primary bg-transparent outline-none transition-all"
        } else {
            "px-4 py-2.5 text-xs font-semibold text-on-surface-variant hover:text-on-surface bg-transparent outline-none transition-all"
        }
    };

    // ── Real data resources ──
    let billing_summary = LocalResource::new(|| async move { get_billing_summary().await.ok() });
    let business_kpis = LocalResource::new(|| async move { get_business_kpis().await.ok() });
    let billing_plans = LocalResource::new(|| async move { get_billing_plans().await.unwrap_or_default() });
    let transactions = LocalResource::new(|| async move { get_all_transactions().await.unwrap_or_default() });


    // Handle Form actions
    let handle_new_subscription = move |_| {
        let tenant = new_sub_tenant.get();
        let product = new_sub_product.get();
        let plan = new_sub_plan.get();

        toast.show_toast(
            "Success",
            &format!("Subscription for {} ({}) on plan '{}' created.", tenant, product, plan),
            "success",
        );
        show_new_sub_modal.set(false);
    };

    let handle_log_payment = move |_| {
        let tenant = log_payment_tenant.get();
        let amount = log_payment_amount.get();
        let reference = log_payment_ref.get();

        if amount.is_empty() { return; }

        toast.show_toast(
            "Success",
            &format!("Manual payment of ${} logged for {}. Ref: {}", amount, tenant, reference),
            "success",
        );
        log_payment_amount.set("".to_string());
        log_payment_ref.set("".to_string());
        show_log_payment_modal.set(false);
    };

    let handle_exempt_tenant = move |_| {
        let tenant = exempt_tenant.get();
        let reason = exempt_reason.get();

        if tenant.is_empty() { return; }

        toast.show_toast(
            "Success",
            &format!("Exemption active for {}. Reason: {}", tenant, reason),
            "success",
        );
        exempt_tenant.set("".to_string());
        exempt_reason.set("".to_string());
        show_exemption_modal.set(false);
    };

    let handle_export = move |_| {
        toast.show_toast(
            "Preparing Export",
            "Financial telemetry export dispatched to downloads.",
            "info",
        );
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumb & Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm animate-fade-in fade-in-up">
                <div>
                    <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">"Billing Overview"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">
                        "Platform financial health · Select a product to drill into its plans and revenue"
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <select class="bg-surface-container-highest border border-outline/20 text-on-surface text-xs rounded-lg p-2 focus:ring-primary focus:border-primary">
                        <option>"June 2026"</option>
                        <option>"May 2026"</option>
                        <option>"Q2 2026"</option>
                        <option>"YTD"</option>
                    </select>
                    <Button variant=ButtonVariant::Outline on:click=handle_export>
                        "↓ Export"
                    </Button>
                </div>
            </div>

            // ── KPI Strip ──
            <div class="grid grid-cols-2 md:grid-cols-7 gap-4 bg-surface-container-low border border-outline-variant/10 p-5 rounded-2xl shadow-xs overflow-x-auto select-none">
                <div class="space-y-1 min-w-[100px]">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Total MRR"</span>
                    <div class="text-xl font-extrabold text-primary font-mono">
                        {move || business_kpis.get().flatten().map(|k| format!("${:.0}k", k.mrr.value / 1000.0)).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[10px] text-emerald-400 font-medium">
                        {move || business_kpis.get().flatten().map(|k| {
                            let delta = k.mrr.value - k.mrr.previous_value;
                            let pct = if k.mrr.previous_value > 0.0 { (delta / k.mrr.previous_value) * 100.0 } else { 0.0 };
                            format!("↑ {:.0}% vs prev", pct)
                        }).unwrap_or("—".to_string())}
                    </p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Active Subs"</span>
                    <div class="text-xl font-extrabold font-mono">
                        {move || billing_summary.get().flatten().map(|s| s.active_subscriptions.to_string()).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[9px] text-on-surface-variant/70 truncate">
                        {move || billing_summary.get().flatten().map(|s| format!("{} trial · {} grace · {} susp.", s.in_trial, s.in_grace_period, s.suspended)).unwrap_or("—".to_string())}
                    </p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Failed Invoices"</span>
                    <div class="text-xl font-extrabold text-amber-500 font-mono">
                        {move || billing_summary.get().flatten().map(|s| s.failed_invoices_count.to_string()).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[10px] text-error font-medium">
                        {move || billing_summary.get().flatten().map(|s| format!("${:.0}k outstanding", s.failed_invoices_value / 1000.0)).unwrap_or("—".to_string())}
                    </p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Collection Rate"</span>
                    <div class="text-xl font-extrabold text-emerald-400 font-mono">
                        {move || billing_summary.get().flatten().map(|s| format!("{:.0}%", s.collection_success_rate * 100.0)).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[10px] text-on-surface-variant/80">"G-03"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Billing Plans"</span>
                    <div class="text-xl font-extrabold font-mono">
                        {move || billing_plans.get().map(|p| p.len().to_string()).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[10px] text-on-surface-variant/80">"active plans"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Transactions"</span>
                    <div class="text-xl font-extrabold font-mono">
                        {move || transactions.get().map(|t| t.len().to_string()).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[10px] text-on-surface-variant/80">"ledger entries"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Churn Rate"</span>
                    <div class="text-xl font-extrabold font-mono">
                        {move || billing_summary.get().flatten().map(|s| format!("{:.1}%", s.gross_churn_rate * 100.0)).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[9px] text-on-surface-variant/70 font-mono">"G-17"</p>
                </div>
            </div>

            // ── Tabs Bar ──
            <div class="flex border-b border-outline-variant/15 overflow-x-auto shrink-0 select-none">
                <button class=move || tab_class("products_plans") on:click=move |_| active_tab.set("products_plans".to_string())>
                    "Products & Plans"
                </button>
                <button class=move || tab_class("ledger") on:click=move |_| active_tab.set("ledger".to_string())>
                    "Ledger · G-03"
                </button>
                <button class=move || tab_class("overdue") on:click=move |_| active_tab.set("overdue".to_string())>
                    "Overdue & Disputes (3)"
                </button>
                <button class=move || tab_class("rails") on:click=move |_| active_tab.set("rails".to_string())>
                    "Payment Rails"
                </button>
                <button class=move || tab_class("commissions") on:click=move |_| active_tab.set("commissions".to_string())>
                    "Commission Plans · G-25"
                </button>
                <button class=move || tab_class("tax") on:click=move |_| active_tab.set("tax".to_string())>
                    "Tax & Filings · G-17"
                </button>
            </div>

            // ── TAB CONTENT: Products & Plans ──
            <Show when=move || active_tab.get() == "products_plans">
                <div class="space-y-6">
                    // Hierarchy Callout
                    <div class="flex items-center gap-3 p-4 bg-primary/10 border border-primary/20 rounded-xl text-xs text-on-surface-variant leading-relaxed animate-fade-in">
                        <span class="material-symbols-outlined text-primary text-base">"info"</span>
                        <span>
                            <strong class="text-on-surface">"Platform Products"</strong> " → Billing Plans → Tenant Subscriptions → Ledger Entries → Ledger Splits. Click a product to drill in."
                        </span>
                    </div>

                    // Product Cards Grid
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 animate-fade-in">
                        // Card 1: Folio
                        <div class="bg-surface-container-low border border-outline-variant/15 rounded-2xl p-5 relative overflow-hidden transition-all duration-200 hover:bg-surface-container hover:border-outline-variant/30 flex flex-col justify-between">
                            <div class="absolute top-0 left-0 right-0 h-1 bg-primary"></div>
                            <div>
                                <div class="flex justify-between items-start mb-3">
                                    <div>
                                        <h3 class="text-lg font-bold text-on-surface">"Folio"</h3>
                                        <p class="text-xs text-on-surface-variant">"Property Management · STR · Long-term"</p>
                                    </div>
                                    <Badge intent=BadgeIntent::Primary>"Active"</Badge>
                                </div>
                                <div class="divide-y divide-outline-variant/10 text-xs my-4 space-y-2 pt-2">
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Folio Starter · $400/mo"</span>
                                        <span class="text-on-surface-variant">"2 tenants"</span>
                                        <span class="font-bold text-primary font-mono">"$800"</span>
                                    </div>
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Folio STR Pro · $1,800/mo"</span>
                                        <span class="text-on-surface-variant">"4 tenants"</span>
                                        <span class="font-bold text-primary font-mono">"$7,200"</span>
                                    </div>
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Folio Enterprise · $6,000/mo"</span>
                                        <span class="text-on-surface-variant">"3 tenants"</span>
                                        <span class="font-bold text-primary font-mono">"$18,000"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="grid grid-cols-3 gap-2 border-t border-outline-variant/10 pt-4 mt-2 text-center">
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"MRR"</div>
                                    <div class="text-base font-extrabold text-primary font-mono mt-0.5">"$26k"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"GMV"</div>
                                    <div class="text-base font-extrabold text-emerald-400 font-mono mt-0.5">"$1.68M"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Tenants"</div>
                                    <div class="text-base font-extrabold font-mono mt-0.5">"9"</div>
                                </div>
                            </div>
                        </div>

                        // Card 2: Anchor
                        <div class="bg-surface-container-low border border-outline-variant/15 rounded-2xl p-5 relative overflow-hidden transition-all duration-200 hover:bg-surface-container hover:border-outline-variant/30 flex flex-col justify-between">
                            <div class="absolute top-0 left-0 right-0 h-1 bg-violet-500"></div>
                            <div>
                                <div class="flex justify-between items-start mb-3">
                                    <div>
                                        <h3 class="text-lg font-bold text-on-surface">"Anchor"</h3>
                                        <p class="text-xs text-on-surface-variant">"Creator OS · Blog · Resume · BTC"</p>
                                    </div>
                                    <Badge intent=BadgeIntent::Default>"Beta"</Badge>
                                </div>
                                <div class="divide-y divide-outline-variant/10 text-xs my-4 space-y-2 pt-2">
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Creator · $900/mo"</span>
                                        <span class="text-on-surface-variant">"3 tenants"</span>
                                        <span class="font-bold text-violet-400 font-mono">"$2,700"</span>
                                    </div>
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Creator Pro · $2,400/mo"</span>
                                        <span class="text-on-surface-variant">"1 tenant"</span>
                                        <span class="font-bold text-violet-400 font-mono">"$2,400"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="grid grid-cols-3 gap-2 border-t border-outline-variant/10 pt-4 mt-2 text-center">
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"MRR"</div>
                                    <div class="text-base font-extrabold text-violet-400 font-mono mt-0.5">"$5.1k"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"GMV"</div>
                                    <div class="text-base font-extrabold text-emerald-400 font-mono mt-0.5">"$220k"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Tenants"</div>
                                    <div class="text-base font-extrabold font-mono mt-0.5">"4"</div>
                                </div>
                            </div>
                        </div>

                        // Card 3: Network
                        <div class="bg-surface-container-low border border-outline-variant/15 rounded-2xl p-5 relative overflow-hidden transition-all duration-200 hover:bg-surface-container hover:border-outline-variant/30 flex flex-col justify-between">
                            <div class="absolute top-0 left-0 right-0 h-1 bg-emerald-500"></div>
                            <div>
                                <div class="flex justify-between items-start mb-3">
                                    <div>
                                        <h3 class="text-lg font-bold text-on-surface">"Network"</h3>
                                        <p class="text-xs text-on-surface-variant">"Gated Community · Membership · Events"</p>
                                    </div>
                                    <Badge intent=BadgeIntent::Primary>"Active"</Badge>
                                </div>
                                <div class="divide-y divide-outline-variant/10 text-xs my-4 space-y-2 pt-2">
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Network Pro · $600/mo"</span>
                                        <span class="text-on-surface-variant">"2 tenants"</span>
                                        <span class="font-bold text-emerald-400 font-mono">"$1,200"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="grid grid-cols-3 gap-2 border-t border-outline-variant/10 pt-4 mt-2 text-center">
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"MRR"</div>
                                    <div class="text-base font-extrabold text-emerald-400 font-mono mt-0.5">"$1.2k"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"GMV"</div>
                                    <div class="text-base font-extrabold text-emerald-400 font-mono mt-0.5">"$155k"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Tenants"</div>
                                    <div class="text-base font-extrabold font-mono mt-0.5">"2"</div>
                                </div>
                            </div>
                        </div>

                        // Card 4: Meridian
                        <div class="bg-surface-container-low border border-outline-variant/15 rounded-2xl p-5 relative overflow-hidden transition-all duration-200 hover:bg-surface-container hover:border-outline-variant/30 flex flex-col justify-between">
                            <div class="absolute top-0 left-0 right-0 h-1 bg-amber-500"></div>
                            <div>
                                <div class="flex justify-between items-start mb-3">
                                    <div>
                                        <h3 class="text-lg font-bold text-on-surface">"Meridian"</h3>
                                        <p class="text-xs text-on-surface-variant">"Fleet Management · G-27 Scorecards"</p>
                                    </div>
                                    <Badge intent=BadgeIntent::Default>"Pre-launch"</Badge>
                                </div>
                                <div class="divide-y divide-outline-variant/10 text-xs my-4 space-y-2 pt-2">
                                    <div class="flex justify-between items-center py-1">
                                        <span class="text-on-surface-variant">"Fleet Depot · $4,200/mo (est.)"</span>
                                        <span class="text-amber-500">"0 tenants"</span>
                                        <span class="text-on-surface-variant/80 font-mono">"Pipeline"</span>
                                    </div>
                                </div>
                            </div>
                            <div class="grid grid-cols-3 gap-2 border-t border-outline-variant/10 pt-4 mt-2 text-center">
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"MRR"</div>
                                    <div class="text-base font-extrabold text-on-surface-variant/80 font-mono mt-0.5">"$0"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Pipeline"</div>
                                    <div class="text-base font-extrabold text-amber-500 font-mono mt-0.5">"$4.2M"</div>
                                </div>
                                <div>
                                    <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Waitlist"</div>
                                    <div class="text-base font-extrabold text-amber-500 font-mono mt-0.5">"12"</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Active Subscriptions Card Table
                    <Card class="bg-card border-border shadow-sm overflow-hidden".to_string()>
                        <div class="px-5 py-4 border-b border-outline-variant/10 bg-surface-container/30 flex justify-between items-center">
                            <div>
                                <span class="font-bold text-sm">"All Active Subscriptions"</span>
                                <p class="text-[10px] text-on-surface-variant">"Maps dynamic billing states from tenant_subscriptions and billing_plans records"</p>
                            </div>
                            <Button variant=ButtonVariant::Default size=ButtonSize::Sm on:click=move |_| show_new_sub_modal.set(true)>
                                "+ New Subscription"
                            </Button>
                        </div>
                        <DataTable class="w-full text-xs">
                            <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Plan Name"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Price / Interval"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Billing Interval"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Currency"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border font-sans">
                                <Suspense fallback=move || view! { <DataTableRow><td class="p-4 text-center text-on-surface-variant/50" colspan="5">"Loading billing plans..."</td></DataTableRow> }>
                            {move || billing_plans.get().unwrap_or_default().into_iter().map(|plan| {
                                let price_formatted = format!("${}.{:02} / {}", plan.price / 100, plan.price % 100, plan.interval);
                                view! {
                                    <DataTableRow>
                                        <DataTableCell class="p-3 font-bold text-on-surface">{plan.name.clone()}</DataTableCell>
                                        <DataTableCell class="p-3 text-right font-mono text-emerald-400 font-bold">{price_formatted}</DataTableCell>
                                        <DataTableCell class="p-3 text-on-surface-variant">{plan.interval.clone()}</DataTableCell>
                                        <DataTableCell class="p-3 text-on-surface-variant font-mono">{plan.currency.clone()}</DataTableCell>
                                        <DataTableCell class="p-3 text-right">
                                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click=move |_| {
                                                toast.show_toast("Plan", "Drilling into plan details...", "info");
                                            }>"Details →"</Button>
                                        </DataTableCell>
                                    </DataTableRow>
                                }
                            }).collect_view()}
                            </Suspense>
                        </DataTableBody>
                        </DataTable>
                    </Card>
                </div>
            </Show>

            // ── TAB CONTENT: Ledger ──
            <Show when=move || active_tab.get() == "ledger">
                <Card class="bg-card border-border shadow-sm overflow-hidden animate-fade-in".to_string()>
                    <div class="px-5 py-4 border-b border-outline-variant/10 bg-surface-container/30 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
                        <div>
                            <span class="font-bold text-sm">"Platform Transaction Ledger Entries"</span>
                            <p class="text-[10px] text-on-surface-variant">"Direct relational transactions split-calculated for payout routing (G-03)"</p>
                        </div>
                        <div class="flex items-center gap-3">
                            <select
                                class="bg-surface-container-highest border border-outline/20 text-on-surface text-xs rounded-lg p-2 focus:ring-primary focus:border-primary"
                                on:change=move |ev| filter_tenant.set(event_target_value(&ev))
                            >
                                <option>"All Tenants"</option>
                                <option>"Nexus PM Group"</option>
                                <option>"Biscayne STR Co."</option>
                            </select>
                            <select
                                class="bg-surface-container-highest border border-outline/20 text-on-surface text-xs rounded-lg p-2 focus:ring-primary focus:border-primary"
                                on:change=move |ev| filter_rail.set(event_target_value(&ev))
                            >
                                <option>"All Rails"</option>
                                <option>"Stripe"</option>
                                <option>"Bitcoin"</option>
                                <option>"PIX"</option>
                            </select>
                            <select
                                class="bg-surface-container-highest border border-outline/20 text-on-surface text-xs rounded-lg p-2 focus:ring-primary focus:border-primary"
                                on:change=move |ev| filter_status.set(event_target_value(&ev))
                            >
                                <option>"All Statuses"</option>
                                <option>"Paid"</option>
                                <option>"Pending"</option>
                                <option>"Overdue"</option>
                            </select>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on:click=handle_export>
                                "Export"
                            </Button>
                        </div>
                    </div>
                    <DataTable class="w-full text-xs font-sans">
                        <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"TX ID"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Tenant"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Provider"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Amount"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Status"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Provider TX ID"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Created"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            <Suspense fallback=move || view! { <DataTableRow><td class="p-4 text-center text-on-surface-variant/50" colspan="7">"Loading transactions..."</td></DataTableRow> }>
                        {move || transactions.get().unwrap_or_default().into_iter().map(|tx| {
                            let amount_fmt = format!("${}.{:02} {}", tx.amount / 100, tx.amount.abs() % 100, tx.currency.to_uppercase());
                            let status_intent = match tx.status.as_str() {
                                "paid" | "completed" | "settled" => BadgeIntent::Success,
                                "overdue" | "failed" => BadgeIntent::Default,
                                "pending" => BadgeIntent::Primary,
                                _ => BadgeIntent::Default,
                            };
                            let status = tx.status.clone();
                            view! {
                                <DataTableRow>
                                    <DataTableCell class="p-3 font-mono text-on-surface-variant/80 text-[10px]">{tx.id[..8].to_string() + "…"}</DataTableCell>
                                    <DataTableCell class="p-3 font-mono text-[10px]">{tx.tenant_id[..8].to_string() + "…"}</DataTableCell>
                                    <DataTableCell class="p-3 text-on-surface-variant">{tx.provider.clone()}</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-mono font-bold text-emerald-400">{amount_fmt}</DataTableCell>
                                    <DataTableCell class="p-3 text-center"><Badge intent=status_intent>{status}</Badge></DataTableCell>
                                    <DataTableCell class="p-3 font-mono text-[10px] text-on-surface-variant/70">{tx.provider_tx_id.clone().unwrap_or("—".to_string())}</DataTableCell>
                                    <DataTableCell class="p-3 font-mono text-[10px]">{tx.created_at.clone().unwrap_or("—".to_string())}</DataTableCell>
                                </DataTableRow>
                            }
                        }).collect_view()}
                        </Suspense>
                    </DataTableBody>
                    </DataTable>
                </Card>
            </Show>

            // ── TAB CONTENT: Overdue & Disputes ──
            <Show when=move || active_tab.get() == "overdue">
                <div class="space-y-6 animate-fade-in">
                    // Alarm Banner
                    <div class="flex items-start gap-3.5 p-4 bg-error/10 border border-error/30 rounded-xl text-xs text-on-surface-variant leading-relaxed">
                        <span class="material-symbols-outlined text-error text-base mt-0.5">"warning"</span>
                        <div>
                            <span class="font-bold text-error">"3 invoices overdue · $38,300 total outstanding."</span>
                            <p class="mt-0.5">"Automated grace period checks run daily. Subscriptions auto-suspend upon threshold violations."</p>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        // Left: Overdue Invoices
                        <div class="lg:col-span-2 space-y-4">
                            <Card class="bg-card border-border shadow-sm overflow-hidden".to_string()>
                                <div class="px-5 py-4 border-b border-outline-variant/10 bg-[#06122d]/30 flex justify-between items-center">
                                    <span class="font-bold text-sm">"Outstanding Invoice Reminders"</span>
                                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on:click=move |_| toast.show_toast("Success", "Bulk reminder emails queued and dispatched.", "success")>
                                        "Send All Reminders"
                                    </Button>
                                </div>
                                <DataTable class="w-full text-xs font-sans">
                                    <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                                        <DataTableRow class="hover:bg-transparent">
                                            <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Tenant"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Product"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Amount"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Due Date"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Overdue"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Grace Status"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Rail"</DataTableHead>
                                            <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Actions"</DataTableHead>
                                        </DataTableRow>
                                    </DataTableHeader>
                                    <DataTableBody class="divide-y divide-border">
                                        <DataTableRow>
                                                <td class="p-8 text-center text-on-surface-variant/50" colspan="7">"Overdue invoice data pending — requires billing reconciliation endpoint (future)"</td>
                                            </DataTableRow>
                                        </DataTableBody>
                                </DataTable>
                            </Card>
                        </div>

                        // Right: Billing Exemptions
                        <div class="space-y-4">
                            <Card class="bg-card border-border shadow-sm p-5".to_string()>
                                <div class="flex justify-between items-center mb-4 font-bold">
                                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Exemption Overrides"</h3>
                                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm class="h-7 text-[10px]".to_string() on:click=move |_| show_exemption_modal.set(true)>
                                        "+ Exempt"
                                    </Button>
                                </div>
                                <p class="text-[11px] text-on-surface-variant mb-4 leading-relaxed font-sans">
                                    "Bypass auto-suspensions and grace period locks for strategic VIP client SLA accounts."
                                </p>
                                <div class="divide-y divide-outline-variant/15 text-xs pt-2">
                                    {move || billing_summary.get().flatten().map(|s| s.exemptions.into_iter().map(|e| view! {
                                        <div class="flex justify-between items-center py-2 px-1">
                                            <span class="font-bold text-on-surface">{e.tenant_name.clone()}</span>
                                            <span class="text-on-surface-variant text-[10px]">{e.reason.clone()}</span>
                                        </div>
                                    }).collect_view()).unwrap_or_default()}
                                </div>
                            </Card>
                    </div>
                </div>
            </div>
            </Show>

            // ── TAB CONTENT: Tax & Filings ──
            <Show when=move || active_tab.get() == "tax">
                <Card class="bg-card border-border shadow-sm overflow-hidden animate-fade-in".to_string()>
                    <div class="px-5 py-4 border-b border-outline-variant/10 bg-surface-container/30 flex justify-between items-center">
                        <div>
                            <span class="font-bold text-sm">"VAT, Sales Tax, and Corporate Filings (G-17)"</span>
                            <p class="text-[10px] text-on-surface-variant">"Corporate compliance reporting registers mapped platform-wide"</p>
                        </div>
                    </div>
                    <DataTable class="w-full text-xs font-sans">
                        <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Filing Code"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Jurisdiction / Context"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Gross Platform Sales"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Tax Collected"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Filing Status"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Filing Deadline"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            <DataTableRow>
                                <td class="p-8 text-center text-on-surface-variant/50" colspan="6">"Tax filing data pending — requires G-17 tax_events entity and endpoint (future)"</td>
                            </DataTableRow>
                        </DataTableBody>
                    </DataTable>
                </Card>
            </Show>

            // ── MODAL: New Subscription ──
            <Show when=move || show_new_sub_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_new_sub_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"New Tenant Subscription"</h3>
                        <p class="text-xs text-on-surface-variant mb-6 font-sans">"Manually provision a product billing subscription plan for a tenant account."</p>

                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Tenant Account"</Label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| new_sub_tenant.set(event_target_value(&ev))
                                >
                                    <option>"Nexus PM Group"</option>
                                    <option>"Biscayne STR Co."</option>
                                    <option>"Harbor Media"</option>
                                    <option>"South Beach Nets"</option>
                                    <option>"Cabana Club Ltd."</option>
                                    <option>"Rio Verde PMC"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Platform Product"</Label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| new_sub_product.set(event_target_value(&ev))
                                >
                                    <option>"Folio"</option>
                                    <option>"Anchor"</option>
                                    <option>"Network"</option>
                                    <option>"Meridian"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Billing Plan & Price Plan"</Label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| new_sub_plan.set(event_target_value(&ev))
                                >
                                    <option>"Starter - $400/mo"</option>
                                    <option>"STR Pro - $1,800/mo"</option>
                                    <option>"Enterprise - $6,000/mo"</option>
                                    <option>"Creator - $900/mo"</option>
                                    <option>"Creator Pro - $2,400/mo"</option>
                                    <option>"Network Pro - $600/mo"</option>
                                </select>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_new_sub_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_new_subscription>"Create Subscription"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── MODAL: Log Manual Payment ──
            <Show when=move || show_log_payment_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_log_payment_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Log Manual Payment Entry"</h3>
                        <p class="text-xs text-on-surface-variant mb-6 font-sans">"Record a bank wire or off-channel transaction payment to settle outstanding invoices."</p>

                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Tenant Account"</Label>
                                <Input
                                    r#type=InputType::Text
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string()
                                    bind_value=log_payment_tenant
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Settle Amount ($)"</Label>
                                <Input
                                    r#type=InputType::Text
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string()
                                    bind_value=log_payment_amount
                                    placeholder="e.g. 28000.00".to_string()
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Reconciliation Reference"</Label>
                                <Input
                                    r#type=InputType::Text
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string()
                                    bind_value=log_payment_ref
                                    placeholder="e.g. wire_98231_bank_amex".to_string()
                                />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_log_payment_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_log_payment>"Log Payment"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── MODAL: Add Billing Exemption ──
            <Show when=move || show_exemption_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_exemption_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Add Billing Exemption Override"</h3>
                        <p class="text-xs text-on-surface-variant mb-6 font-sans">"Exempt a tenant from automated suspension routines and grace timers."</p>

                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Tenant Account Name"</Label>
                                <Input
                                    r#type=InputType::Text
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string()
                                    bind_value=exempt_tenant
                                    placeholder="e.g. Urban Core Mgmt".to_string()
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Exemption Basis / Reason"</Label>
                                <Input
                                    r#type=InputType::Text
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string()
                                    bind_value=exempt_reason
                                    placeholder="e.g. VIP SLA Contract Waiver".to_string()
                                />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_exemption_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_exempt_tenant>"Exempt Tenant"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

