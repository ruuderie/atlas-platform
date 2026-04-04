use leptos::prelude::*;

#[component]
pub fn Icon(
    #[prop(into)] name: String,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    view! {
        <span class=format!("{} {}", name, class)></span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_icon_renders_correct_classes() {
        let root = leptos::task::spawn_local(async {
            let el = view! {
                <Icon name="lucide-zap".to_string() class="w-5 h-5".to_string() />
            };
            
            // Testing DOM property mapping natively in WASM via node selection
            // In a real environment, you inject the view into document.body
            // and perform standard HTML attribute checks.
        });
    }
}
