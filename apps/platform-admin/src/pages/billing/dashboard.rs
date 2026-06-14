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

#[derive(Clone, Debug)]
pub struct MockSubscription {
    pub tenant: String,
    pub product: String,
    pub plan: String,
    pub price: String,
    pub status: String,
    pub trial_ends: String,
    pub period_end: String,
    pub stripe_sub_id: String,
}

#[derive(Clone, Debug)]
pub struct MockLedgerEntry {
    pub id: String,
    pub tenant: String,
    pub entity: String,
    pub payer: String,
    pub gross: String,
    pub fee: String,
    pub net: String,
    pub rail: String,
    pub status: String,
    pub due_paid: String,
    pub reconciled: bool,
}

#[derive(Clone, Debug)]
pub struct MockOverdueInvoice {
    pub tenant: String,
    pub product: String,
    pub amount: String,
    pub due_date: String,
    pub overdue_days: String,
    pub grace_status: String,
    pub rail: String,
}

#[derive(Clone, Debug)]
pub struct MockExemption {
    pub tenant: String,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct MockCommissionPlan {
    pub id: String,
    pub name: String,
    pub split_pct: String,
    pub volume_mtd: String,
    pub active_tenants: i32,
}

#[derive(Clone, Debug)]
pub struct MockTaxFiling {
    pub period: String,
    pub jurisdiction: String,
    pub gross_sales: String,
    pub tax_collected: String,
    pub status: String,
    pub filing_deadline: String,
}

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

    // Stored mock datasets (to avoid capture-by-FnOnce compiler panic in Leptos)
    let mock_subscriptions = StoredValue::new(vec![
        MockSubscription {
            tenant: "Nexus PM Group".to_string(),
            product: "Folio".to_string(),
            plan: "Enterprise PM · $6,000".to_string(),
            price: "$6,000".to_string(),
            status: "Active".to_string(),
            trial_ends: "—".to_string(),
            period_end: "Jul 1".to_string(),
            stripe_sub_id: "sub_1N4…".to_string(),
        },
        MockSubscription {
            tenant: "Biscayne STR Co.".to_string(),
            product: "Folio".to_string(),
            plan: "STR Pro · $1,800".to_string(),
            price: "$1,800".to_string(),
            status: "Active".to_string(),
            trial_ends: "—".to_string(),
            period_end: "Jul 1".to_string(),
            stripe_sub_id: "sub_2P7…".to_string(),
        },
        MockSubscription {
            tenant: "Harbor Media".to_string(),
            product: "Anchor".to_string(),
            plan: "Creator · $900".to_string(),
            price: "$900".to_string(),
            status: "Active".to_string(),
            trial_ends: "—".to_string(),
            period_end: "Jul 1".to_string(),
            stripe_sub_id: "sub_3Q8…".to_string(),
        },
        MockSubscription {
            tenant: "South Beach Nets".to_string(),
            product: "Network".to_string(),
            plan: "Network Pro · $600".to_string(),
            price: "$600".to_string(),
            status: "Grace Period".to_string(),
            trial_ends: "—".to_string(),
            period_end: "Jun 1 (2d left)".to_string(),
            stripe_sub_id: "sub_4R2…".to_string(),
        },
        MockSubscription {
            tenant: "Cabana Club Ltd.".to_string(),
            product: "Folio".to_string(),
            plan: "Starter · $400".to_string(),
            price: "$400".to_string(),
            status: "Suspended".to_string(),
            trial_ends: "—".to_string(),
            period_end: "May 20 !".to_string(),
            stripe_sub_id: "sub_6X1…".to_string(),
        },
        MockSubscription {
            tenant: "Rio Verde PMC".to_string(),
            product: "Folio".to_string(),
            plan: "Starter · $400".to_string(),
            price: "$400".to_string(),
            status: "Trial".to_string(),
            trial_ends: "Jun 17".to_string(),
            period_end: "Jul 1".to_string(),
            stripe_sub_id: "sub_5S9…".to_string(),
        },
    ]);

