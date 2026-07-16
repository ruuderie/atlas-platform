use crate::api::admin::{
    CampaignModel, CreateCampaignInput, campaign_export_url, create_campaign, get_campaign,
    list_campaign_members, list_campaign_referrers, list_campaigns,
};
use crate::api::products::get_products;
use crate::components::gtm_process_strip::{GtmProcessStrip, GtmStage};
/// # Campaigns — Go-to-Market Command Center
///
/// Route: /campaigns           → campaign list
/// Route: /campaigns/:id       → campaign detail (members, funnel, landing pages)
///
/// Marketing Architect view:
///   Campaign → UTM URL → Landing Page → Lead Capture → Attribution → Conversion
///
/// The `utm_campaign` field is the connective tissue between a campaign record
/// and the landing page variant that receives its traffic.
use leptos::prelude::*;

// ── helpers ──────────────────────────────────────────────────────────────────

fn campaign_type_label(t: &str) -> &'static str {
    match t {
        "direct_mail" => "Direct Mail",
        "cold_email" => "Cold Email",
        "ppc" => "Paid Search",
        "social" => "Social",
        "event_based" => "Event",
        "sms" => "SMS",
        "content" => "Content",
        "referral" => "Referral",
        "retargeting" => "Retargeting",
        _ => "Campaign",
    }
}

fn type_color_class(t: &str) -> &'static str {
    match t {
        "direct_mail" => "bg-amber-500/15 border-amber-500/30 text-amber-300",
        "cold_email" => {
            "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)"
        }
        "ppc" => "bg-purple-500/15 border-purple-500/30 text-purple-300",
        "social" => "bg-pink-500/15 border-pink-500/30 text-pink-300",
        "event_based" => "bg-emerald-500/15 border-emerald-500/30 text-emerald-300",
        _ => "bg-outline-variant/20 border-outline-variant/30 text-on-surface-variant",
    }
}

fn status_dot(s: &str) -> &'static str {
    match s {
        "active" => "text-emerald-400",
        "scheduled" => "text-amber-400",
        "paused" => "text-orange-400",
        "completed" => "text-on-surface-variant",
        "draft" => "text-on-surface-variant",
        _ => "text-on-surface-variant",
    }
}

fn fmt_money(cents: i64) -> String {
    if cents == 0 {
        return "$0".to_string();
    }
    let dollars = cents / 100;
    if dollars >= 1_000_000 {
        return format!("${:.1}M", dollars as f64 / 1_000_000.0);
    }
    if dollars >= 1_000 {
        return format!("${:.0}k", dollars as f64 / 1_000.0);
    }
    format!("${}", dollars)
}

fn conv_rate(conversions: i32, total: i32) -> String {
    if total == 0 {
        return "0%".to_string();
    }
    format!("{:.0}%", conversions as f64 / total as f64 * 100.0)
}

fn cac(spent_cents: i64, conversions: i32) -> String {
    if conversions == 0 {
        return "—".to_string();
    }
    fmt_money(spent_cents / conversions as i64)
}

// ── Campaigns List Page ───────────────────────────────────────────────────────

