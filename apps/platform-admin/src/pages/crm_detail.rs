use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::button::{Button, ButtonVariant};

#[component]
pub fn CrmDetail() -> impl IntoView {
    let params = use_params_map();
    
    // In a real app we would use distinct routes or typed params.
    // Here we read the generic "entity" and "id" from the URL map.
    let entity_type = move || params.get().get("entity").unwrap_or_default().to_string();
    let record_id = move || params.get().get("id").unwrap_or_default().to_string();

    view! {
        <div class="max-w-5xl mx-auto space-y-6 p-6">
            <header class="flex flex-col md:flex-row md:items-start justify-between gap-4 mb-6">
                <div class="space-y-2">
                    <div class="flex items-center space-x-3">
                        <h2 class="text-3xl font-bold tracking-tight capitalize">{move || entity_type()} " Record"</h2>
                        <Badge intent=BadgeIntent::Default>{move || record_id()}</Badge>
                    </div>
                </div>
                <div class="flex items-center space-x-2">
                    <Button variant=ButtonVariant::Outline>"Edit"</Button>
                    <Button variant=ButtonVariant::Destructive>"Delete"</Button>
                </div>
            </header>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="md:col-span-2 space-y-6">
                    <Card class="p-6 bg-card border border-border".to_string()>
                        <h3 class="text-lg font-semibold mb-4">"Details"</h3>
                        <div class="grid grid-cols-2 gap-y-4 gap-x-8">
                            {move || match entity_type().as_str() {
                                "user" => view! {
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Username"</span><p class="text-sm font-medium">"alice_admin"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">"alice@example.com"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Role"</span><Badge intent=BadgeIntent::Warning>"Admin"</Badge></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Success>"Active"</Badge></div>
                                }.into_any(),
                                "lead" => view! {
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Name"</span><p class="text-sm font-medium">"John Prospect"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">"john.p@example.com"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Phone"</span><p class="text-sm font-medium">"+1 (555) 123-4567"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Default>"New"</Badge></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Converted"</span><p class="text-sm font-medium">"False"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Associated Deal ID"</span><p class="text-sm font-medium">"None"</p></div>
                                }.into_any(),
                                "customer" => view! {
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Company Name"</span><p class="text-sm font-medium">"Acme Corp"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Customer Type"</span><p class="text-sm font-medium">"Enterprise"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">"billing@acme.example.com"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Phone"</span><p class="text-sm font-medium">"+1 800-ACME-CORP"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Annual Revenue"</span><p class="text-sm font-medium">"$5,000,000.00"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Employee Count"</span><p class="text-sm font-medium">"250"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Website"</span><p class="text-sm font-medium text-primary hover:underline cursor-pointer">"www.acme.example.com"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Success>"Active"</Badge></div>
                                }.into_any(),
                                "contact" => view! {
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Name"</span><p class="text-sm font-medium">"Jane Smith"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">"jane.smith@acme.example.com"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Phone"</span><p class="text-sm font-medium">"+1 (555) 987-6543"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Role"</span><p class="text-sm font-medium">"VP of Purchasing"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Customer ID"</span><p class="text-sm font-medium text-primary hover:underline cursor-pointer">"CST-8821"</p></div>
                                }.into_any(),
                                "deal" => view! {
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Deal Name"</span><p class="text-sm font-medium">"Q4 License Expansion"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Amount"</span><p class="text-sm text-green-600 font-bold">"$125,000.00"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Success>"Won"</Badge></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Stage"</span><p class="text-sm font-medium">"Closed"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Close Date"</span><p class="text-sm font-medium">"Dec 15, 2024"</p></div>
                                    <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Customer ID"</span><p class="text-sm font-medium text-primary hover:underline cursor-pointer">"CST-8821"</p></div>
                                }.into_any(),
                                _ => view! {
                                    <div class="space-y-1 col-span-2">
                                        <span class="text-sm font-medium text-muted-foreground">"Description / Notes"</span>
                                        <p class="text-sm text-foreground/80 mt-1">
                                            "This entity type is not fully mapped in the UI mock. In a fully implemented system, this would display the detailed notes, history, and custom field values associated with this specific entity ID."
                                        </p>
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    </Card>

                    <Card class="p-6 bg-card border border-border".to_string()>
                        <h3 class="text-lg font-semibold mb-4">"Related Records"</h3>
                        <div class="text-sm text-muted-foreground text-center py-8 border-2 border-dashed border-border rounded-lg">
                            "No related records found."
                        </div>
                    </Card>
                </div>

                <div class="space-y-6">
                    <Card class="p-6 bg-card border border-border".to_string()>
                        <h3 class="text-lg font-semibold mb-4">"System Info"</h3>
                        <div class="space-y-4">
                            <div class="flex justify-between">
                                <span class="text-sm text-muted-foreground">"Record ID"</span>
                                <span class="text-sm font-medium truncate max-w-[120px]" title={move || record_id()}>{move || record_id()}</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-muted-foreground">"Created Date"</span>
                                <span class="text-sm font-medium">"Jan 1, 2024"</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-muted-foreground">"Owner"</span>
                                <span class="text-sm font-medium text-primary hover:underline cursor-pointer">"Bob Agent"</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-muted-foreground">"Source"</span>
                                <span class="text-sm font-medium">"System Gen"</span>
                            </div>
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    }
}
