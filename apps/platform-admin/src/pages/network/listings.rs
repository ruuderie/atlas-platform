use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use crate::api::models::{ListingModel, ListingStatus};
use crate::api::listings::search_listings;

#[component]
pub fn Listings() -> impl IntoView {
    let (listings, set_listings) = signal(Vec::<ListingModel>::new());
    
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(data) = search_listings("").await {
                set_listings.set(data);
            }
        });
    });

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                    <h2 class="text-3xl font-bold tracking-tight text-foreground">"Listings Network"</h2>
                    <p class="text-muted-foreground mt-1">"Manage and moderate all active network listings globally."</p>
                </div>
                <div class="flex space-x-2">
                    <a href="/network/listings/new">
                        <Button variant=ButtonVariant::Default>"Create Listing"</Button>
                    </a>
                </div>
            </header>

            <Card class="bg-card border-border shadow-sm overflow-hidden p-0".to_string()>
                <div class="overflow-x-auto">
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Title"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Type"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Created"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            {move || listings.get().into_iter().map(|item| {
                                let id = item.id.clone();
                                let detail_url = format!("/network/listings/{}", id);
                                let status_str = match item.status {
                                    ListingStatus::Active => "Active",
                                    ListingStatus::Pending => "Pending",
                                    ListingStatus::Approved => "Approved",
                                    ListingStatus::Rejected => "Rejected",
                                };
                                let status_class = match item.status {
                                    ListingStatus::Active => "text-primary bg-primary/10",
                                    ListingStatus::Pending => "text-amber-600 bg-amber-600/10",
                                    ListingStatus::Approved => "text-green-600 bg-green-600/10",
                                    ListingStatus::Rejected => "text-red-500 bg-red-500/10",
                                };
                                
                                view! {
                                    <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted group">
                                        <DataTableCell class="p-4 align-middle font-semibold text-foreground">{item.title}</DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-muted-foreground">
                                            <span class="px-2 py-1 rounded bg-secondary/20 text-secondary text-xs">{item.listing_type}</span>
                                        </DataTableCell>
                                        <DataTableCell class="p-4 align-middle">
                                            <span class=format!("px-2 py-1 rounded text-xs font-semibold {}", status_class)>{status_str}</span>
                                            {if item.is_featured {
                                                view!{ <span class="ml-2 px-2 py-1 bg-amber-100 text-amber-700 text-xs font-bold rounded">"Featured"</span> }.into_any()
                                            } else {
                                                view!{ <span></span> }.into_any()
                                            }}
                                        </DataTableCell>
                                        <DataTableCell class="p-4 align-middle text-muted-foreground text-xs">{item.created_at}</DataTableCell>
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
