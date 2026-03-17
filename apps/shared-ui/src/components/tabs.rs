use leptos::prelude::*;

use crate::components::ui::tabs::{Tabs as RustUITabs, TabsList};

#[component]
pub fn Tabs(
    #[prop(into, optional)] default_value: String,
    children: Children
) -> impl IntoView {
    view! {
        <RustUITabs class="w-full flex-1 flex flex-col".to_string() default_value=default_value>
            {children()}
        </RustUITabs>
    }
}

use crate::components::ui::tabs::TabsTrigger;

#[component]
pub fn TabButton(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] active_value: Signal<String>,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let value_clone1 = value.clone();
    let value_clone2 = value.clone();
    let is_active = move || active_value.get() == value_clone1;
    
    view! {
        <TabsTrigger 
            value=value_clone2.clone()
            class=if is_active() { 
                "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-background text-foreground shadow".to_string() 
            } else { 
                "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-background/50 hover:text-foreground".to_string() 
            }
            on:click=move |_| on_select.run(value_clone2.clone())
        >
            {label}
        </TabsTrigger>
    }
}
