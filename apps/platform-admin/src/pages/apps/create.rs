use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use crate::api::provision::{provision_tenant, ProvisionTenantPayload};

#[component]
pub fn AppCreate() -> impl IntoView {
    let site_name = RwSignal::new("".to_string());
    let slug = RwSignal::new("".to_string());
    let domain = RwSignal::new("".to_string());
    
    let admin_email = RwSignal::new("".to_string());
    let admin_first_name = RwSignal::new("".to_string());
    let admin_last_name = RwSignal::new("".to_string());
    
    let is_submitting = RwSignal::new(false);
    let setup_url = RwSignal::new(None::<String>);
    let provisioned_domain = RwSignal::new(String::new());

    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    // Automatically generate slug from display name if slug is untouched
    let on_name_input = move |val: String| {
        site_name.set(val.clone());
        let current_slug = slug.get();
        // Only auto-generate if slug is empty or matches previous auto-gen
        if current_slug.is_empty() || current_slug == val.to_lowercase().replace(" ", "-").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect::<String>() {
            let derived = val.to_lowercase()
                .replace(" ", "-")
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect::<String>();
            slug.set(derived);
        }
    };

    let handle_submit = move |_| {
        if is_submitting.get() { return; }
        
        let display = site_name.get();
        let tenant = slug.get().trim().to_lowercase();
        let dom = domain.get().trim().to_lowercase();
        let email = admin_email.get().trim().to_string();
        let first = admin_first_name.get().trim().to_string();
        let last = admin_last_name.get().trim().to_string();

        if display.is_empty() || tenant.is_empty() || dom.is_empty() {
            toast.show_toast("Validation", "Tenant Name, Slug, and Domain are required.", "error");
            return;
        }

        if email.is_empty() || first.is_empty() || last.is_empty() {
            toast.show_toast("Validation", "Administrator email, first name, and last name are required.", "error");
            return;
        }

        is_submitting.set(true);
        toast.show_toast("Provisioning", "Provisioning fully-wired tenant...", "info");

        let payload = ProvisionTenantPayload {
            tenant_name: tenant,
            display_name: display,
            domain: dom.clone(),
            admin_email: email,
            admin_first_name: first,
            admin_last_name: last,
            apps: Some(vec!["anchor".to_string()]),
        };

        leptos::task::spawn_local(async move {
            match provision_tenant(payload).await {
                Ok(res) => {
                    toast.show_toast("Success", "Tenant successfully provisioned!", "success");
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
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();
            let _ = clipboard.write_text(&url_val);
            toast.show_toast("Clipboard", "Setup link copied to clipboard!", "success");
        }
    };

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8 pb-16">
            {move || if let Some(url) = setup_url.get() {
                view! {
                    <div class="animation-fade-in space-y-6">
                        <header class="text-center mb-8">
                            <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-emerald-500/10 text-emerald-500 mb-4 animate-bounce">
                                <span class="material-symbols-outlined text-4xl">"check_circle"</span>
                            </div>
                            <h2 class="text-3xl font-bold tracking-tight text-foreground">"Tenant Provisioned Successfully!"</h2>
                            <p class="text-muted-foreground mt-2">
                                "The multi-tenant database records, Dynamic CORS origins, and CMS configurations have been wired up."
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
                                        "Provide this secure setup link to the tenant owner. They will use it to bind their passkey credential in a passwordless-first flow."
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
                                        <strong class="text-amber-500 font-semibold">"Security Notice:"</strong>
                                        " This link contains a secure setup token that will expire in 24 hours. It can only be used once."
                                    </div>
                                </div>

                                <div class="flex justify-center gap-4 pt-6 border-t border-border mt-8">
                                    <Button variant=ButtonVariant::Outline on:click=move |_| {
                                        setup_url.set(None);
                                        site_name.set("".to_string());
                                        slug.set("".to_string());
                                        domain.set("".to_string());
                                        admin_email.set("".to_string());
                                        admin_first_name.set("".to_string());
                                        admin_last_name.set("".to_string());
                                    }>
                                        "Provision Another"
                                    </Button>
                                    <a href="/apps">
                                        <Button variant=ButtonVariant::Default>
                                            "Go to Dashboard"
                                        </Button>
                                    </a>
                                </div>
                            </div>
                        </Card>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="animation-fade-in space-y-6">
                        <header class="mb-8">
                            <a href="/apps" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                            <h2 class="text-3xl font-bold tracking-tight">"Register New Application"</h2>
                            <p class="text-muted-foreground mt-2">"Configure and provision a brand new multi-tenant application sandboxed environment."</p>
                        </header>
                        
                        <Card class="p-8 bg-card border border-border shadow-sm".to_string()>
                            <div class="space-y-8">
                                <div>
                                    <h3 class="text-lg font-bold border-b border-border pb-2 mb-4 text-foreground">"1. Tenant & Routing Configuration"</h3>
                                    
                                    <div class="space-y-6">
                                        <div class="space-y-2">
                                            <Label>"Application / Tenant Name"</Label>
                                            <Input 
                                                r#type=InputType::Text 
                                                placeholder="e.g. Acme Corporation".to_string() 
                                                bind_value=site_name
                                                on:input=move |ev| on_name_input(event_target_value(&ev))
                                            />
                                            <p class="text-xs text-muted-foreground">"The human-readable title for the tenant organization."</p>
                                        </div>

                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                            <div class="space-y-2">
                                                <Label>"Tenant Slug (ID)"</Label>
                                                <Input 
                                                    r#type=InputType::Text 
                                                    placeholder="e.g. acme-corp".to_string() 
                                                    bind_value=slug
                                                />
                                                <p class="text-xs text-muted-foreground">"Globally unique URL slug (lowercase, hyphens, alphanumeric)."</p>
                                            </div>
                                            <div class="space-y-2">
                                                <Label>"Routing Hostname (FQDN)"</Label>
                                                <Input 
                                                    r#type=InputType::Text 
                                                    placeholder="e.g. acme.platform.localhost".to_string() 
                                                    bind_value=domain
                                                />
                                                <p class="text-xs text-muted-foreground">"FQDN where users will access this app (binds Dynamic CORS)."</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div>
                                    <h3 class="text-lg font-bold border-b border-border pb-2 mb-4 text-foreground">"2. Security & Administrator Setup"</h3>
                                    
                                    <div class="space-y-6">
                                        <div class="space-y-2">
                                            <Label>"Administrator Email"</Label>
                                            <Input 
                                                r#type=InputType::Email 
                                                placeholder="e.g. admin@company.com".to_string() 
                                                bind_value=admin_email
                                            />
                                            <p class="text-xs text-muted-foreground">"The initial user receives a secure link to register their passwordless passkey."</p>
                                        </div>

                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                            <div class="space-y-2">
                                                <Label>"First Name"</Label>
                                                <Input 
                                                    r#type=InputType::Text 
                                                    placeholder="e.g. Jane".to_string() 
                                                    bind_value=admin_first_name
                                                />
                                            </div>
                                            <div class="space-y-2">
                                                <Label>"Last Name"</Label>
                                                <Input 
                                                    r#type=InputType::Text 
                                                    placeholder="e.g. Doe".to_string() 
                                                    bind_value=admin_last_name
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div class="bg-indigo-500/5 border border-indigo-500/20 p-4 rounded-xl flex items-start gap-3">
                                    <span class="material-symbols-outlined text-indigo-500 shrink-0">"info"</span>
                                    <div class="text-xs text-muted-foreground">
                                        <strong class="text-indigo-500 font-semibold">"Shared Infrastructure Mode:"</strong>
                                        " Tenants are created instantly in the central multi-tenant cluster. Routing tables and CORS permissions are updated on-the-fly."
                                    </div>
                                </div>
                            </div>

                            <div class="flex justify-end gap-4 mt-8 pt-6 border-t border-border">
                                <a href="/apps">
                                    <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                                </a>
                                <Button variant=ButtonVariant::Default on:click=handle_submit>
                                    {move || if is_submitting.get() { "Provisioning..." } else { "Deploy Application" }}
                                </Button>
                            </div>
                        </Card>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
