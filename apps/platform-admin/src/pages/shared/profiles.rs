use leptos::prelude::*;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::table::{Table as DataTable, TableHeader as DataTableHeader, TableRow as DataTableRow, TableHead as DataTableHead, TableBody as DataTableBody, TableCell as DataTableCell};
use leptos_router::hooks::use_params_map;
use shared_ui::components::ui::label::Label;
use crate::api::provision::{provision_admin, ProvisionAdminPayload};
use crate::api::admin::{create_invite, CreateInviteInput};

#[component]
pub fn ProfilesPanel() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());
    let site_id_str = site_id().to_string();

    let (show_invite, set_show_invite) = signal(false);
    let (show_provision, set_show_provision) = signal(false);
    let (managing_user_name, set_managing_user_name) = signal(None::<String>);
    
    let invite_email = RwSignal::new("".to_string());
    
    let provision_email = RwSignal::new("".to_string());
    let provision_first_name = RwSignal::new("".to_string());
    let provision_last_name = RwSignal::new("".to_string());
    let is_provisioning = RwSignal::new(false);
    let provision_setup_url = RwSignal::new(None::<String>);

    let dirs = use_context::<LocalResource<Vec<crate::api::models::PlatformAppModel>>>().expect("dirs context");

    let tenant_id_sig = Signal::derive(move || {
        let current_id = site_id();
        if let Some(d) = dirs.get() {
            d.into_iter()
                .find(|dir| dir.instance_id.to_string() == current_id)
                .and_then(|dir| uuid::Uuid::parse_str(&dir.tenant_id).ok())
        } else {
            None
        }
    });

    let profiles_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::admin::get_users(uuid::Uuid::parse_str(&sid).ok()).await.unwrap_or_default() }
        }
    });

    let handle_invite = move |_| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        let email = invite_email.get().trim().to_string();
        if email.is_empty() {
            toast.show_toast("Validation", "Email is required.", "error");
            return;
        }
        let t = toast.clone();
        leptos::task::spawn_local(async move {
            match create_invite(CreateInviteInput {
                email: email.clone(),
                display_name: None,
                role: "Admin".to_string(),
                app_role: None,
                tenant: String::new(),
                app_instance_id: None,
                target_app_url: None,
                personal_message: None,
                expires_days: Some(7),
            }).await {
                Ok(_) => t.show_toast("Invite Sent", &format!("Invite sent to {}.", email), "success"),
                Err(e) => t.show_toast("Error", &format!("Invite failed: {e}"), "error"),
            }
        });
    };

    let handle_provision = move |_| {
        if is_provisioning.get() { return; }
        
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        let email = provision_email.get().trim().to_string();
        let first = provision_first_name.get().trim().to_string();
        let last = provision_last_name.get().trim().to_string();
        
        if email.is_empty() || first.is_empty() || last.is_empty() {
            toast.show_toast("Validation", "All credentials are required.", "error");
            return;
        }
        
        let t_id_opt = tenant_id_sig.get();
        if t_id_opt.is_none() {
            toast.show_toast("Error", "Could not retrieve active Tenant ID context.", "error");
            return;
        }
        let tenant_id = t_id_opt.unwrap();
        
        is_provisioning.set(true);
        toast.show_toast("Provisioning", "Seeding tenant administrator...", "info");
        
        let payload = ProvisionAdminPayload {
            email,
            first_name: first,
            last_name: last,
        };
        
        leptos::task::spawn_local(async move {
            match provision_admin(tenant_id, payload).await {
                Ok(res) => {
                    toast.show_toast("Success", "Administrator seeded!", "success");
                    provision_setup_url.set(Some(res.setup_url));
                    provision_email.set("".to_string());
                    provision_first_name.set("".to_string());
                    provision_last_name.set("".to_string());
                }
                Err(e) => {
                    toast.show_toast("Error", &format!("Seeding failed: {}", e), "error");
                }
            }
            is_provisioning.set(false);
        });
    };

    view! {
        <div class="w-full animation-fade-in relative">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h3 class="text-xl font-semibold dark:text-white">"Identity & Access Management"</h3>
                    <p class="text-slate-500 text-sm">"Manage access controls, roles, and connected profiles."</p>
                </div>
                <div class="flex gap-2">
                    <Button variant=ButtonVariant::Outline on:click=move |_| set_show_provision.set(true)>
                        "Provision Administrator"
                    </Button>
                    <Button variant=ButtonVariant::Default on:click=move |_| set_show_invite.set(true)>
                        "Invite Team Member"
                    </Button>
                </div>
            </div>

            <div class="w-full rounded-md border border-border">
                <DataTable class="w-full">
                    <DataTableHeader>
                        <DataTableRow class="hover:bg-transparent">
                            <DataTableHead>"User"</DataTableHead>
                            <DataTableHead>"Email"</DataTableHead>
                            <DataTableHead>"Role"</DataTableHead>
                            <DataTableHead class="text-right">"Actions"</DataTableHead>
                        </DataTableRow>
                    </DataTableHeader>
                    <DataTableBody>
                        <Suspense fallback=move || view! { <tr><td colspan="4" class="p-4 text-center text-muted-foreground">"Loading users..."</td></tr> }>
                        {move || profiles_res.get().map(|users| {
                            if users.is_empty() {
                                view! {
                                    <tr><td colspan="4" class="p-8 text-center text-muted-foreground text-sm">"No users found for this site."</td></tr>
                                }.into_any()
                            } else {
                                view! {
                                    <For each=move || users.clone() key=|u| u.id.to_string() children=move |u| {
                                        let display = u.email.split('@').next().unwrap_or("User").to_string();
                                        let display2 = display.clone();
                                        let email = u.email.clone();
                                        let _uid = u.id.to_string();
                                        let uid_short = u.id.to_string().chars().take(8).collect::<String>();
                                        let status = if u.is_active { "Active" } else { "Inactive" };
                                        view! {
                                            <DataTableRow>
                                                <DataTableCell class="font-medium">
                                                    <div class="flex items-center gap-2">
                                                        <div class="w-8 h-8 rounded-full bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-indigo-700 dark:text-indigo-300 font-bold text-xs">
                                                            {display.chars().next().unwrap_or('U').to_ascii_uppercase()}
                                                        </div>
                                                        <span>{display}</span>
                                                        <span class="text-xs text-muted-foreground ml-2 font-mono">"#" {uid_short}</span>
                                                    </div>
                                                </DataTableCell>
                                                <DataTableCell>{email}</DataTableCell>
                                                <DataTableCell>
                                                    <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700">
                                                        {status}
                                                    </span>
                                                </DataTableCell>
                                                <DataTableCell class="text-right">
                                                    <Button variant=ButtonVariant::Ghost class="h-8 px-2".to_string() on:click=move |_| set_managing_user_name.set(Some(display2.clone()))>
                                                        "Manage Access"
                                                    </Button>
                                                </DataTableCell>
                                            </DataTableRow>
                                        }
                                    }/>
                                }.into_any()
                            }
                        })}
                        </Suspense>
                    </DataTableBody>
                </DataTable>
            </div>

            <Show when=move || show_invite.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_invite.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Invite Team Member"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Send a Magic Link invitation to grant access."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Email Address"</Label>
                                <Input r#type=InputType::Email placeholder="user@example.com".to_string() bind_value=invite_email />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_invite.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=handle_invite>"Send Invite"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_provision.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| {
                            set_show_provision.set(false);
                            provision_setup_url.set(None);
                        }>"✕"</button>
                        
                        {move || if let Some(url) = provision_setup_url.get() {
                            view! {
                                <div class="space-y-4">
                                    <h3 class="text-xl font-semibold text-foreground flex items-center gap-2">
                                        <span class="material-symbols-outlined text-emerald-500">"check_circle"</span>
                                        "Administrator Provisioned!"
                                    </h3>
                                    <p class="text-muted-foreground text-sm">
                                        "A secure, passwordless-first onboarding credential has been registered. Share the setup link below with the user."
                                    </p>
                                    <div class="space-y-2 mt-4">
                                        <input 
                                            type="text" 
                                            value=url.clone() 
                                            readonly=true 
                                            class="w-full bg-muted font-mono text-xs px-3 py-2 rounded border border-border outline-none select-all text-foreground"
                                        />
                                        <Button variant=ButtonVariant::Default class="w-full justify-center".to_string() on:click=move |_| {
                                            let window = web_sys::window().unwrap();
                                            let navigator = window.navigator();
                                            let clipboard = navigator.clipboard();
                                            let _ = clipboard.write_text(&url);
                                            let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
                                            toast.show_toast("Clipboard", "Setup link copied!", "success");
                                        }>
                                            "Copy Link"
                                        </Button>
                                    </div>
                                    <div class="flex justify-end pt-4">
                                        <Button variant=ButtonVariant::Outline on:click=move |_| {
                                            set_show_provision.set(false);
                                            provision_setup_url.set(None);
                                        }>"Close"</Button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-4">
                                    <h3 class="text-xl font-semibold text-foreground">"Provision Tenant Administrator"</h3>
                                    <p class="text-muted-foreground text-sm">
                                        "Bypass typical invitations to directly seed a brand new Tenant Administrator (Owner) with passwordless setup token."
                                    </p>
                                    <div class="space-y-4 my-6 text-left">
                                        <div class="grid gap-2">
                                            <Label>"Administrator Email"</Label>
                                            <Input r#type=InputType::Email placeholder="admin@company.com".to_string() bind_value=provision_email />
                                        </div>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div class="grid gap-2">
                                                <Label>"First Name"</Label>
                                                <Input r#type=InputType::Text placeholder="Jane".to_string() bind_value=provision_first_name />
                                            </div>
                                            <div class="grid gap-2">
                                                <Label>"Last Name"</Label>
                                                <Input r#type=InputType::Text placeholder="Doe".to_string() bind_value=provision_last_name />
                                            </div>
                                        </div>
                                    </div>
                                    <div class="flex justify-end gap-3 pt-2">
                                        <Button variant=ButtonVariant::Outline on:click=move |_| set_show_provision.set(false)>"Cancel"</Button>
                                        <Button variant=ButtonVariant::Default on:click=handle_provision attr:disabled=move || is_provisioning.get()>
                                            {move || if is_provisioning.get() { "Provisioning..." } else { "Provision Administrator" }}
                                        </Button>
                                    </div>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>
            </Show>

            <Show when=move || managing_user_name.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_managing_user_name.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">{move || format!("Manage {}", managing_user_name.get().unwrap_or_default())}</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Configure robust access and permissions."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_managing_user_name.set(None)>"Close"</Button>
                            <Button
                                variant=ButtonVariant::Destructive
                                attr:disabled=true
                                attr:title="Revoke requires a user UUID — use toggle_admin or the Users API directly"
                                class="opacity-40 cursor-not-allowed".to_string()
                            >"Revoke Access"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
