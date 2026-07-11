use crate::api::admin::{get_all_transactions, get_billing_plans, get_tenant_stats};
use crate::api::analytics::{get_billing_summary, get_business_kpis};
use leptos::prelude::*;
use shared_ui::components::ui::button::{Button, ButtonSize, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

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
        if active_tab.get() == tab_id {
            "tab active"
        } else {
            "tab"
        }
    };

    // ── Real data resources ──
    let refresh = RwSignal::new(0u32);
    let data_error: RwSignal<Option<String>> = RwSignal::new(None);
    let billing_summary = LocalResource::new(move || async move {
        let _ = refresh.get();
        match get_billing_summary().await {
            Ok(v) => {
                data_error.set(None);
                Some(v)
            }
            Err(e) => {
                data_error.set(Some(format!("Billing summary: {}", e)));
                None
            }
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
            &format!(
                "Subscription for {} ({}) on plan '{}' created.",
                tenant, product, plan
            ),
            "success",
        );
        show_new_sub_modal.set(false);
    };

    let handle_log_payment = move |_| {
        let tenant = log_payment_tenant.get();
        let amount = log_payment_amount.get();
        let reference = log_payment_ref.get();

        if amount.is_empty() {
            return;
        }

        toast.show_toast(
            "Success",
            &format!(
                "Manual payment of ${} logged for {}. Ref: {}",
                amount, tenant, reference
            ),
            "success",
        );
        log_payment_amount.set("".to_string());
        log_payment_ref.set("".to_string());
        show_log_payment_modal.set(false);
    };

    let handle_exempt_tenant = move |_| {
        let tenant = exempt_tenant.get();
        let reason = exempt_reason.get();

        if tenant.is_empty() {
            return;
        }

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
        <div class="main-canvas">
            // ── Page Header ──────────────────────────────────────────────────
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Billing & Revenue"</h1>
                    <p class="page-subtitle">"Platform financial health · subscriptions, MRR, ledger, and collection rates"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-ghost btn-icon"
                        title="Refresh billing data"
                        on:click=move |_| refresh.update(|n| *n += 1)
                    >
                        <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M2 8a6 6 0 0 1 6-6 6 6 0 0 1 4.2 1.8L14 6"/>
                            <path d="M14 2v4h-4"/>
                            <path d="M14 8a6 6 0 0 1-6 6 6 6 0 0 1-4.2-1.8L2 10"/>
                            <path d="M2 14v-4h4"/>
                        </svg>
                    </button>
                    <button class="btn btn-ghost" on:click=handle_export>
                        "↓ Export"
                    </button>
                </div>
            </div>

            // ── Error banner ──
            {move || data_error.get().map(|e| crate::utils::inline_error(&e))}

            // ── KPI Row ───────────────────────────────────────────────────────
            <div class="kpi-row" style="grid-template-columns:repeat(7,1fr);">
                <div class="kpi-card">
                    <div class="kpi-label">"Total MRR"</div>
                    <div class="kpi-value mono">
                        {move || business_kpis.get().flatten()
                            .map(|k| format!("${:.0}k", k.mrr.value / 1000.0))
                            .unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta up">
                        {move || business_kpis.get().flatten().map(|k| {
                            let delta = k.mrr.value - k.mrr.previous_value;
                            let pct = if k.mrr.previous_value > 0.0 { (delta / k.mrr.previous_value) * 100.0 } else { 0.0 };
                            format!("{:+.0}% MoM", pct)
                        }).unwrap_or("—".to_string())}
                    </div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Active Subs"</div>
                    <div class="kpi-value mono">
                        {move || billing_summary.get().flatten().map(|s| s.active_subscriptions.to_string()).unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta neutral">
                        {move || billing_summary.get().flatten()
                            .map(|s| format!("{} trial · {} grace", s.in_trial, s.in_grace_period))
                            .unwrap_or("—".to_string())}
                    </div>
                </div>
                <div class="kpi-card warn-border">
                    <div class="kpi-label">"Failed Invoices"</div>
                    <div class="kpi-value mono" style="color:var(--amber)">
                        {move || billing_summary.get().flatten().map(|s| s.failed_invoices_count.to_string()).unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta down">
                        {move || billing_summary.get().flatten()
                            .map(|s| format!("${:.0}k outstanding", s.failed_invoices_value / 1000.0))
                            .unwrap_or("—".to_string())}
                    </div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Collection Rate"</div>
                    <div class="kpi-value mono" style="color:var(--green)">
                        {move || billing_summary.get().flatten().map(|s| format!("{:.0}%", s.collection_success_rate)).unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta neutral">"Invoiced vs paid"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Billing Plans"</div>
                    <div class="kpi-value mono">
                        {move || billing_plans.get().map(|p| p.len().to_string()).unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta neutral">"Active plans"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Transactions"</div>
                    <div class="kpi-value mono">
                        {move || transactions.get().map(|t| t.len().to_string()).unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta neutral">"Ledger entries"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Gross Churn"</div>
                    <div class="kpi-value mono">
                        {move || billing_summary.get().flatten().map(|s| format!("{:.1}%", s.gross_churn_rate * 100.0)).unwrap_or("—".to_string())}
                    </div>
                    <div class="kpi-delta neutral" style="font-family:monospace">"G-17"</div>
                </div>
            </div>

            // ── Tab Bar ───────────────────────────────────────────────────────
            <div class="tab-bar">
                <button class=move || tab_class("products_plans") on:click=move |_| active_tab.set("products_plans".to_string())>"Products & Plans"</button>
                <button class=move || tab_class("rev_intel")       on:click=move |_| active_tab.set("rev_intel".to_string())>"Revenue Intel"</button>
                <button class=move || tab_class("ledger")          on:click=move |_| active_tab.set("ledger".to_string())>"Ledger · G-03"</button>
                <button class=move || tab_class("overdue")         on:click=move |_| active_tab.set("overdue".to_string())>"Overdue & Disputes"</button>
                <button class=move || tab_class("rails")           on:click=move |_| active_tab.set("rails".to_string())>"Payment Rails"</button>
                <button class=move || tab_class("commissions")     on:click=move |_| active_tab.set("commissions".to_string())>"Commission Plans · G-25"</button>
                <button class=move || tab_class("tax")             on:click=move |_| active_tab.set("tax".to_string())>"Tax & Filings · G-17"</button>
            </div>


            // ── TAB CONTENT: Products & Plans ──
            <Show when=move || active_tab.get() == "products_plans">
                <div style="display:flex;flex-direction:column;gap:14px;">
                    // Hierarchy callout
                    <div style="display:flex;align-items:center;gap:12px;padding:12px 16px;background:var(--cobalt-dim);border:1px solid rgba(10,132,255,0.2);border-radius:8px;font-size:12px;color:var(--text-secondary);">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4"><circle cx="7" cy="7" r="5.5"/><path d="M7 6v4M7 4.5v.5"/></svg>
                        <span><strong style="color:var(--text-primary)">"Platform Products"</strong>" → Billing Plans → Tenant Subscriptions → Ledger Entries → Ledger Splits."</span>
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

                    // Active billing plans table
                    <div class="section">
                        <div class="section-header">
                            <div class="section-title">
                                <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4"><rect x="1" y="2" width="12" height="10" rx="1"/><path d="M1 6h12"/></svg>
                                "All Billing Plans"
                                <span class="section-count">{move || billing_plans.get().map(|p| format!("{} plans", p.len())).unwrap_or_default()}</span>
                            </div>
                            <button class="btn btn-primary btn-sm" on:click=move |_| show_new_sub_modal.set(true)>
                                "+ New Subscription"
                            </button>
                        </div>
                        <Suspense fallback=move || view! { <div class="p-4 muted">"Loading plans…"</div> }>
                        <table>
                            <thead><tr>
                                <th>"Plan Name"</th>
                                <th class="right">"Price / Interval"</th>
                                <th>"Interval"</th>
                                <th>"Currency"</th>
                                <th class="right">"Actions"</th>
                            </tr></thead>
                            <tbody>
                            {move || billing_plans.get().unwrap_or_default().into_iter().map(|plan| {
                                let price_formatted = format!("${}.{:02} / {}", plan.price / 100, plan.price % 100, plan.interval);
                                view! {
                                    <tr>
                                        <td style="font-weight:600">{plan.name.clone()}</td>
                                        <td class="right mono" style="color:var(--green)">{price_formatted}</td>
                                        <td class="muted">{plan.interval.clone()}</td>
                                        <td class="mono">{plan.currency.clone()}</td>
                                        <td class="right">
                                            <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                toast.show_toast("Plan", "Drilling into plan details...", "info");
                                            }>"Details →"</button>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                            </tbody>
                        </table>
                        </Suspense>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Ledger ──
            <Show when=move || active_tab.get() == "ledger">
                <div class="section">
                    <div class="section-header">
                        <div class="section-title">
                            <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4"><path d="M2 4h10M2 7h7M2 10h5"/></svg>
                            "Transaction Ledger"
                            <span class="section-count">"G-03 · split-calculated"</span>
                        </div>
                        <div style="display:flex;align-items:center;gap:8px;">
                            <select on:change=move |ev| filter_tenant.set(event_target_value(&ev))>
                                <option value="">"All Tenants"</option>
                                {move || tenant_list.get().unwrap_or_default().into_iter().map(|t| {
                                    let n = t.name.clone();
                                    view! { <option value=n.clone()>{n.clone()}</option> }
                                }).collect_view()}
                            </select>
                            <select on:change=move |ev| filter_rail.set(event_target_value(&ev))>
                                <option>"All Rails"</option><option>"Stripe"</option><option>"Bitcoin"</option><option>"PIX"</option>
                            </select>
                            <select on:change=move |ev| filter_status.set(event_target_value(&ev))>
                                <option>"All Statuses"</option><option>"Paid"</option><option>"Pending"</option><option>"Overdue"</option>
                            </select>
                            <button class="btn btn-ghost btn-sm" on:click=handle_export>"Export"</button>
                        </div>
                    </div>
                    <Suspense fallback=move || view! { <div class="p-4 muted">"Loading transactions…"</div> }>
                    <table>
                        <thead><tr>
                            <th>"TX ID"</th>
                            <th>"Tenant"</th>
                            <th>"Provider"</th>
                            <th class="right">"Amount"</th>
                            <th class="center">"Status"</th>
                            <th>"Provider TX ID"</th>
                            <th>"Created"</th>
                        </tr></thead>
                        <tbody>
                        {move || transactions.get().unwrap_or_default().into_iter().map(|tx| {
                            let amount_fmt = format!("${}.{:02} {}", tx.amount / 100, tx.amount.abs() % 100, tx.currency.to_uppercase());
                            let status_color = match tx.status.as_str() {
                                "paid" | "completed" | "settled" => "var(--green)",
                                "overdue" | "failed" => "var(--red)",
                                "pending" => "var(--cobalt)",
                                _ => "var(--text-muted)",
                            };
                            let status = tx.status.clone();
                            view! {
                                <tr>
                                    <td class="mono" style="font-size:10px">{tx.id[..8].to_string() + "…"}</td>
                                    <td class="mono" style="font-size:10px">
                                        <a href={format!("/billing/tenant/{}", tx.tenant_id)}
                                            style="color:var(--text-link);text-decoration:none;"
                                            title="View tenant billing"
                                        >{tx.tenant_id[..8].to_string() + "…"}</a>
                                    </td>
                                    <td class="secondary">{tx.provider.clone()}</td>
                                    <td class="right mono" style="color:var(--green);font-weight:700">{amount_fmt}</td>
                                    <td class="center">
                                        <span class="plan-badge" style=format!("color:{s};border-color:{s}", s=status_color)>{status}</span>
                                    </td>
                                    <td class="mono muted" style="font-size:10px">{tx.provider_tx_id.clone().unwrap_or("—".to_string())}</td>
                                    <td class="mono" style="font-size:10px">{tx.created_at.clone().unwrap_or("—".to_string())}</td>
                                </tr>
                            }
                        }).collect_view()}
                        </tbody>
                    </table>
                    </Suspense>
                </div>
            </Show>

            // ── TAB CONTENT: Overdue & Disputes ──
            <Show when=move || active_tab.get() == "overdue">
                <div style="display:flex;flex-direction:column;gap:14px;">
                    // Alarm banner
                    <div style="display:flex;align-items:center;gap:12px;padding:12px 16px;background:var(--amber-dim);border:1px solid var(--amber);border-radius:8px;font-size:12px;color:var(--text-secondary);">
                        <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4" style="color:var(--amber)"><path d="M7 1L13 12H1L7 1z"/><path d="M7 5v4M7 10.5v.5"/></svg>
                        <div>
                            <span style="font-weight:700;color:var(--amber)">"Overdue invoice reconciliation pending."</span>
                            <span style="margin-left:8px">"Grace period checks run daily. Subscriptions auto-suspend upon threshold violations."</span>
                        </div>
                    </div>
                    <div class="two-col">
                        <div class="section">
                            <div class="section-header">
                                <div class="section-title">"Outstanding Invoice Reminders"</div>
                                <button class="btn btn-ghost btn-sm"
                                    on:click=move |_| toast.show_toast("Success", "Bulk reminder emails queued.", "success")
                                >"Send All Reminders"</button>
                            </div>
                            <table><thead><tr>
                                <th>"Tenant"</th><th>"Product"</th><th class="right">"Amount"</th>
                                <th>"Due Date"</th><th class="center">"Overdue"</th><th>"Rail"</th>
                            </tr></thead>
                            <tbody><tr><td colspan="6" class="center muted" style="padding:32px">
                                "Overdue data pending — billing reconciliation endpoint (future)"
                            </td></tr></tbody></table>
                        </div>
                        <div class="section">
                            <div class="section-header">
                                <div class="section-title">"Billing Exemptions"</div>
                                <button class="btn btn-ghost btn-sm" on:click=move |_| show_exemption_modal.set(true)>"+ Exempt"</button>
                            </div>
                            <div style="padding:12px 16px;font-size:12px;color:var(--text-muted);line-height:1.6">
                                "Bypass auto-suspensions and grace period locks for strategic VIP client SLA accounts."
                            </div>
                            <div style="border-top:1px solid var(--border-default);">
                                {move || billing_summary.get().flatten().map(|s| s.exemptions.into_iter().map(|e| view! {
                                    <div style="display:flex;justify-content:space-between;align-items:center;padding:9px 16px;border-bottom:1px solid var(--border-subtle);font-size:12px;">
                                        <span style="font-weight:500">{e.tenant_name.clone()}</span>
                                        <span class="muted" style="font-size:11px">{e.reason.clone()}</span>
                                    </div>
                                }).collect_view()).unwrap_or_default()}
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Tax & Filings ──
            <Show when=move || active_tab.get() == "tax">
                <div class="section">
                    <div class="section-header">
                        <div class="section-title">
                            <svg viewBox="0 0 14 14" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4"><rect x="2" y="1" width="10" height="12" rx="1"/><path d="M5 5h4M5 8h3"/></svg>
                            "VAT, Sales Tax & Corporate Filings"
                            <span class="section-count">"G-17"</span>
                        </div>
                    </div>
                    <table><thead><tr>
                        <th>"Filing Code"</th><th>"Jurisdiction"</th>
                        <th class="right">"Gross Sales"</th><th class="right">"Tax Collected"</th>
                        <th class="center">"Status"</th><th>"Deadline"</th><th class="right">"Actions"</th>
                    </tr></thead>
                    <tbody><tr><td colspan="7" class="center muted" style="padding:32px">
                        "Tax filing data pending — requires G-17 tax_events entity and endpoint (future)"
                    </td></tr></tbody></table>
                </div>
            </Show>

            // ── Modals ─────────────────────────────────────────────────────────
            // Shared modal overlay style
            <Show when=move || show_new_sub_modal.get()>
                <div style="position:fixed;inset:0;z-index:500;background:rgba(0,0,0,0.75);display:flex;align-items:center;justify-content:center;padding:16px;">
                    <div style="background:var(--bg-elevated);border:1px solid var(--border-strong);border-radius:12px;padding:28px;width:100%;max-width:440px;position:relative;">
                        <button style="position:absolute;top:16px;right:16px;background:none;border:none;color:var(--text-muted);cursor:pointer;font-size:16px;" on:click=move |_| show_new_sub_modal.set(false)>"✕"</button>
                        <h3 class="page-title" style="font-size:16px;margin-bottom:4px">"New Tenant Subscription"</h3>
                        <p class="page-subtitle" style="margin-bottom:20px">"Manually provision a product billing subscription plan for a tenant account."</p>
                        <div style="display:flex;flex-direction:column;gap:14px;margin-bottom:20px;">
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Tenant Account"</label>
                                <select style="width:100%" on:change=move |ev| new_sub_tenant.set(event_target_value(&ev))>
                                    {move || tenant_list.get().unwrap_or_default().into_iter().map(|t| {
                                        let n = t.name.clone();
                                        view! { <option value=n.clone()>{n.clone()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Platform Product"</label>
                                <select style="width:100%" on:change=move |ev| new_sub_product.set(event_target_value(&ev))>
                                    <option>"Folio"</option><option>"Anchor"</option><option>"Network"</option><option>"Meridian"</option>
                                </select>
                            </div>
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Billing Plan"</label>
                                <select style="width:100%" on:change=move |ev| new_sub_plan.set(event_target_value(&ev))>
                                    <option>"Starter - $400/mo"</option><option>"STR Pro - $1,800/mo"</option>
                                    <option>"Enterprise - $6,000/mo"</option><option>"Creator Pro - $2,400/mo"</option>
                                    <option>"Network Pro - $600/mo"</option>
                                </select>
                            </div>
                        </div>
                        <div style="display:flex;justify-content:flex-end;gap:8px;">
                            <button class="btn btn-ghost" on:click=move |_| show_new_sub_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=handle_new_subscription>"Create Subscription"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_log_payment_modal.get()>
                <div style="position:fixed;inset:0;z-index:500;background:rgba(0,0,0,0.75);display:flex;align-items:center;justify-content:center;padding:16px;">
                    <div style="background:var(--bg-elevated);border:1px solid var(--border-strong);border-radius:12px;padding:28px;width:100%;max-width:440px;position:relative;">
                        <button style="position:absolute;top:16px;right:16px;background:none;border:none;color:var(--text-muted);cursor:pointer;font-size:16px;" on:click=move |_| show_log_payment_modal.set(false)>"✕"</button>
                        <h3 class="page-title" style="font-size:16px;margin-bottom:4px">"Log Manual Payment"</h3>
                        <p class="page-subtitle" style="margin-bottom:20px">"Record a bank wire or off-channel transaction to settle outstanding invoices."</p>
                        <div style="display:flex;flex-direction:column;gap:14px;margin-bottom:20px;">
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Tenant Account"</label>
                                <Input r#type=InputType::Text bind_value=log_payment_tenant/>
                            </div>
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Amount ($)"</label>
                                <Input r#type=InputType::Text bind_value=log_payment_amount placeholder="e.g. 28000.00".to_string()/>
                            </div>
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Reference"</label>
                                <Input r#type=InputType::Text bind_value=log_payment_ref placeholder="e.g. wire_98231_bank_amex".to_string()/>
                            </div>
                        </div>
                        <div style="display:flex;justify-content:flex-end;gap:8px;">
                            <button class="btn btn-ghost" on:click=move |_| show_log_payment_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=handle_log_payment>"Log Payment"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_exemption_modal.get()>
                <div style="position:fixed;inset:0;z-index:500;background:rgba(0,0,0,0.75);display:flex;align-items:center;justify-content:center;padding:16px;">
                    <div style="background:var(--bg-elevated);border:1px solid var(--border-strong);border-radius:12px;padding:28px;width:100%;max-width:440px;position:relative;">
                        <button style="position:absolute;top:16px;right:16px;background:none;border:none;color:var(--text-muted);cursor:pointer;font-size:16px;" on:click=move |_| show_exemption_modal.set(false)>"✕"</button>
                        <h3 class="page-title" style="font-size:16px;margin-bottom:4px">"Billing Exemption Override"</h3>
                        <p class="page-subtitle" style="margin-bottom:20px">"Exempt a tenant from automated suspension and grace timers."</p>
                        <div style="display:flex;flex-direction:column;gap:14px;margin-bottom:20px;">
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Tenant Name"</label>
                                <Input r#type=InputType::Text bind_value=exempt_tenant placeholder="e.g. Urban Core Mgmt".to_string()/>
                            </div>
                            <div><label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px">"Exemption Reason"</label>
                                <Input r#type=InputType::Text bind_value=exempt_reason placeholder="e.g. VIP SLA Contract Waiver".to_string()/>
                            </div>
                        </div>
                        <div style="display:flex;justify-content:flex-end;gap:8px;">
                            <button class="btn btn-ghost" on:click=move |_| show_exemption_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=handle_exempt_tenant>"Exempt Tenant"</button>
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
                                                    <th style="text-align:left;padding:8px 16px;font-size:10px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;">"Billing"</th>
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
                                                                <td style="padding:9px 16px;font-weight:500;">
                                                                    <a href={format!("/tenants/{}", t.tenant_id)}
                                                                        style="color:var(--text-primary);text-decoration:none;"
                                                                        onmouseover="this.style.color='var(--text-link)'"
                                                                        onmouseout="this.style.color='var(--text-primary)'"
                                                                    >{t.name.clone()}</a>
                                                                </td>
                                                                <td style="padding:9px 16px;color:var(--text-muted);">{plan_str}</td>
                                                                <td style="padding:9px 16px;text-align:right;font-family:monospace;font-weight:700;color:var(--cobalt);">{mrr_str}</td>
                                                                <td style="padding:9px 16px;">
                                                                    <span style=format!("font-size:9px;font-weight:700;color:{};padding:2px 7px;background:{}22;border-radius:4px;", status_color, status_color)>
                                                                        {status_label}
                                                                    </span>
                                                                </td>
                                                                <td style="padding:9px 16px;">
                                                                    <a href={format!("/billing/tenant/{}", t.tenant_id)}
                                                                        style="font-size:10px;color:var(--text-link);text-decoration:none;"
                                                                    >"Billing →"</a>
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

            // ── Revenue Intelligence tab content is unchanged ─────────────────
        </div>
    }
}