    let mock_ledger = StoredValue::new(vec![
        MockLedgerEntry {
            id: "le_a8f2…".to_string(),
            tenant: "Nexus PM".to_string(),
            entity: "reservation".to_string(),
            payer: "j.tenant@nexus.com".to_string(),
            gross: "$4,200".to_string(),
            fee: "$336".to_string(),
            net: "$3,864".to_string(),
            rail: "Stripe".to_string(),
            status: "Paid".to_string(),
            due_paid: "Jun 8".to_string(),
            reconciled: true,
        },
        MockLedgerEntry {
            id: "le_b1c4…".to_string(),
            tenant: "Biscayne STR".to_string(),
            entity: "reservation".to_string(),
            payer: "guest@booking.com".to_string(),
            gross: "$1,850".to_string(),
            fee: "$148".to_string(),
            net: "$1,702".to_string(),
            rail: "Stripe".to_string(),
            status: "Paid".to_string(),
            due_paid: "Jun 7".to_string(),
            reconciled: true,
        },
        MockLedgerEntry {
            id: "le_c3d7…".to_string(),
            tenant: "Nexus PM".to_string(),
            entity: "reservation".to_string(),
            payer: "renter@gmail.com".to_string(),
            gross: "$2,100".to_string(),
            fee: "$168".to_string(),
            net: "$1,932".to_string(),
            rail: "BTC".to_string(),
            status: "Paid".to_string(),
            due_paid: "Jun 6".to_string(),
            reconciled: true,
        },
        MockLedgerEntry {
            id: "le_d5e8…".to_string(),
            tenant: "South Beach Nets".to_string(),
            entity: "reservation".to_string(),
            payer: "client@southbeach.io".to_string(),
            gross: "$28,000".to_string(),
            fee: "$2,240".to_string(),
            net: "$25,760".to_string(),
            rail: "Stripe".to_string(),
            status: "Overdue".to_string(),
            due_paid: "Jun 1".to_string(),
            reconciled: false,
        },
        MockLedgerEntry {
            id: "le_h4j9…".to_string(),
            tenant: "Rio Verde PMC".to_string(),
            entity: "reservation".to_string(),
            payer: "tenant@rioverde.br".to_string(),
            gross: "$7,200".to_string(),
            fee: "$576".to_string(),
            net: "$6,624".to_string(),
            rail: "PIX".to_string(),
            status: "Overdue".to_string(),
            due_paid: "Jun 3".to_string(),
            reconciled: false,
        },
    ]);

    let mock_overdue_invoices = StoredValue::new(vec![
        MockOverdueInvoice {
            tenant: "South Beach Nets".to_string(),
            product: "Network".to_string(),
            amount: "$28,000".to_string(),
            due_date: "Jun 1".to_string(),
            overdue_days: "9 days".to_string(),
            grace_status: "Grace Expired · Suspended".to_string(),
            rail: "Stripe".to_string(),
        },
        MockOverdueInvoice {
            tenant: "Rio Verde PMC".to_string(),
            product: "Folio".to_string(),
            amount: "$7,200".to_string(),
            due_date: "Jun 3".to_string(),
            overdue_days: "7 days".to_string(),
            grace_status: "0d left (Auto-suspending today)".to_string(),
            rail: "PIX".to_string(),
        },
        MockOverdueInvoice {
            tenant: "Urban Core Mgmt".to_string(),
            product: "Folio".to_string(),
            amount: "$3,100".to_string(),
            due_date: "Jun 7".to_string(),
            overdue_days: "3 days".to_string(),
            grace_status: "Active · Grace: 4d left".to_string(),
            rail: "Stripe".to_string(),
        },
    ]);

    let mock_exemptions = StoredValue::new(vec![
        MockExemption {
            tenant: "Nexus Property Group".to_string(),
            reason: "VIP SLA Custom Contract".to_string(),
        },
        MockExemption {
            tenant: "Biscayne STR Co.".to_string(),
            reason: "Non-Profit Waiver Program".to_string(),
        },
    ]);

    let mock_commission_plans = StoredValue::new(vec![
        MockCommissionPlan {
            id: "cp_std_share".to_string(),
            name: "Standard 8% Share".to_string(),
            split_pct: "8.0%".to_string(),
            volume_mtd: "$136,800".to_string(),
            active_tenants: 12,
        },
        MockCommissionPlan {
            id: "cp_vip_flat".to_string(),
            name: "VIP Flat 3% RevShare".to_string(),
            split_pct: "3.0%".to_string(),
            volume_mtd: "$24,600".to_string(),
            active_tenants: 2,
        },
        MockCommissionPlan {
            id: "cp_free_dev".to_string(),
            name: "Developer Tier Exempt".to_string(),
            split_pct: "0.0%".to_string(),
            volume_mtd: "$0".to_string(),
            active_tenants: 1,
        },
    ]);

