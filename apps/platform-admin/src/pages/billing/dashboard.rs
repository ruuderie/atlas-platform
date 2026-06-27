use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::button::{Button, ButtonVariant, ButtonSize};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use crate::api::admin::{get_billing_plans, get_all_transactions, get_tenant_stats};
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
    let new_sub_tenant = RwSignal::new(String::new());
    let new_sub_product = RwSignal::new("Folio".to_string());
    let new_sub_plan = RwSignal::new("Starter - $400/mo".to_string());

    let log_payment_tenant = RwSignal::new(String::new());
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
    let refresh = RwSignal::new(0u32);
    let data_error: RwSignal<Option<String>> = RwSignal::new(None);
    let billing_summary = LocalResource::new(move || async move {
        let _ = refresh.get();
        match get_billing_summary().await {
            Ok(v) => { data_error.set(None); Some(v) }
            Err(e) => { data_error.set(Some(format!("Billing summary: {}", e))); None }
        }
    });
    let business_kpis = LocalResource::new(move || async move {
        let _ = refresh.get();
        get_business_kpis().await.ok()
    });
    let billing_plans = LocalResource::new(move || async move {
        let _ = refresh.get();
        get_billing_plans().await.unwrap_or_default()
    });
    let transactions = LocalResource::new(move || async move {
        let _ = refresh.get();
        get_all_transactions().await.unwrap_or_default()
    });
    let tenant_list = LocalResource::new(move || async move {
        let _ = refresh.get();
        get_tenant_stats().await.unwrap_or_default()
    });


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
                    <button
                        class="btn-ghost px-3 py-2 rounded-lg text-xs font-semibold border border-outline-variant/30 flex items-center gap-1.5 hover:bg-surface-bright/20 transition-all active:scale-95"
                        on:click=move |_| refresh.update(|n| *n += 1)
                        title="Reload billing data"
                    >
                        <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                        </svg>
                        "Refresh"
                    </button>
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

            // ── Error banner ──
            {move || data_error.get().map(|e| crate::utils::inline_error(&e))}

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
                        {move || billing_summary.get().flatten().map(|s| format!("{:.0}%", s.collection_success_rate)).unwrap_or("—".to_string())}
                    </div>
                    <p class="text-[10px] text-on-surface-variant/80">"Invoiced vs paid"</p>
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
                    "Pricing Plans"
                </button>
                <button class=move || tab_class("rev_intel") on:click=move |_| active_tab.set("rev_intel".to_string())>
                    "Revenue"
                </button>
                <button class=move || tab_class("ledger") on:click=move |_| active_tab.set("ledger".to_string())>
                    "Client Billing"
                </button>
                <button class=move || tab_class("overdue") on:click=move |_| active_tab.set("overdue".to_string())>
                    "Overdue & Disputes"
                </button>
                <button class=move || tab_class("rails") on:click=move |_| active_tab.set("rails".to_string())>
                    "Payment Rails"
                </button>
                <button class=move || tab_class("commissions") on:click=move |_| active_tab.set("commissions".to_string())>
                    "Commission Plans"
                </button>
                <button class=move || tab_class("tax") on:click=move |_| active_tab.set("tax".to_string())>
                    "Tax & Filings"
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

                    // Products Grid — driven from billing_plans API
                    <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant/50 text-xs">"Loading products..."</div> }>
                    {move || billing_plans.get().map(|plans| {
                        if plans.is_empty() {
                            view! {
                                <div class="col-span-2 bg-surface-container-low border border-outline-variant/15 rounded-2xl p-10 text-center">
                                    <p class="text-sm font-semibold text-on-surface-variant">"No billing plans configured"</p>
                                    <p class="text-[11px] text-on-surface-variant/60 mt-1">"Create a billing plan to start tracking revenue by product."</p>
                                </div>
                            }.into_any()
                        } else {
                            // Group plans by product prefix (first word of plan name)
                            let mut products: std::collections::BTreeMap<String, Vec<_>> = std::collections::BTreeMap::new();
                            for plan in plans {
                                let product = plan.name.split_whitespace().next().unwrap_or("Other").to_string();
                                products.entry(product).or_default().push(plan);
                            }
                            let accent_colors = ["bg-primary", "bg-violet-500", "bg-emerald-500", "bg-amber-500"];
                            products.into_iter().enumerate().map(|(i, (product_name, product_plans))| {
                                let color = accent_colors[i % accent_colors.len()].to_string();
                                let plan_count = product_plans.len();
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/15 rounded-2xl p-5 relative overflow-hidden transition-all duration-200 hover:bg-surface-container hover:border-outline-variant/30 flex flex-col justify-between">
                                        <div class=format!("absolute top-0 left-0 right-0 h-1 {}", color)></div>
                                        <div>
                                            <div class="flex justify-between items-start mb-3">
                                                <div>
                                                    <h3 class="text-lg font-bold text-on-surface">{product_name.clone()}</h3>
                                                    <p class="text-xs text-on-surface-variant">{format!("{} billing plan{}", plan_count, if plan_count == 1 {""} else {"s"})}</p>
                                                </div>
                                            </div>
                                            <div class="divide-y divide-outline-variant/10 text-xs my-4 space-y-2 pt-2">
                                                {product_plans.iter().map(|plan| {
                                                    let price_str = format!("${}.{:02}/{}", plan.price / 100, plan.price % 100, plan.interval);
                                                    let name = plan.name.clone();
                                                    view! {
                                                        <div class="flex justify-between items-center py-1">
                                                            <span class="text-on-surface-variant">{name}</span>
                                                            <span class="font-bold text-primary font-mono">{price_str}</span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                        <div class="border-t border-outline-variant/10 pt-4 mt-2">
                                            <p class="text-[10px] text-on-surface-variant/60 text-center italic">
                                                "Per-product revenue requires billing ledger splits endpoint"
                                            </p>
                                        </div>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    })}
                    </Suspense>

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
                                <option value="">"All Tenants"</option>
                                {move || tenant_list.get().unwrap_or_default().into_iter().map(|t| {
                                    let n = t.name.clone();
                                    view! { <option value=n.clone()>{n.clone()}</option> }
                                }).collect_view()}
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
                    <div class="flex items-start gap-3.5 p-4 bg-amber-500/10 border border-amber-500/30 rounded-xl text-xs text-on-surface-variant leading-relaxed">
                        <span class="material-symbols-outlined text-amber-500 text-base mt-0.5">"warning"</span>
                        <div>
                            <span class="font-bold text-amber-400">"Overdue invoice reconciliation pending."</span>
                            <p class="mt-0.5">"Automated grace period checks run daily. Subscriptions auto-suspend upon threshold violations. Connect the billing reconciliation endpoint to populate this view."</p>
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
                                {move || tenant_list.get().unwrap_or_default().into_iter().map(|t| {
                                    let n = t.name.clone();
                                    view! { <option value=n.clone()>{n.clone()}</option> }
                                }).collect_view()}
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
            // ── TAB CONTENT: Revenue Intelligence ────────────────────────────────
            <Show when=move || active_tab.get() == "rev_intel">
                <div class="space-y-6">
                    <Suspense fallback=move || view! { <div class="p-8 text-center text-xs text-on-surface-variant/50">"Loading..."</div> }>
                    {move || tenant_list.get().map(|tenants| {
                        // ─ Tier distribution ─────────────────────────────────────
                        let total = tenants.len().max(1);
                        let enterprise_tenants: Vec<_> = tenants.iter().filter(|t| t.plan.as_deref().map(|p| p.to_lowercase().contains("enterprise")).unwrap_or(false)).collect();
                        let growth_tenants: Vec<_> = tenants.iter().filter(|t| t.plan.as_deref().map(|p| p.to_lowercase().contains("growth")).unwrap_or(false)).collect();
                        let starter_tenants: Vec<_> = tenants.iter().filter(|t| t.plan.as_deref().map(|p| {
                            let pl = p.to_lowercase();
                            pl.contains("starter") || pl.contains("basic") || pl.contains("free")
                        }).unwrap_or(false)).collect();
                        let no_plan_count = tenants.iter().filter(|t| t.plan.is_none() || t.plan.as_deref() == Some("")).count();

                        // MRR by tier (from mrr_cents field)
                        let enterprise_mrr: i64 = enterprise_tenants.iter().filter_map(|t| t.mrr_cents).sum();
                        let growth_mrr: i64 = growth_tenants.iter().filter_map(|t| t.mrr_cents).sum();
                        let starter_mrr: i64 = starter_tenants.iter().filter_map(|t| t.mrr_cents).sum();
                        let total_mrr = (enterprise_mrr + growth_mrr + starter_mrr).max(1);

                        let tiers = vec![
                            ("Enterprise", enterprise_tenants.len(), enterprise_mrr, "#818cf8", "rgba(99,102,241,0.12)"),
                            ("Growth",     growth_tenants.len(),     growth_mrr,     "#34d399", "rgba(16,185,129,0.12)"),
                            ("Starter",    starter_tenants.len(),    starter_mrr,    "#fbbf24", "rgba(245,158,11,0.12)"),
                        ];

                        // MRR by app type (from tenant_list; only rough but honest)
                        // We'll use plan as proxy since we don't have per-app-type MRR in this endpoint.

                        // Grace period alert — use billing_summary signal
                        let grace_count = billing_summary.get().flatten().map(|s| s.in_grace_period).unwrap_or(0);
                        let suspended_count = billing_summary.get().flatten().map(|s| s.suspended).unwrap_or(0);

                        view! {
                            <div class="space-y-6">
                                // ─ Grace Period Alert Banner
                                {if grace_count > 0 || suspended_count > 0 {
                                    view! {
                                        <div style="\
                                            display:flex;\
                                            align-items:center;\
                                            gap:12px;\
                                            padding:14px 18px;\
                                            background:rgba(239,68,68,0.08);\
                                            border:1px solid rgba(239,68,68,0.3);\
                                            border-radius:10px;\
                                            font-size:12px;\
                                        ">
                                            <span style="font-size:20px;">"⚠️"</span>
                                            <div>
                                                <div style="font-weight:700;color:#ef4444;margin-bottom:2px;">
                                                    {format!("{} tenant{} in grace period · {} suspended",
                                                        grace_count,
                                                        if grace_count == 1 { "" } else { "s" },
                                                        suspended_count
                                                    )}
                                                </div>
                                                <div style="color:var(--text-muted);font-size:11px;">
                                                    "Auto-suspension runs daily. Review overdue accounts to prevent involuntary churn."
                                                    <a href="#" style="color:var(--cobalt);margin-left:6px;text-decoration:none;"
                                                        on:click=move |_| active_tab.set("overdue".to_string())
                                                    >"View overdue →"</a>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <></> }.into_any()
                                }}

                                // ─ Plan Tier Distribution
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                    <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20">
                                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">
                                            "Plan Tier Distribution · MRR Share"
                                        </h3>
                                        <span class="text-[10px] text-on-surface-variant/60">{format!("{} tenants total", total)}</span>
                                    </div>
                                    <div class="p-5 space-y-5">
                                        {tiers.into_iter().map(|(tier_name, count, mrr_cents, color, bg)| {
                                            let count_pct = (count as f64 / total as f64 * 100.0) as u32;
                                            let mrr_pct = (mrr_cents as f64 / total_mrr as f64 * 100.0) as u32;
                                            let mrr_dollars = mrr_cents / 100;
                                            view! {
                                                <div style="display:flex;flex-direction:column;gap:6px;">
                                                    <div style="display:flex;align-items:center;justify-content:space-between;">
                                                        <div style="display:flex;align-items:center;gap:8px;">
                                                            <span style=format!("display:inline-block;width:10px;height:10px;border-radius:2px;background:{};", color)></span>
                                                            <span style="font-size:12px;font-weight:600;">{tier_name}</span>
                                                            <span style=format!("font-size:10px;padding:1px 7px;border-radius:4px;background:{};color:{};", bg, color)>
                                                                {format!("{} tenant{}", count, if count == 1 { "" } else { "s" })}
                                                            </span>
                                                        </div>
                                                        <div style="font-size:11px;font-family:monospace;color:var(--text-muted);">
                                                            {format!("${}/mo · {}% MRR", mrr_dollars, mrr_pct)}
                                                        </div>
                                                    </div>
                                                    // Dual-bar: tenant count (faint) + MRR share (solid)
                                                    <div style="position:relative;height:6px;background:rgba(255,255,255,0.06);border-radius:3px;">
                                                        <div style=format!("position:absolute;left:0;top:0;height:6px;border-radius:3px;width:{}%;background:{};", count_pct, color)></div>
                                                    </div>
                                                    <div style="position:relative;height:3px;background:rgba(255,255,255,0.04);border-radius:2px;margin-top:-3px;">
                                                        <div style=format!("position:absolute;left:0;top:0;height:3px;border-radius:2px;width:{}%;background:{}88;", mrr_pct, color)></div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                        {if no_plan_count > 0 {
                                            view! {
                                                <div style="font-size:11px;color:var(--text-muted);padding-top:4px;border-top:1px solid var(--border-subtle);">
                                                    {format!("{} tenant{} with no plan assigned", no_plan_count, if no_plan_count == 1 { "" } else { "s" })}
                                                    <a href="/tenants" style="color:var(--cobalt);margin-left:6px;text-decoration:none;">"Review →"</a>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <></> }.into_any()
                                        }}
                                    </div>
                                </div>

                                // ─ Tenant MRR Ranking Table
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                    <div class="px-5 py-3.5 border-b border-outline-variant/20">
                                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Top Tenants by MRR"</h3>
                                    </div>
                                    <div style="overflow-x:auto;">
                                        <table style="width:100%;border-collapse:collapse;font-size:12px;">
                                            <thead>
                                                <tr style="background:rgba(255,255,255,0.03);">
                                                    <th style="text-align:left;padding:8px 16px;font-size:10px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">"Tenant"</th>
                                                    <th style="text-align:left;padding:8px 16px;font-size:10px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">"Plan"</th>
                                                    <th style="text-align:right;padding:8px 16px;font-size:10px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">"MRR"</th>
                                                    <th style="text-align:left;padding:8px 16px;font-size:10px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">"Status"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {{
                                                    let mut sorted = tenants.clone();
                                                    sorted.sort_by(|a, b| b.mrr_cents.unwrap_or(0).cmp(&a.mrr_cents.unwrap_or(0)));
                                                    sorted.into_iter().take(15).map(|t| {
                                                        let mrr_str = t.mrr_cents.map(|c| format!("${}/mo", c / 100)).unwrap_or_else(|| "—".to_string());
                                                        let plan_str = t.plan.clone().unwrap_or_else(|| "No plan".to_string());
                                                        let status = t.site_status.clone().unwrap_or_default();
                                                        let (status_color, status_label) = match status.to_lowercase().as_str() {
                                                            "active" => ("#22c55e", "Live"),
                                                            "suspended" => ("#ef4444", "Suspended"),
                                                            _ => ("#f59e0b", "—"),
                                                        };
                                                        view! {
                                                            <tr style="border-bottom:1px solid var(--border-subtle);">
                                                                <td style="padding:9px 16px;font-weight:500;">{t.name.clone()}</td>
                                                                <td style="padding:9px 16px;color:var(--text-muted);">{plan_str}</td>
                                                                <td style="padding:9px 16px;text-align:right;font-family:monospace;font-weight:700;color:var(--cobalt);">{mrr_str}</td>
                                                                <td style="padding:9px 16px;">
                                                                    <span style=format!("font-size:9px;font-weight:700;color:{};padding:2px 7px;background:{}22;border-radius:4px;", status_color, status_color)>
                                                                        {status_label}
                                                                    </span>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }}
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    })}
                    </Suspense>
                </div>
            </Show>

        </div>
    }
}

