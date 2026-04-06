use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use crate::api::models::{ListingCreate, PlatformAppModel, UserInfo};
use crate::api::listings::create_listing;
use crate::app::GlobalToast;
use std::collections::HashMap;

#[component]
pub fn ListingCreate() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<GlobalToast>().expect("toast");
    let dirs_res = use_context::<LocalResource<Vec<PlatformAppModel>>>().expect("dirs context");
    let user_ctx = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");

    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (listing_type, set_listing_type) = signal(String::new());
    let (directory_id, set_directory_id) = signal(String::new());
    let (is_submitting, set_is_submitting) = signal(false);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_submitting.set(true);

        let payload = ListingCreate {
            title: title.get(),
            description: description.get(),
            network_id: directory_id.get(),
            profile_id: user_ctx.get().map(|u| u.id).unwrap_or_default(),
            category_id: None,
            listing_type: Some(listing_type.get()),
            price: None,
            price_type: None,
            country: None,
            state: None,
            city: None,
            neighborhood: None,
            latitude: None,
            longitude: None,
            additional_info: None,
            is_featured: Some(false),
            is_based_on_template: Some(false),
            based_on_template_id: None,
            is_ad_placement: Some(false),
            is_active: Some(true),
            slug: None,
        };

        let nav = navigate.clone();
        leptos::task::spawn_local(async move {
            match create_listing(payload).await {
                Ok(_) => {
                    toast.message.set(Some("Listing created successfully".to_string()));
                    nav("/listings", Default::default());
                }
                Err(e) => {
                    toast.message.set(Some(format!("Failed to create: {}", e)));
                }
            }
            set_is_submitting.set(false);
        });
    };

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/network/listings" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                <h2 class="text-3xl font-bold tracking-tight text-foreground">"Create Listing"</h2>
                <p class="text-muted-foreground mt-2">"Publish a new listing and assign it to a network directory."</p>
            </header>
            
            <Card class="p-8 bg-card border border-border shadow-sm".to_string()>
                <form class="space-y-6" on:submit=handle_submit>
                    <div class="space-y-2 flex flex-col">
                        <label class="text-sm font-medium text-foreground">"Listing Title"</label>
                        <input
                            type="text"
                            required
                            class="flex h-10 w-full rounded-md border border-border bg-transparent px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent placeholder:text-muted-foreground"
                            placeholder="e.g. Modern Web Development Services"
                            prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="space-y-2 flex flex-col">
                        <label class="text-sm font-medium text-foreground">"Description"</label>
                        <textarea
                            required
                            class="flex min-h-[100px] w-full rounded-md border border-border bg-transparent px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent placeholder:text-muted-foreground"
                            placeholder="Describe the listing..."
                            prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                        ></textarea>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2 flex flex-col">
                            <label class="text-sm font-medium text-foreground">"Listing Type"</label>
                            <select
                                required
                                class="flex h-10 w-full rounded-md border border-border bg-transparent px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
                                on:change=move |ev| set_listing_type.set(event_target_value(&ev))
                            >
                                <option value="" disabled selected=move || listing_type.get().is_empty()>"Select Type"</option>
                                <option value="Service">"Service"</option>
                                <option value="Product">"Product"</option>
                                <option value="Event">"Event"</option>
                                <option value="Guide">"Guide"</option>
                                <option value="Real Estate">"Real Estate"</option>
                            </select>
                        </div>
                        
                        <div class="space-y-2 flex flex-col">
                            <label class="text-sm font-medium text-foreground">"Assign to Directory"</label>
                            <select
                                required
                                class="flex h-10 w-full rounded-md border border-border bg-transparent px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
                                on:change=move |ev| set_directory_id.set(event_target_value(&ev))
                            >
                                <option value="" disabled selected=move || directory_id.get().is_empty()>"Select Network"</option>
                                <Suspense fallback=move || view! { <option>"Loading directories..."</option> }>
                                    {move || dirs_res.get().map(|directories| view! {
                                        <For
                                            each=move || directories.clone()
                                            key=|dir| dir.tenant_id.clone()
                                            children=move |dir| {
                                                view! {
                                                    <option value=dir.tenant_id.to_string()>{dir.name.clone()}</option>
                                                }
                                            }
                                        />
                                    })}
                                </Suspense>
                            </select>
                        </div>
                    </div>

                    <div class="flex justify-end gap-4 mt-8 pt-6 border-t border-border">
                        <a href="/network/listings">
                            <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                        </a>
                        <Button variant=ButtonVariant::Default>
                            {move || if is_submitting.get() { "Publishing..." } else { "Create Listing" }}
                        </Button>
                    </div>
                </form>
            </Card>
        </div>
    }
}
