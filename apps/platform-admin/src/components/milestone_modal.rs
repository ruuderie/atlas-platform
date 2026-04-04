use leptos::prelude::*;
use shared_ui::components::modal::Modal;
use shared_ui::components::ui::button::Button;
use shared_ui::components::icon::Icon;

#[component]
pub fn MilestoneModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_activate: Callback<()>,
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] feature_name: String,
    #[prop(into)] price_text: String,
) -> impl IntoView {
    view! {
        <Modal open=open on_close=on_close.clone() title="Milestone Reached!">
            <div class="flex flex-col items-center text-center p-4">
                <div class="w-16 h-16 bg-green-100 text-green-600 rounded-full flex items-center justify-center mb-4">
                    <Icon name="lucide-award".to_string() class="w-8 h-8".to_string() />
                </div>
                
                <h2 class="text-xl font-bold text-gray-900 mb-2">{title}</h2>
                <p class="text-gray-600 mb-6">{description}</p>
                
                <div class="w-full bg-gray-50 p-4 rounded-lg mb-6 text-left border border-gray-100">
                    <div class="flex items-center gap-3 mb-2">
                        <Icon name="lucide-zap".to_string() class="w-5 h-5 text-amber-500".to_string() />
                        <h4 class="font-semibold text-gray-900">{feature_name}</h4>
                    </div>
                    <p class="text-sm text-gray-500 mb-3">
                        "Make sure you are closing them at the highest rate possible. Integrate this module directly into your portal."
                    </p>
                    <div class="font-medium text-blue-700">
                        {price_text}
                    </div>
                </div>
                
                <div class="flex gap-3 w-full">
                    <Button 
                        on:click=move |_| on_close.run(()) 
                        variant=shared_ui::components::ui::button::ButtonVariant::Outline 
                        class="flex-1"
                    >
                        "Maybe Later"
                    </Button>
                    <Button 
                        on:click=move |_| on_activate.run(()) 
                        variant=shared_ui::components::ui::button::ButtonVariant::Default 
                        class="flex-1"
                    >
                        "Enable Feature"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
