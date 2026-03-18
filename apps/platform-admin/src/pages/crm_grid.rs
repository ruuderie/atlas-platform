use leptos::prelude::*;
use shared_ui::components::data_table::DataTable;
use shared_ui::components::tabs::{Tabs, TabButton};
use shared_ui::components::card::Card;
use shared_ui::components::ui::tabs::{TabsContent, TabsList};
use shared_ui::components::ui::dialog::{Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter, DialogClose, DialogAction};
use shared_ui::components::ui::input::Input;
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::button::{Button, ButtonVariant};

use crate::api::crm::{get_users, get_leads, get_accounts, get_deals, create_lead, create_account};
use crate::api::models::{UserInfo, LeadModel, AccountModel, DealModel, CreateLead, CreateAccount};

#[component]
pub fn CrmGrid() -> impl IntoView {
    // Form signals
    let new_record_name = RwSignal::new("".to_string());
    let new_record_email = RwSignal::new("".to_string());
    let new_record_type = RwSignal::new("Lead".to_string());
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // ---- Data Fetching Resources ----
    let users_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_users().await.unwrap_or_default() }});
    let leads_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_leads().await.unwrap_or_default() }});
    let accounts_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_accounts().await.unwrap_or_default() }});
    let deals_res = LocalResource::new(move || { trigger_fetch.get(); async move { get_deals().await.unwrap_or_default() }});

    let handle_save_record = move |_| {
        let name = new_record_name.get();
        let email = new_record_email.get();
        let rtype = new_record_type.get();
        
        leptos::task::spawn_local(async move {
            if rtype == "Lead" {
                let data = CreateLead { name, email: Some(email) };
                if let Err(e) = create_lead(data).await { toast.message.set(Some(e)); }
            } else if rtype == "Account" {
                let data = CreateAccount { name };
                if let Err(e) = create_account(data).await { toast.message.set(Some(e)); }
            }
            set_trigger_fetch.update(|v| *v += 1);
            new_record_name.set("".to_string());
            new_record_email.set("".to_string());
        });
    };

    // ---- Headers ----
    let user_headers = vec!["ID".to_string(), "Name".to_string(), "Email".to_string(), "Role".to_string(), "Status".to_string()];
    let lead_headers = vec!["Lead ID".to_string(), "Name".to_string(), "Email".to_string(), "Status".to_string(), "Converted?".to_string()];
    let account_headers = vec!["Account ID".to_string(), "Name".to_string()];
    let deal_headers = vec!["Deal ID".to_string(), "Name".to_string(), "Customer ID".to_string(), "Amount".to_string(), "Status".to_string(), "Stage".to_string()];

    // ---- Derived Data Tables ----
    let user_data = Signal::derive(move || {
        users_res.get().unwrap_or_default().into_iter().map(|u| {
            vec![
                u.id, 
                format!("{} {}", u.first_name, u.last_name), 
                u.email, 
                if u.is_admin { "Admin".into() } else { "User".into() }, 
                "Active".into()
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let lead_data = Signal::derive(move || {
        leads_res.get().unwrap_or_default().into_iter().map(|l| {
            vec![
                l.id,
                l.name,
                l.email.unwrap_or_else(|| "-".to_string()),
                l.status.unwrap_or_else(|| "New".to_string()),
                l.is_converted.to_string(),
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let account_data = Signal::derive(move || {
        accounts_res.get().unwrap_or_default().into_iter().map(|a| {
            vec![
                a.id,
                a.name,
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let deal_data = Signal::derive(move || {
        deals_res.get().unwrap_or_default().into_iter().map(|d| {
            vec![
                d.id,
                d.name,
                d.customer_id,
                format!("${:.2}", d.amount),
                d.status,
                d.stage,
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let selected_user = RwSignal::new(None::<Vec<String>>);
    let selected_lead = RwSignal::new(None::<Vec<String>>);
    let selected_account = RwSignal::new(None::<Vec<String>>);

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-8 p-6">
            <header class="space-y-2">
                <h2 class="text-3xl font-bold tracking-tight">"CRM"</h2>
                <p class="text-muted-foreground text-lg">"Manage the relationships between entities across your systems."</p>
            </header>

            <Suspense fallback=move || view! { <div class="p-8 text-center text-muted-foreground">"Loading CRM data..."</div> }>
                <Card class="p-6 bg-card border border-border flex flex-col min-h-[500px]".to_string()>
                    <Tabs default_value="users".to_string()>
                        <div class="flex justify-between items-center mb-6">
                            <TabsList class="inline-flex h-9 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground self-start".to_string()>
                                <TabButton label="Users" value="users" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                                <TabButton label="Leads" value="leads" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                                <TabButton label="Accounts & Contacts" value="accounts_contacts" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                                <TabButton label="Deals" value="deals" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                            </TabsList>

                            <Dialog>
                                <DialogTrigger class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2".to_string()>
                                    "+ New Record"
                                </DialogTrigger>
                                <DialogContent class="sm:max-w-[425px]".to_string()>
                                    <DialogHeader>
                                        <DialogTitle>"Add New Record"</DialogTitle>
                                        <DialogDescription>"Fill out the details to register a new entity in the CRM."</DialogDescription>
                                    </DialogHeader>
                                    <DialogBody>
                                        <div class="grid gap-4 py-4">
                                            <div class="grid grid-cols-4 items-center gap-4">
                                                <Label class="text-right".to_string()>"Name"</Label>
                                                <Input class="col-span-3".to_string() placeholder="Entity name...".to_string() bind_value=new_record_name />
                                            </div>
                                            <div class="grid grid-cols-4 items-center gap-4">
                                                <Label class="text-right".to_string()>"Email"</Label>
                                                <Input class="col-span-3".to_string() placeholder="contact@example.com".to_string() bind_value=new_record_email />
                                            </div>
                                            <div class="grid grid-cols-4 items-center gap-4">
                                                <Label class="text-right".to_string()>"Type"</Label>
                                                <select 
                                                    class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 col-span-3"
                                                    on:change=move |ev| new_record_type.set(event_target_value(&ev))
                                                >
                                                    <option selected=true>"Lead"</option>
                                                    <option>"Account"</option>
                                                </select>
                                            </div>
                                        </div>
                                    </DialogBody>
                                    <DialogFooter>
                                        <DialogClose class="mt-2 sm:mt-0".to_string()>"Cancel"</DialogClose>
                                        <Button on:click=handle_save_record>"Save Record"</Button>
                                    </DialogFooter>
                                </DialogContent>
                            </Dialog>
                        </div>

                        <div class="flex-1 flex flex-col">
                            <TabsContent value="users".to_string()>
                                <div class="flex flex-col xl:flex-row gap-6 items-start">
                                    <div class="flex-1 min-w-0 overflow-x-auto border border-border/50 rounded-md">
                                        <DataTable 
                                            headers=user_headers.clone() 
                                            data=user_data 
                                            on_row_click=Callback::new(move |row: Vec<String>| selected_user.set(Some(row)))
                                        />
                                    </div>
                                    <Show when=move || selected_user.get().is_some() fallback=|| view! { <div class="w-full xl:w-96 text-center p-4 text-muted-foreground">"Select a user to view details"</div> }>
                                        <div class="w-full xl:w-96 shrink-0 bg-muted/30 p-6 rounded-xl border border-border flex flex-col space-y-6">
                                            <div class="flex items-center justify-between border-b border-border pb-4">
                                                <div class="space-y-1">
                                                    <h4 class="text-xl font-semibold tracking-tight">{move || selected_user.get().map(|u| u.get(1).cloned().unwrap_or_default()).unwrap_or_default()}</h4>
                                                    <p class="text-sm text-muted-foreground">{move || selected_user.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default()}</p>
                                                </div>
                                                <div class="flex items-center space-x-2">
                                                    <a href=move || format!("/crm/user/{}", selected_user.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default())>
                                                        <Button variant=ButtonVariant::Outline>"View Details"</Button>
                                                    </a>
                                                </div>
                                            </div>
                                            <div class="space-y-4">
                                                <div class="grid gap-1">
                                                    <span class="text-sm font-medium">"Email"</span>
                                                    <span class="text-sm text-muted-foreground">{move || selected_user.get().map(|u| u.get(2).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                                </div>
                                                <div class="grid gap-1">
                                                    <span class="text-sm font-medium">"Role"</span>
                                                    <span class="text-sm text-muted-foreground">{move || selected_user.get().map(|u| u.get(3).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                                </div>
                                                <div class="grid gap-1">
                                                    <span class="text-sm font-medium">"Status"</span>
                                                    <span class="text-sm text-muted-foreground">{move || selected_user.get().map(|u| u.get(4).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            </TabsContent>

                            <TabsContent value="leads".to_string()>
                                <div class="flex flex-col xl:flex-row gap-6 items-start">
                                    <div class="flex-1 min-w-0 overflow-x-auto border border-border/50 rounded-md">
                                        <DataTable 
                                            headers=lead_headers.clone() 
                                            data=lead_data 
                                            on_row_click=Callback::new(move |row: Vec<String>| selected_lead.set(Some(row)))
                                        />
                                    </div>
                                    <Show when=move || selected_lead.get().is_some() fallback=|| view! { <div class="w-full xl:w-96 text-center p-4 text-muted-foreground">"Select a lead to view details"</div> }>
                                        <div class="w-full xl:w-96 shrink-0 bg-muted/30 p-6 rounded-xl border border-border flex flex-col space-y-6">
                                            <div class="flex items-center justify-between border-b border-border pb-4">
                                                <div class="space-y-1">
                                                    <h4 class="text-xl font-semibold tracking-tight">{move || selected_lead.get().map(|u| u.get(1).cloned().unwrap_or_default()).unwrap_or_default()}</h4>
                                                    <p class="text-sm text-muted-foreground">{move || selected_lead.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default()}</p>
                                                </div>
                                                <div class="flex items-center space-x-2">
                                                    <a href=move || format!("/crm/lead/{}", selected_lead.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default())>
                                                        <Button variant=ButtonVariant::Outline>"View Details"</Button>
                                                    </a>
                                                </div>
                                            </div>
                                            <div class="space-y-4">
                                                <div class="grid gap-1">
                                                    <span class="text-sm font-medium">"Email"</span>
                                                    <span class="text-sm text-muted-foreground">{move || selected_lead.get().map(|u| u.get(2).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                                </div>
                                                <div class="grid gap-1">
                                                    <span class="text-sm font-medium">"Status"</span>
                                                    <span class="text-sm text-muted-foreground">{move || selected_lead.get().map(|u| u.get(3).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            </TabsContent>

                            <TabsContent value="accounts_contacts".to_string()>
                                <div class="flex flex-col xl:flex-row gap-6 items-start">
                                    <div class="flex-1 min-w-0 overflow-x-auto border border-border/50 rounded-md">
                                        <DataTable 
                                            headers=account_headers.clone() 
                                            data=account_data 
                                            on_row_click=Callback::new(move |row: Vec<String>| selected_account.set(Some(row)))
                                        />
                                    </div>
                                    <Show when=move || selected_account.get().is_some() fallback=|| view! { <div class="w-full xl:w-96 text-center p-4 text-muted-foreground">"Select an account to view details"</div> }>
                                        <div class="w-full xl:w-96 shrink-0 bg-muted/30 p-6 rounded-xl border border-border flex flex-col space-y-6">
                                            <div class="flex items-center justify-between border-b border-border pb-4">
                                                <div class="space-y-1">
                                                    <h4 class="text-xl font-semibold tracking-tight">{move || selected_account.get().map(|u| u.get(1).cloned().unwrap_or_default()).unwrap_or_default()}</h4>
                                                    <p class="text-sm text-muted-foreground">{move || selected_account.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default()}</p>
                                                </div>
                                                <div class="flex items-center space-x-2">
                                                    <a href=move || format!("/crm/account/{}", selected_account.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default())>
                                                        <Button variant=ButtonVariant::Outline>"View Details"</Button>
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            </TabsContent>

                            <TabsContent value="deals".to_string()>
                                <div class="overflow-x-auto border border-border/50 rounded-md">
                                    <DataTable headers=deal_headers.clone() data=deal_data />
                                </div>
                            </TabsContent>
                            
                            <TabsContent value="contacts".to_string()>
                                <p class="text-muted-foreground text-sm">"Coming soon..."</p>
                            </TabsContent>
                        </div>
                    </Tabs>
                </Card>
            </Suspense>
        </div>
    }
}
