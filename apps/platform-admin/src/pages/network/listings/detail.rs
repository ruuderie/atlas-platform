use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::related_list::RelatedList;
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::components::card::Card;
use crate::api::admin::end_ab_test;
use crate::app::GlobalToast;

#[component]
pub fn ListingDetail() -> impl IntoView {
    let params = use_params_map();
    let listing_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let listing_id_str = StoredValue::new(listing_id());
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");
    let ab_refetch = RwSignal::new(0u32);

    let ab_tests_res = LocalResource::new(move || {
        let lid = listing_id_str.get_value();
        let _ = ab_refetch.get();
        async move { crate::api::listings::get_listing_ab_tests(&lid).await.unwrap_or_default() }
    });

    let listing_props = RwSignal::new(Some(serde_json::json!({"Service Area": "New York, NJ, PA"})));

    let end_test_action = Action::new_local(move |test_id: &Uuid| {
        let t = toast.clone();
        let tid = *test_id;
        async move {
            match end_ab_test(tid).await {
                Ok(()) => {
                    t.show_toast("Test Ended", "A/B test has been set to Ended.", "success");
                    ab_refetch.update(|v| *v += 1);
                }
                Err(e) => t.show_toast("Error", &e, "error"),
            }
        }
    });

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-6 pt-8 pb-12 px-6">
            <header class="flex flex-col md:flex-row justify-between md:items-end gap-4 border-b border-border pb-4">
                <div>
                     <div class="flex items-center space-x-3 mb-2">
                        <a href="/network/listings">
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
                    // Summary table
                    <DataTable class="w-full text-sm">
                        <DataTableHeader class="bg-muted/50 border-b border-border">
                            <DataTableRow class="hover:bg-transparent">
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Test ID"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Strategy"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Total Visitors"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Avg CVR"</DataTableHead>
                                <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                            </DataTableRow>
                        </DataTableHeader>
                        <DataTableBody class="divide-y divide-border">
                            <Suspense fallback=move || view! { <div class="p-4">"Loading A/B tests…"</div> }>
                                {move || ab_tests_res.get().map(|tests| {
                                    view! {
                                        <For each=move || tests.clone() key=|t| t.id.to_string() children=move |t| {
                                            let id_str = t.id.to_string();
                                            let short_id = id_str.chars().take(8).collect::<String>();
                                            let status = t.status.clone();
                                            let total_views: i32 = t.variants.iter().map(|v| v.views).sum();
                                            let total_conv: i32 = t.variants.iter().map(|v| v.conversions).sum();
                                            let avg_cvr = if total_views > 0 {
                                                format!("{:.1}%", (total_conv as f64 / total_views as f64) * 100.0)
                                            } else {
                                                "—".to_string()
                                            };
                                            let test_id_uuid = t.id;
                                            let is_active = status == "Active";

                                            let status_class = match status.as_str() {
                                                "Active" => "bg-green-500/10 text-green-600",
                                                "Ended" | "Completed" => "bg-muted text-muted-foreground",
                                                _ => "bg-primary/10 text-primary",
                                            };

                                            view! {
                                                <DataTableRow class="transition-colors hover:bg-muted/50">
                                                    <DataTableCell class="p-4 align-middle font-mono text-xs text-muted-foreground">{short_id}"…"</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-foreground font-semibold">{t.traffic_split_strategy.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">
                                                        <span class=format!("px-2 py-1 rounded text-xs font-bold {}", status_class)>{status.clone()}</span>
                                                    </DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right text-muted-foreground">{total_views.to_string()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right font-semibold text-primary">{avg_cvr}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right flex gap-2 justify-end">
                                                        {if is_active {
                                                            view! {
                                                                <button
                                                                    on:click=move |_| { end_test_action.dispatch(test_id_uuid); }
                                                                    class="px-2 py-1 text-xs font-semibold border border-error/30 text-error rounded hover:bg-error/10 transition-all"
                                                                >
                                                                    "End Test"
                                                                </button>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span class="text-xs text-muted-foreground">"—"</span> }.into_any()
                                                        }}
                                                    </DataTableCell>
                                                </DataTableRow>
                                                // Variant breakdown rows
                                                <For each=move || t.variants.clone() key=|v| v.id.to_string() children=move |v| {
                                                    let cvr = if v.views > 0 {
                                                        format!("{:.1}%", (v.conversions as f64 / v.views as f64) * 100.0)
                                                    } else {
                                                        "—".to_string()
                                                    };
                                                    let bar_pct = if v.views > 0 {
                                                        ((v.conversions as f64 / v.views as f64) * 100.0).min(100.0)
                                                    } else {
                                                        0.0
                                                    };
                                                    let is_ctrl = v.is_control;
                                                    view! {
                                                        <DataTableRow class="bg-muted/20 hover:bg-muted/30 transition-colors text-xs">
                                                            <DataTableCell class="pl-8 py-2 text-muted-foreground">
                                                                <div class="flex items-center gap-2">
                                                                    {if is_ctrl {
                                                                        view! { <span class="px-1.5 py-0.5 rounded bg-secondary/20 text-secondary text-[10px] font-bold">"Control"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="px-1.5 py-0.5 rounded bg-primary/10 text-primary text-[10px] font-bold">"Variant"</span> }.into_any()
                                                                    }}
                                                                    <span class="font-medium text-foreground">{v.name.clone()}</span>
                                                                </div>
                                                            </DataTableCell>
                                                            <DataTableCell class="py-2 text-muted-foreground">"—"</DataTableCell>
                                                            <DataTableCell class="py-2 text-right text-muted-foreground">
                                                                {v.views.to_string()} " views · " {v.conversions.to_string()} " conv"
                                                            </DataTableCell>
                                                            <DataTableCell class="py-2 pr-4">
                                                                <div class="flex items-center gap-2">
                                                                    <div class="flex-1 h-1.5 rounded-full bg-border overflow-hidden">
                                                                        <div
                                                                            class="h-full bg-primary rounded-full transition-all"
                                                                            style=format!("width: {}%", bar_pct)
                                                                        />
                                                                    </div>
                                                                    <span class="text-xs font-semibold text-primary w-10 text-right">{cvr}</span>
                                                                </div>
                                                            </DataTableCell>
                                                        </DataTableRow>
                                                    }
                                                }/>
                                            }
                                        }/>
                                    }
                                })}
                            </Suspense>
                        </DataTableBody>
                    </DataTable>
                </RelatedList>
            </div>
        </div>
    }
}
