use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

use crate::components::upsell_banner::UpsellBanner;
use crate::components::recommended_partners::RecommendedPartners;

use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::components::crm_stage_bar::CrmStageBar;
use shared_ui::components::crm_timeline::CrmTimeline;
use shared_ui::components::crm_timeline::{CrmNote as SharedCrmNote, CrmActivity as SharedCrmActivity};

use serde::{Serialize, Deserialize};
use crate::api::crm::{
    get_user_by_id, get_lead_by_id, get_account_by_id, get_deal_by_id, get_contact_by_id,
    update_contact, convert_lead, update_lead, get_contact_notes, add_contact_note,
    get_contact_activities, log_contact_activity, get_crm_status_options
};
use crate::api::models::{UserInfo, LeadModel, AccountModel, DealModel, ContactModel, CreateContact, CrmNote, CrmActivity, CrmStatusOption};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityDetail {
    User(UserInfo),
    Lead(LeadModel),
    Contact(ContactModel),
    Account(AccountModel),
    Deal(DealModel),
    Unknown,
}

#[component]
pub fn CrmDetail() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let entity_type = move || params.get().get("entity").unwrap_or_default().to_string();
    let record_id = move || params.get().get("id").unwrap_or_default().to_string();

    let (trigger_refresh, set_trigger_refresh) = signal(0);

    let details_res = LocalResource::new(
        move || {
            trigger_refresh.get();
            let entity = entity_type();
            let id = record_id();
            async move {
                match entity.as_str() {
                    "user" => get_user_by_id(&id).await.map(EntityDetail::User).unwrap_or(EntityDetail::Unknown),
                    "lead" => get_lead_by_id(&id).await.map(EntityDetail::Lead).unwrap_or(EntityDetail::Unknown),
                    "contact" => get_contact_by_id(&id).await.map(EntityDetail::Contact).unwrap_or(EntityDetail::Unknown),
                    "account" => get_account_by_id(&id).await.map(EntityDetail::Account).unwrap_or(EntityDetail::Unknown),
                    "deal" => get_deal_by_id(&id).await.map(EntityDetail::Deal).unwrap_or(EntityDetail::Unknown),
                    _ => EntityDetail::Unknown,
                }
            }
        }
    );

    let status_options_res = LocalResource::new(move || async move {
        get_crm_status_options("Lead").await.unwrap_or_default()
    });

    let notes_res = LocalResource::new(move || {
        trigger_refresh.get();
        let entity = entity_type();
        let id = record_id();
        async move {
            if entity == "contact" {
                get_contact_notes(&id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    let activities_res = LocalResource::new(move || {
        trigger_refresh.get();
        let entity = entity_type();
        let id = record_id();
        async move {
            if entity == "contact" {
                get_contact_activities(&id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    let contact_properties = RwSignal::new(None::<serde_json::Value>);

    Effect::new(move |_| {
        if let Some(EntityDetail::Contact(c)) = details_res.get() {
            contact_properties.set(c.properties.clone());
        }
    });

    let handle_save_properties = move |_| {
        let id = record_id();
        let props = contact_properties.get();
        
        if let Some(EntityDetail::Contact(c)) = details_res.get_untracked() {
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                let data = CreateContact {
                    name: c.name,
                    email: c.email,
                    phone: c.phone,
                    whatsapp: c.whatsapp,
                    telegram: c.telegram,
                    twitter: c.twitter,
                    instagram: c.instagram,
                    facebook: c.facebook,
                    properties: props,
                };
                match update_contact(&id, data).await {
                    Ok(_) => {
                        toast.message.set(Some("Contact properties saved successfully!".to_string()));
                        set_trigger_refresh.update(|v| *v += 1);
                    }
                    Err(e) => {
                        toast.message.set(Some(format!("Failed to save properties: {}", e)));
                    }
                }
            });
        }
    };

    let handle_convert_lead = move |_| {
        let id = record_id();
        let toast = toast.clone();
        let navigate = leptos_router::hooks::use_navigate();
        leptos::task::spawn_local(async move {
            match convert_lead(&id).await {
                Ok(contact) => {
                    toast.message.set(Some("Lead qualified and converted to Contact!".to_string()));
                    navigate(&format!("/crm/contact/{}", contact.id), Default::default());
                }
                Err(e) => {
                    toast.message.set(Some(format!("Failed to convert lead: {}", e)));
                }
            }
        });
    };

    let current_lead_status = move || {
        if let Some(EntityDetail::Lead(l)) = details_res.get() {
            l.status.clone().unwrap_or_else(|| "new".to_string())
        } else {
            "".to_string()
        }
    };

    let timeline_notes = move || {
        notes_res.get().unwrap_or_default().into_iter().map(|n| SharedCrmNote {
            id: n.id,
            content: n.content,
            created_at: n.created_at,
        }).collect::<Vec<_>>()
    };

    let timeline_activities = move || {
        activities_res.get().unwrap_or_default().into_iter().map(|a| SharedCrmActivity {
            id: a.id,
            activity_type: a.activity_type,
            description: a.description,
            created_at: a.created_at,
        }).collect::<Vec<_>>()
    };

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

            <Show when=move || entity_type() == "lead">
                <Suspense fallback=move || view! { <div class="h-12 animate-pulse bg-surface-container rounded-lg"></div> }>
                    <div class="mb-6">
                        <CrmStageBar
                            stages={status_options_res.get().unwrap_or_default().into_iter().map(|s| {
                                shared_ui::components::crm_stage_bar::CrmStatusOption {
                                    status_key: s.status_key,
                                    label: s.label,
                                    color: s.color,
                                    sort_order: s.sort_order,
                                    is_system: s.is_system,
                                }
                            }).collect::<Vec<_>>()}
                            current_stage={Signal::derive(move || current_lead_status())}
                            on_stage_change={Callback::new(move |new_stage: String| {
                                let id = record_id();
                                let toast = toast.clone();
                                leptos::task::spawn_local(async move {
                                    match update_lead(&id, &new_stage).await {
                                        Ok(_) => {
                                            toast.message.set(Some("Lead status updated successfully!".to_string()));
                                            set_trigger_refresh.update(|v| *v += 1);
                                        }
                                        Err(e) => {
                                            toast.message.set(Some(format!("Failed to update lead stage: {}", e)));
                                        }
                                    }
                                });
                            })}
                        />
                    </div>
                </Suspense>
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
                                    Some(EntityDetail::Lead(l)) => {
                                        let is_conv = l.is_converted;
                                        view! {
                                            <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Name"</span><p class="text-sm font-medium">{l.name.clone()}</p></div>
                                            <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">{l.email.unwrap_or_else(|| "-".to_string())}</p></div>
                                            <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Status"</span><Badge intent=BadgeIntent::Default>{l.status.unwrap_or_else(|| "New".to_string())}</Badge></div>
                                            <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Converted"</span><p class="text-sm font-medium">{l.is_converted.to_string()}</p></div>
                                            <Show when=move || !is_conv>
                                                <div class="col-span-2 pt-4 mt-4 border-t border-border flex justify-end">
                                                    <Button 
                                                        variant=ButtonVariant::Default
                                                        class="btn-primary-gradient text-white flex items-center gap-2"
                                                        on:click=handle_convert_lead
                                                    >
                                                        <span class="material-symbols-outlined text-[16px]">"celebration"</span>
                                                        "Convert Lead to Contact"
                                                    </Button>
                                                </div>
                                            </Show>
                                        }.into_any()
                                    },
                                    Some(EntityDetail::Contact(c)) => view! {
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Name"</span><p class="text-sm font-medium">{c.name.clone()}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Email"</span><p class="text-sm font-medium">{c.email.clone().unwrap_or_else(|| "-".to_string())}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Phone"</span><p class="text-sm font-medium">{c.phone.clone().unwrap_or_else(|| "-".to_string())}</p></div>
                                        <div class="space-y-1"><span class="text-sm font-medium text-muted-foreground">"Customer ID"</span><p class="text-sm font-medium">{c.customer_id.clone().unwrap_or_else(|| "-".to_string())}</p></div>
                                        <div class="space-y-1 col-span-2 mt-4 pt-4 border-t border-border">
                                            <span class="text-sm font-medium text-muted-foreground block mb-2">"Social Media Channels"</span>
                                            <div class="grid grid-cols-2 gap-4">
                                                <div class="flex items-center gap-2">
                                                    <span class="text-xs text-muted-foreground w-16">"WhatsApp:"</span>
                                                    <span class="text-xs font-semibold">{c.whatsapp.clone().unwrap_or_else(|| "-".to_string())}</span>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-xs text-muted-foreground w-16">"Telegram:"</span>
                                                    <span class="text-xs font-semibold">{c.telegram.clone().unwrap_or_else(|| "-".to_string())}</span>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-xs text-muted-foreground w-16">"Twitter:"</span>
                                                    <span class="text-xs font-semibold">{c.twitter.clone().unwrap_or_else(|| "-".to_string())}</span>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-xs text-muted-foreground w-16">"Instagram:"</span>
                                                    <span class="text-xs font-semibold">{c.instagram.clone().unwrap_or_else(|| "-".to_string())}</span>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-xs text-muted-foreground w-16">"Facebook:"</span>
                                                    <span class="text-xs font-semibold">{c.facebook.clone().unwrap_or_else(|| "-".to_string())}</span>
                                                </div>
                                            </div>
                                        </div>
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

                        <Show when=move || entity_type() == "contact">
                            <Card class="p-6 bg-card border border-border".to_string()>
                                <h3 class="text-lg font-semibold mb-6 flex items-center gap-2">
                                    <span class="material-symbols-outlined text-primary">"history"</span>
                                    "Chronological Timeline"
                                </h3>
                                <Suspense fallback=move || view! { <div class="text-muted-foreground text-sm">"Loading timeline..."</div> }>
                                    <CrmTimeline
                                        notes={Signal::derive(move || timeline_notes())}
                                        activities={Signal::derive(move || timeline_activities())}
                                        on_add_note={Callback::new(move |content: String| {
                                            let id = record_id();
                                            let toast = toast.clone();
                                            leptos::task::spawn_local(async move {
                                                match add_contact_note(&id, &content).await {
                                                    Ok(_) => {
                                                        toast.message.set(Some("Note added successfully!".to_string()));
                                                        set_trigger_refresh.update(|v| *v += 1);
                                                    }
                                                    Err(e) => {
                                                        toast.message.set(Some(format!("Failed to add note: {}", e)));
                                                    }
                                                }
                                            });
                                        })}
                                        on_log_activity={Callback::new(move |(act_type, desc): (String, String)| {
                                            let id = record_id();
                                            let toast = toast.clone();
                                            leptos::task::spawn_local(async move {
                                                match log_contact_activity(&id, &act_type, &desc).await {
                                                    Ok(_) => {
                                                        toast.message.set(Some("Activity logged successfully!".to_string()));
                                                        set_trigger_refresh.update(|v| *v += 1);
                                                    }
                                                    Err(e) => {
                                                        toast.message.set(Some(format!("Failed to log activity: {}", e)));
                                                    }
                                                }
                                            });
                                        })}
                                    />
                                </Suspense>
                            </Card>
                        </Show>

                        <Show when=move || entity_type() != "contact">
                            <Card class="p-6 bg-card border border-border".to_string()>
                                <h3 class="text-lg font-semibold mb-4">"Related Records"</h3>
                                <div class="text-sm text-muted-foreground text-center py-8 border-2 border-dashed border-border rounded-lg">
                                    "No related records found."
                                </div>
                            </Card>
                        </Show>
                    </div>

                    <div class="space-y-6">
                        <Show when=move || entity_type() == "contact">
                            <Card class="p-6 bg-card border border-border".to_string()>
                                <div class="space-y-4">
                                    <PropertiesEditor properties=contact_properties />
                                    <div class="flex justify-end pt-2">
                                        <Button 
                                            variant=ButtonVariant::Default
                                            class="text-xs"
                                            on:click=handle_save_properties
                                        >
                                            "Save Custom Properties"
                                        </Button>
                                    </div>
                                </div>
                            </Card>
                        </Show>

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
