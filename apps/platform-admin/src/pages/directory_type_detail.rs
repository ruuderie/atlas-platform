use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::related_list::RelatedList;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};

#[component]
pub fn DirectoryTypeDetail() -> impl IntoView {
    let params = use_params_map();
    let type_id = move || params.with(|p| p.get("id").unwrap_or_default());
    
    let dirs_res = LocalResource::new(move || async move { 
        crate::api::directories::get_directories().await.unwrap_or_default() 
    });
    
    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                     <div class="flex items-center space-x-3 mb-2">
                        <a href="/directory-types">
                            <Button variant=ButtonVariant::Outline class="h-8 px-2".to_string()>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1"><path d="m15 18-6-6 6-6"/></svg>
                                "Back"
                            </Button>
                        </a>
                        <span class="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium">"Directory Type"</span>
                    </div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Manage Type: " {type_id}</h2>
                    <p class="text-muted-foreground mt-1">"Configure specific schemas for directories."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Outline>"Edit Setup"</Button>
                </div>
            </header>
            
            <RelatedList
                title="Dependent Directories".to_string()
                description="Directories currently configured to use this type.".to_string()
                icon="account_tree".to_string()
                action_label="Provision Directory".to_string()
                on_action=Callback::new(move |_| {})
                count=2
            >
                <DataTable class="w-full text-sm">
                    <DataTableHeader class="bg-muted/50 border-b border-border">
                        <DataTableRow class="hover:bg-transparent">
                            <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Dir ID"</DataTableHead>
                            <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                            <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                            <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                        </DataTableRow>
                    </DataTableHeader>
                    <DataTableBody class="divide-y divide-border">
                        <Suspense fallback=move || view! { <div class="p-4">"Loading..."</div> }>
                        {move || dirs_res.get().map(|dirs| view! {
                            <For each=move || dirs.clone() key=|d| d.id.clone() children=move |d| {
                                let id = d.id.clone();
                                let site_url = format!("/sites/{}", id);
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                        <DataTableCell class="p-4 align-middle font-medium text-muted-foreground">{id.clone()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-foreground font-semibold">{d.name.clone()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle">{d.site_status.clone()}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-right">
                                            <a href=site_url>
                                                <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Manage Site"</Button>
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
    }
}
