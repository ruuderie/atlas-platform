use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::ui::tabs::{Tabs, TabsList, TabsTrigger, TabsContent};
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use shared_ui::components::badge::{Badge, BadgeIntent};

#[component]
pub fn SiteDashboard() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());

    // Mock data for Listings (linked to this directory)
    let mock_listings = vec![
        ("L-101", "Acme Movers", "Transportation", "Active"),
        ("L-102", "Globe Freight", "Logistics", "Pending"),
        ("L-103", "QuickShip Plus", "Courier", "Active"),
        ("L-104", "City Transit Co", "Public Transit", "Inactive"),
    ];

    // Mock data for Profiles (users linked to this directory)
    let mock_profiles = vec![
        ("usr_8821", "Alice Admin", "alice@example.com", "Site Admin"),
        ("usr_3194", "Bob Driver", "bob@example.com", "Contributor"),
        ("usr_5561", "Charlie Dispatch", "charlie@example.com", "Editor"),
    ];

    view! {
        <div class="max-w-7xl mx-auto space-y-6">
            <header class="flex flex-col md:flex-row justify-between items-start md:items-end gap-4 border-b border-border pb-4">
                <div>
                    <div class="flex items-center space-x-3 mb-2">
                        <Button variant=ButtonVariant::Outline class="h-8 px-2".to_string() on:click=move |_| {
                            let window = web_sys::window().unwrap();
                            let _ = window.history().unwrap().back();
                        }>
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1"><path d="m15 18-6-6 6-6"/></svg>
                            "Back to Registry"
                        </Button>
                        <Badge intent=BadgeIntent::Success>"Active"</Badge>
                    </div>
                    <h2 class="text-3xl font-bold tracking-tight">"Directory: " {site_id}</h2>
                    <p class="text-muted-foreground mt-1">"Manage directory-specific listings, users, and configuration."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Outline class="bg-background".to_string()>"View Live Site"</Button>
                    <Button variant=ButtonVariant::Default>"Directory Settings"</Button>
                </div>
            </header>

            <Tabs default_value="listings".to_string() class="w-full relative z-0 mt-6">
                <TabsList class="flex w-full max-w-md mb-6 bg-muted p-1 rounded-md">
                    <TabsTrigger value="listings".to_string()>"Listings"</TabsTrigger>
                    <TabsTrigger value="profiles".to_string()>"User Profiles"</TabsTrigger>
                    <TabsTrigger value="settings".to_string()>"Settings"</TabsTrigger>
                </TabsList>

                <TabsContent value="listings".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <Card class="bg-card border-border shadow-sm p-0 overflow-hidden relative z-0".to_string()>
                        <div class="p-6 border-b border-border flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
                            <div>
                                <h3 class="text-lg font-semibold leading-none tracking-tight">"Business Listings"</h3>
                                <p class="text-sm text-muted-foreground mt-1">"Businesses registered in this specific directory network."</p>
                            </div>
                            <Button variant=ButtonVariant::Default>"Add Listing"</Button>
                        </div>
                        <div class="overflow-x-auto">
                            <DataTable class="w-full text-sm">
                                <DataTableHeader class="bg-muted/50 border-b border-border">
                                    <DataTableRow class="hover:bg-transparent">
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Listing ID"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Business Name"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Category"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                                    </DataTableRow>
                                </DataTableHeader>
                                <DataTableBody class="divide-y divide-border">
                                    {mock_listings.into_iter().map(|(id, name, cat, status)| {
                                        let badge_intent = match status {
                                            "Active" => BadgeIntent::Success,
                                            "Pending" => BadgeIntent::Warning,
                                            _ => BadgeIntent::Default,
                                        };
                                        view! {
                                            <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                                <DataTableCell class="p-4 align-middle font-medium">{id}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle">{name}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle text-muted-foreground">{cat}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle">
                                                    <Badge intent=badge_intent>{status}</Badge>
                                                </DataTableCell>
                                                <DataTableCell class="p-4 align-middle text-right">
                                                    <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Edit"</Button>
                                                </DataTableCell>
                                            </DataTableRow>
                                        }
                                    }).collect::<Vec<_>>()}
                                </DataTableBody>
                            </DataTable>
                        </div>
                    </Card>
                </TabsContent>

                <TabsContent value="profiles".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <Card class="bg-card border-border shadow-sm p-0 overflow-hidden relative z-0".to_string()>
                        <div class="p-6 border-b border-border flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
                            <div>
                                <h3 class="text-lg font-semibold leading-none tracking-tight">"Directory Profiles"</h3>
                                <p class="text-sm text-muted-foreground mt-1">"Users who have registered accounts specifically within this site."</p>
                            </div>
                            <Button variant=ButtonVariant::Default>"Invite User"</Button>
                        </div>
                        <div class="overflow-x-auto">
                            <DataTable class="w-full text-sm">
                                <DataTableHeader class="bg-muted/50 border-b border-border">
                                    <DataTableRow class="hover:bg-transparent">
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"User ID"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Email"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Role"</DataTableHead>
                                        <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                                    </DataTableRow>
                                </DataTableHeader>
                                <DataTableBody class="divide-y divide-border">
                                    {mock_profiles.into_iter().map(|(id, name, email, role)| {
                                        let role_badge = match role {
                                            "Site Admin" => BadgeIntent::Error,
                                            "Editor" => BadgeIntent::Primary,
                                            _ => BadgeIntent::Default,
                                        };
                                        view! {
                                            <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                                <DataTableCell class="p-4 align-middle font-medium">{id}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle">{name}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle text-muted-foreground">{email}</DataTableCell>
                                                <DataTableCell class="p-4 align-middle">
                                                    <Badge intent=role_badge>{role}</Badge>
                                                </DataTableCell>
                                                <DataTableCell class="p-4 align-middle text-right">
                                                    <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Manage"</Button>
                                                </DataTableCell>
                                            </DataTableRow>
                                        }
                                    }).collect::<Vec<_>>()}
                                </DataTableBody>
                            </DataTable>
                        </div>
                    </Card>
                </TabsContent>

                <TabsContent value="settings".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <Card class="bg-card border-border shadow-sm p-6".to_string()>
                        <h3 class="text-lg font-semibold mb-4">"Directory Configuration"</h3>
                        <div class="space-y-4 max-w-lg">
                            <div class="space-y-2">
                                <label class="text-sm font-medium leading-none">"Custom Domain"</label>
                                <input class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50" value=move || format!("{}.example.com", site_id()) disabled=true />
                                <p class="text-xs text-muted-foreground">"Contact platform admin to change domain routing."</p>
                            </div>
                            <Button variant=ButtonVariant::Destructive>"Deactivate Directory"</Button>
                        </div>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    }
}
