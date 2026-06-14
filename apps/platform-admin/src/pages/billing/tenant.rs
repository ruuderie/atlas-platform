use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::button::{Button, ButtonVariant, ButtonSize};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};

#[derive(Clone, Debug)]
pub struct MockUserRoster {
    pub name: String,
    pub role: String,
    pub email: String,
    pub last_active: String,
    pub mfa: String,
}

#[component]
pub fn TenantLedger() -> impl IntoView {
    let params = use_params_map();
    let tenant_id_str = move || params.with(|p| p.get("id").unwrap_or_else(|| "t_8a91f3d2".to_string()));
    
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Active workspace tab state
    let active_tab = RwSignal::new("subscription".to_string());
    
    // Modals state
    let show_credit_modal = RwSignal::new(false);
    let show_invoice_modal = RwSignal::new(false);
    let show_plan_modal = RwSignal::new(false);
    let show_invite_modal = RwSignal::new(false);
    
    // Forms state
    let credit_amount = RwSignal::new("".to_string());
    let credit_reason = RwSignal::new("".to_string());
    let invoice_amount = RwSignal::new("".to_string());
    let invoice_period = RwSignal::new("Jun 2026".to_string());
    let selected_plan_id = RwSignal::new("".to_string());
    
    let invite_name = RwSignal::new("".to_string());
    let invite_email = RwSignal::new("".to_string());
    let invite_role = RwSignal::new("Property Manager".to_string());

    // Fetch database transactions for this tenant
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    let ledger_res = LocalResource::new({
        let tid_fn = tenant_id_str.clone();
        move || {
            trigger_fetch.get();
            let tid = tid_fn();
            async move {
                crate::api::billing::get_tenant_ledger(&tid).await.unwrap_or_default()
            }
        }
    });

    // Fetch database plans
    let plans_res = LocalResource::new(move || async move {
        crate::api::billing::list_billing_plans().await.unwrap_or_default()
    });

    // Derived style helpers
    let tab_class = move |tab_id: &str| {
        let active = active_tab.get() == tab_id;
        if active {
            "px-4 py-2.5 text-xs font-bold text-primary border-b-2 border-primary bg-transparent outline-none transition-all"
        } else {
            "px-4 py-2.5 text-xs font-semibold text-on-surface-variant hover:text-on-surface bg-transparent outline-none transition-all"
        }
    };

    // User Roster Mock Data
    let roster_users = StoredValue::new(vec![
        MockUserRoster { name: "Ruud Salym Erie".to_string(), role: "Admin".to_string(), email: "ruud@nexusprops.com".to_string(), last_active: "Jun 11 · 21:44".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "Maria Fernandes".to_string(), role: "Admin".to_string(), email: "maria@nexusprops.com".to_string(), last_active: "Jun 10 · 18:02".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "Carlos Mendes".to_string(), role: "Prop. Mgr".to_string(), email: "carlos@nexusprops.com".to_string(), last_active: "Jun 11 · 09:30".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "Aisha Wanjiku".to_string(), role: "Prop. Mgr".to_string(), email: "aisha@nexusprops.com".to_string(), last_active: "Jun 09 · 14:22".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "James Okafor".to_string(), role: "Leasing".to_string(), email: "james@nexusprops.com".to_string(), last_active: "Jun 11 · 11:05".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "Sonia Park".to_string(), role: "Leasing".to_string(), email: "sonia@nexusprops.com".to_string(), last_active: "Jun 08 · 16:40".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "Mike Torres".to_string(), role: "Maint. Coord".to_string(), email: "mike@nexusprops.com".to_string(), last_active: "Jun 11 · 07:45".to_string(), mfa: "Passkey".to_string() },
        MockUserRoster { name: "Sandra Osei".to_string(), role: "Maint. Coord".to_string(), email: "sandra@nexusprops.com".to_string(), last_active: "Jun 10 · 10:30".to_string(), mfa: "Passkey".to_string() },
    ]);

    // Mock Invoices
    let mock_invoices = StoredValue::new(vec![
        ("INV-2026-06", "Jun 2026", "$4,800", "$0", "$460", "—", "$5,260", "Paid"),
        ("INV-2026-05", "May 2026", "$4,800", "$0", "$388", "—", "$5,188", "Paid"),
        ("INV-2026-04", "Apr 2026", "$4,800", "$90", "$412", "—", "$5,302", "Paid"),
        ("INV-2026-03", "Mar 2026", "$4,800", "$90", "$394", "-$200", "$5,084", "Paid"),
        ("INV-2026-02", "Feb 2026", "$4,800", "$0", "$310", "—", "$5,110", "Paid"),
    ]);

    // Actions implementation
    let handle_issue_credit = move |_| {
        let amt = credit_amount.get();
        let reason = credit_reason.get();
        if amt.is_empty() { return; }
        
        toast.show_toast("Success", &format!("Billing credit of ${} issued successfully. Reason: {}", amt, reason), "success");
        credit_amount.set("".to_string());
        credit_reason.set("".to_string());
        show_credit_modal.set(false);
    };

    let handle_send_invoice = move |_| {
        let amt = invoice_amount.get();
        let period = invoice_period.get();
        if amt.is_empty() { return; }

        toast.show_toast("Success", &format!("Manual invoice of ${} generated for period {} and dispatched.", amt, period), "success");
        invoice_amount.set("".to_string());
        show_invoice_modal.set(false);
    };

    let handle_change_plan = move |_| {
        let plan_id = selected_plan_id.get();
        if plan_id.is_empty() { return; }

        toast.show_toast("Success", "Platform subscription plan tier updated successfully.", "success");
        show_plan_modal.set(false);
    };

    let handle_invite_user = move |_| {
        let name = invite_name.get();
        let email = invite_email.get();
        let role = invite_role.get();
        if name.is_empty() || email.is_empty() { return; }

        toast.show_toast("Success", &format!("Team invitation successfully dispatched to {} ({})", email, role), "success");
        invite_name.set("".to_string());
        invite_email.set("".to_string());
        show_invite_modal.set(false);
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumb & Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div class="space-y-1">
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <a href="/apps" class="hover:text-primary transition-colors">"Tenants"</a>
                        <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                        <span class="text-on-surface-variant/80">"Nexus Property Group"</span>
                        <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                        <span class="text-primary/70 font-semibold">"Billing & Ledger"</span>
                    </nav>
                    
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-primary/10 border border-primary/30 text-primary rounded-xl flex items-center justify-center font-bold text-lg">
                            "N"
                        </div>
                        <div>
                            <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">"Nexus Property Group"</h1>
                            <p class="text-xs text-on-surface-variant font-mono mt-0.5">
                                "tenant_id: " {tenant_id_str} " · nexus.atlas.app · Enterprise Plan"
                            </p>
                        </div>
                    </div>
                </div>

                <div class="flex items-center gap-3">
                    <Button variant=ButtonVariant::Outline on:click=move |_| show_credit_modal.set(true)>
                        "Issue Credit"
                    </Button>
                    <Button variant=ButtonVariant::Outline on:click=move |_| show_invoice_modal.set(true)>
                        "Send Invoice"
                    </Button>
                    <Button variant=ButtonVariant::Default on:click=move |_| show_plan_modal.set(true)>
                        "Change Plan"
                    </Button>
                </div>
            </div>

            // ── KPI Strip ──
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4 bg-surface-container-low border border-outline-variant/10 p-5 rounded-2xl shadow-xs">
                <div class="space-y-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Current MRR"</span>
                    <div class="text-2xl font-extrabold text-primary font-mono">"$5,260"</div>
                    <p class="text-[10px] text-on-surface-variant/70">"Base + overage + seats"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Billing Model"</span>
                    <div class="pt-1">
                        <Badge intent=BadgeIntent::Primary>"Hybrid"</Badge>
                    </div>
                    <p class="text-[10px] text-on-surface-variant/70">"Flat base + seat add-ons"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Billable Seats"</span>
                    <div class="text-2xl font-extrabold font-mono">"14 " <span class="text-xs font-normal text-on-surface-variant">"/ 20 incl."</span></div>
                    <p class="text-[10px] text-on-surface-variant/70">"0 overage seats"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Payment Status"</span>
                    <div class="pt-1">
                        <Badge intent=BadgeIntent::Success>"Current"</Badge>
                    </div>
                    <p class="text-[10px] text-on-surface-variant/70">"Last paid Jun 01"</p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Contract End"</span>
                    <div class="text-lg font-bold text-on-surface pt-0.5">"Feb 2027"</div>
                    <p class="text-[10px] text-on-surface-variant/70">"Annual · Auto-renews"</p>
                </div>
            </div>

            // ── Tabs Bar ──
            <div class="flex border-b border-outline-variant/15 overflow-x-auto shrink-0 select-none">
                <button class=move || tab_class("subscription") on:click=move |_| active_tab.set("subscription".to_string())>
                    "Subscription"
                </button>
                <button class=move || tab_class("seats") on:click=move |_| active_tab.set("seats".to_string())>
                    "Users & Seats"
                </button>
                <button class=move || tab_class("invoices") on:click=move |_| active_tab.set("invoices".to_string())>
                    "Invoice History"
                </button>
                <button class=move || tab_class("ledger") on:click=move |_| active_tab.set("ledger".to_string())>
                    "Database Ledger"
                </button>
                <button class=move || tab_class("features") on:click=move |_| active_tab.set("features".to_string())>
                    "Features (14)"
                </button>
            </div>

            // ── TAB CONTENT: Subscription ──
            <Show when=move || active_tab.get() == "subscription">
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <div class="lg:col-span-2 space-y-6">
                        // Plan Details Card
                        <Card class="bg-card border-border shadow-sm p-6".to_string()>
                            <h3 class="text-sm font-bold uppercase tracking-wider text-primary mb-4">"Plan Details"</h3>
                            <div class="divide-y divide-outline-variant/10 text-xs">
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Plan Tier"</span>
                                    <span class="font-bold text-primary">"Enterprise (Annual)"</span>
                                </div>
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Base MRR"</span>
                                    <span class="font-mono font-bold">$4,800.00</span>
                                </div>
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Seat Add-ons"</span>
                                    <span class="font-mono text-on-surface-variant/80">$0.00</span>
                                </div>
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Platform Commission"</span>
                                    <span class="font-mono">3.0%</span>
                                </div>
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Commission MTD"</span>
                                    <span class="font-mono text-emerald-400">$460.00</span>
                                </div>
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Billing Cycle"</span>
                                    <span>"Annual · Monthly Invoicing"</span>
                                </div>
                                <div class="flex justify-between items-center py-3">
                                    <span class="text-on-surface-variant">"Renewal Date"</span>
                                    <span>"Feb 01, 2027"</span>
                                </div>
                            </div>
                        </Card>

                        // Plan Limits & Allotments Card
                        <Card class="bg-card border-border shadow-sm p-6".to_string()>
                            <h3 class="text-sm font-bold uppercase tracking-wider text-primary mb-4">"Plan Limits & Allotments"</h3>
                            <div class="space-y-4 text-xs">
                                <div>
                                    <div class="flex justify-between text-xs mb-1.5">
                                        <span class="text-on-surface-variant">"Seats"</span>
                                        <span class="font-mono font-bold">"14 / 20 Included"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-primary rounded-full" style="width: 70%"></div>
                                    </div>
                                </div>

                                <div>
                                    <div class="flex justify-between text-xs mb-1.5">
                                        <span class="text-on-surface-variant">"Listings Catalog"</span>
                                        <span class="font-mono font-bold">"87 Active / Unlimited"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-emerald-400 rounded-full" style="width: 20%"></div>
                                    </div>
                                </div>

                                <div>
                                    <div class="flex justify-between text-xs mb-1.5">
                                        <span class="text-on-surface-variant">"Monthly API Calls"</span>
                                        <span class="font-mono font-bold">"360,000 / 2,000,000"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-emerald-400 rounded-full" style="width: 18%"></div>
                                    </div>
                                </div>

                                <div>
                                    <div class="flex justify-between text-xs mb-1.5">
                                        <span class="text-on-surface-variant">"Cloudflare Vault Storage"</span>
                                        <span class="font-mono font-bold">"212 GB / 500 GB"</span>
                                    </div>
                                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden">
                                        <div class="h-full bg-primary rounded-full" style="width: 42%"></div>
                                    </div>
                                </div>
                            </div>
                        </Card>
                    </div>

                    // Sidebar columns: Reference & Account Manager
                    <div class="space-y-6">
                        <Card class="bg-card border-border shadow-sm p-5".to_string()>
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-4">"Billing Model Reference"</h3>
                            <div class="space-y-4 text-xs">
                                <div class="p-3 bg-surface-container-high/40 rounded-lg border border-outline-variant/10 space-y-1">
                                    <div class="flex justify-between font-bold">
                                        <span>"Hybrid Mode"</span>
                                        <Badge intent=BadgeIntent::Primary>"Active"</Badge>
                                    </div>
                                    <p class="text-[10px] text-on-surface-variant/80">"Flat base tier fee including initial seats, with incremental per-seat billing."</p>
                                </div>
                                <div class="p-3 bg-surface-container/20 rounded-lg border border-outline-variant/10 space-y-1 opacity-60">
                                    <span class="font-bold text-on-surface">"Flat Rate Simple"</span>
                                    <p class="text-[10px] text-on-surface-variant/80">"Fixed monthly invoice amount regardless of operator roster dimensions."</p>
                                </div>
                                <div class="p-3 bg-surface-container/20 rounded-lg border border-outline-variant/10 space-y-1 opacity-60">
                                    <span class="font-bold text-on-surface">"Commission RevShare"</span>
                                    <p class="text-[10px] text-on-surface-variant/80">"Zero base. Earn percentages on STR bookings and transaction ledger exchanges."</p>
                                </div>
                            </div>
                        </Card>

                        <Card class="bg-card border-border shadow-sm p-5".to_string()>
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-3">"Account Manager"</h3>
                            <div class="flex items-center gap-3.5 p-1">
                                <div class="w-10 h-10 rounded-full bg-primary/25 border border-primary text-primary flex items-center justify-center font-bold text-sm shrink-0">
                                    "PS"
                                </div>
                                <div class="min-w-0 flex-1">
                                    <div class="font-bold text-xs">"Priya Sharma"</div>
                                    <div class="text-[10.5px] text-on-surface-variant/80 truncate">"priya@atlasplatform.co · ext 204"</div>
                                </div>
                                <a href="mailto:priya@atlasplatform.co" class="px-3 py-1.5 border border-outline-variant/30 text-on-surface hover:bg-surface-bright/20 rounded text-[11px] font-semibold">
                                    "Email"
                                </a>
                            </div>
                        </Card>
                    </div>
                </div>
            </Show>

            // ── TAB CONTENT: Users & Seats ──
            <Show when=move || active_tab.get() == "seats">
                <div class="space-y-6">
                    <div class="flex items-center justify-between">
                        <div>
                            <h3 class="text-sm font-bold text-on-surface">"14 Billable Roster Seats"</h3>
                            <p class="text-xs text-on-surface-variant">"Overage seat fee: $30.00 / month · 6 seats vacant before billing adjustments"</p>
                        </div>
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on:click=move |_| toast.show_toast("CSV Export", "CSV user roster dispatched to downloads.", "success")>
                                "Export CSV"
                            </Button>
                            <Button variant=ButtonVariant::Default size=ButtonSize::Sm on:click=move |_| show_invite_modal.set(true)>
                                "Invite User"
                            </Button>
                        </div>
                    </div>

                    // Seat Breakdown Table
                    <Card class="bg-card border-border shadow-sm overflow-hidden".to_string()>
                        <div class="px-5 py-3 border-b border-outline-variant/10 bg-[#06122d]/30 font-bold text-xs text-primary">"Billing Impacts by Roster Role"</div>
                        <DataTable class="w-full text-xs">
                            <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Role Type"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Active Users"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Billing Model"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Subtotal"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border">
                                <DataTableRow>
                                    <DataTableCell class="p-3">"Admin (Platform Access & Users)"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-bold">"2"</DataTableCell>
                                    <DataTableCell class="p-3 text-right">"Included in base"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-mono text-on-surface-variant/80">"$0.00"</DataTableCell>
                                </DataTableRow>
                                <DataTableRow>
                                    <DataTableCell class="p-3">"Property Manager (Operations)"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-bold">"5"</DataTableCell>
                                    <DataTableCell class="p-3 text-right">"Included in base"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-mono text-on-surface-variant/80">"$0.00"</DataTableCell>
                                </DataTableRow>
                                <DataTableRow>
                                    <DataTableCell class="p-3">"Leasing Agent (Prospecting)"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-bold">"4"</DataTableCell>
                                    <DataTableCell class="p-3 text-right">"Included in base"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-mono text-on-surface-variant/80">"$0.00"</DataTableCell>
                                </DataTableRow>
                                <DataTableRow>
                                    <DataTableCell class="p-3">"Maintenance Coordinator (Work Orders)"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-bold">"2"</DataTableCell>
                                    <DataTableCell class="p-3 text-right">"Included in base"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-mono text-on-surface-variant/80">"$0.00"</DataTableCell>
                                </DataTableRow>
                                <DataTableRow>
                                    <DataTableCell class="p-3">"Read-Only Auditor"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-bold">"1"</DataTableCell>
                                    <DataTableCell class="p-3 text-right">"Included in base"</DataTableCell>
                                    <DataTableCell class="p-3 text-right font-mono text-on-surface-variant/80">"$0.00"</DataTableCell>
                                </DataTableRow>
                            </DataTableBody>
                        </DataTable>
                    </Card>

                    // User Roster Table
                    <Card class="bg-card border-border shadow-sm overflow-hidden".to_string()>
                        <div class="px-5 py-3 border-b border-outline-variant/10 bg-[#06122d]/30 font-bold text-xs text-primary">"User Roster"</div>
                        <DataTable class="w-full text-xs">
                            <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Name"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Role"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Email Address"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Last Active"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"MFA Security"</DataTableHead>
                                    <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border">
                                <For
                                    each=move || roster_users.get_value()
                                    key=|u| u.email.clone()
                                    children=move |u| {
                                        let u_email = u.email.clone();
                                        view! {
                                            <DataTableRow>
                                                <DataTableCell class="p-3 font-bold">{u.name.clone()}</DataTableCell>
                                                <DataTableCell class="p-3">
                                                    <span class="px-2 py-0.5 rounded bg-surface-container border border-outline-variant/20 font-semibold text-[10px]">
                                                        {u.role.clone()}
                                                    </span>
                                                </DataTableCell>
                                                <DataTableCell class="p-3 font-mono text-on-surface-variant/80">{u.email.clone()}</DataTableCell>
                                                <DataTableCell class="p-3 text-on-surface-variant">{u.last_active.clone()}</DataTableCell>
                                                <DataTableCell class="p-3 text-emerald-400">"● " {u.mfa.clone()}</DataTableCell>
                                                <DataTableCell class="p-3 text-right">
                                                    <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click={
                                                        let email = u_email.clone();
                                                        move |_| toast.show_toast("Edit User", &format!("Editing details for {}", email), "info")
                                                    }>
                                                        "Edit"
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

            // ── TAB CONTENT: Invoice History ──
            <Show when=move || active_tab.get() == "invoices">
                <Card class="bg-card border-border shadow-sm overflow-hidden".to_string()>
                    <DataTable class="w-full text-xs">
                        <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Invoice ID"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Billing Period"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Base Fee"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Seats Overage"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Commission Share"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Issued Credits"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Total Due"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Status"</DataTableHead>
                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Vault Receipt"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            <For
                                each=move || mock_invoices.get_value()
                                key=|inv| inv.0
                                children=move |inv| {
                                    let inv_id = inv.0;
                                    view! {
                                        <DataTableRow>
                                            <DataTableCell class="p-3 font-mono font-bold text-primary">{inv.0}</DataTableCell>
                                            <DataTableCell class="p-3">{inv.1}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono">{inv.2}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono text-on-surface-variant/80">{inv.3}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono">{inv.4}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono text-emerald-400">{inv.5}</DataTableCell>
                                            <DataTableCell class="p-3 text-right font-mono font-bold">{inv.6}</DataTableCell>
                                            <DataTableCell class="p-3 text-center">
                                                <Badge intent=BadgeIntent::Success>{inv.7}</Badge>
                                            </DataTableCell>
                                            <DataTableCell class="p-3 text-right">
                                                <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm class="h-7 px-2 text-xs".to_string() on:click={
                                                    let id = inv_id;
                                                    move |_| toast.show_toast("Vault Reference", &format!("Downloading invoice PDF file: {}.pdf", id), "info")
                                                }>
                                                    "PDF"
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

            // ── TAB CONTENT: Database Ledger ──
            <Show when=move || active_tab.get() == "ledger">
                <Card class="bg-card border-border shadow-sm overflow-hidden".to_string()>
                    <div class="px-5 py-4 bg-surface-container/30 border-b border-outline-variant/10 flex justify-between items-center">
                        <div>
                            <span class="font-semibold text-sm">"Real-Time Database Ledger Records"</span>
                            <p class="text-[10px] text-on-surface-variant">"Direct transactional ledger mappings loaded live from target PostgreSQL nodes"</p>
                        </div>
                        <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on:click=move |_| set_trigger_fetch.update(|v| *v += 1)>
                            "Refresh Ledger"
                        </Button>
                    </div>
                    
                    <Suspense fallback=move || view! { <div class="p-6 text-center text-xs text-on-surface-variant">"Loading PostgreSQL ledger..."</div> }>
                        {move || {
                            let txs = ledger_res.get().unwrap_or_default();
                            if txs.is_empty() {
                                view! {
                                    <div class="p-8 text-center text-xs text-on-surface-variant/70 bg-[#06122d]/10">
                                        "No transaction ledger instances mapped in database. Showing mock seed transactions."
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <DataTable class="w-full text-xs">
                                        <DataTableHeader class="bg-surface-container-highest border-b border-outline-variant/30">
                                            <DataTableRow class="hover:bg-transparent">
                                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Transaction ID"</DataTableHead>
                                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Gateway Provider"</DataTableHead>
                                                <DataTableHead class="h-8 px-4 text-right font-medium text-on-surface-variant">"Amount (cents)"</DataTableHead>
                                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Currency"</DataTableHead>
                                                <DataTableHead class="h-8 px-4 text-left font-medium text-on-surface-variant">"Gateway TX ID"</DataTableHead>
                                                <DataTableHead class="h-8 px-4 text-center font-medium text-on-surface-variant">"Status"</DataTableHead>
                                            </DataTableRow>
                                        </DataTableHeader>
                                        <DataTableBody class="divide-y divide-border">
                                            <For
                                                each=move || txs.clone()
                                                key=|tx| tx.id
                                                children=move |tx| {
                                                    view! {
                                                        <DataTableRow>
                                                            <DataTableCell class="p-3 font-mono text-[10.5px] text-primary">{tx.id.to_string()}</DataTableCell>
                                                            <DataTableCell class="p-3 font-semibold">{tx.provider.clone()}</DataTableCell>
                                                            <DataTableCell class="p-3 text-right font-mono">{tx.amount}</DataTableCell>
                                                            <DataTableCell class="p-3 font-mono">{tx.currency.clone()}</DataTableCell>
                                                            <DataTableCell class="p-3 font-mono text-on-surface-variant/80">{tx.provider_tx_id.clone().unwrap_or_else(|| "—".to_string())}</DataTableCell>
                                                            <DataTableCell class="p-3 text-center">
                                                                <Badge intent=if tx.status == "completed" { BadgeIntent::Success } else { BadgeIntent::Default }>
                                                                    {tx.status.clone()}
                                                                </Badge>
                                                            </DataTableCell>
                                                        </DataTableRow>
                                                    }
                                                }
                                            />
                                        </DataTableBody>
                                    </DataTable>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </Card>
            </Show>

            // ── TAB CONTENT: Features ──
            <Show when=move || active_tab.get() == "features">
                <Card class="bg-card border-border shadow-sm p-6".to_string()>
                    <h3 class="text-sm font-bold uppercase tracking-wider text-primary mb-4">"Provisioned Platform Features"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
                        {[
                            ("G-01: Spatial Service Areas", "PostGIS dynamic location bounding boxes and mapping queries.", true),
                            ("G-02: Private Vault (R2)", "Secure tenant file attachments with short-lived pre-signed URLs.", true),
                            ("G-04: Hybrid Subscription Engine", "Flat base platform pricing + user-overage seat calculations.", true),
                            ("G-11: Contracts & SLA", "Vault-isolated contract PDFs, signed dates, and renewal periods.", true),
                            ("G-16: Regulatory Registry", "Short-term rental certificates, municipal license check loops.", true),
                            ("G-27: Contributor Calibration", "Scorecard ratings pipelines, time-series aggregation.", true),
                            ("G-31: High-Ticket Leads", "Waitlist submissions, city-based target parameters.", true),
                            ("G-33: Custom Domain Matrix", "CNAME domain overrides, dynamic Ingress route sidecar.", true),
                        ].into_iter().map(|(title, desc, active)| view! {
                            <div class="flex items-start gap-3 p-4 bg-surface-container-high/40 rounded-xl border border-outline-variant/10">
                                <span class=format!("material-symbols-outlined text-base mt-0.5 {}", if active { "text-emerald-400" } else { "text-on-surface-variant/40" })>
                                    {if active { "check_circle" } else { "cancel" }}
                                </span>
                                <div>
                                    <div class="font-bold text-on-surface">{title}</div>
                                    <p class="text-[10px] text-on-surface-variant/80 mt-1 leading-relaxed">{desc}</p>
                                </div>
                            </div>
                        }).collect_view()}
                    </div>
                </Card>
            </Show>

            // ── MODAL: Issue Credit ──
            <Show when=move || show_credit_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_credit_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Issue Custom Billing Credit"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Apply a manual ledger deduction adjustment to the tenant's current MRR cycle balance."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Credit Amount ($)"</Label>
                                <Input r#type=InputType::Text class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string() bind_value=credit_amount placeholder="e.g. 250.00".to_string() />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Reason / Adjustment Basis"</Label>
                                <Input r#type=InputType::Text class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string() bind_value=credit_reason placeholder="e.g. Support SLA SLA compensation".to_string() />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_credit_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_issue_credit>"Issue Credit"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── MODAL: Send Invoice ──
            <Show when=move || show_invoice_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_invoice_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Generate Manual Invoice"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Dispatched direct invoice request containing flat fees or custom platform transaction totals."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Invoice Total Due ($)"</Label>
                                <Input r#type=InputType::Text class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string() bind_value=invoice_amount placeholder="e.g. 4800.00".to_string() />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Billing Period"</Label>
                                <Input r#type=InputType::Text class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string() bind_value=invoice_period placeholder="e.g. Jun 2026".to_string() />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_invoice_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_send_invoice>"Send Invoice"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── MODAL: Change Plan ──
            <Show when=move || show_plan_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_plan_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Modify Subscription Plan Tier"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Select a new template pricing model list loaded from platform database nodes."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Active Plan Tier"</Label>
                                <select 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| selected_plan_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select pricing plan..."</option>
                                    <Suspense fallback=move || view! { <option>"Loading plans..."</option> }>
                                        {move || plans_res.get().map(|plans| view! {
                                            <For
                                                each=move || plans.clone()
                                                key=|p| p.id
                                                children=move |p| {
                                                    view! {
                                                        <option value=p.id.to_string()>{p.name.clone()} " ($" {p.base_price_cents / 100} "/mo)"</option>
                                                    }
                                                }
                                            />
                                        })}
                                    </Suspense>
                                </select>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_plan_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_change_plan>"Update Tier Plan"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── MODAL: Invite User ──
            <Show when=move || show_invite_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_invite_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Invite Team Member"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Invite an operator to join Nexus Property Group's active roster."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <Label>"Full Name"</Label>
                                <Input r#type=InputType::Text class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string() bind_value=invite_name placeholder="e.g. Ruud Salym".to_string() />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Email Address"</Label>
                                <Input r#type=InputType::Text class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary".to_string() bind_value=invite_email placeholder="e.g. ruud@nexusprops.com".to_string() />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label>"Roster Role"</Label>
                                <select 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| invite_role.set(event_target_value(&ev))
                                >
                                    <option value="Admin">"Admin"</option>
                                    <option value="Property Manager">"Property Manager"</option>
                                    <option value="Leasing Agent">"Leasing Agent"</option>
                                    <option value="Maintenance Coordinator">"Maintenance Coordinator"</option>
                                    <option value="Read-Only Auditor">"Read-Only Auditor"</option>
                                </select>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Ghost on:click=move |_| show_invite_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_invite_user>"Dispatch Invite"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
