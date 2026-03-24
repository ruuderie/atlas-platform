use leptos::prelude::*;
use crate::components::card::Card;
use crate::components::ui::button::{Button, ButtonVariant};

#[component]
pub fn RelatedList(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(into, optional)] icon: Option<String>,
    #[prop(optional)] action_label: Option<String>,
    #[prop(optional)] on_action: Option<Callback<()>>,
    #[prop(optional)] count: Option<usize>,
    children: Children,
) -> impl IntoView {
    view! {
        <Card class="bg-card border-border shadow-sm p-0 overflow-hidden relative z-0 mb-6".to_string()>
            <div class="p-6 border-b border-border flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 bg-muted/20">
                <div class="flex items-center gap-4">
                    {move || icon.clone().map(|ic| view! {
                        <div class="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center shrink-0 border border-primary/20">
                            <span class="material-symbols-outlined text-primary">{ic}</span>
                        </div>
                    })}
                    <div>
                        <div class="flex items-center gap-2">
                            <h3 class="text-lg font-semibold leading-none tracking-tight">{title.clone()}</h3>
                            {move || count.map(|c| view! {
                                <span class="px-2 py-0.5 rounded-full bg-surface-container-high text-xs font-bold text-muted-foreground">{c.to_string()}</span>
                            })}
                        </div>
                        {move || description.clone().map(|desc| view! {
                            <p class="text-sm text-muted-foreground mt-1">{desc}</p>
                        })}
                    </div>
                </div>
                {move || match (action_label.clone(), on_action) {
                    (Some(lbl), Some(cb)) => {
                        view! {
                            <Button variant=ButtonVariant::Default on:click=move |_| cb.run(())>
                                <span class="material-symbols-outlined text-sm mr-2 leading-none">"add"</span>
                                {lbl}
                            </Button>
                        }.into_any()
                    },
                    _ => view! { <div></div> }.into_any(),
                }}
            </div>
            <div class="overflow-x-auto w-full">
                {children()}
            </div>
        </Card>
    }
}