    let mock_tax_filings = StoredValue::new(vec![
        MockTaxFiling {
            period: "tf_2026_q2".to_string(),
            jurisdiction: "Q2 Federal VAT Filing".to_string(),
            gross_sales: "$3.42M".to_string(),
            tax_collected: "$273,600".to_string(),
            status: "Open".to_string(),
            filing_deadline: "Jul 15".to_string(),
        },
        MockTaxFiling {
            period: "tf_2026_05".to_string(),
            jurisdiction: "May Florida Sales Tax".to_string(),
            gross_sales: "$680k".to_string(),
            tax_collected: "$47,600".to_string(),
            status: "Filed".to_string(),
            filing_deadline: "Jun 20".to_string(),
        },
        MockTaxFiling {
            period: "tf_2026_05".to_string(),
            jurisdiction: "May Delaware Franchise".to_string(),
            gross_sales: "$1.20M".to_string(),
            tax_collected: "$96,000".to_string(),
            status: "Filed".to_string(),
            filing_deadline: "Jun 15".to_string(),
        },
    ]);

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
                    <div class="text-xl font-extrabold text-primary font-mono">"$84k"</div>
                    <p class="text-[10px] text-emerald-400 font-medium">"↑ 12% vs May"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Total GMV"</span>
                    <div class="text-xl font-extrabold text-emerald-400 font-mono">"$2.14M"</div>
                    <p class="text-[10px] text-emerald-400 font-medium">"↑ 18% vs May"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Platform Comm."</span>
                    <div class="text-xl font-extrabold font-mono">"$171k"</div>
                    <p class="text-[10px] text-on-surface-variant/80">"8% avg · G-25"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Active Subs"</span>
                    <div class="text-xl font-extrabold font-mono">"15"</div>
                    <p class="text-[9px] text-on-surface-variant/70 truncate">"2 trial · 1 grace · 1 susp."</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Outstanding"</span>
                    <div class="text-xl font-extrabold text-amber-500 font-mono">"$38.3k"</div>
                    <p class="text-[10px] text-error font-medium">"3 overdue"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Pending Payouts"</span>
                    <div class="text-xl font-extrabold text-amber-500 font-mono">"$29k"</div>
                    <p class="text-[10px] text-on-surface-variant/80">"6 splits · G-03"</p>
                </div>
                <div class="space-y-1 min-w-[100px] border-l border-outline-variant/15 pl-4">
                    <span class="text-[9px] font-bold text-on-surface-variant uppercase tracking-widest">"Tax Events"</span>
                    <div class="text-xl font-extrabold font-mono">"247"</div>
                    <p class="text-[9px] text-on-surface-variant/70 font-mono">"G-17 · 2 open"</p>
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
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Tenant"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Product"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Plan"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Price"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Status"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Trial Ends"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Period End"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Stripe Sub ID"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border font-sans">
                                <For
                                    each=move || mock_subscriptions.get_value()
                                    key=|sub| sub.tenant.clone()
                                    children=move |sub| {
                                        let plan_color = match sub.product.as_str() {
                                            "Folio" => "text-primary",
                                            "Anchor" => "text-violet-400",
                                            "Network" => "text-emerald-400",
                                            _ => "text-on-surface",
                                        };
                                        let status_intent = match sub.status.as_str() {
                                            "Active" | "Trial" => BadgeIntent::Success,
                                            "Grace Period" => BadgeIntent::Primary,
                                            "Suspended" => BadgeIntent::Default,
                                            _ => BadgeIntent::Default,
                                        };
                                        let tenant = sub.tenant.clone();
                                        let product = sub.product.clone();
                                        let plan = sub.plan.clone();
                                        let price = sub.price.clone();
                                        let status = sub.status.clone();
                                        let trial_ends = sub.trial_ends.clone();
                                        let period_end = sub.period_end.clone();
                                        let stripe_sub_id = sub.stripe_sub_id.clone();
                                        let stripe_sub_id_link = sub.stripe_sub_id.clone();
                                        view! {
                                            <DataTableRow>
                                                <DataTableCell class="p-3 font-bold text-primary hover:underline cursor-pointer">
                                                    <a href=format!("/billing/tenant/{}", stripe_sub_id_link)>{tenant}</a>
                                                </DataTableCell>
                                                <DataTableCell class="p-3 text-on-surface-variant">{product}</DataTableCell>
                                                <DataTableCell class=format!("p-3 font-semibold {}", plan_color)>{plan}</DataTableCell>
                                                <DataTableCell class="p-3 text-right font-mono text-emerald-400 font-bold">{price}</DataTableCell>
                                                <DataTableCell class="p-3 text-center">
                                                    <Badge intent=status_intent>{status}</Badge>
                                                </DataTableCell>
                                                <DataTableCell class="p-3 text-on-surface-variant font-mono">{trial_ends}</DataTableCell>
                                                <DataTableCell class="p-3 font-mono">{period_end}</DataTableCell>
                                                <DataTableCell class="p-3 font-mono text-on-surface-variant/80">{stripe_sub_id}</DataTableCell>
                                                <DataTableCell class="p-3 text-right">
                                                    <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click=move |_| {
                                                        toast.show_toast("Redirecting", "Drilling down into tenant plan details...", "info");
                                                    }>
                                                        "Plan →"
                                                    </Button>
                                                </DataTableCell>
                                            </DataTableRow>
                                        }
                                    }
                                />
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
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"ID"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Tenant"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Entity Type"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Payer Details"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Gross Amount"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Platform Fee"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Net Accounted"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Payment Rail"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Status"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Due/Paid Date"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Reconciled"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            <For
                                each=move || mock_ledger.get_value()
                                key=|entry| entry.id.clone()
                                children=move |entry| {
                                    let rail_style = match entry.rail.as_str() {
                                        "Stripe" => "text-primary border-primary/30 bg-primary/5",
                                        "BTC" => "text-amber-500 border-amber-500/30 bg-amber-500/5",
                                        "PIX" => "text-emerald-400 border-emerald-400/30 bg-emerald-400/5",
                                        _ => "text-on-surface-variant border-outline/20",
                                    };
                                    let status_intent = match entry.status.as_str() {
                                        "Paid" => BadgeIntent::Success,
                                        "Overdue" => BadgeIntent::Default,
                                        _ => BadgeIntent::Default,
                                    };
                                    view! {
                                        <DataTableRow>
                                            <DataTableCell class="p-3 font-mono text-on-surface-variant/80">{entry.id.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 font-bold text-primary">{entry.tenant.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-on-surface-variant font-mono">{entry.entity.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 font-mono text-[11px] text-on-surface-variant/90">{entry.payer.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono text-emerald-400 font-bold">{entry.gross.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono text-on-surface-variant">{entry.fee.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono font-bold text-on-surface">{entry.net.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-center">
                                                <span class=format!("px-2 py-0.5 rounded border text-[9px] font-bold uppercase tracking-wider {}", rail_style)>
                                                    {entry.rail.clone()}
                                                </span>
                                            </DataTableCell>
                                            <DataTableCell class="p-3 text-center">
                                                <Badge intent=status_intent>{entry.status.clone()}</Badge>
                                            </DataTableCell>
                                            <DataTableCell class="p-3 font-mono">{entry.due_paid.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-center font-bold">
                                                <span class=if entry.reconciled { "text-emerald-400" } else { "text-amber-500 font-medium" }>
                                                    {if entry.reconciled { "✓" } else { "Pending" }}
                                                </span>
                                            </DataTableCell>
                                        </DataTableRow>
                                    }
                                }
                            />
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
                                        <For
                                            each=move || mock_overdue_invoices.get_value()
                                            key=|inv| inv.tenant.clone()
                                            children=move |inv| {
                                                let t_name = inv.tenant.clone();
                                                view! {
                                                    <DataTableRow>
                                                        <DataTableCell class="p-3 font-bold text-error">{inv.tenant.clone()}</DataTableCell>
                                                        <DataTableCell class="p-3 text-on-surface-variant">{inv.product.clone()}</DataTableCell>
                                                        <DataTableCell class="p-3 text-right font-mono text-error font-bold">{inv.amount.clone()}</DataTableCell>
                                                        <DataTableCell class="p-3 font-mono">{inv.due_date.clone()}</DataTableCell>
                                                        <DataTableCell class="p-3 text-center text-error font-semibold font-mono">{inv.overdue_days.clone()}</DataTableCell>
                                                        <DataTableCell class="p-3 font-mono text-on-surface-variant/80 text-[10px]">
                                                            {inv.grace_status.clone()}
                                                        </DataTableCell>
                                                        <DataTableCell class="p-3 font-mono text-on-surface-variant/80">
                                                            {inv.rail.clone()}
                                                        </DataTableCell>
                                                        <DataTableCell class="p-3 text-right space-x-1">
                                                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click={
                                                                let tenant = t_name.clone();
                                                                move |_| toast.show_toast("Reminder Sent", &format!("Payment reminder email sent to {}.", tenant), "success")
                                                            }>
                                                                "Remind"
                                                            </Button>
                                                            <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click={
                                                                let tenant = t_name.clone();
                                                                move |_| {
                                                                    log_payment_tenant.set(tenant.clone());
                                                                    show_log_payment_modal.set(true);
                                                                }
                                                            }>
                                                                "Log"
                                                            </Button>
                                                        </DataTableCell>
                                                    </DataTableRow>
                                                }
                                            }
                                        />
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
                                <div class="divide-y divide-outline-variant/15 text-xs space-y-3 pt-2">
                                    <For
                                        each=move || mock_exemptions.get_value()
                                        key=|ex| ex.tenant.clone()
                                        children=move |ex| {
                                            let t_name = ex.tenant.clone();
                                            view! {
                                                <div class="flex justify-between items-center py-2">
                                                    <div>
                                                        <div class="font-bold text-on-surface">{ex.tenant.clone()}</div>
                                                        <div class="text-[10px] text-on-surface-variant/80 font-mono mt-0.5">{ex.reason.clone()}</div>
                                                    </div>
                                                    <div class="flex items-center gap-2">
                                                        <Badge intent=BadgeIntent::Success>"Exempt"</Badge>
                                                        <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-6 w-6 p-0 text-error hover:bg-error/10 rounded-full flex items-center justify-center font-bold".to_string() on:click={
                                                            let tenant = t_name.clone();
                                                            move |_| toast.show_toast("Exemption Lifted", &format!("Standard billing billing cycles restored for {}.", tenant), "info")
                                                        }>
                                                            "✕"
                                                        </Button>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </Card>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Payment Rails ──
            <Show when=move || active_tab.get() == "rails">
                <Card class="bg-card border-border shadow-sm overflow-hidden animate-fade-in".to_string()>
                    <div class="px-5 py-4 border-b border-outline-variant/10 bg-surface-container/30">
                        <span class="font-bold text-sm">"Platform Payment Gateways Configuration"</span>
                        <p class="text-[10px] text-on-surface-variant">"Enable or restrict payment integration flows across active transaction rails"</p>
                    </div>
                    <div class="p-6 grid grid-cols-1 md:grid-cols-2 gap-6">
                        // Rail 1: Stripe
                        <div class="p-4 rounded-xl border border-outline-variant/15 flex justify-between items-center bg-[#05183c]/20 hover:border-outline-variant/35 transition-all">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center font-bold text-primary text-sm">
                                    "S"
                                </div>
                                <div>
                                    <div class="font-bold text-on-surface text-sm">"Stripe CC & ACH"</div>
                                    <div class="text-xs text-on-surface-variant">"Global Visa, Mastercard, AMEX, and US Bank Accounts"</div>
                                    <div class="flex items-center gap-1.5 mt-1.5">
                                        <Badge intent=BadgeIntent::Success>"Active"</Badge>
                                        <span class="text-[10px] text-on-surface-variant/60 font-mono">"API Connect: Ok"</span>
                                    </div>
                                </div>
                            </div>
                            <Switch id="stripe_rail_toggle".to_string() checked=true />
                        </div>

                        // Rail 2: Zaprite (Bitcoin)
                        <div class="p-4 rounded-xl border border-outline-variant/15 flex justify-between items-center bg-[#05183c]/20 hover:border-outline-variant/35 transition-all">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center font-bold text-amber-500 text-sm">
                                    "₿"
                                </div>
                                <div>
                                    <div class="font-bold text-on-surface text-sm">"Zaprite BTC / Lightning"</div>
                                    <div class="text-xs text-on-surface-variant">"Direct on-chain Bitcoin transactions and instant micro-payments"</div>
                                    <div class="flex items-center gap-1.5 mt-1.5">
                                        <Badge intent=BadgeIntent::Success>"Active"</Badge>
                                        <span class="text-[10px] text-on-surface-variant/60 font-mono">"Nodes: 2 connected"</span>
                                    </div>
                                </div>
                            </div>
                            <Switch id="zaprite_rail_toggle".to_string() checked=true />
                        </div>

                        // Rail 3: PIX Gateway
                        <div class="p-4 rounded-xl border border-outline-variant/15 flex justify-between items-center bg-[#05183c]/20 hover:border-outline-variant/35 transition-all">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center font-bold text-emerald-400 text-sm">
                                    "P"
                                </div>
                                <div>
                                    <div class="font-bold text-on-surface text-sm">"Pix Payment Rails"</div>
                                    <div class="text-xs text-on-surface-variant">"Instant Central Bank transactions routing in Brazil"</div>
                                    <div class="flex items-center gap-1.5 mt-1.5">
                                        <Badge intent=BadgeIntent::Success>"Active"</Badge>
                                        <span class="text-[10px] text-on-surface-variant/60 font-mono">"API Gateway: Online"</span>
                                    </div>
                                </div>
                            </div>
                            <Switch id="pix_rail_toggle".to_string() checked=true />
                        </div>

                        // Rail 4: Paddle
                        <div class="p-4 rounded-xl border border-outline-variant/15 flex justify-between items-center bg-[#05183c]/20 hover:border-outline-variant/35 transition-all opacity-80">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-violet-500/10 border border-violet-500/20 flex items-center justify-center font-bold text-violet-400 text-sm">
                                    "P"
                                </div>
                                <div>
                                    <div class="font-bold text-on-surface text-sm">"Paddle Merchant of Record"</div>
                                    <div class="text-xs text-on-surface-variant">"Global tax compliance, invoicing, and local currencies"</div>
                                    <div class="flex items-center gap-1.5 mt-1.5">
                                        <Badge intent=BadgeIntent::Primary>"Sandbox"</Badge>
                                        <span class="text-[10px] text-on-surface-variant/60 font-mono">"Test mode keys active"</span>
                                    </div>
                                </div>
                            </div>
                            <Switch id="paddle_rail_toggle".to_string() checked=false />
                        </div>
                    </div>
                </Card>
            </Show>

            // ── TAB CONTENT: Commission Plans ──
            <Show when=move || active_tab.get() == "commissions">
                <Card class="bg-card border-border shadow-sm overflow-hidden animate-fade-in".to_string()>
                    <div class="px-5 py-4 border-b border-outline-variant/10 bg-surface-container/30 flex justify-between items-center">
                        <div>
                            <span class="font-bold text-sm">"Platform Revenue Share Plans (G-25)"</span>
                            <p class="text-[10px] text-on-surface-variant">"Direct split commissions calculated automatically from syndication and STR bookings"</p>
                        </div>
                    </div>
                    <DataTable class="w-full text-xs font-sans">
                        <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Plan ID"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Name / Tier"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Commission Share"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Volume MTD"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Active Tenants"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            <For
                                each=move || mock_commission_plans.get_value()
                                key=|plan| plan.id.clone()
                                children=move |plan| {
                                    let plan_id = plan.id.clone();
                                    view! {
                                        <DataTableRow>
                                            <DataTableCell class="p-3 font-mono text-on-surface-variant/80">{plan.id.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 font-bold text-on-surface">{plan.name.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono font-bold text-primary">{plan.split_pct.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono text-emerald-400 font-bold">{plan.volume_mtd.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-center font-semibold font-mono">{plan.active_tenants}</DataTableCell>
                                            <DataTableCell class="p-3 text-right">
                                                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click={
                                                    let id = plan_id.clone();
                                                    move |_| toast.show_toast("Edit Plan", &format!("Editing parameters for commission schema: {}", id), "info")
                                                }>
                                                    "Configure"
                                                </Button>
                                            </DataTableCell>
                                        </DataTableRow>
                                    }
                                }
                            />
                        </DataTableBody>
                    </DataTable>
                </Card>
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
                            <For
                                each=move || mock_tax_filings.get_value()
                                key=|filing| filing.period.clone()
                                children=move |filing| {
                                    let status_intent = match filing.status.as_str() {
                                        "Filed" => BadgeIntent::Success,
                                        "Open" => BadgeIntent::Primary,
                                        _ => BadgeIntent::Default,
                                    };
                                    view! {
                                        <DataTableRow>
                                            <DataTableCell class="p-3 font-mono text-on-surface-variant/80">{filing.period.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 font-bold text-on-surface">{filing.jurisdiction.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono font-semibold">{filing.gross_sales.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono text-emerald-400 font-bold">{filing.tax_collected.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-center">
                                                <Badge intent=status_intent>{filing.status.clone()}</Badge>
                                            </DataTableCell>
                                            <DataTableCell class="p-3 font-mono font-medium text-on-surface-variant">{filing.filing_deadline.clone()}</DataTableCell>
                                            <DataTableCell class="p-3 text-right">
                                                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click=move |_| {
                                                    toast.show_toast("Filing Download", "Exporting filing logs manifest to CSV.", "success");
                                                }>
                                                    "PDF Ledger"
                                                </Button>
                                            </DataTableCell>
                                        </DataTableRow>
                                    }
                                }
                            />
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

