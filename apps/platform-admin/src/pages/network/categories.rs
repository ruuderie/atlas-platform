use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use crate::api::models::CategoryModel;
use crate::api::categories::get_categories;
use crate::api::models::PlatformAppModel;

#[component]
pub fn Categories() -> impl IntoView {
    let dirs_res = use_context::<LocalResource<Vec<PlatformAppModel>>>().expect("dirs context");
    let (categories, set_categories) = signal(Vec::<CategoryModel>::new());
    
    // Create query tracking signals similar to platform_admins.rs
    let active_network = use_context::<ReadSignal<Option<uuid::Uuid>>>().expect("active dir");
    let (selected_network, set_selected_network) = signal(active_network.get().map(|u| u.to_string()).unwrap_or_default());
    
    // Use Resource for reactive network fetching
    let cats_res = LocalResource::new(move || {
        let current_dir = selected_network.get();
        async move {
            let filter = if current_dir.is_empty() { None } else { Some(current_dir) };
            get_categories(filter).await.unwrap_or_default()
        }
    });

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Categories"</h2>
                    <p class="text-muted-foreground mt-1">"Manage standardized industry and listing categories natively."</p>
                </div>
                <div class="flex items-center gap-4">
                    <div class="flex flex-col">
                        <label class="text-xs font-semibold text-muted-foreground mb-1 uppercase tracking-wider">"Network Filter"</label>
                        <select
                            class="h-10 bg-surface-container-high px-3 rounded-md text-sm font-medium border border-outline-variant/20 hover:bg-surface-bright/20 focus:ring-primary focus:border-primary text-on-surface min-w-[200px]"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_selected_network.set(val);
                            }
                        >
                            <option value="" selected=move || selected_network.get().is_empty()>"All Networks"</option>
                            <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                                {move || dirs_res.get().map(|networks| view! {
                                    <For
                                        each=move || networks.clone()
                                        key=|dir| dir.tenant_id.clone()
                                        children=move |dir| {
                                            view! {
                                                <option 
                                                    value=dir.tenant_id.to_string()
                                                    selected=move || selected_network.get() == dir.tenant_id.to_string()
                                                >
                                                    {dir.name.clone()}
                                                </option>
                                            }
                                        }
                                    />
                                })}
                            </Suspense>
                        </select>
                    </div>
                    <a href="/network/categories/new" class="mt-5">
                        <Button variant=ButtonVariant::Default>"Create Category"</Button>
                    </a>
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
                            <Suspense fallback=move || view! { <DataTableRow><DataTableCell attr:colspan="5" class="text-center p-8 text-muted-foreground">"Loading categories..."</DataTableCell></DataTableRow> }>
                            {move || cats_res.get().unwrap_or_default().into_iter().map(|item| {
                                let id = item.id.clone();
                                let detail_url = format!("/network/categories/{}", id);
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
                            </Suspense>
                        </DataTableBody>
                    </DataTable>
                </div>
            </Card>
        </div>
    }
}
