use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use crate::api::provision::{provision_tenant, ProvisionTenantPayload};

/// Available app types that can be included at provision time.
/// Each entry: (slug, icon, name, description, required)
const APP_TYPES: &[(&str, &str, &str, &str, bool)] = &[
    (
        "anchor",
        "⚓",
        "Anchor CMS",
        "Content management, CRM, and lead capture. Required for every tenant.",
        true,
    ),
    (
        "property_management",
        "🏠",
        "Folio PM",
        "Full property management: leases, maintenance, G-27 scorecards, STR permitting.",
        false,
    ),
    (
        "network_instance",
        "🔗",
        "Network Directory",
        "Multi-sided marketplace with listings, profiles, and syndication links.",
        false,
    ),
];

#[component]
pub fn AppCreate() -> impl IntoView {
    let site_name       = RwSignal::new("".to_string());
    let slug            = RwSignal::new("".to_string());
    let domain          = RwSignal::new("".to_string());
    let admin_email     = RwSignal::new("".to_string());
    let admin_first     = RwSignal::new("".to_string());
    let admin_last      = RwSignal::new("".to_string());

    // App type selections — anchor always selected
    let include_folio   = RwSignal::new(false);
    let include_network = RwSignal::new(false);
    let bypass_dns      = RwSignal::new(false);

    // Multi-step state
    let current_step    = RwSignal::new(1_u8); // 1 = Identity, 2 = Apps, 3 = Domain & Review

    let is_submitting   = RwSignal::new(false);
    let setup_url       = RwSignal::new(None::<String>);
    let provisioned_domain = RwSignal::new(String::new());

    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    // Automatically derive slug from display name while slug is untouched
    let on_name_input = move |val: String| {
        site_name.set(val.clone());
        let current = slug.get();
        let auto = val.to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect::<String>();
        if current.is_empty() || current == auto.clone() {
            slug.set(auto);
        }
    };

    // Step validation helpers
    let step1_valid = move || {
        !site_name.get().trim().is_empty()
            && !slug.get().trim().is_empty()
            && !admin_email.get().trim().is_empty()
            && !admin_first.get().trim().is_empty()
            && !admin_last.get().trim().is_empty()
    };
    let step3_valid = move || !domain.get().trim().is_empty();

    let handle_submit = move |_| {
        if is_submitting.get() { return; }

        let display = site_name.get().trim().to_string();
        let tenant  = slug.get().trim().to_lowercase();
        let dom     = domain.get().trim().to_lowercase();
        let email   = admin_email.get().trim().to_string();
        let first   = admin_first.get().trim().to_string();
        let last    = admin_last.get().trim().to_string();

        if display.is_empty() || tenant.is_empty() || dom.is_empty() || email.is_empty() {
            toast.show_toast("Validation", "All required fields must be filled.", "error");
            return;
        }

        let mut apps = vec!["anchor".to_string()];
        if include_folio.get()   { apps.push("property_management".to_string()); }
        if include_network.get() { apps.push("network_instance".to_string()); }

        is_submitting.set(true);
        toast.show_toast("Provisioning", "Wiring tenant, app instances, and CORS…", "info");

        let bypass = if bypass_dns.get() { Some(true) } else { None };
        let payload = ProvisionTenantPayload {
            tenant_name: tenant,
            display_name: display,
            domain: dom,
            admin_email: email,
            admin_first_name: first,
            admin_last_name: last,
            apps: Some(apps),
            bypass_dns_verification: bypass,
        };

        leptos::task::spawn_local(async move {
            match provision_tenant(payload).await {
                Ok(res) => {
                    toast.show_toast("Success", "Tenant provisioned successfully!", "success");
                    provisioned_domain.set(res.domain);
                    setup_url.set(Some(res.setup_url));
                }
                Err(e) => {
                    toast.show_toast("Error", &format!("Provisioning failed: {}", e), "error");
                }
            }
            is_submitting.set(false);
        });
    };

    let copy_setup_link = move |_| {
        if let Some(url_val) = setup_url.get() {
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&url_val);
                toast.show_toast("Clipboard", "Setup link copied!", "success");
            }
        }
    };

    view! {
        <div class="main-canvas">
        <div class="max-w-3xl mx-auto space-y-6 pb-16">
            {move || if let Some(url) = setup_url.get() {
                // ── Success state ──
                view! {
                    <div class="animation-fade-in space-y-6">
                        <header class="text-center mb-8">
                            <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-emerald-500/10 text-emerald-500 mb-4 animate-bounce">
                                <span class="material-symbols-outlined text-4xl">"check_circle"</span>
                            </div>
                            <h2 class="text-3xl font-bold tracking-tight text-foreground">"Tenant Provisioned!"</h2>
                            <p class="text-muted-foreground mt-2">
                                "Database records, CORS origins, and CMS scaffolding are fully wired."
                            </p>
                        </header>

                        <Card class="p-8 bg-card border border-border shadow-lg relative overflow-hidden".to_string()>
                            <div class="absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-emerald-500 via-teal-500 to-indigo-500"></div>
                            <div class="space-y-6">
                                <div class="bg-muted/50 border border-border p-4 rounded-xl space-y-2">
                                    <div class="text-xs text-muted-foreground uppercase font-bold tracking-wider">"Active Environment"</div>
                                    <div class="font-mono text-foreground font-bold text-lg">{provisioned_domain.get()}</div>
                                </div>

                                <div class="space-y-2">
                                    <h4 class="font-bold text-foreground">"One-Time Administrator Setup Link"</h4>
                                    <p class="text-xs text-muted-foreground">
                                        "Send this to the tenant owner. They will use it to register their passkey credential."
                                    </p>
                                    <div class="flex items-center gap-2 mt-4">
                                        <input
                                            type="text"
                                            value=url.clone()
                                            readonly=true
                                            class="flex-1 bg-muted font-mono text-sm px-4 py-3 rounded-lg border border-border outline-none select-all text-foreground"
                                        />
                                        <Button variant=ButtonVariant::Default on:click=copy_setup_link class="h-[46px]".to_string()>
                                            <span class="material-symbols-outlined mr-2">"content_copy"</span>
                                            "Copy"
                                        </Button>
                                    </div>
                                </div>

                                <div class="bg-amber-500/5 border border-amber-500/20 p-4 rounded-xl flex items-start gap-3">
                                    <span class="material-symbols-outlined text-amber-500 shrink-0">"warning"</span>
                                    <div class="text-xs text-muted-foreground">
                                        <strong class="text-amber-500 font-semibold">"Security Notice: "</strong>
                                        "This link contains a secure setup token. It expires in 24 hours and can only be used once."
                                    </div>
                                </div>

                                <div class="flex justify-center gap-4 pt-6 border-t border-border mt-8">
                                    <Button variant=ButtonVariant::Outline on:click=move |_| {
                                        setup_url.set(None);
                                        site_name.set("".to_string());
                                        slug.set("".to_string());
                                        domain.set("".to_string());
                                        admin_email.set("".to_string());
                                        admin_first.set("".to_string());
                                        admin_last.set("".to_string());
                                        include_folio.set(false);
                                        include_network.set(false);
                                        bypass_dns.set(false);
                                        current_step.set(1);
                                    }>"Provision Another"</Button>
                                    <a href="/apps">
                                        <Button variant=ButtonVariant::Default>"Go to Dashboard"</Button>
                                    </a>
                                </div>
                            </div>
                        </Card>
                    </div>
                }.into_any()
            } else {
                // ── Wizard form ──
                view! {
                    <div class="animation-fade-in space-y-6">
                        <header class="mb-8">
                            <a href="/apps" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back to Dashboard"</a>
                            <h2 class="text-3xl font-bold tracking-tight">"Provision New Tenant"</h2>
                            <p class="text-muted-foreground mt-2">"Create a fully-wired multi-tenant environment: tenant record, app instances, domain, CMS, and admin passkey setup link — in one step."</p>
                        </header>

                        // ── Step indicator ──
                        <div class="flex items-center gap-0 mb-2">
                            {(1_u8..=3).map(|step| {
                                let label = match step {
                                    1 => "Identity",
                                    2 => "App Selection",
                                    _ => "Domain & Review",
                                };
                                let step_s = StoredValue::new(step);
                                view! {
                                    <>
                                    <div class="flex flex-col items-center gap-1">
                                        <div class=move || {
                                            let active = current_step.get();
                                            if step_s.get_value() == active {
                                                "w-8 h-8 rounded-full bg-primary text-on-primary flex items-center justify-center text-xs font-bold shadow-md"
                                            } else if step_s.get_value() < active {
                                                "w-8 h-8 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 flex items-center justify-center text-xs font-bold"
                                            } else {
                                                "w-8 h-8 rounded-full bg-surface-container border border-outline-variant/30 text-on-surface-variant flex items-center justify-center text-xs font-bold"
                                            }
                                        }>{step.to_string()}</div>
                                        <span class="text-[10px] text-on-surface-variant whitespace-nowrap">{label}</span>
                                    </div>
                                    {if step < 3 {
                                        view! {
                                            <div class=move || {
                                                let active = current_step.get();
                                                if step_s.get_value() < active {
                                                    "flex-1 h-px bg-emerald-500/40 mt-[-12px] mx-1"
                                                } else {
                                                    "flex-1 h-px bg-outline-variant/20 mt-[-12px] mx-1"
                                                }
                                            }></div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    </>
                                }
                            }).collect_view()}
                        </div>

                        <Card class="p-8 bg-card border border-border shadow-sm".to_string()>
                            // ── Step 1: Tenant Identity ──
                            <Show when=move || current_step.get() == 1>
                                <div class="space-y-6">
                                    <h3 class="text-lg font-bold border-b border-border pb-2 mb-4 text-foreground">"1. Tenant Identity"</h3>

                                    <div class="space-y-2">
                                        <Label>"Display Name"</Label>
                                        <Input
                                            r#type=InputType::Text
                                            placeholder="e.g. Acme Corporation".to_string()
                                            bind_value=site_name
                                            on:input=move |ev| on_name_input(event_target_value(&ev))
                                        />
                                        <p class="text-xs text-muted-foreground">"Human-readable tenant name shown in platform-admin."</p>
                                    </div>

                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                        <div class="space-y-2">
                                            <Label>"Tenant Slug"</Label>
                                            <Input
                                                r#type=InputType::Text
                                                placeholder="e.g. acme-corp".to_string()
                                                bind_value=slug
                                            />
                                            <p class="text-xs text-muted-foreground">"Globally unique. Lowercase, hyphens, alphanumeric."</p>
                                            <div class=move || if !slug.get().is_empty() {
                                                "text-xs font-mono text-primary/80 bg-primary/5 border border-primary/20 px-2 py-1 rounded"
                                            } else { "hidden" }>
                                                {move || format!("slug: {}", slug.get())}
                                            </div>
                                        </div>
                                        <div class="space-y-2">
                                            <Label>"Administrator Email"</Label>
                                            <Input
                                                r#type=InputType::Email
                                                placeholder="e.g. admin@company.com".to_string()
                                                bind_value=admin_email
                                            />
                                            <p class="text-xs text-muted-foreground">"Receives the one-time passkey setup link."</p>
                                        </div>
                                    </div>

                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                        <div class="space-y-2">
                                            <Label>"First Name"</Label>
                                            <Input r#type=InputType::Text placeholder="e.g. Jane".to_string() bind_value=admin_first />
                                        </div>
                                        <div class="space-y-2">
                                            <Label>"Last Name"</Label>
                                            <Input r#type=InputType::Text placeholder="e.g. Doe".to_string() bind_value=admin_last />
                                        </div>
                                    </div>

                                    <div class="flex justify-end pt-4 border-t border-border">
                                        <Button
                                            variant=ButtonVariant::Default
                                            attr:disabled=move || !step1_valid()
                                            on:click=move |_| { if step1_valid() { current_step.set(2); } }
                                        >
                                            "Next: App Selection →"
                                        </Button>
                                    </div>
                                </div>
                            </Show>

                            // ── Step 2: App Selection ──
                            <Show when=move || current_step.get() == 2>
                                <div class="space-y-6">
                                    <h3 class="text-lg font-bold border-b border-border pb-2 mb-4 text-foreground">"2. App Selection"</h3>
                                    <p class="text-sm text-muted-foreground -mt-2">
                                        "Choose which Atlas apps to provision for this tenant. Anchor CMS is always included."
                                    </p>

                                    <div class="space-y-3">
                                        {APP_TYPES.iter().map(|(slug, icon, name, desc, required)| {
                                            let slug_sv  = StoredValue::new(*slug);
                                            let icon_sv  = StoredValue::new(*icon);
                                            let name_sv  = StoredValue::new(*name);
                                            let desc_sv  = StoredValue::new(*desc);
                                            let required = *required;
                                            view! {
                                                <label class=move || {
                                                    let checked = required || match slug_sv.get_value() {
                                                        "property_management" => include_folio.get(),
                                                        "network_instance"    => include_network.get(),
                                                        _                     => true,
                                                    };
                                                    if checked {
                                                        "flex items-start gap-4 p-4 rounded-xl border border-primary/40 bg-primary/5 cursor-pointer transition-all"
                                                    } else {
                                                        "flex items-start gap-4 p-4 rounded-xl border border-outline-variant/20 bg-surface-container-low hover:border-primary/30 hover:bg-primary/5 cursor-pointer transition-all"
                                                    }
                                                }>
                                                    <input
                                                        type="checkbox"
                                                        class="mt-1 accent-primary w-4 h-4 shrink-0"
                                                        prop:checked=move || required || match slug_sv.get_value() {
                                                            "property_management" => include_folio.get(),
                                                            "network_instance"    => include_network.get(),
                                                            _                     => true,
                                                        }
                                                        prop:disabled=required
                                                        on:change=move |ev| {
                                                            let checked = event_target_checked(&ev);
                                                            match slug_sv.get_value() {
                                                                "property_management" => include_folio.set(checked),
                                                                "network_instance"    => include_network.set(checked),
                                                                _                     => {},
                                                            }
                                                        }
                                                    />
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-lg">{icon_sv.get_value()}</span>
                                                            <span class="font-semibold text-sm text-on-surface">{name_sv.get_value()}</span>
                                                            {if required {
                                                                view! {
                                                                    <span class="text-[9px] font-bold bg-primary/15 text-primary px-1.5 py-0.5 rounded uppercase tracking-wider">"Required"</span>
                                                                }.into_any()
                                                            } else {
                                                                view! { <span></span> }.into_any()
                                                            }}
                                                        </div>
                                                        <p class="text-xs text-muted-foreground mt-1">{desc_sv.get_value()}</p>
                                                    </div>
                                                </label>
                                            }
                                        }).collect_view()}
                                    </div>

                                    // Live summary
                                    <div class="bg-surface-container border border-outline-variant/20 rounded-xl px-4 py-3 text-xs text-on-surface-variant">
                                        "Will provision: "
                                        <span class="font-semibold text-on-surface">
                                            {move || {
                                                let mut list = vec!["Anchor CMS"];
                                                if include_folio.get()   { list.push("Folio PM"); }
                                                if include_network.get() { list.push("Network Directory"); }
                                                list.join(" + ")
                                            }}
                                        </span>
                                    </div>

                                    <div class="flex justify-between pt-4 border-t border-border">
                                        <Button variant=ButtonVariant::Outline on:click=move |_| current_step.set(1)>
                                            "← Back"
                                        </Button>
                                        <Button variant=ButtonVariant::Default on:click=move |_| current_step.set(3)>
                                            "Next: Domain & Review →"
                                        </Button>
                                    </div>
                                </div>
                            </Show>

                            // ── Step 3: Domain & Review ──
                            <Show when=move || current_step.get() == 3>
                                <div class="space-y-6">
                                    <h3 class="text-lg font-bold border-b border-border pb-2 mb-4 text-foreground">"3. Domain & Review"</h3>

                                    <div class="space-y-2">
                                        <Label>"Routing Hostname (FQDN)"</Label>
                                        <Input
                                            r#type=InputType::Text
                                            placeholder="e.g. acme.platform.localhost".to_string()
                                            bind_value=domain
                                        />
                                        <p class="text-xs text-muted-foreground">"FQDN where users access this tenant (binds Dynamic CORS + Ingress). No scheme, no port."</p>
                                    </div>

                                    // DNS bypass toggle
                                    <label class="flex items-start gap-3 p-4 rounded-xl border border-amber-500/20 bg-amber-500/5 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="mt-0.5 accent-amber-500 w-4 h-4 shrink-0"
                                            prop:checked=move || bypass_dns.get()
                                            on:change=move |ev| bypass_dns.set(event_target_checked(&ev))
                                        />
                                        <div>
                                            <div class="text-sm font-semibold text-amber-500">"Skip DNS TXT Ownership Check"</div>
                                            <p class="text-xs text-muted-foreground mt-0.5">"For dev / staging environments only. Bypasses the Cloudflare DNS TXT record verification step."</p>
                                        </div>
                                    </label>

                                    // Review summary
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden">
                                        <div class="px-4 py-3 border-b border-outline-variant/10 bg-surface-container-high/30">
                                            <span class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Provision Summary"</span>
                                        </div>
                                        <div class="divide-y divide-outline-variant/10 text-xs">
                                            <div class="flex justify-between px-4 py-2.5">
                                                <span class="text-on-surface-variant">"Tenant Name"</span>
                                                <span class="font-semibold text-on-surface">{move || site_name.get()}</span>
                                            </div>
                                            <div class="flex justify-between px-4 py-2.5">
                                                <span class="text-on-surface-variant">"Slug"</span>
                                                <span class="font-mono text-on-surface">{move || slug.get()}</span>
                                            </div>
                                            <div class="flex justify-between px-4 py-2.5">
                                                <span class="text-on-surface-variant">"Domain"</span>
                                                <span class="font-mono text-on-surface">{move || domain.get()}</span>
                                            </div>
                                            <div class="flex justify-between px-4 py-2.5">
                                                <span class="text-on-surface-variant">"Admin"</span>
                                                <span class="text-on-surface">{move || format!("{} {} · {}", admin_first.get(), admin_last.get(), admin_email.get())}</span>
                                            </div>
                                            <div class="flex justify-between px-4 py-2.5">
                                                <span class="text-on-surface-variant">"Apps"</span>
                                                <span class="font-semibold text-on-surface">{move || {
                                                    let mut list = vec!["Anchor CMS"];
                                                    if include_folio.get()   { list.push("Folio PM"); }
                                                    if include_network.get() { list.push("Network Directory"); }
                                                    list.join(" + ")
                                                }}</span>
                                            </div>
                                            {move || if bypass_dns.get() {
                                                view! {
                                                    <div class="flex justify-between px-4 py-2.5 bg-amber-500/5">
                                                        <span class="text-amber-500">"DNS Bypass"</span>
                                                        <span class="font-semibold text-amber-500">"ENABLED (dev/staging)"</span>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <div></div> }.into_any()
                                            }}
                                        </div>
                                    </div>

                                    <div class="section flex items-start gap-3">
                                        <span class="material-symbols-outlined" style="color:var(--cobalt);flex-shrink:0">"info"</span>
                                        <div class="text-xs text-muted-foreground">
                                            <strong style="color:var(--cobalt);font-weight:600">"Shared Infrastructure: "</strong>
                                            "Tenant, account, app instances, domain, CMS scaffolding, and WebAuthn registry entry are created atomically. CORS and routing update live without a pod restart."
                                        </div>
                                    </div>

                                    <div class="flex justify-between pt-4 border-t border-border">
                                        <Button variant=ButtonVariant::Outline on:click=move |_| current_step.set(2)>
                                            "← Back"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Default
                                            attr:disabled=move || is_submitting.get() || !step3_valid()
                                            on:click=handle_submit
                                        >
                                            {move || if is_submitting.get() { "Provisioning…" } else { "🚀 Deploy Tenant" }}
                                        </Button>
                                    </div>
                                </div>
                            </Show>
                        </Card>
                    </div>
                }.into_any()
            }}
        </div>
        </div>
    }
}
