use leptos::prelude::*;
use crate::components::ui::data_table::{
    DataTable as RustUITable, DataTableBody, DataTableCell, DataTableHead, DataTableHeader, DataTableRow
};

#[component]
pub fn DataTable(
    headers: Vec<String>,
    #[prop(into)] data: Signal<Vec<Vec<String>>>,
    #[prop(into, optional)] on_row_click: Option<Callback<Vec<String>>>,
) -> impl IntoView {
    view! {
        <div class="w-full space-y-4">
            <RustUITable class="w-full caption-bottom text-sm".to_string()>
                <DataTableHeader>
                    <DataTableRow>
                        {headers.into_iter().map(|header| {
                            view! { <DataTableHead>{header}</DataTableHead> }
                        }).collect::<Vec<_>>()}
                    </DataTableRow>
                </DataTableHeader>
                <DataTableBody>
                    <For
                        each=move || data.get()
                        key=|row| row.join("-")
                        children=move |row| {
                            let row_clone = row.clone();
                            view! {
                                <DataTableRow 
                                    class=if on_row_click.is_some() { "cursor-pointer hover:bg-muted/50".to_string() } else { "".to_string() }
                                    on:click=move |_| {
                                        if let Some(cb) = on_row_click {
                                            cb.run(row_clone.clone());
                                        }
                                    }
                                >
                                    {row.into_iter().map(|cell| {
                                        view! { <DataTableCell>{cell}</DataTableCell> }
                                    }).collect::<Vec<_>>()}
                                </DataTableRow>
                            }
                        }
                    />
                </DataTableBody>
            </RustUITable>
            /* Pagination placeholder */
            <div class="flex items-center justify-between px-2">
                <div class="flex-1 text-sm text-muted-foreground">
                    "Showing " {move || data.get().len()} " entries"
                </div>
                <div class="flex items-center space-x-2">
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-input hover:bg-accent hover:text-accent-foreground h-8 px-4" disabled=true>"Previous"</button>
                    <button class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border border-input hover:bg-accent hover:text-accent-foreground h-8 px-4">"Next"</button>
                </div>
            </div>
        </div>
    }
}
