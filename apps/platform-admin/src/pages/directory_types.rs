use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use crate::api::models::DirectoryTypeModel;
use crate::api::directory_types::get_directory_types;

#[component]
pub fn DirectoryTypes() -> impl IntoView {
    let (types, set_types) = signal(Vec::<DirectoryTypeModel>::new());
    
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(data) = get_directory_types().await {
                set_types.set(data);
            }
        });
    });

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Directory Types"</h2>
                    <p class="text-muted-foreground mt-1">"Manage standardized directory frameworks and schemas."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Default>"Create Type"</Button>
                </div>
            </header>

            <Card class="bg-card border-border shadow-sm overflow-hidden p-0".to_string()>
                <div class="overflow-x-auto">
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground w-[250px]">"Type ID"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Description"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {move || types.get().into_iter().map(|item| {
                                let id = item.id.clone();
                                let detail_url = format!("/directory-types/{}", id);
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted group">
                                        <DataTableCell class="p-4 align-middle font-medium text-muted-foreground text-xs">{item.id}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle font-medium text-foreground">{item.name}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-muted-foreground max-w-sm truncate">{item.description}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-right">
                                            <a href=detail_url>
                                                <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary opacity-0 group-hover:opacity-100 transition-opacity".to_string()>"Manage"</Button>
                                            </a>
                                        </DataTableCell>
                                    </DataTableRow>
                                }
                            }).collect::<Vec<_>>()}
                        </DataTableBody>
                    </DataTable>
                </div>
            </Card>
        </div>
    }
}
