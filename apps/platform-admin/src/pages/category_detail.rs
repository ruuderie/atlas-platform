use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::related_list::RelatedList;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};

#[component]
pub fn CategoryDetail() -> impl IntoView {
    let params = use_params_map();
    let cat_id = move || params.with(|p| p.get("id").unwrap_or_default());
    
    // Mock child entities
    let mock_templates = vec![
        ("TPL-1", "Standard Plumber Form"),
    ];
    let mock_subcategories = vec![
        ("CAT-SUB-1", "Emergency Plumbing"),
        ("CAT-SUB-2", "Commercial Pipemakers"),
    ];
    let mock_listings = vec![
        ("LST-100", "Bob's Plumbing"),
    ];
    
    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                     <div class="flex items-center space-x-3 mb-2">
                        <a href="/categories">
                            <Button variant=ButtonVariant::Outline class="h-8 px-2".to_string()>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1"><path d="m15 18-6-6 6-6"/></svg>
                                "Back"
                            </Button>
                        </a>
                        <span class="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium">"Category Record"</span>
                    </div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Manage Category: " {cat_id}</h2>
                    <p class="text-muted-foreground mt-1">"Metadata and nested relationships for this category."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Outline>"Edit Data"</Button>
                </div>
            </header>
            
            <div class="grid grid-cols-1 gap-6">
                <RelatedList
                    title="Sub-Categories".to_string()
                    description="Nested child categories.".to_string()
                    icon="account_tree".to_string()
                    action_label="Add Sub-Category".to_string()
                    on_action=Callback::new(move |_| {})
                    count=2
                >
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Cat ID"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {mock_subcategories.into_iter().map(|(id, name)| {
                                let target_url = format!("/categories/{}", id);
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                        <DataTableCell class="p-4 align-middle font-medium text-muted-foreground">{id.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-foreground font-semibold">{name.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-right">
                                            <a href=target_url>
                                                <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Drill down"</Button>
                                            </a>
                                        </DataTableCell>
                                    </DataTableRow>
                                }
                            }).collect::<Vec<_>>()}
                        </DataTableBody>
                    </DataTable>
                </RelatedList>

                <RelatedList
                    title="Assigned Templates".to_string()
                    description="Templates active specifically for records under this category.".to_string()
                    icon="draw".to_string()
                    action_label="Link Template".to_string()
                    on_action=Callback::new(move |_| {})
                    count=1
                >
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Template ID"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {mock_templates.into_iter().map(|(id, name)| {
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                        <DataTableCell class="p-4 align-middle font-medium text-muted-foreground">{id.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-foreground font-semibold">{name.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-right">
                                            <a href=format!("/templates/{}", id)>
                                                <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Inspect"</Button>
                                            </a>
                                        </DataTableCell>
                                    </DataTableRow>
                                }
                            }).collect::<Vec<_>>()}
                        </DataTableBody>
                    </DataTable>
                </RelatedList>

                 <RelatedList
                    title="Listings".to_string()
                    description="Listings explicitly classified under this category.".to_string()
                    icon="store".to_string()
                    action_label="Provision Listing".to_string()
                    on_action=Callback::new(move |_| {})
                    count=1
                >
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Listing ID"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Title"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {mock_listings.into_iter().map(|(id, name)| {
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                        <DataTableCell class="p-4 align-middle font-medium text-muted-foreground">{id.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-foreground font-semibold">{name.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-right">
                                            <a href=format!("/listings/{}", id)>
                                                <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Manage Listing"</Button>
                                            </a>
                                        </DataTableCell>
                                    </DataTableRow>
                                }
                            }).collect::<Vec<_>>()}
                        </DataTableBody>
                    </DataTable>
                </RelatedList>
            </div>
        </div>
    }
}