#[component]
pub fn CampaignsPage() -> impl IntoView {
    let campaigns_version = RwSignal::new(0u32);
    let campaigns = LocalResource::new(move || async move {
        let _ = campaigns_version.get();
        list_campaigns().await
    });

    let show_new_modal = RwSignal::new(false);
    let navigate = leptos_router::hooks::use_navigate();

    // new campaign form signals
    let new_name = RwSignal::new(String::new());
    let new_type = RwSignal::new("direct_mail".to_string());
    let new_goal = RwSignal::new("lead_capture".to_string());
    let new_budget = RwSignal::new(String::new());
    let new_utm_src = RwSignal::new("manual".to_string());
    let new_utm_med = RwSignal::new("direct_mail".to_string());
    let new_utm_cmp = RwSignal::new(String::new());
    let new_provider = RwSignal::new("dm_manual".to_string());
    let new_lp_path = RwSignal::new("/lp/miami-landlords".to_string());
    let create_error = RwSignal::new(String::new());

    let create_action = Action::new_local(move |_: &()| {
        let name = new_name.get();
        let budget = new_budget.get().parse::<i64>().ok().map(|d| d * 100);
        let utm_medium = if new_type.get() == "direct_mail" {
            Some("direct_mail".to_string())
        } else {
            Some(new_utm_med.get()).filter(|s| !s.is_empty())
        };
        let input = CreateCampaignInput {
            name: name.clone(),
            campaign_type: new_type.get(),
            tenant_id: uuid::Uuid::nil(),
            goal_type: Some(new_goal.get()),
            budget_cents: budget,
            utm_source: Some(new_utm_src.get()).filter(|s| !s.is_empty()),
            utm_medium,
            utm_campaign: Some(new_utm_cmp.get()).filter(|s| !s.is_empty()),
            starts_at: None,
            ends_at: None,
        };
        let _provider = new_provider.get();
        let _lp = new_lp_path.get();
        let nav = navigate.clone();
        async move {
            if name.is_empty() {
                create_error.set("Campaign name is required.".into());
                return;
            }
            create_error.set(String::new());
            match create_campaign(input).await {
                Ok(created) => {
                    show_new_modal.set(false);
                    campaigns_version.update(|n| *n += 1);
                    new_name.set(String::new());
                    new_utm_cmp.set(String::new());
                    new_budget.set(String::new());
                    nav(
                        &format!("/campaigns/{}", created.id),
                        leptos_router::NavigateOptions::default(),
                    );
                }
                Err(e) => create_error.set(e),
            }
        }
    });

    view! {
        <div class="main-canvas">

            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Campaigns"</h1>
                    <p class="page-subtitle">
                        "Manage outreach campaigns. Each campaign connects to a landing page via its UTM slug — "
                        "giving you full funnel visibility from postcard to client."
                    </p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-primary" on:click=move |_| show_new_modal.set(true)>
                        "+ New Campaign"
                    </button>
                </div>
            </div>

            <GtmProcessStrip
                active=GtmStage::Campaigns
                subtitle="Coordinate outbound, paid, and content campaigns against acquisition pages."
            />

            // ── Funnel Explainer Banner ──────────────────────────────────────
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-5 py-4 flex items-center gap-6 text-xs text-on-surface-variant overflow-x-auto">
                <div class="flex items-center gap-2 shrink-0">
                    <span class="w-6 h-6 rounded-full bg-amber-500/20 border border-amber-500/30 flex items-center justify-center text-amber-400 font-bold text-[10px]">"1"</span>
                    <span class="font-semibold text-on-surface">"Campaign"</span>
                    <span>"Direct mail / email / paid"</span>
                </div>
                <svg class="w-4 h-4 text-outline-variant/50 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor"><path d="M4 8h8M9 5l3 3-3 3"/></svg>
                <div class="flex items-center gap-2 shrink-0">
                    <span class="plan-badge" style="color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)">"2"</span>
                    <span class="font-semibold text-on-surface">"Landing Page"</span>
                    <span>"Linked by utm_campaign slug"</span>
                </div>
                <svg class="w-4 h-4 text-outline-variant/50 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor"><path d="M4 8h8M9 5l3 3-3 3"/></svg>
                <div class="flex items-center gap-2 shrink-0">
                    <span class="w-6 h-6 rounded-full bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400 font-bold text-[10px]">"3"</span>
                    <span class="font-semibold text-on-surface">"Lead Capture"</span>
                    <span>"Form submission → atlas_leads"</span>
                </div>
                <svg class="w-4 h-4 text-outline-variant/50 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor"><path d="M4 8h8M9 5l3 3-3 3"/></svg>
                <div class="flex items-center gap-2 shrink-0">
                    <span class="w-6 h-6 rounded-full bg-primary/20 border border-primary/30 flex items-center justify-center text-primary font-bold text-[10px]">"4"</span>
                    <span class="font-semibold text-on-surface">"Conversion"</span>
                    <span>"Subscriber → MRR"</span>
                </div>
            </div>

            // ── Campaign Cards ───────────────────────────────────────────────
            <Suspense fallback=|| view! {
                <div class="text-sm text-on-surface-variant/60 animate-pulse py-8 text-center">"Loading campaigns..."</div>
            }>
                {move || campaigns.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="bg-surface-container-low border border-error/30 rounded-xl p-8 text-center space-y-3">
                            <svg class="w-10 h-10 text-error/40 mx-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
                                <circle cx="12" cy="12" r="10"/>
                                <line x1="12" y1="8" x2="12" y2="12"/>
                                <circle cx="12" cy="16" r="0.5" fill="currentColor"/>
                            </svg>
                            <p class="text-sm font-semibold text-on-surface">
                                "Failed to load campaigns"
                            </p>
                            <p class="text-xs text-error/80 font-mono bg-error/5 border border-error/15 rounded px-3 py-2 max-w-lg mx-auto text-left break-all">
                                {e.clone()}
                            </p>
                            <p class="text-xs text-on-surface-variant/60 max-w-xs mx-auto">
                                "This is usually a backend or database configuration issue. Contact your platform administrator if this persists."
                            </p>
                            <button
                                class="mt-2 btn btn-ghost btn-sm"
                                on:click=move |_| { let _ = campaigns.refetch(); }
                            >
                                "↺ Retry"
                            </button>
                        </div>
                    }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-12 text-center">
                            <svg class="w-12 h-12 text-on-surface-variant/20 mx-auto mb-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M3 3h18v14H3zM7 21h10M12 17v4"/></svg>
                            <p class="text-sm font-semibold text-on-surface-variant">"No campaigns yet"</p>
                            <p class="text-xs text-on-surface-variant/60 mt-1 max-w-xs mx-auto">
                                "Create a campaign to start tracking outreach, members, and landing page conversions."
                            </p>
                            <button
                                class="mt-4 btn btn-primary"
                                on:click=move |_| show_new_modal.set(true)
                            >"+ New Campaign"</button>
                        </div>
                    }.into_any(),
                    Ok(list) => view! {
                        <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
                            {list.into_iter().map(|c| view! { <CampaignCard campaign=c /> }).collect_view()}
                        </div>
                    }.into_any(),
                })}
            </Suspense>

        </div>

        // ── New Campaign Modal ───────────────────────────────────────────────
        <Show when=move || show_new_modal.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                <div class="bg-surface-container border border-outline-variant/30 rounded-2xl shadow-2xl w-full max-w-2xl mx-4 overflow-hidden max-h-[90vh] flex flex-col">
                    <div class="px-6 py-4 border-b border-outline-variant/20 flex items-center justify-between shrink-0">
                        <div>
                            <h2 class="text-sm font-bold text-on-surface">"New Campaign"</h2>
                            <p class="text-[10px] text-on-surface-variant mt-0.5">
                                "Direct mail defaults on — unique utm_campaign + offer code before first stamp."
                            </p>
                        </div>
                        <button class="btn btn-ghost btn-icon btn-sm" on:click=move |_| show_new_modal.set(false)>
                            <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4l8 8M12 4l-8 8"/></svg>
                        </button>
                    </div>
                    <div class="px-6 py-5 space-y-4 overflow-y-auto">
                        <Show when=move || !create_error.get().is_empty()>
                            <p class="text-xs text-error bg-error/10 border border-error/20 rounded-lg px-3 py-2">{move || create_error.get()}</p>
                        </Show>
                        // Name
                        <div>
                            <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Campaign Name"</label>
                            <input
                                type="text" placeholder="e.g. Miami Q3 Landlord Postcard"
                                class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                on:input=move |ev| new_name.set(event_target_value(&ev))
                                prop:value=move || new_name.get()
                            />
                        </div>

                        // Type + Goal row
                        <div class="grid grid-cols-2 gap-3">
                            <div>
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Type"</label>
                                <select
                                    class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface focus:border-primary/60 outline-none"
                                    prop:value=move || new_type.get()
                                    on:change=move |ev| {
                                        let t = event_target_value(&ev);
                                        if t == "direct_mail" {
                                            new_utm_med.set("direct_mail".to_string());
                                            if new_utm_src.get().is_empty() {
                                                new_utm_src.set("manual".to_string());
                                            }
                                        }
                                        new_type.set(t);
                                    }
                                >
                                    <option value="direct_mail">"Direct Mail"</option>
                                    <option value="cold_email">"Cold Email"</option>
                                    <option value="ppc">"Paid Search"</option>
                                    <option value="social">"Social"</option>
                                    <option value="event_based">"Event"</option>
                                    <option value="sms">"SMS"</option>
                                    <option value="referral">"Referral"</option>
                                    <option value="retargeting">"Retargeting"</option>
                                </select>
                            </div>
                            <div>
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Goal"</label>
                                <select
                                    class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface focus:border-primary/60 outline-none"
                                    prop:value=move || new_goal.get()
                                    on:change=move |ev| new_goal.set(event_target_value(&ev))
                                >
                                    <option value="lead_capture">"Lead Capture"</option>
                                    <option value="demo_booking">"Demo / Meeting"</option>
                                    <option value="trial_signup">"Trial Signup"</option>
                                    <option value="paid_conversion">"Paid Conversion"</option>
                                    <option value="referral">"Referral"</option>
                                    <option value="retention">"Retention"</option>
                                </select>
                            </div>
                        </div>

                        <Show when=move || new_type.get() == "direct_mail">
                            <div class="space-y-3">
                                <div>
                                    <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Mail provider"</label>
                                    <select
                                        class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface focus:border-primary/60 outline-none"
                                        prop:value=move || new_provider.get()
                                        on:change=move |ev| new_provider.set(event_target_value(&ev))
                                    >
                                        <option value="dm_manual">"Manual CSV (ship now)"</option>
                                        <option value="dm_lob" disabled>"Lob (coming soon)"</option>
                                        <option value="dm_property_radar" disabled>"PropertyRadar (coming soon)"</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Landing page path"</label>
                                    <input type="text"
                                        class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface font-mono focus:border-primary/60 outline-none"
                                        prop:value=move || new_lp_path.get()
                                        on:input=move |ev| new_lp_path.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="bg-amber-500/10 border border-amber-500/25 rounded-lg px-3 py-2.5 text-[11px] text-on-surface-variant leading-relaxed">
                                    <span class="font-semibold text-amber-300">"Before you mail: "</span>
                                    "unique utm_campaign · per-drop utm_content · offer code on piece · QR → tracked LP · record spend on invoice · GA4/Meta pixels live."
                                </div>
                            </div>
                        </Show>

                        // UTM section
                        <div class="bg-surface-container-high/30 rounded-lg p-3 space-y-2">
                            <p class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-2">
                                "UTM Tracking — links campaign to landing page"
                            </p>
                            <div class="grid grid-cols-3 gap-2">
                                <div>
                                    <label class="block text-[10px] text-on-surface-variant/70 mb-1">"utm_source"</label>
                                    <input type="text" placeholder="manual"
                                        class="w-full bg-surface-container-low border border-outline-variant/30 rounded px-2 py-1.5 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                        prop:value=move || new_utm_src.get()
                                        on:input=move |ev| new_utm_src.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="block text-[10px] text-on-surface-variant/70 mb-1">"utm_medium"</label>
                                    <input type="text" placeholder="direct_mail"
                                        class="w-full bg-surface-container-low border border-outline-variant/30 rounded px-2 py-1.5 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                        prop:value=move || new_utm_med.get()
                                        prop:readonly=move || new_type.get() == "direct_mail"
                                        on:input=move |ev| new_utm_med.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="block text-[10px] text-on-surface-variant/70 mb-1">"utm_campaign"</label>
                                    <input type="text" placeholder="miami_q3_dm"
                                        class="w-full bg-surface-container-low border border-outline-variant/30 rounded px-2 py-1.5 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                        prop:value=move || new_utm_cmp.get()
                                        on:input=move |ev| new_utm_cmp.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <p class="text-[10px] text-on-surface-variant/50 font-mono break-all">
                                {move || format!(
                                    "{}?utm_source={}&utm_medium={}&utm_campaign={}",
                                    new_lp_path.get(),
                                    new_utm_src.get(),
                                    new_utm_med.get(),
                                    new_utm_cmp.get()
                                )}
                            </p>
                        </div>

                        // Budget
                        <div>
                            <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Budget (USD)"</label>
                            <input type="number" placeholder="2500"
                                class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                prop:value=move || new_budget.get()
                                on:input=move |ev| new_budget.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <div class="px-6 py-4 border-t border-outline-variant/20 flex justify-end gap-3 shrink-0">
                        <button class="btn btn-ghost"
                            on:click=move |_| show_new_modal.set(false)>"Cancel"</button>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| { create_action.dispatch(()); }
                        >{move || if new_type.get() == "direct_mail" { "Create & add first drop" } else { "Create Campaign" }}</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// ── Campaign Card ─────────────────────────────────────────────────────────────

