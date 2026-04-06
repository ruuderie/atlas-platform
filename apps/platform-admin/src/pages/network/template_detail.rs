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
pub fn TemplateDetail() -> impl IntoView {
    let params = use_params_map();
    let template_id = move || params.with(|p| p.get("id").unwrap_or_default());
    
    let listings_res = LocalResource::new(move || async move { 
        // We fetch a specific directory's listings or just a mock fallback for MVP since we don't have get_listings_by_template
        vec![] as Vec<crate::api::models::ListingModel>
    });
    let template_schema = RwSignal::new(Some(serde_json::json!({
        "License Number": "Text",
        "Emergency Phone": "String",
        "Years in Business": "Number"
    })));
    
    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                     <div class="flex items-center space-x-3 mb-2">
                        <a href="/network/templates">
                            <Button variant=ButtonVariant::Outline class="h-8 px-2".to_string()>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1"><path d="m15 18-6-6 6-6"/></svg>
                                "Back"
                            </Button>
                        </a>
                        <span class="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium">"Template Record"</span>
                    </div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Manage Template: " {template_id}</h2>
                    <p class="text-muted-foreground mt-1">"Configure the exact data schema required for listings using this template."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Outline>"Edit Data"</Button>
                </div>
            </header>
            
            <div class="grid grid-cols-1 gap-6">
                <Card class="bg-card border-border shadow-sm p-6".to_string()>
                    <PropertiesEditor properties=template_schema />
                </Card>

                 <RelatedList
                    title="Active Listings".to_string()
                    description="Listings actively utilizing this template schema.".to_string()
                    icon="store".to_string()
                    action_label="Provision Listing".to_string()
                    on_action=Callback::new(move |_| {})
                    count=2
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
                            <Suspense fallback=move || view! { <div class="p-4">"Loading..."</div> }>
                            {move || listings_res.get().map(|listings| view! {
                                <For each=move || listings.clone() key=|l| l.id.clone() children=move |l| {
                                    let id1 = l.id.clone();
                                    let id2 = l.id.clone();
                                    view! {
                                        <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                            <DataTableCell class="p-4 align-middle font-medium text-muted-foreground">{id1}</DataTableCell>
                                            <DataTableCell class="p-4 align-middle text-foreground font-semibold">{l.title.clone()}</DataTableCell>
                                            <DataTableCell class="p-4 align-middle text-right">
                                                <a href=format!("/listings/{}", id2)>
                                                    <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Manage Listing"</Button>
                                                </a>
                                            </DataTableCell>
                                        </DataTableRow>
                                    }
                                }/>
                            })}
                        </Suspense>
                        </DataTableBody>
                    </DataTable>
                </RelatedList>
            </div>
        </div>
    }
}
