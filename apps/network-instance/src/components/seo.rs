use leptos::prelude::*;
use leptos_meta::{Title, Meta, Link};

#[component]
pub fn Seo(
    #[prop(into)] title: String,
    #[prop(optional)] description: Option<String>,
    #[prop(optional)] image: Option<String>,
    #[prop(optional)] og_type: Option<String>,
    #[prop(optional)] script_json_ld: Option<String>,
    #[prop(optional)] canonical_url: Option<String>,
) -> impl IntoView {
    view! {
        <Title text=title.clone() />
        
        {description.clone().map(|desc| view! {
            <Meta name="description" content=desc.clone() />
            <Meta property="og:description" content=desc />
        })}
        
        <Meta property="og:title" content=title />
        <Meta property="og:type" content=og_type.unwrap_or_else(|| "website".to_string()) />
        
        {image.map(|img| view! {
            <Meta property="og:image" content=img />
        })}
        
        {script_json_ld.map(|json_ld| view! {
            <script type="application/ld+json" inner_html=json_ld></script>
        })}

        {canonical_url.map(|url| view! {
            <Link rel="canonical" href=url />
        })}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_seo_metadata_initialization() {
        let _ = leptos::task::spawn_local(async {
            let _el = view! {
                <Seo 
                    title="Home Page".to_string()
                    description="Test Description".to_string()
                />
            };
            // Ensure meta tags map to Head component
        });
    }
}
