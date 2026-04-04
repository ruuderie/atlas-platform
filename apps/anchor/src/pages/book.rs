use leptos::*;
use leptos_meta::{Meta, Title};

use crate::pages::landing::get_site_settings;

#[component]
pub fn BookDiscovery() -> impl IntoView {
    let settings_res = create_resource(|| (), |_| get_site_settings());

    view! {
        <Title text="Book Discovery | Anchor" />
        <Meta name="description" content="Schedule a discovery call with Ruud Salym Erie to discuss technical architecture, software engineering, and infrastructure modernization."/>
        <Meta property="og:title" content="Book Discovery | Anchor"/>
        <Meta property="og:description" content="Schedule a discovery call with Ruud Salym Erie to discuss technical architecture, software engineering, and infrastructure modernization."/>
        <main class="min-h-screen pt-32 pb-24 px-6 md:px-12 max-w-7xl mx-auto flex flex-col items-center">

            <div class="max-w-3xl text-center mb-12 flex flex-col items-center">
                <crate::components::dynamic_header::DynamicPageHeader route_path="/book".to_string() badge_color="primary".to_string() />
            </div>

            <Transition fallback=move || view! { <div class="text-outline jetbrains">"Loading calendar..."</div> }>
                {move || match settings_res.get() {
                    Some(Ok(settings)) => {
                        let bu = settings.booking_url;
                        if bu.is_empty() {
                            view! { <div class="text-error bg-error-container text-on-error-container p-6 w-full max-w-2xl text-center">"Booking is currently unavailable. Please reach out via email."</div> }.into_view()
                        } else {
                            view! {
                                <div class="w-full max-w-4xl bg-surface-container-low rounded-3xl overflow-hidden border border-outline-variant/30 shadow-xl h-[700px]">
                                    <iframe
                                        src=bu
                                        width="100%"
                                        height="100%"
                                        frameborder="0"
                                        class="w-full h-full"
                                    ></iframe>
                                </div>
                            }.into_view()
                        }
                    },
                    _ => view! { <div class="text-error">"Failed to load booking configuration."</div> }.into_view()
                }}
            </Transition>
        </main>
    }
}
