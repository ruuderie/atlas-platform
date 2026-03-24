use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::checkbox::Checkbox;
use shared_ui::components::ui::select::{
    Select, SelectContent, SelectGroup, SelectOption, SelectTrigger, SelectValue, SelectPosition
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DynamicFieldType {
    Text,
    Email,
    Number,
    Textarea,
    Checkbox,
    Select,
    Switch,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DynamicSelectOption {
    pub label: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DynamicField {
    pub id: String,
    pub name: String,
    pub label: String,
    pub field_type: DynamicFieldType,
    pub required: bool,
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
    pub options: Option<Vec<DynamicSelectOption>>,
}

#[component]
pub fn DynamicForm(
    #[prop(into)] layout: Vec<DynamicField>,
    #[prop(into, optional)] on_submit: Option<Callback<HashMap<String, String>>>,
    #[prop(into, optional)] default_values: Option<HashMap<String, String>>,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let mut initial_state = HashMap::new();
    if let Some(defaults) = default_values {
        initial_state = defaults;
    } else {
        for field in &layout {
            if let Some(val) = &field.default_value {
                initial_state.insert(field.id.clone(), val.clone());
            }
        }
    }

    let field_signals: HashMap<String, RwSignal<String>> = layout
        .iter()
        .map(|f| {
            let val = initial_state.get(&f.id).cloned().unwrap_or_default();
            (f.id.clone(), RwSignal::new(val))
        })
        .collect();

    let signals_store = store_value(field_signals);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(cb) = on_submit {
            let mut final_state = HashMap::new();
            signals_store.with_value(|sigs| {
                for (id, sig) in sigs.iter() {
                    final_state.insert(id.clone(), sig.get());
                }
            });
            cb.run(final_state);
        }
    };

    view! {
        <form class=class on:submit=handle_submit>
            <div class="space-y-6">
                {layout.into_iter().map(move |field| {
                    let sig = signals_store.with_value(|sigs| *sigs.get(&field.id).unwrap());
                    
                    let input_view = match field.field_type {
                        DynamicFieldType::Text => view! {
                            <Input r#type=InputType::Text placeholder=field.placeholder.unwrap_or_default() bind_value=sig required=field.required />
                        }.into_any(),
                        DynamicFieldType::Email => view! {
                            <Input r#type=InputType::Email placeholder=field.placeholder.unwrap_or_default() bind_value=sig required=field.required />
                        }.into_any(),
                        DynamicFieldType::Number => view! {
                            <Input r#type=InputType::Number placeholder=field.placeholder.unwrap_or_default() bind_value=sig required=field.required />
                        }.into_any(),
                        DynamicFieldType::Textarea => view! {
                            <textarea 
                                class="border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm"
                                placeholder=field.placeholder.unwrap_or_default()
                                required=field.required
                                prop:value=move || sig.get()
                                on:input=move |ev| sig.set(event_target_value(&ev))
                            />
                        }.into_any(),
                        DynamicFieldType::Checkbox => {
                            let checked = Signal::derive(move || sig.get().to_lowercase() == "true");
                            let on_change = Callback::new(move |new_val: bool| {
                                sig.set(new_val.to_string());
                            });
                            view! {
                                <div class="mt-2">
                                    <Checkbox checked=checked on_checked_change=on_change />
                                </div>
                            }.into_any()
                        },
                        DynamicFieldType::Switch => {
                             let checked = move || sig.get().to_lowercase() == "true";
                             let handle_change = move |ev: leptos::ev::Event| {
                                 let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                 sig.set(target.checked().to_string());
                             };
                             view! {
                                 <div class="mt-2 text-sm text-muted-foreground">
                                    <label class="inline-flex relative items-center cursor-pointer" tabindex="0">
                                        <input
                                            type="checkbox"
                                            value=""
                                            class="hidden peer"
                                            checked=checked()
                                            on:change=handle_change
                                        />
                                        <div
                                            data-name="Switch"
                                            class="w-11 h-6 bg-gray-200 rounded-full peer-focus:outline-hidden peer-focus:ring-ring/50 peer-focus:ring-[3px] peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:size-5 after:transition-all peer-checked:bg-primary"
                                        />
                                    </label>
                                </div>
                             }.into_any()
                        },
                        DynamicFieldType::Select => {
                            let opts = field.options.unwrap_or_default();
                            let default_val = sig.get();
                            
                            let on_change = Callback::new(move |new_val: Option<String>| {
                                if let Some(v) = new_val {
                                    sig.set(v);
                                }
                            });
                            
                            view! {
                                <Select default_value=default_val on_change=on_change>
                                    <SelectTrigger class="w-full".to_string()>
                                        <SelectValue placeholder=field.placeholder.unwrap_or("Select an option".to_string()) />
                                    </SelectTrigger>
                                    <SelectContent position=SelectPosition::Below>
                                        <SelectGroup>
                                            {opts.into_iter().map(|opt| {
                                                view! {
                                                    <SelectOption value=opt.value>
                                                        {opt.label}
                                                    </SelectOption>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </SelectGroup>
                                    </SelectContent>
                                </Select>
                            }.into_any()
                        }
                    };

                    view! {
                        <div class="flex flex-col space-y-2">
                            <Label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70".to_string()>{field.label}</Label>
                            {input_view}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            {children()}
        </form>
    }
}
