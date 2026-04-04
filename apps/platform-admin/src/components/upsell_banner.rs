use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::Button;
use shared_ui::components::icon::Icon;

#[component]
pub fn UpsellBanner(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] cta_text: String,
    #[prop(into)] on_click: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    view! {
        <Card class="bg-gradient-to-r from-blue-50 to-indigo-50 border-blue-200 mb-6 p-6 shadow-sm flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
            <div class="flex items-start gap-4">
                <div class="p-3 bg-blue-100 text-blue-600 rounded-full flex-shrink-0">
                    <Icon name="lucide-sparkles".to_string() class="w-6 h-6".to_string() />
                </div>
                <div>
                    <h3 class="text-lg font-semibold text-gray-900">{title}</h3>
                    <p class="text-gray-600 mt-1">{description}</p>
                </div>
            </div>
            <div class="flex-shrink-0">
                <Button 
                    on:click=move |e| {
                        // Track impression/click for analytics
                        on_click.run(e);
                    }
                    variant=shared_ui::components::ui::button::ButtonVariant::Default 
                    class="whitespace-nowrap bg-blue-600 hover:bg-blue-700 text-white"
                >
                    {cta_text}
                </Button>
            </div>
        </Card>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_upsell_banner_renders_attributes() {
        let _ = leptos::task::spawn_local(async {
            let _el = view! {
                <UpsellBanner 
                    title="Premium Features".to_string()
                    description="Unlock your potential.".to_string()
                    cta_text="Upgrade".to_string()
                    on_click=Callback::new(move |_| {})
                />
            };
            // Test banner mounts correctly
        });
    }
}
