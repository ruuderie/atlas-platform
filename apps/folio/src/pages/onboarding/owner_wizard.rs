// apps/folio/src/pages/onboarding/owner_wizard.rs
//
// OwnerWizard — /onboard/owner
//
// Steps mirroring wiz_owner_onboard/code.html:
//   1. Identity & Profile
//   2. Review Your Portfolio
//   3. Statement Preferences
//   4. Grow your network (optional)
//
// Invite code support: ?code= pre-populates the context panel.
// Email OTP is handled by WizardShell as a pre-auth gate.

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;
use leptos::prelude::*;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "identity",
        label: "Identity & Profile",
        skippable: false,
    },
    WizardStepDesc {
        id: "portfolio",
        label: "Review Your Portfolio",
        skippable: false,
    },
    WizardStepDesc {
        id: "statements",
        label: "Statement Preferences",
        skippable: false,
    },
    WizardStepDesc {
        id: "network",
        label: "Grow your network",
        skippable: true,
    },
];

#[component]
pub fn OwnerWizard() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| {
        if let Some(Ok(r)) = code_resource.get() {
            invite_sig.set(r);
        }
    });

    let current_idx = RwSignal::new(0usize);
    let total = STEPS.len();
    let is_last = Signal::derive(move || current_idx.get() == total - 1);
    let next_label = Signal::derive(move || {
        if is_last.get() {
            "Go to Owner Portal"
        } else {
            "Continue"
        }
    });

    // Identity
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let ownership = RwSignal::new("individual".to_string());
    let entity_name = RwSignal::new(String::new());
    let tax_id = RwSignal::new(String::new());

    // Portfolio / approvals
    let approval_threshold = RwSignal::new("1000".to_string());

    // Statements
    let dist_method = RwSignal::new("ach".to_string());
    let stmt_freq = RwSignal::new("monthly_1st".to_string());
    let stmt_format = RwSignal::new("pdf_email".to_string());
    let acct_email = RwSignal::new(String::new());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/o".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_) => nav("/o", Default::default()),
                }
            });
        } else {
            current_idx.set(idx + 1);
        }
    });
    let on_prev = Callback::new(move |_| {
        let i = current_idx.get();
        if i > 0 {
            current_idx.set(i - 1);
        }
    });

    let ctx_body = ViewFn::from(|| {
        view! {
            <p class="wiz-ctx-p">"Your owner portal gives you read-only visibility into your portfolio managed by your property manager."</p>
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Monthly owner statements"</li>
                <li><span class="ms msf">"check_circle"</span>"Real-time occupancy & income"</li>
                <li><span class="ms msf">"check_circle"</span>"Maintenance approval requests"</li>
                <li><span class="ms msf">"check_circle"</span>"Distribution schedule and payment history"</li>
            </ul>
        }
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Owner"
            persona_icon="account_balance" accent_color="#7c3aed" panel_bg="#140f20"
            ctx_headline="Welcome to your Owner Portal" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            // ── Step 1: Identity & Profile ──────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Identity & Profile"</h1>
                    <p class="wiz-s-sub">"Basic info for your owner account. Your property manager will see this when communicating with you."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Robert"
                                    prop:value=move || first_name.get()
                                    on:input=move |e| first_name.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Chen"
                                    prop:value=move || last_name.get()
                                    on:input=move |e| last_name.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Display Name (shown to your PM)"</label>
                            <input class="wiz-inp" type="text" placeholder="Robert Chen or RC Holdings LLC"
                                prop:value=move || display_name.get()
                                on:input=move |e| display_name.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Phone"</label>
                            <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                prop:value=move || phone.get()
                                on:input=move |e| phone.set(event_target_value(&e))/></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Ownership Structure"</div>
                        <div class="wiz-f"><label class="wiz-label">"Ownership Type"</label>
                            <select class="wiz-inp"
                                prop:value=move || ownership.get()
                                on:change=move |e| ownership.set(event_target_value(&e))>
                                <option value="individual">"Individual"</option>
                                <option value="llc">"LLC"</option>
                                <option value="trust">"Trust"</option>
                                <option value="corporation">"Corporation"</option>
                                <option value="partnership">"Partnership"</option>
                            </select></div>
                        <div class="wiz-f"><label class="wiz-label">"Entity Name (if applicable)"</label>
                            <input class="wiz-inp" type="text" placeholder="RC Holdings LLC"
                                prop:value=move || entity_name.get()
                                on:input=move |e| entity_name.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Tax ID (EIN or SSN)"</label>
                            <input class="wiz-inp" type="text" placeholder="XX-XXXXXXX"
                                prop:value=move || tax_id.get()
                                on:input=move |e| tax_id.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Review Your Portfolio ───────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"apartment"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Your Portfolio"</h1>
                    <p class="wiz-s-sub">"Review the properties your manager has linked to your owner account."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Portfolio Summary"</div>
                        <div style="display:grid; grid-template-columns:repeat(2,1fr); gap:12px;">
                            <div style="padding:14px; background:#f4f5f9; border-radius:8px;">
                                <div style="font-size:11px; color:#64748b; font-weight:600; text-transform:uppercase; letter-spacing:.06em;">"Properties"</div>
                                <div style="font-size:22px; font-weight:800; margin-top:4px;">
                                    {move || invite_sig.get().and_then(|c| c.context.asset_count).unwrap_or(0).to_string()}
                                </div>
                            </div>
                            <div style="padding:14px; background:#f4f5f9; border-radius:8px;">
                                <div style="font-size:11px; color:#64748b; font-weight:600; text-transform:uppercase; letter-spacing:.06em;">"Units"</div>
                                <div style="font-size:22px; font-weight:800; margin-top:4px;">"—"</div>
                            </div>
                            <div style="padding:14px; background:#f4f5f9; border-radius:8px;">
                                <div style="font-size:11px; color:#64748b; font-weight:600; text-transform:uppercase; letter-spacing:.06em;">"Monthly Revenue"</div>
                                <div style="font-size:22px; font-weight:800; margin-top:4px;">"—"</div>
                            </div>
                            <div style="padding:14px; background:#f4f5f9; border-radius:8px;">
                                <div style="font-size:11px; color:#64748b; font-weight:600; text-transform:uppercase; letter-spacing:.06em;">"Occupancy Rate"</div>
                                <div style="font-size:22px; font-weight:800; margin-top:4px;">"—"</div>
                            </div>
                        </div>
                        <p style="font-size:12px; color:#94a3b8; margin-top:12px;">
                            "Full property details appear after activation. Your PM can add or remove assets anytime."
                        </p>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Maintenance Approval Threshold"</div>
                        <p style="font-size:13px; color:#64748b; margin-bottom:14px;">
                            "Repairs above this amount require your approval before your PM can dispatch."
                        </p>
                        <div class="wiz-f"><label class="wiz-label">"Approval Threshold"</label>
                            <select class="wiz-inp"
                                prop:value=move || approval_threshold.get()
                                on:change=move |e| approval_threshold.set(event_target_value(&e))>
                                <option value="250">"$250"</option>
                                <option value="500">"$500"</option>
                                <option value="1000">"$1,000"</option>
                                <option value="2500">"$2,500"</option>
                                <option value="5000">"$5,000"</option>
                            </select></div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Statement Preferences ───────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"receipt_long"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Statement Preferences"</h1>
                    <p class="wiz-s-sub">"Choose how you want to receive owner statements and distributions."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Distribution Method"</div>
                        <div class="wiz-og wiz-og3">
                            <button type="button"
                                class=move || if dist_method.get() == "ach" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| dist_method.set("ach".into())>
                                <span class="ms msf">"account_balance"</span>
                                <div class="wiz-oc-label">"ACH Bank Transfer"</div>
                                <div class="wiz-oc-desc">"Direct deposit to your bank"</div>
                            </button>
                            <button type="button"
                                class=move || if dist_method.get() == "wire" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| dist_method.set("wire".into())>
                                <span class="ms msf">"swap_horiz"</span>
                                <div class="wiz-oc-label">"Wire Transfer"</div>
                                <div class="wiz-oc-desc">"Same-day wire available"</div>
                            </button>
                            <button type="button"
                                class=move || if dist_method.get() == "crypto" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| dist_method.set("crypto".into())>
                                <span class="ms msf">"currency_bitcoin"</span>
                                <div class="wiz-oc-label">"Cryptocurrency"</div>
                                <div class="wiz-oc-desc">"USDC or BTC payout"</div>
                            </button>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Statement Schedule"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Statement Frequency"</label>
                                <select class="wiz-inp"
                                    prop:value=move || stmt_freq.get()
                                    on:change=move |e| stmt_freq.set(event_target_value(&e))>
                                    <option value="monthly_1st">"Monthly (1st of each month)"</option>
                                    <option value="monthly_custom">"Monthly (custom date)"</option>
                                    <option value="quarterly">"Quarterly"</option>
                                </select></div>
                            <div class="wiz-f"><label class="wiz-label">"Preferred Format"</label>
                                <select class="wiz-inp"
                                    prop:value=move || stmt_format.get()
                                    on:change=move |e| stmt_format.set(event_target_value(&e))>
                                    <option value="pdf_email">"PDF via email"</option>
                                    <option value="pdf_csv">"PDF + CSV"</option>
                                    <option value="in_app">"In-app only"</option>
                                </select></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Accounting Email (optional)"</label>
                            <input class="wiz-inp" type="email" placeholder="accounting@rchholdings.com"
                                prop:value=move || acct_email.get()
                                on:input=move |e| acct_email.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // ── Step 4: Grow your network ───────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"group_add"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Grow your network"</h1>
                    <p class="wiz-s-sub">"Share this portal with fellow investors, or introduce Folio to a self-managed landlord in your circle."</p>
                    {
                        use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                        view! {
                            <NetworkInvitePanel
                                actor_role="owner"
                                preferred_slug="owner_invite_peers"
                                angles=vec![
                                    AngleCard {
                                        icon: "star",
                                        title: "Other managed owners",
                                        body: "Fellow investors get the same visibility into statements and approvals.",
                                        benefit_icon: None,
                                        benefit_label: None,
                                    },
                                    AngleCard {
                                        icon: "apartment",
                                        title: "Self-managed landlords",
                                        body: "Share Folio with landlords who still track rent in spreadsheets.",
                                        benefit_icon: None,
                                        benefit_label: None,
                                    },
                                ]
                                section_title="Who to invite".to_string()
                                send_label="Send invite".to_string()
                                allow_multi=true
                                footnote="They will get a personal invite. You can invite more anytime from your portal.".to_string()
                                show_history=false
                            />
                        }
                    }
                </div>
            </Show>
        </WizardShell>
    }
}
