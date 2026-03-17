use leptos::prelude::*;
use shared_ui::components::data_table::DataTable;
use shared_ui::components::tabs::{Tabs, TabButton};
use shared_ui::components::card::Card;
use shared_ui::components::ui::tabs::{TabsContent, TabsList};
use shared_ui::components::ui::dialog::{Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter, DialogClose, DialogAction};
use shared_ui::components::ui::input::Input;
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::button::{Button, ButtonVariant};

#[component]
pub fn CrmGrid() -> impl IntoView {
    
    // User Model: ID, Name, Email, Role, Status
    let user_headers = vec!["ID".to_string(), "Name".to_string(), "Email".to_string(), "Role".to_string(), "Status".to_string()];
    let (user_data, _) = signal(vec![
        vec!["U-100".to_string(), "Alice Admin".to_string(), "alice@example.com".to_string(), "Administrator".to_string(), "Active".to_string()],
        vec!["U-101".to_string(), "Bob Builder".to_string(), "bob@example.com".to_string(), "Editor".to_string(), "Active".to_string()],
    ]);

    // Lead Model: Lead ID, Company, Contact, Score, Status
    let lead_headers = vec!["Lead ID".to_string(), "Company".to_string(), "Contact".to_string(), "Score".to_string(), "Status".to_string()];
    let (lead_data, _) = signal(vec![
        vec!["L-500".to_string(), "Stark Industries".to_string(), "Tony Stark".to_string(), "95".to_string(), "Qualifying".to_string()],
        vec!["L-501".to_string(), "Wayne Enterprises".to_string(), "Bruce Wayne".to_string(), "88".to_string(), "Contacted".to_string()],
    ]);

    // Account Model: Name, Type, Attributes, Email, Revenue, Active
    let account_headers = vec![
        "UUID".to_string(), "Name".to_string(), "Type".to_string(), "Email".to_string(), "Revenue".to_string(), "Attributes".to_string(), "Active".to_string()
    ];
    let (account_data, _) = signal(vec![
        vec!["8f6d7a-112".to_string(), "Acme Corp".to_string(), "BusinessEntity".to_string(), "contact@acme.inc".to_string(), "$1.2M".to_string(), "Shipper, Tenant".to_string(), "true".to_string()],
        vec!["3a91bc-445".to_string(), "John Doe".to_string(), "Person".to_string(), "john@doe.local".to_string(), "-".to_string(), "Bitcoiner".to_string(), "true".to_string()],
        vec!["c15b9e-781".to_string(), "Initech".to_string(), "BusinessEntity".to_string(), "admin@initech.co".to_string(), "$150k".to_string(), "Software Vendor".to_string(), "false".to_string()],
    ]);

    // Deal Model: Customer ID, Name, Amount, Status, Stage, Close Date
    let deal_headers = vec![
        "UUID".to_string(), "Deal Name".to_string(), "Customer ID".to_string(), "Amount".to_string(), "Status".to_string(), "Stage".to_string()
    ];
    let (deal_data, _) = signal(vec![
        vec!["d11-45a".to_string(), "Q3 Expansion".to_string(), "8f6d7a-112".to_string(), "$150,000".to_string(), "Qualification".to_string(), "Needs Analysis".to_string()],
        vec!["d22-89c".to_string(), "New Subscription".to_string(), "c15b9e-781".to_string(), "$25,000".to_string(), "Closed Won".to_string(), "Contract Signed".to_string()],
    ]);

    let selected_user = RwSignal::new(user_data.get().first().cloned());
    let selected_lead = RwSignal::new(lead_data.get().first().cloned());
    let selected_account = RwSignal::new(account_data.get().first().cloned());

    view! {
        <div class="max-w-7xl mx-auto space-y-8 p-6">
            <header class="space-y-2">
                <h2 class="text-3xl font-bold tracking-tight">"CRM"</h2>
                <p class="text-muted-foreground text-lg">"Manage the relationships between entities across your systems."</p>
            </header>

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
                                            <Input class="col-span-3".to_string() placeholder="Entity name...".to_string() />
                                        </div>
                                        <div class="grid grid-cols-4 items-center gap-4">
                                            <Label class="text-right".to_string()>"Email"</Label>
                                            <Input class="col-span-3".to_string() placeholder="contact@example.com".to_string() />
                                        </div>
                                        <div class="grid grid-cols-4 items-center gap-4">
                                            <Label class="text-right".to_string()>"Type"</Label>
                                            <select class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 col-span-3">
                                                <option>"Lead"</option>
                                                <option>"Account"</option>
                                                <option>"Contact"</option>
                                                <option>"User"</option>
                                            </select>
                                        </div>
                                    </div>
                                </DialogBody>
                                <DialogFooter>
                                    <DialogClose class="mt-2 sm:mt-0".to_string()>"Cancel"</DialogClose>
                                    <DialogAction>"Save Record"</DialogAction>
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
                                            <Button variant=ButtonVariant::Destructive>"Delete"</Button>
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
                                            <Button variant=ButtonVariant::Destructive>"Delete"</Button>
                                        </div>
                                    </div>
                                    <div class="space-y-4">
                                        <div class="grid gap-1">
                                            <span class="text-sm font-medium">"Contact"</span>
                                            <span class="text-sm text-muted-foreground">{move || selected_lead.get().map(|u| u.get(2).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                        </div>
                                        <div class="grid gap-1">
                                            <span class="text-sm font-medium">"Score"</span>
                                            <span class="text-sm text-muted-foreground">{move || selected_lead.get().map(|u| u.get(3).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                        </div>
                                        <div class="grid gap-1">
                                            <span class="text-sm font-medium">"Status"</span>
                                            <span class="text-sm text-muted-foreground">{move || selected_lead.get().map(|u| u.get(4).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                        </div>
                                    </div>
                                </div>
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
                                <div class="w-full xl:w-96 shrink-0 bg-muted/30 p-6 rounded-xl border border-border flex flex-col space-y-6">
                                    <div class="flex items-center justify-between border-b border-border pb-4">
                                        <div class="space-y-1">
                                            <h4 class="text-xl font-semibold tracking-tight">{move || selected_account.get().map(|u| u.get(1).cloned().unwrap_or_default()).unwrap_or_default()}</h4>
                                            <p class="text-sm text-muted-foreground">{move || selected_account.get().map(|u| u.get(2).cloned().unwrap_or_default()).unwrap_or_default()}</p>
                                        </div>
                                        <div class="flex items-center space-x-2">
                                            <a href=move || format!("/crm/account/{}", selected_account.get().map(|u| u.get(0).cloned().unwrap_or_default()).unwrap_or_default())>
                                                <Button variant=ButtonVariant::Outline>"View Details"</Button>
                                            </a>
                                            <Button variant=ButtonVariant::Destructive>"Delete"</Button>
                                        </div>
                                    </div>
                                    <div class="space-y-4">
                                        <div class="grid gap-1">
                                            <span class="text-sm font-medium">"Email"</span>
                                            <span class="text-sm text-muted-foreground">{move || selected_account.get().map(|u| u.get(3).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                        </div>
                                        <div class="grid gap-1">
                                            <span class="text-sm font-medium">"Revenue"</span>
                                            <span class="text-sm text-muted-foreground">{move || selected_account.get().map(|u| u.get(4).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                        </div>
                                        <div class="grid gap-1">
                                            <span class="text-sm font-medium">"Attributes"</span>
                                            <span class="text-sm text-muted-foreground">{move || selected_account.get().map(|u| u.get(5).cloned().unwrap_or_default()).unwrap_or_default()}</span>
                                        </div>
                                    </div>
                                </div>
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
        </div>
    }
}
