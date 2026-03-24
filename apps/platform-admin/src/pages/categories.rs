use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use crate::api::models::CategoryModel;
use crate::api::categories::get_categories;

#[component]
pub fn Categories() -> impl IntoView {
    let (categories, set_categories) = signal(Vec::<CategoryModel>::new());
    
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(data) = get_categories().await {
                set_categories.set(data);
            }
        });
    });

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Categories"</h2>
                    <p class="text-muted-foreground mt-1">"Manage standardized industry and listing categories natively."</p>
                </div>
                <div class="flex space-x-2">
                    <Button variant=ButtonVariant::Default>"Create Category"</Button>
                </div>
            </header>

            <Card class="bg-card border-border shadow-sm overflow-hidden p-0".to_string()>
                <div class="overflow-x-auto">
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Icon"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Slug"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {move || categories.get().into_iter().map(|item| {
                                let id = item.id.clone();
                                let detail_url = format!("/categories/{}", id);
                                let icon = item.icon.clone().unwrap_or_else(|| "category".to_string());
                                let status_class = if item.is_active { "text-primary bg-primary/10" } else { "text-muted-foreground bg-muted" };
                                let status_text = if item.is_active { "Active" } else { "Inactive" };
                                let slug = item.slug.clone().unwrap_or_default();
                                
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted group">
                                        <DataTableCell class="p-4 align-middle text-muted-foreground w-12 text-center">
                                            <span class="material-symbols-outlined text-lg">{icon}</span>
                                        </DataTableCell>
                                        <DataTableCell class="p-4 align-middle font-medium text-foreground">{item.name}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-muted-foreground">{slug}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle">
                                            <span class=format!("px-2 py-1 rounded text-xs font-semibold {}", status_class)>{status_text}</span>
                                            {if item.is_custom {
                                                view! { <span class="ml-2 px-2 py-1 rounded bg-secondary/20 text-secondary text-xs font-semibold">"Custom"</span> }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </DataTableCell>
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
