use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

use crate::components::upsell_banner::UpsellBanner;
use crate::components::recommended_partners::RecommendedPartners;

use serde::{Serialize, Deserialize};
use crate::api::crm::{get_user_by_id, get_lead_by_id, get_account_by_id, get_deal_by_id};
use crate::api::models::{UserInfo, LeadModel, AccountModel, DealModel};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityDetail {
    User(UserInfo),
    Lead(LeadModel),
    Account(AccountModel),
    Deal(DealModel),
    Unknown,
}

#[component]
pub fn CrmDetail() -> impl IntoView {
    let params = use_params_map();
    
    let entity_type = move || params.get().get("entity").unwrap_or_default().to_string();
    let record_id = move || params.get().get("id").unwrap_or_default().to_string();

    let details_res = LocalResource::new(
        move || {
            let entity = entity_type();
            let id = record_id();
            async move {
                match entity.as_str() {
                "user" => get_user_by_id(&id).await.map(EntityDetail::User).unwrap_or(EntityDetail::Unknown),
                "lead" => get_lead_by_id(&id).await.map(EntityDetail::Lead).unwrap_or(EntityDetail::Unknown),
                "account" => get_account_by_id(&id).await.map(EntityDetail::Account).unwrap_or(EntityDetail::Unknown),
                "deal" => get_deal_by_id(&id).await.map(EntityDetail::Deal).unwrap_or(EntityDetail::Unknown),
                _ => EntityDetail::Unknown,
            }
        }
    });

    let (show_edit, set_show_edit) = signal(false);

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
                    <Button variant=ButtonVariant::Outline on:click=move |_| set_show_edit.set(true)>"Edit"</Button>
                    <Button variant=ButtonVariant::Destructive>"Delete"</Button>
                </div>
            </header>
            
            <Show when=move || entity_type() == "lead">
                <UpsellBanner 
                    title="Having trouble reaching this lead?".to_string()
                    description="Our dedicated intake team can automatically chase unresponsive leads via phone and SMS.".to_string()
                    cta_text="Enable Intake Pros - $99/mo".to_string()
                    on_click=Callback::new(move |_| {
                        leptos::logging::log!("Upsell Clicked: Intake Pros on Lead {}", record_id());
                    })
                />
            </Show>

            <Suspense fallback=move || view! { <div class="text-muted-foreground p-8">"Loading details..."</div> }>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div class="md:col-span-2 space-y-6">
                        <Card class="p-6 bg-card border border-border".to_string()>
                            <h3 class="text-lg font-semibold mb-4">"Details"</h3>
                            <div class="grid grid-cols-2 gap-y-4 gap-x-8">
                                {move || match details_res.get() {
                                    Some(EntityDetail::User(u)) => view! {
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Name"</span><p class="text-sm font-medium">{format!("{} {}", u.first_name, u.last_name)}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">{u.email}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Admin"</span><Badge intent=if u.is_admin { BadgeIntent::Warning } else { BadgeIntent::Default }>{u.is_admin.to_string()}</Badge></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Success>"Active"</Badge></div>
                                    }.into_any(),
                                    Some(EntityDetail::Lead(l)) => view! {
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Name"</span><p class="text-sm font-medium">{l.name}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">{l.email.unwrap_or_else(|| "-".to_string())}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Default>{l.status.unwrap_or_else(|| "New".to_string())}</Badge></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Converted"</span><p class="text-sm font-medium">{l.is_converted.to_string()}</p></div>
                                    }.into_any(),
                                    Some(EntityDetail::Account(a)) => view! {
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Account Name"</span><p class="text-sm font-medium">{a.name}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Success>"Active"</Badge></div>
                                    }.into_any(),
                                    Some(EntityDetail::Deal(d)) => view! {
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Deal Name"</span><p class="text-sm font-medium">{d.name}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Amount"</span><p class="text-sm text-green-600 font-bold">{format!("${:.2}", d.amount)}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Success>{d.status}</Badge></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Stage"</span><p class="text-sm font-medium">{d.stage}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Customer ID"</span><p class="text-sm font-medium text-primary hover:underline cursor-pointer">{d.customer_id}</p></div>
                                    }.into_any(),
                                    _ => view! {
                                        <div class="space-y-1 col-span-2 text-muted-foreground">"Failed to load details or unknown entity."</div>
                                    }.into_any(),
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
                                    <span class="text-sm text-muted-foreground">"Source"</span>
                                    <span class="text-sm font-medium">"System Gen"</span>
                                </div>
                            </div>
                        </Card>
                        
                        <RecommendedPartners />
                    </div>
                </div>
            </Suspense>

            <Show when=move || show_edit.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_edit.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground capitalize">{move || format!("Edit {}", entity_type())}</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Update core record fields."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2">
                                <Label>"Entity Identifier"</Label>
                                <Input r#type=InputType::Text bind_value=RwSignal::new(record_id()) />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_edit.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| set_show_edit.set(false)>"Save Updates"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
