use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::related_list::RelatedList;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::components::card::Card;

#[component]
pub fn ListingDetail() -> impl IntoView {
    let params = use_params_map();
    let listing_id = move || params.with(|p| p.get("id").unwrap_or_default());
    
    let mock_ab_tests = vec![
        ("AB-99", "Hero Image Optimization", "Running", "450 visitors"),
    ];
    let listing_props = RwSignal::new(Some(serde_json::json!({"Service Area": "New York, NJ, PA"})));

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                     <div class="flex items-center space-x-3 mb-2">
                        <a href="/listings">
                            <Button variant=ButtonVariant::Outline class="h-8 px-2".to_string()>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1"><path d="m15 18-6-6 6-6"/></svg>
                                "Back"
                            </Button>
                        </a>
                        <span class="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium">"Listing Management"</span>
                    </div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Listing: " {listing_id}</h2>
                    <p class="text-muted-foreground mt-1">"Deep inspection of listing components, fields, and optimization tests."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Outline>"Edit Listing"</Button>
                </div>
            </header>
            
            <div class="grid grid-cols-1 gap-6">
                 <Card class="bg-card border-border shadow-sm p-6".to_string()>
                     <PropertiesEditor properties=listing_props />
                 </Card>

                 <RelatedList
                    title="A/B Tests (Active & Historical)".to_string()
                    description="Optimization tests running against this listing's traffic.".to_string()
                    icon="science".to_string()
                    action_label="New A/B Test".to_string()
                    on_action=Callback::new(move |_| {})
                    count=1
                >
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Test ID"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Hypothesis"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Sample Size"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {mock_ab_tests.into_iter().map(|(id, name, status, traffic)| {
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                        <DataTableCell class="p-4 align-middle font-medium text-muted-foreground">{id.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-foreground font-semibold">{name.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle"><span class="px-2 py-1 bg-primary/10 text-primary rounded text-xs font-bold">{status.to_string()}</span></DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-muted-foreground">{traffic.to_string()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-right">
                                            <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"View Report"</Button>
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