#[component]
fn CampaignCard(campaign: CampaignModel) -> impl IntoView {
    let id = campaign.id;
    let budget_str = campaign
        .budget_cents
        .map(|b| fmt_money(b))
        .unwrap_or("—".to_string());
    let spent_str = fmt_money(campaign.spent_cents);
    let cac_str = cac(campaign.spent_cents, campaign.total_conversions);
    let conv_rate_str = conv_rate(campaign.total_conversions, campaign.total_contacts);
    let type_label = campaign_type_label(&campaign.campaign_type).to_string();
    let type_class = type_color_class(&campaign.campaign_type).to_string();
    let status_class = status_dot(&campaign.status).to_string();
    let budget_pct = campaign
        .budget_cents
        .filter(|&b| b > 0)
        .map(|b| ((campaign.spent_cents as f64 / b as f64) * 100.0).min(100.0) as u32)
        .unwrap_or(0);

    let utm_campaign = campaign.utm_campaign.clone().unwrap_or_default();
    let global_name = campaign.global_name.clone();

    view! {
        <a href=format!("/campaigns/{}", id)
            class="block bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden hover:border-outline-variant/50 hover:shadow-lg transition-all duration-200 group"
        >
            // Header bar
            <div class="px-5 py-3.5 border-b border-outline-variant/15 bg-surface-container-high/20 flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase border {}", type_class)>{type_label}</span>
                    <span class=format!("text-[10px] font-semibold {}", status_class)>
                        {format!("● {}", campaign.status)}
                    </span>
                </div>
                <svg class="w-3.5 h-3.5 text-on-surface-variant/30 group-hover:text-on-surface-variant transition-colors"
                    viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M6 4l4 4-4 4"/>
                </svg>
            </div>

            // Campaign name + global_name + UTM
            <div class="px-5 pt-4 pb-3">
                <h3 class="text-sm font-bold text-on-surface leading-tight group-hover:text-primary transition-colors">
                    {campaign.name}
                </h3>
                {if !global_name.is_empty() {
                    view! {
                        <p class="text-[10px] text-on-surface-variant mt-0.5 font-mono">{global_name}</p>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
                {if utm_campaign.is_empty() {
                    view! { <p class="text-[10px] mt-0.5 font-mono italic" style="color:var(--amber)">"No UTM slug — not linked to landing page"</p> }.into_any()
                } else {
                    view! {
                        <p class="text-[10px] text-primary mt-0.5 font-mono">
                            {format!("utm_campaign={}", utm_campaign)}
                        </p>
                    }.into_any()
                }}
            </div>

            // KPI grid
            <div class="px-5 pb-4 grid grid-cols-4 gap-3 text-center">
                <div>
                    <div class="text-sm font-bold font-mono text-on-surface">
                        {campaign.total_contacts}
                    </div>
                    <div class="text-[9px] uppercase tracking-wider text-on-surface-variant mt-0.5">"Members"</div>
                </div>
                <div>
                    <div class="text-sm font-bold font-mono text-emerald-400">
                        {campaign.total_conversions}
                    </div>
                    <div class="text-[9px] uppercase tracking-wider text-on-surface-variant mt-0.5">"Converted"</div>
                </div>
                <div class="flex flex-col gap-1">
                    <div class="text-sm font-bold font-mono text-primary">
                        {conv_rate_str.clone()}
                    </div>
                    <div class="text-[9px] uppercase tracking-wider text-on-surface-variant mt-0.5">"Conv. Rate"</div>
                </div>
                <div class="flex flex-col gap-1">
                    <div class=format!("text-sm font-bold font-mono {}",
                        if cac_str == "—" { "text-on-surface-variant" } else { "text-amber-400" })>
                        {cac_str.clone()}
                    </div>
                    <div class="text-[9px] uppercase tracking-wider text-on-surface-variant mt-0.5">"CAC"</div>
                </div>
            </div>

            // Budget progress bar
            <div class="px-5 pb-4">
                <div class="flex justify-between text-[9px] text-on-surface-variant mb-1.5">
                    <span>{format!("Spent: {}", spent_str)}</span>
                    <span>{format!("Budget: {}", budget_str)}</span>
                </div>
                <div class="h-1 bg-surface-container-high/40 rounded-full overflow-hidden">
                    <div
                        class=format!("h-full rounded-full transition-all {}", if budget_pct >= 90 { "bg-error" } else { "bg-primary" })
                        style=format!("width: {}%", budget_pct)
                    />
                </div>
            </div>
        </a>
    }
}

// ── Campaign Detail Page ──────────────────────────────────────────────────────

#[component]
pub fn CampaignDetail() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let campaign_id =
        move || params.with(|p| p.get("id").and_then(|s| s.parse::<uuid::Uuid>().ok()));

    let campaign_res = LocalResource::new(move || async move {
        match campaign_id() {
            Some(id) => get_campaign(id).await.ok(),
            None => None,
        }
    });

    let active_tab = RwSignal::<String>::new("overview".to_string());

    view! {
        <Suspense fallback=|| view! {
            <div class="p-12 text-center text-on-surface-variant/60 animate-pulse text-sm">"Loading..."</div>
        }>
            {move || campaign_res.get().map(|opt| match opt {
                None => view! {
                    <div class="p-12 text-center text-on-surface-variant/60 text-sm">"Campaign not found."</div>
                }.into_any(),
                Some(campaign) => {
                    let id = campaign.id;
                    let export_url = campaign_export_url(id);
                    let budget_str = campaign.budget_cents.map(|b| fmt_money(b)).unwrap_or("—".to_string());
                    let cac_str = cac(campaign.spent_cents, campaign.total_conversions);
                    let conv_str = conv_rate(campaign.total_conversions, campaign.total_contacts);
                    let utm_cmp = campaign.utm_campaign.clone().unwrap_or_default();
                    let type_label = campaign_type_label(&campaign.campaign_type).to_string();
                    let type_class = type_color_class(&campaign.campaign_type).to_string();
                    let status_class = status_dot(&campaign.status).to_string();
                    let campaign_name = campaign.name.clone();
                    let global_name = campaign.global_name.clone();
                    let utm_source = campaign.utm_source.clone().unwrap_or_default();
                    let utm_medium = campaign.utm_medium.clone().unwrap_or_default();
                    let utm_query = if utm_cmp.is_empty() {
                        None
                    } else {
                        Some(format!(
                            "?utm_source={}&utm_medium={}&utm_campaign={}",
                            utm_source, utm_medium, &utm_cmp
                        ))
                    };
                    let utm_for_referrers = utm_cmp.clone();
                    let utm_for_landing_pages = utm_cmp;
                    let global_name_display = global_name.clone();

                    view! {
                        <div class="space-y-6">

                            // ── Breadcrumb ───────────────────────────────────
                            <div class="flex items-center gap-2 text-xs text-on-surface-variant/60">
                                <a href="/campaigns" class="hover:text-on-surface transition-colors">"Campaigns"</a>
                                <span>"/"</span>
                                <span class="text-on-surface">{campaign_name.clone()}</span>
                            </div>

                            // ── Detail Header ────────────────────────────────
                            <div class="flex items-start justify-between flex-wrap gap-4">
                                <div class="flex items-center gap-3">
                                    <div>
                                        <div class="flex items-center gap-2 mb-1">
                                            <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase border {}", type_class)>{type_label}</span>
                                            <span class=format!("text-xs font-semibold {}", status_class)>
                                                {format!("● {}", campaign.status)}
                                            </span>
                                        </div>
                                        <h1 class="text-xl font-extrabold text-on-surface tracking-tight">{campaign_name.clone()}</h1>
                                        <p class="text-xs text-on-surface-variant mt-0.5 font-mono select-all">{global_name_display}</p>
                                        {match utm_query {
                                            None => view! { <p class="text-xs text-on-surface-variant mt-0.5 font-mono italic">"utm_campaign not set"</p> }.into_any(),
                                            Some(q) => view! {
                                                <p class="text-xs text-primary/70 mt-0.5 font-mono">{q}</p>
                                            }.into_any(),
                                        }}
                                    </div>
                                </div>
                                // Export button — opens CSV download
                                <a href=export_url target="_blank"
                                    class="btn btn-ghost"
                                    style="text-decoration:none"
                                >
                                    <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                        <path d="M8 2v8M5 7l3 3 3-3M2 12v1a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-1"/>
                                    </svg>
                                    "Export Members CSV"
                                </a>
                            </div>

                            // ── KPI Bar ──────────────────────────────────────
                            <div class="grid grid-cols-2 sm:grid-cols-4 xl:grid-cols-6 gap-3">
                                {[
                                    ("Members", campaign.total_contacts.to_string(), "text-on-surface"),
                                    ("Conversions", campaign.total_conversions.to_string(), "text-emerald-400"),
                                    ("Conv. Rate", conv_str.clone(), "text-primary"),
                                    ("CAC", cac_str.clone(), "text-amber-400"),
                                    ("Budget", budget_str.clone(), "text-on-surface"),
                                    ("Spent", fmt_money(campaign.spent_cents), "text-on-surface"),
                                ].iter().map(|(label, val, color)| {
                                    let val = val.clone();
                                    let color = color.to_string();
                                    let label = label.to_string();
                                    view! {
                                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                            <div class=format!("text-lg font-extrabold font-mono {}", color)>{val}</div>
                                            <div class="text-[9px] uppercase tracking-wider text-on-surface-variant/60 mt-0.5">{label}</div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>

                            // ── Tabs ─────────────────────────────────────────
                            <div class="tab-bar">
                                {[("overview", "Overview"), ("drops", "Drops"), ("spend", "Spend"), ("attribution", "Attribution"), ("referrers", "Referrers"), ("ambassadors", "Ambassadors"), ("members", "Members"), ("landing-pages", "Landing + QR"), ("programs", "Programs"), ("sequence", "Sequence")].iter().map(|(slug, label)| {
                                    let slug = slug.to_string();
                                    let label = label.to_string();
                                    let slug2 = slug.clone();
                                    view! {
                                        <button
                                            class=move || if active_tab.get() == slug {
                                                "tab active"
                                            } else {
                                                "tab"
                                            }
                                            on:click=move |_| active_tab.set(slug2.clone())
                                        >{label}</button>
                                    }
                                }).collect_view()}
                            </div>

                            // ── Tab: Overview ────────────────────────────────
                            <Show when=move || active_tab.get() == "overview">
                                <OverviewTab campaign_id=id />
                            </Show>

                            <Show when=move || active_tab.get() == "drops">
                                <DmDropsTab campaign_id=id />
                            </Show>
                            <Show when=move || active_tab.get() == "spend">
                                <DmSpendTab campaign_id=id />
                            </Show>
                            <Show when=move || active_tab.get() == "attribution">
                                <DmAttributionTab campaign_id=id />
                            </Show>

                            // ── Tab: Referrers ───────────────────────────────
                            <Show when=move || active_tab.get() == "referrers">
                                <ReferrersTab campaign_id=id utm_campaign=utm_for_referrers.clone() />
                            </Show>

                            // ── Tab: Ambassadors ─────────────────────────────
                            <Show when=move || active_tab.get() == "ambassadors">
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 space-y-3">
                                    <h3 class="text-sm font-bold text-on-surface">"Growth ambassadors"</h3>
                                    <p class="text-xs text-on-surface-variant max-w-lg">
                                        "Mint partner codes and dual QR card packs (landlord + vendor) on the Ambassadors page. Attachments default to both Friends & Family campaigns."
                                    </p>
                                    <a href="/ambassadors" class="btn btn-primary btn-sm inline-flex" style="text-decoration:none">
                                        "Open Ambassadors"
                                    </a>
                                </div>
                            </Show>

                            // ── Tab: Members ─────────────────────────────────
                            <Show when=move || active_tab.get() == "members">
                                <MembersTab campaign_id=id />
                            </Show>

                            // ── Tab: Landing + QR ────────────────────────────
                            <Show when=move || active_tab.get() == "landing-pages">
                                <LandingPagesTab
                                    campaign_id=id
                                    utm_campaign=utm_for_landing_pages.clone()
                                    utm_source=utm_source.clone()
                                    utm_medium=utm_medium.clone()
                                />
                            </Show>

                            // ── Tab: Sequence ────────────────────────────────
                            <Show when=move || active_tab.get() == "programs">
                                <ProgramsTab />
                            </Show>

                            // ── Tab: Sequence ────────────────────────────────
                            <Show when=move || active_tab.get() == "sequence">
                                <SequenceTab />
                            </Show>

                        </div>
                    }.into_any()
                }
            })}
        </Suspense>
    }
}

// ── Overview Tab ─────────────────────────────────────────────────────────────

#[component]
fn OverviewTab(campaign_id: uuid::Uuid) -> impl IntoView {
    view! {
        <div class="space-y-5">
            // Funnel visualization
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                <div class="px-5 py-3.5 border-b border-outline-variant/15 bg-surface-container-high/20">
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Campaign Funnel"</h3>
                </div>
                <div class="p-6">
                    <div class="flex items-center gap-0 overflow-x-auto">
                        {[
                            ("Mailed / Sent", "Members enrolled in this campaign", "bg-amber-500/20 border-amber-500/30 text-amber-300", "text-amber-400"),
                            ("Visited LP", "Clicked through to landing page", "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)", "color:var(--cobalt)"),
                            ("Filled Form", "Submitted lead capture form", "bg-purple-500/20 border-purple-500/30 text-purple-300", "text-purple-400"),
                            ("Converted", "Became paying subscribers", "bg-emerald-500/20 border-emerald-500/30 text-emerald-300", "text-emerald-400"),
                        ].iter().enumerate().map(|(i, (stage, desc, bg, text))| {
                            let stage = stage.to_string();
                            let desc = desc.to_string();
                            let bg = bg.to_string();
                            let text = text.to_string();
                            view! {
                                <div class="flex items-center">
                                    <div class=format!("rounded-xl border px-5 py-4 text-center shrink-0 w-36 {}", bg)>
                                        <div class=format!("text-2xl font-extrabold font-mono {}", text)>"—"</div>
                                        <div class="text-[10px] font-bold text-white/80 mt-1">{stage}</div>
                                        <div class="text-[9px] text-white/50 mt-0.5 leading-tight">{desc}</div>
                                    </div>
                                    {if i < 3 {
                                        view! {
                                            <div class="w-8 h-px bg-outline-variant/30 relative">
                                                <div class="absolute right-0 top-1/2 -translate-y-1/2 w-0 h-0 border-l-4 border-l-outline-variant/40 border-y-2 border-y-transparent"/>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                    <p class="text-[10px] text-on-surface-variant mt-4">
                        "Funnel data aggregates attribution touchpoints. Enable UTM tracking on your landing page to see live funnel metrics."
                    </p>
                </div>
            </div>

            // Attribution window + channel info
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-4">"Attribution Settings"</h3>
                <div class="grid grid-cols-2 sm:grid-cols-3 gap-4 text-xs">
                    <div>
                        <div class="text-[10px] text-on-surface-variant/60 uppercase tracking-wider mb-1">"Attribution Window"</div>
                        <div class="font-mono text-on-surface font-semibold">"30 days"</div>
                    </div>
                    <div>
                        <div class="text-[10px] text-on-surface-variant/60 uppercase tracking-wider mb-1">"Attribution Model"</div>
                        <div class="font-mono text-on-surface font-semibold">"Last Touch"</div>
                    </div>
                    <div>
                        <div class="text-[10px] text-on-surface-variant/60 uppercase tracking-wider mb-1">"Opens"</div>
                        <div class="font-mono text-on-surface font-semibold">"—"</div>
                    </div>
                    <div>
                        <div class="text-[10px] text-on-surface-variant/60 uppercase tracking-wider mb-1">"Clicks"</div>
                        <div class="font-mono text-on-surface font-semibold">"—"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── Referrers Tab ────────────────────────────────────────────────────────────

#[component]
fn ReferrersTab(campaign_id: uuid::Uuid, utm_campaign: String) -> impl IntoView {
    let board_res = LocalResource::new(move || async move {
        list_campaign_referrers(campaign_id).await
    });
    let share_hint = if utm_campaign.is_empty() {
        "/refer/{code}".to_string()
    } else {
        format!("/refer/{{code}}?utm_campaign={utm_campaign}")
    };

    view! {
        <div class="space-y-4">
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-4">
                <p class="text-xs text-on-surface-variant">
                    "Share template: "
                    <span class="font-mono text-primary/80">{share_hint}</span>
                </p>
                <p class="text-[11px] text-on-surface-variant/70 mt-1">
                    "Attributed waitlist signups grouped by referred_by / utm_content."
                </p>
            </div>

            <Suspense fallback=|| view! {
                <div class="text-xs text-on-surface-variant/60 animate-pulse">"Loading referrers..."</div>
            }>
                {move || board_res.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-8 text-center">
                            <p class="text-sm text-on-surface-variant">{e}</p>
                        </div>
                    }.into_any(),
                    Ok(board) if board.referrers.is_empty() => view! {
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-10 text-center">
                            <p class="text-sm text-on-surface-variant">"No attributed referrals yet."</p>
                            <p class="text-xs text-on-surface-variant mt-1">
                                "Signups from /refer/{code} will appear here once they join the waitlist."
                            </p>
                        </div>
                    }.into_any(),
                    Ok(board) => {
                        let total = board.total_attributed;
                        let top = board.referrers.first().map(|r| r.referred_by.clone()).unwrap_or_default();
                        view! {
                            <div class="grid grid-cols-2 gap-3 mb-2">
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                    <div class="text-lg font-extrabold font-mono text-on-surface">{total}</div>
                                    <div class="text-[9px] uppercase tracking-wider text-on-surface-variant/60 mt-0.5">"Attributed signups"</div>
                                </div>
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                    <div class="text-lg font-extrabold font-mono text-emerald-400">{top}</div>
                                    <div class="text-[9px] uppercase tracking-wider text-on-surface-variant/60 mt-0.5">"Top referrer"</div>
                                </div>
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                <table class="w-full text-left border-collapse text-xs">
                                    <thead>
                                        <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/20">
                                            <th class="py-3 px-4 font-semibold">"Rank"</th>
                                            <th class="py-3 px-4 font-semibold">"Referrer"</th>
                                            <th class="py-3 px-4 font-semibold text-right">"Signups"</th>
                                            <th class="py-3 px-4 font-semibold">"Last signup"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {board.referrers.into_iter().enumerate().map(|(i, row)| {
                                            let rank = i + 1;
                                            let last = row.latest_signup_at.clone().unwrap_or_else(|| "—".to_string());
                                            view! {
                                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container-high/20">
                                                    <td class="py-3 px-4 font-mono text-on-surface-variant">{rank}</td>
                                                    <td class="py-3 px-4 font-semibold text-on-surface">{row.referred_by}</td>
                                                    <td class="py-3 px-4 text-right font-mono font-bold">{row.signup_count}</td>
                                                    <td class="py-3 px-4 text-on-surface-variant font-mono text-[11px]">{last}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}

// ── Members Tab ──────────────────────────────────────────────────────────────

#[component]
fn MembersTab(campaign_id: uuid::Uuid) -> impl IntoView {
    let members_res = LocalResource::new(move || async move {
        list_campaign_members(campaign_id).await.unwrap_or_default()
    });
    let filter = RwSignal::new("all".to_string());

    view! {
        <div class="space-y-4">
            // Controls row
            <div class="flex items-center justify-between gap-4">
                <div class="flex gap-1">
                    {["all", "active", "converted", "exited"].iter().map(|f| {
                        let f = f.to_string();
                        let f2 = f.clone();
                        view! {
                            <button
                                class=move || if filter.get() == f {
                                    "pill active"
                                } else {
                                    "pill"
                                }
                                on:click=move |_| filter.set(f2.clone())
                            >{f.clone()}</button>
                        }
                    }).collect_view()}
                </div>
                <div class="flex gap-2">
                    // Add leads/contacts button (simplified — in production opens a picker modal)
                    <button class="btn btn-ghost btn-sm">
                        <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="2" x2="8" y2="14"/><line x1="2" y1="8" x2="14" y2="8"/></svg>
                        "Add Leads"
                    </button>
                    <button class="btn btn-ghost btn-sm">
                        <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="2" x2="8" y2="14"/><line x1="2" y1="8" x2="14" y2="8"/></svg>
                        "Add Contacts"
                    </button>
                </div>
            </div>

            // Members table
            <Suspense fallback=|| view! { <div class="text-xs text-on-surface-variant/60 animate-pulse">"Loading members..."</div> }>
                {move || members_res.get().map(|members| {
                    let filtered: Vec<_> = members.iter().filter(|m| {
                        let f = filter.get();
                        f == "all" || m.status == f
                    }).cloned().collect();

                    if filtered.is_empty() {
                        view! {
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-10 text-center">
                                <p class="text-sm text-on-surface-variant">"No members match this filter."</p>
                                <p class="text-xs text-on-surface-variant mt-1">
                                    "Use \"Add Leads\" or \"Add Contacts\" to enroll members, or use "
                                    <span class="font-mono text-primary/60">"POST /api/folio/campaigns/{id}/enroll-leads"</span>
                                    " via API."
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse text-xs">
                                        <thead>
                                            <tr class="text-[10px] uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/15 bg-surface-container-high/20">
                                                <th class="py-3 px-4 font-semibold">"Name"</th>
                                                <th class="py-3 px-4 font-semibold">"Email"</th>
                                                <th class="py-3 px-4 font-semibold">"Company"</th>
                                                <th class="py-3 px-4 font-semibold">"Status"</th>
                                                <th class="py-3 px-4 font-semibold">"Address Ready"</th>
                                                <th class="py-3 px-4 font-semibold">"Enrolled"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-outline-variant/5">
                                            {filtered.into_iter().map(|m| {
                                                let has_address = m.contact_metadata.as_ref()
                                                    .and_then(|meta| meta.get("street_address"))
                                                    .and_then(|v| v.as_str())
                                                    .map(|s| !s.is_empty())
                                                    .unwrap_or(false);
                                                let company = m.contact_metadata.as_ref()
                                                    .and_then(|meta| meta.get("company"))
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("—")
                                                    .to_string();
                                                let status_color = match m.status.as_str() {
                                                    "active" => "text-emerald-400",
                                                    "converted" => "text-primary",
                                                    "exited" => "text-error/70",
                                                    _ => "text-on-surface-variant",
                                                };
                                                view! {
                                                    <tr class="hover:bg-surface-bright/5 transition-colors">
                                                        <td class="py-3 px-4 font-semibold text-on-surface">
                                                            {m.contact_name.clone().unwrap_or("—".into())}
                                                        </td>
                                                        <td class="py-3 px-4 text-on-surface-variant font-mono">
                                                            {m.contact_email.clone().unwrap_or("—".into())}
                                                        </td>
                                                        <td class="py-3 px-4 text-on-surface-variant">{company}</td>
                                                        <td class="py-3 px-4">
                                                            <span class=format!("font-semibold {}", status_color)>{m.status.clone()}</span>
                                                        </td>
                                                        <td class="py-3 px-4">
                                                            {if has_address {
                                                                view! { <span class="text-emerald-400 font-semibold">"✓ Yes"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="text-on-surface-variant text-[10px]">"Missing"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="py-3 px-4 text-on-surface-variant/60 font-mono">
                                                            {m.enrolled_at.chars().take(10).collect::<String>()}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_any()
                    }
                })}
            </Suspense>

            // Direct mail readiness note
            <div class="bg-amber-500/5 border border-amber-500/20 rounded-lg px-4 py-3 flex gap-3 text-xs">
                <svg class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M8 2l5 12H3L8 2z"/><line x1="8" y1="10" x2="8" y2="7"/><circle cx="8" cy="12" r="0.5" fill="currentColor"/>
                </svg>
                <div>
                    <span class="font-semibold text-amber-300">"Direct Mail Export: "</span>
                    <span class="text-on-surface-variant/70">
                        "Only members with "
                        <span class="font-semibold text-on-surface">"Address Ready = ✓"</span>
                        " will have complete mailing data in the CSV. "
                        "Address comes from the lead's atlas_leads record or from the linked account for contact members."
                    </span>
                </div>
            </div>
        </div>
    }
}

// ── Landing Pages Tab ─────────────────────────────────────────────────────────

#[component]
fn LandingPagesTab(
    campaign_id: uuid::Uuid,
    utm_campaign: String,
    utm_source: String,
    utm_medium: String,
) -> impl IntoView {
    let products_res =
        LocalResource::new(move || async move { get_products().await.unwrap_or_default() });

    let utm_cmp = utm_campaign.clone();
    let qr_url = crate::api::admin::campaign_qr_url(campaign_id);
    let tracked_preview = format!(
        "?utm_source={}&utm_medium={}&utm_campaign={}&utm_content={{drop}}",
        utm_source, utm_medium, utm_cmp
    );

    view! {
        <div class="space-y-4">
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                <div class="px-4 py-3 border-b border-outline-variant/15 flex items-center justify-between gap-3">
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Landing + QR"</h3>
                    <a class="btn btn-ghost btn-sm" href=qr_url.clone()
                        style="text-decoration:none" target="_blank" rel="noopener">
                        "Download QR PNG"
                    </a>
                </div>
                <div class="px-5 py-4 text-xs text-on-surface-variant leading-relaxed space-y-2">
                    <p>
                        "Print the QR on the piece so it lands on the tracked LP. Full query shape:"
                    </p>
                    <p class="font-mono text-[11px] text-primary/80 break-all">{tracked_preview}</p>
                    <p class="text-[10px] text-on-surface-variant/60">
                        "Replace the drop placeholder with each drop’s utm_content. Offer codes are redeemed on waitlist, not in the URL."
                    </p>
                </div>
            </div>

            // Explainer
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5">
                <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant mb-2">"How Campaign → Landing Page Linking Works"</h3>
                <p class="text-xs text-on-surface-variant/70 leading-relaxed">
                    "When you send a campaign (direct mail postcard, email, paid ad), the destination URL includes UTM parameters. "
                    "Your landing page captures those params when a prospect visits. When they fill the form, Atlas records an attribution touchpoint "
                    "linking the lead back to this campaign. Below are the landing page variants whose URL should include "
                    <span class="font-mono text-primary/80">{format!("utm_campaign={}", utm_cmp.clone())}</span>
                    "."
                </p>
                {if utm_cmp.is_empty() {
                    view! {
                        <div class="mt-3 bg-error/10 border border-error/30 rounded px-3 py-2 text-xs text-error/80">
                            "⚠ This campaign has no utm_campaign slug set. Edit the campaign to add UTM tracking, "
                            "then link landing pages."
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}
            </div>

            // Products/landing pages list
            <Suspense fallback=|| view! { <div class="text-xs text-on-surface-variant/60 animate-pulse">"Loading landing pages..."</div> }>
                {move || {
                    let cmp = utm_campaign.clone();
                    let src = utm_source.clone();
                    let med = utm_medium.clone();
                    products_res.get().map(|products| {
                        if products.is_empty() {
                            return view! {
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-8 text-center">
                                    <p class="text-sm text-on-surface-variant/60">"No landing pages found."</p>
                                    <a href="/landing-pages" class="text-xs text-primary hover:underline mt-2 block">"Go to Landing Pages →"</a>
                                </div>
                            }.into_any();
                        }

                        view! {
                            <div class="space-y-3">
                                {products.into_iter().map(|p| {
                                    let slug = p.slug.clone();
                                    let lp_url = format!(
                                        "/lp/{}?utm_source={}&utm_medium={}&utm_campaign={}",
                                        slug, src, med, cmp
                                    );
                                    let lp_url2 = lp_url.clone();
                                    #[cfg(not(target_arch = "wasm32"))]
                                    let _ = &lp_url2;
                                    let name = p.name.clone();

                                    view! {
                                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-5 py-4 flex items-center justify-between gap-4">
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm font-semibold text-on-surface">{name}</span>
                                                </div>
                                                <p class="text-[10px] font-mono text-on-surface-variant/60 mt-0.5 truncate">
                                                    {format!("/lp/{}", slug)}
                                                </p>
                                            </div>
                                            <div class="flex items-center gap-3 shrink-0">
                                                <div class="hidden lg:block text-right">
                                                    <p class="text-[9px] text-on-surface-variant/50 mb-0.5">"Campaign URL"</p>
                                                    <p class="text-[9px] font-mono text-primary/60 max-w-xs truncate">{lp_url.clone()}</p>
                                                </div>
                                                <button
                                                    class="btn btn-ghost btn-sm"
                                                    on:click=move |_| {
                                                        #[cfg(target_arch = "wasm32")]
                                                        if let Some(w) = web_sys::window() {
                                                            let url = lp_url2.clone();
                                                            let _ = w.navigator().clipboard().write_text(&url);
                                                        }
                                                    }
                                                >"Copy URL"</button>
                                                <a href="/landing-pages"
                                                    class="btn btn-ghost btn-sm"
                                                    style="text-decoration:none"
                                                >"Edit Page →"</a>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    })
                }}
            </Suspense>
        </div>
    }
}

// ── Programs Tab ───────────────────────────────────────────────────────────────

#[component]
fn ProgramsTab() -> impl IntoView {
    view! {
        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-8 text-center">
            <svg class="w-10 h-10 text-on-surface-variant/20 mx-auto mb-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
                <path d="M4 6h16M4 12h16M4 18h10"/>
            </svg>
            <p class="text-sm font-semibold text-on-surface-variant">"Campaign Programs"</p>
            <p class="text-xs text-on-surface-variant/50 mt-1 max-w-sm mx-auto">
                "Link programs via campaign_id on /programs. Program API data is not exposed in platform-admin yet."
            </p>
            <a href="/programs" class="btn btn-ghost btn-sm mt-4" style="text-decoration:none">
                "Open Programs →"
            </a>
        </div>
    }
}

// ── Sequence Tab ──────────────────────────────────────────────────────────────

#[component]
fn SequenceTab() -> impl IntoView {
    view! {
        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-8 text-center">
            <svg class="w-10 h-10 text-on-surface-variant/20 mx-auto mb-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
                <path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2"/>
            </svg>
            <p class="text-sm font-semibold text-on-surface-variant">"Sequence Steps"</p>
            <p class="text-xs text-on-surface-variant/50 mt-1 max-w-sm mx-auto">
                "Define the follow-up sequence for this campaign — email drips, call prompts, "
                "or direct mail follow-ups. Steps are executed by the campaign engine based on enrollment date."
            </p>
            <p class="text-xs text-primary/60 mt-3">"Sequence builder coming next."</p>
        </div>
    }
}

// ── Direct Mail: Drops / Spend / Attribution ──────────────────────────────────

#[component]
fn DmDropsTab(campaign_id: uuid::Uuid) -> impl IntoView {
    use crate::api::admin::{create_mail_drop, create_offer_code, list_mail_drops, list_offer_codes};
    let version = RwSignal::new(0u32);
    let drop_name = RwSignal::new(String::new());
    let offer_code = RwSignal::new(String::new());
    let piece_count = RwSignal::new("500".to_string());
    let unit_cost = RwSignal::new(String::new());
    let drops = LocalResource::new(move || async move {
        let _ = version.get();
        list_mail_drops(campaign_id).await.unwrap_or_default()
    });
    let codes = LocalResource::new(move || async move {
        let _ = version.get();
        list_offer_codes(campaign_id).await.unwrap_or_default()
    });

    view! {
        <div class="space-y-4">
            <div class="bg-amber-500/10 border border-amber-500/25 rounded-xl px-4 py-3 text-[11px] text-on-surface-variant leading-relaxed">
                <span class="font-semibold text-amber-300">"Print checklist. "</span>
                "One drop = one creative. Print offer code on the piece + QR to tracked LP. Export CSV for the mail house, then record spend when invoiced."
            </div>
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-4 flex flex-wrap gap-3 items-end">
                <div>
                    <label class="text-[10px] uppercase text-on-surface-variant">"Drop name"</label>
                    <input class="block mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs"
                        placeholder="Postcard A"
                        prop:value=move || drop_name.get()
                        on:input=move |ev| drop_name.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] uppercase text-on-surface-variant">"Offer code"</label>
                    <input class="block mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs font-mono"
                        placeholder="MIAMI-A"
                        prop:value=move || offer_code.get()
                        on:input=move |ev| offer_code.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] uppercase text-on-surface-variant">"Pieces"</label>
                    <input class="block mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs w-20"
                        prop:value=move || piece_count.get()
                        on:input=move |ev| piece_count.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] uppercase text-on-surface-variant">"Unit cost (USD)"</label>
                    <input class="block mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs w-24"
                        placeholder="2.40"
                        prop:value=move || unit_cost.get()
                        on:input=move |ev| unit_cost.set(event_target_value(&ev))
                    />
                </div>
                <button class="btn btn-primary btn-sm"
                    on:click=move |_| {
                        let name = drop_name.get();
                        let code = offer_code.get();
                        let pieces: i32 = piece_count.get().parse().unwrap_or(0);
                        let unit_cents = unit_cost.get().parse::<f64>().ok().map(|d| (d * 100.0) as i64);
                        if name.is_empty() { return; }
                        let utm = name.to_lowercase().replace(' ', "_");
                        leptos::task::spawn_local(async move {
                            if let Ok(drop) = create_mail_drop(campaign_id, &name, Some(&utm), pieces, unit_cents).await {
                                if !code.is_empty() {
                                    let _ = create_offer_code(campaign_id, &code, Some(drop.id)).await;
                                }
                                drop_name.set(String::new());
                                offer_code.set(String::new());
                                version.update(|n| *n += 1);
                            }
                        });
                    }
                >"+ Add drop"</button>
                <a class="btn btn-ghost btn-sm" href=crate::api::admin::campaign_qr_url(campaign_id)
                    style="text-decoration:none" target="_blank">"QR PNG"</a>
                <a class="btn btn-ghost btn-sm" href=crate::api::admin::campaign_export_url(campaign_id)
                    style="text-decoration:none" target="_blank">"Export CSV"</a>
            </div>
            <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant">"Loading drops…"</p> }>
                {move || {
                    let list = drops.get().unwrap_or_default();
                    let code_list = codes.get().unwrap_or_default();
                    view! {
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                            <table class="w-full text-left text-xs">
                                <thead><tr class="border-b border-outline-variant/20 text-[10px] uppercase text-on-surface-variant">
                                    <th class="px-4 py-2">"Drop"</th>
                                    <th class="px-4 py-2">"utm_content"</th>
                                    <th class="px-4 py-2">"Offer"</th>
                                    <th class="px-4 py-2 text-right">"Pieces"</th>
                                    <th class="px-4 py-2 text-right">"Cost"</th>
                                    <th class="px-4 py-2">"Status"</th>
                                </tr></thead>
                                <tbody>
                                    {list.into_iter().map(|d| {
                                        let offer = code_list.iter()
                                            .find(|c| c.mail_drop_id == Some(d.id))
                                            .map(|c| c.code.clone())
                                            .unwrap_or_else(|| "—".into());
                                        let cost = match (d.unit_cost_cents, d.piece_count) {
                                            (Some(u), n) if n > 0 => format!("${:.0}", (u * n as i64) as f64 / 100.0),
                                            _ => "—".into(),
                                        };
                                        view! {
                                            <tr class="border-b border-outline-variant/10">
                                                <td class="px-4 py-2 font-semibold">{d.drop_name}</td>
                                                <td class="px-4 py-2 font-mono">{d.utm_content.unwrap_or_default()}</td>
                                                <td class="px-4 py-2 font-mono">{offer}</td>
                                                <td class="px-4 py-2 text-right font-mono">{d.piece_count}</td>
                                                <td class="px-4 py-2 text-right font-mono">{cost}</td>
                                                <td class="px-4 py-2">{d.status}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn DmSpendTab(campaign_id: uuid::Uuid) -> impl IntoView {
    use crate::api::admin::{get_campaign, record_campaign_spend};
    let amount = RwSignal::new(String::new());
    let source = RwSignal::new("mail_house_invoice".to_string());
    let external_ref = RwSignal::new(String::new());
    let msg = RwSignal::new(String::new());
    let version = RwSignal::new(0u32);
    let spent = LocalResource::new(move || async move {
        let _ = version.get();
        get_campaign(campaign_id).await.map(|c| (c.spent_cents, c.budget_cents, c.total_conversions)).ok()
    });

    view! {
        <div class="space-y-4 max-w-xl">
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 space-y-4">
                <Suspense fallback=|| view! { <p class="text-xs">"…"</p> }>
                    {move || spent.get().flatten().map(|(s, b, conv)| {
                        let cac = if conv > 0 { format!("${:.0}", s as f64 / 100.0 / conv as f64) } else { "—".into() };
                        view! {
                            <div class="grid grid-cols-3 gap-3 text-center">
                                <div><div class="text-[10px] uppercase text-on-surface-variant">"Spent"</div>
                                    <div class="text-lg font-bold font-mono text-amber-400">{format!("${:.0}", s as f64 / 100.0)}</div></div>
                                <div><div class="text-[10px] uppercase text-on-surface-variant">"Budget"</div>
                                    <div class="text-lg font-bold font-mono">{b.map(|x| format!("${:.0}", x as f64 / 100.0)).unwrap_or("—".into())}</div></div>
                                <div><div class="text-[10px] uppercase text-on-surface-variant">"CAC"</div>
                                    <div class="text-lg font-bold font-mono text-primary">{cac}</div></div>
                            </div>
                        }
                    })}
                </Suspense>
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                    <div>
                        <label class="text-[10px] uppercase text-on-surface-variant">"Amount (USD)"</label>
                        <input class="w-full mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs"
                            prop:value=move || amount.get()
                            on:input=move |ev| amount.set(event_target_value(&ev))
                            placeholder="1200"
                        />
                    </div>
                    <div>
                        <label class="text-[10px] uppercase text-on-surface-variant">"Source"</label>
                        <input class="w-full mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs font-mono"
                            prop:value=move || source.get()
                            on:input=move |ev| source.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="text-[10px] uppercase text-on-surface-variant">"Invoice ref"</label>
                        <input class="w-full mt-1 px-2 py-1.5 rounded border border-outline-variant/30 bg-surface text-xs"
                            prop:value=move || external_ref.get()
                            on:input=move |ev| external_ref.set(event_target_value(&ev))
                            placeholder="INV-7841"
                        />
                    </div>
                </div>
                <button class="btn btn-primary btn-sm"
                    on:click=move |_| {
                        let dollars: f64 = amount.get().parse().unwrap_or(0.0);
                        if dollars <= 0.0 { return; }
                        let cents = (dollars * 100.0) as i64;
                        let src = source.get();
                        let xref = external_ref.get();
                        let xref = if xref.is_empty() { None } else { Some(xref) };
                        leptos::task::spawn_local(async move {
                            match record_campaign_spend(campaign_id, cents, &src, xref.as_deref()).await {
                                Ok(_) => {
                                    msg.set("Spend recorded.".into());
                                    amount.set(String::new());
                                    version.update(|n| *n += 1);
                                }
                                Err(e) => msg.set(e),
                            }
                        });
                    }
                >"Record spend"</button>
                <p class="text-xs text-on-surface-variant">{move || msg.get()}</p>
            </div>
        </div>
    }
}

#[component]
fn DmAttributionTab(campaign_id: uuid::Uuid) -> impl IntoView {
    use crate::api::admin::{get_campaign, get_campaign_attribution};
    let meta = LocalResource::new(move || async move {
        get_campaign(campaign_id).await.ok()
    });
    let data = LocalResource::new(move || async move {
        get_campaign_attribution(campaign_id).await.unwrap_or_default()
    });
    view! {
        <div class="space-y-4">
            <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant">"…"</p> }>
                {move || meta.get().flatten().map(|c| {
                    let conv = c.total_conversions.max(0);
                    let cac = if conv > 0 {
                        format!("${:.0}", c.spent_cents as f64 / 100.0 / conv as f64)
                    } else {
                        "—".into()
                    };
                    view! {
                        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                <div class="text-[10px] uppercase text-on-surface-variant">"Spent"</div>
                                <div class="text-lg font-bold font-mono">{format!("${:.0}", c.spent_cents as f64 / 100.0)}</div>
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                <div class="text-[10px] uppercase text-on-surface-variant">"Conversions"</div>
                                <div class="text-lg font-bold font-mono text-emerald-400">{conv}</div>
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                <div class="text-[10px] uppercase text-on-surface-variant">"CAC"</div>
                                <div class="text-lg font-bold font-mono text-primary">{cac}</div>
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl px-4 py-3">
                                <div class="text-[10px] uppercase text-on-surface-variant">"Model"</div>
                                <div class="text-sm font-semibold">"last_touch · 30d"</div>
                            </div>
                        </div>
                    }
                })}
            </Suspense>
            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                <div class="px-4 py-3 border-b border-outline-variant/15 flex items-center justify-between">
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"G-20 touchpoints"</h3>
                    <span class="text-[10px] text-on-surface-variant/60">"Campaign-scoped · not a global Attribution Dashboard"</span>
                </div>
                <Suspense fallback=|| view! { <p class="text-xs text-on-surface-variant p-4">"Loading…"</p> }>
                    {move || {
                        let list = data.get().unwrap_or_default();
                        if list.is_empty() {
                            view! {
                                <p class="text-xs text-on-surface-variant p-6 text-center">
                                    "No touchpoints yet. LP views and waitlist with UTMs / offer codes will appear here."
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <table class="w-full text-left text-xs">
                                    <thead>
                                        <tr class="border-b border-outline-variant/20 text-[10px] uppercase text-on-surface-variant">
                                            <th class="px-4 py-2">"When"</th>
                                            <th class="px-4 py-2">"Channel"</th>
                                            <th class="px-4 py-2">"Identity"</th>
                                            <th class="px-4 py-2">"utm_content"</th>
                                            <th class="px-4 py-2">"Event"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {list.into_iter().map(|tp| {
                                            let identity = tp.contact_email.clone()
                                                .or_else(|| tp.anonymous_id.map(|a| {
                                                    let short = if a.len() > 8 { format!("anon:{}…", &a[..8]) } else { format!("anon:{a}") };
                                                    short
                                                }))
                                                .unwrap_or_else(|| "—".into());
                                            let event = match (&tp.conversion_entity_type, tp.conversion_value_cents) {
                                                (Some(t), Some(v)) => format!("{t} · ${:.0}", v as f64 / 100.0),
                                                (Some(t), None) => t.clone(),
                                                _ => "touch".into(),
                                            };
                                            let when = tp.occurred_at.replace('T', " ").chars().take(16).collect::<String>();
                                            view! {
                                                <tr class="border-b border-outline-variant/10">
                                                    <td class="px-4 py-2 font-mono text-[11px]">{when}</td>
                                                    <td class="px-4 py-2"><span class="text-[9px] uppercase font-bold tracking-wider text-amber-400 border border-amber-500/30 bg-amber-500/10 rounded px-1.5 py-0.5">{tp.channel}</span></td>
                                                    <td class="px-4 py-2 font-mono text-[11px]">{identity}</td>
                                                    <td class="px-4 py-2 font-mono text-[11px]">{tp.utm_content.unwrap_or_default()}</td>
                                                    <td class="px-4 py-2">{event}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
