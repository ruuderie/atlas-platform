use leptos::prelude::*;
use super::*;

#[component]
pub fn WebformsTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let webforms_res = Resource::new(move || refresh.get(), |_| get_webforms());

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h3 class="text-xl font-bold text-on-surface">"Lead Capture & Origination Schemas"</h3>
                    <p class="text-sm text-on-surface-variant">"Manage multi-step form sequences mapped into the JSON layout blocks."</p>
                </div>
            </div>

            <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline p-6">"QUERYING_DB..."</div> }>
                {move || match webforms_res.get() {
                    Some(Ok(items)) => {
                        if items.is_empty() {
                            view! {
                                <div class="bg-surface-container border border-outline-variant/30 p-8 text-center mt-6">
                                    <span class="material-symbols-outlined text-4xl text-primary mb-4 block">"view_list"</span>
                                    <p class="text-on-surface-variant max-w-lg mx-auto">
                                        "No active webforms found for this tenant. Create landing page forms to list them here."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="bg-surface-container overflow-hidden border border-outline-variant/30 hidden md:block">
                                    <table class="w-full text-left border-collapse">
                                        <thead>
                                            <tr class="bg-surface-container-high border-b border-outline-variant/30 text-xs tracking-wider uppercase text-on-surface-variant jetbrains">
                                                <th class="px-6 py-4 font-medium">"Form ID (Slug)"</th>
                                                <th class="px-6 py-4 font-medium">"Name"</th>
                                                <th class="px-6 py-4 font-medium">"Description"</th>
                                                <th class="px-6 py-4 font-medium">"Integrations"</th>
                                                <th class="px-6 py-4 font-medium text-right">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-outline-variant/30">
                                            {items.into_iter().map(|form| {
                                                let desc = form.description.unwrap_or_default();
                                                view! {
                                                    <tr class="hover:bg-surface-container-high/50 transition-colors">
                                                        <td class="px-6 py-4 jetbrains text-xs text-primary font-bold">{form.slug}</td>
                                                        <td class="px-6 py-4 text-sm font-medium">{form.name}</td>
                                                        <td class="px-6 py-4 text-sm text-on-surface-variant truncate max-w-[200px]">{desc}</td>
                                                        <td class="px-6 py-4">
                                                            {if form.webhook_url.is_some() {
                                                                view! { <span class="bg-primary/10 text-primary px-2 py-1 text-xs font-bold rounded">"Webhook Active"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="bg-outline-variant/15 text-outline px-2 py-1 text-xs font-bold rounded">"Local Only"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="px-6 py-4 text-right">
                                                            <button class="text-primary hover:underline text-xs jetbrains font-bold uppercase tracking-widest mr-4">"EDIT JSON"</button>
                                                            <button class="text-error hover:underline text-xs jetbrains font-bold uppercase tracking-widest">"DELETE"</button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }
                    Some(Err(_)) => view! { <div class="text-error jetbrains text-sm p-6">"ERR_NO_DATA"</div> }.into_any(),
                    None => view! { <div class="hidden" /> }.into_any(),
                }}
            </Transition>
        </div>
    }
}

#[component]
pub fn PageHeaderTable() -> impl IntoView {
    use crate::components::dynamic_header::get_all_page_headers;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let headers_resource = Resource::new(move || refresh.get(), |_| get_all_page_headers());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = headers_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Route"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Badge"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(headers) => headers.into_iter().map(|h| {
                            let h_clone = h.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 font-bold text-outline">{h.route_path.clone()}</td>
                                <td class="py-4 px-4 text-outline-variant">{h.badge_text.clone().unwrap_or_default()}</td>
                                <td class="py-4 px-4 text-on-surface">{h.title.clone()}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::PageHeader(Some(h_clone.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                    </div>
                                </td>
                            </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn ResumeProfileTable() -> impl IntoView {
    use crate::resume_engine::{delete_resume_profile, download_resume, get_entry_collections};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = Resource::new(move || refresh.get(), |_| get_entry_collections());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING..."</div> }>
            {move || {
                let res = items_res.get();
                view! {
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b-2 border-outline-variant/30">
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"ID"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"PROFILE NAME"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"ACTIONS"</th>
                    </tr>
                </thead>
                <tbody class="jetbrains text-sm">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| {
                            let id_val = item.id;
                            let clone_item = item.clone();
                            view! {
                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                    <td class="py-4 text-outline-variant font-medium">{id_val.to_string()}</td>
                                    <td class="py-4 font-bold text-on-surface">{item.name.clone()}</td>
                                    <td class="py-4 text-right space-x-4">
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(bytes) = download_resume(id_val).await {
                                                        use web_sys::js_sys::{Array, Uint8Array};
                                                        use web_sys::{Blob, BlobPropertyBag, Url};

                                                        let uint8_arr = Uint8Array::from(bytes.as_slice());
                                                        let parts = Array::new();
                                                        parts.push(&uint8_arr);

                                                        let props = BlobPropertyBag::new();
                                                        props.set_type("application/pdf");

                                                        if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &props) {
                                                            if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                                                if let Some(window) = web_sys::window() {
                                                                    let _ = window.open_with_url_and_target(&url, "_blank");
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                            class="text-primary hover:text-primary-container font-medium tracking-wide"
                                        >"[PREVIEW]"</button>
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(bytes) = download_resume(id_val).await {
                                                        use web_sys::js_sys::{Array, Uint8Array};
                                                        use web_sys::{Blob, BlobPropertyBag, Url};

                                                        let uint8_arr = Uint8Array::from(bytes.as_slice());
                                                        let parts = Array::new();
                                                        parts.push(&uint8_arr);

                                                        let props = BlobPropertyBag::new();
                                                        props.set_type("application/pdf");

                                                        if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &props) {
                                                            if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                                                let document = web_sys::window().unwrap().document().unwrap();
                                                                if let Ok(a) = document.create_element("a") {
                                                                    let _ = a.set_attribute("href", &url);
                                                                    let _ = a.set_attribute("download", &format!("Profile_{}_Resume.pdf", id_val));
                                                                    use web_sys::wasm_bindgen::JsCast;
                                                                    let html_a = a.unchecked_into::<web_sys::HtmlElement>();
                                                                    html_a.click();
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                            class="text-primary hover:text-primary-container font-medium tracking-wide"
                                        >"[DOWNLOAD]"</button>
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Profile(Some(clone_item.clone()))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    let _ = delete_resume_profile(id_val).await;
                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                });
                                            }
                                            class="text-error hover:text-error/80 font-medium tracking-wide"
                                        >"[DEL]"</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn BaseResumeEntryTable() -> impl IntoView {
    use crate::resume_engine::{delete_base_entry, get_all_base_entries, ResumeCategory};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = Resource::new(move || refresh.get(), |_| get_all_base_entries());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING DATA..."</div> }>
            {move || match ResourceState::from(items_res.get()) {
                ResourceState::Ready(items) => {
                    if items.is_empty() {
                        view! { <div class="py-8 text-center text-outline-variant">"NO ENTRIES IN DATABASE"</div> }.into_any()
                    } else {
                        let categories = vec![
                            ResumeCategory::Work,
                            ResumeCategory::Education,
                            ResumeCategory::Certification,
                            ResumeCategory::Project,
                            ResumeCategory::Skill,
                            ResumeCategory::Language,
                            ResumeCategory::Volunteer,
                            ResumeCategory::Extracurricular,
                            ResumeCategory::Hobby,
                        ];

                        categories.into_iter().map(|cat| {
                            let cat_items: Vec<_> = items.iter().filter(|i| i.category == cat).cloned().collect();
                            if cat_items.is_empty() {
                                view! { <div class="hidden"></div> }.into_any()
                            } else {
                                let category_str = cat.to_string();
                                view! {
                                    <div class="mb-12">
                                        <div class="flex justify-between items-center mb-4">
                                            <div class="inline-block bg-secondary-container/20 px-3 py-1 border border-secondary/30">
                                                <span class="font-label text-[0.6875rem] text-secondary font-bold tracking-tighter uppercase">{category_str.clone()} " ENTRIES"</span>
                                            </div>
                                            <button
                                                on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::BaseEntry(None, Some(cat)))
                                                class="bg-surface-container-high hover:bg-surface-container-highest text-primary px-3 py-1 text-xs font-bold font-label uppercase transition-colors border border-outline-variant/30 flex items-center gap-2"
                                            >
                                                <span class="material-symbols-outlined text-[0.8rem]">"add"</span>
                                                {format!("NEW {}", category_str.clone())}
                                            </button>
                                        </div>
                                        <table class="w-full text-left border-collapse">
                                            <thead>
                                                <tr class="border-b-2 border-outline-variant/30">
                                                    <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline w-16">"ID"</th>
                                                    <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"TITLE"</th>
                                                    <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"ACTIONS"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="jetbrains text-sm">
                                                {cat_items.into_iter().map(|item| {
                                                    let id_val = item.id;
                                                    let clone_item = item.clone();
                                                    view! {
                                                        <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                                            <td class="py-4 text-outline-variant font-medium">{id_val.to_string()}</td>
                                                            <td class="py-4 font-bold text-on-surface truncate">{item.title.clone()}</td>
                                                            <td class="py-4 text-right space-x-4">
                                                                 <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::BaseEntry(Some(clone_item.clone()), Some(clone_item.category))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                                                <button
                                                                    on:click=move |_| {
                                                                        leptos::task::spawn_local(async move {
                                                                            let _ = delete_base_entry(id_val).await;
                                                                            set_refresh.set(refresh.get_untracked() + 1);
                                                                        });
                                                                    }
                                                                    class="text-error hover:text-error/80 font-medium tracking-wide"
                                                                >"[DEL]"</button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }).collect::<Vec<_>>().into_any()
                    }
                },
                ResourceState::Loading => view! { <div class="hidden"></div> }.into_any(),
                ResourceState::Error(_) => view! { <div class="py-8 text-center text-error">"ERR_NO_DATA"</div> }.into_any(),
            }}
        </Transition>
    }
}

#[component]
pub fn LeadOptionTable() -> impl IntoView {
    use crate::pages::landing::{delete_lead_option, get_all_lead_options};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();

    let items_res = Resource::new(move || refresh.get(), |_| get_all_lead_options());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"LOADING DATA..."</div> }>
            {move || {
                let res = items_res.get();
                view! {
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b-2 border-outline-variant/30">
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"ID"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Order"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Key"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Label"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline">"Status"</th>
                        <th class="py-4 font-label text-[0.65rem] uppercase tracking-widest text-outline text-right">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="jetbrains text-sm">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| {
                            let id_val = item.id;
                            let clone_item = item.clone();
                            view! {
                                <tr class="border-b border-outline-variant/10 hover:bg-surface-container/50 transition-colors">
                                    <td class="py-4 text-outline-variant font-medium">{id_val.to_string()}</td>
                                    <td class="py-4 text-on-surface font-medium">{item.display_order}</td>
                                    <td class="py-4 text-on-surface font-mono text-xs">{item.value_key.clone()}</td>
                                    <td class="py-4 font-bold text-on-surface truncate">{item.label.clone()}</td>
                                    <td class="py-4 font-medium">
                                        <div class="inline-flex items-center gap-2">
                                            <div class="w-1.5 h-1.5 rounded-full" style=if item.is_active { "background-color: #4ade80" } else { "background-color: #f87171" }></div>
                                            {if item.is_active { "ACTIVE" } else { "INACTIVE" }}
                                        </div>
                                    </td>
                                    <td class="py-4 text-right space-x-4">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::LeadOption(Some(clone_item.clone()))) class="text-secondary hover:text-on-secondary-fixed-variant font-medium tracking-wide">"[EDIT]"</button>
                                        <button
                                            on:click=move |_| {
                                                leptos::task::spawn_local(async move {
                                                    let _ = delete_lead_option(id_val).await;
                                                    set_refresh.set(refresh.get_untracked() + 1);
                                                });
                                            }
                                            class="text-error hover:text-error/80 font-medium tracking-wide"
                                        >"[DEL]"</button>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="6" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn PostTable() -> impl IntoView {
    use crate::pages::blog::get_posts;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let posts_resource = Resource::new(move || refresh.get(), |_| get_posts());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = posts_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"ID"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Slug"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(posts) => posts.into_iter().map(|p| {
                            let p_clone = p.clone();
                            let del_id = p.id.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">"#" {p.id.clone()}</td>
                                <td class="py-4 px-4 font-bold text-outline">{p.subtitle.clone().unwrap_or_default()}</td>
                                <td class="py-4 px-4 text-primary font-bold">{p.title.clone()}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(ModalState::Post(Some(p_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let target_id = del_id.clone();
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = crate::pages::blog::delete_post(target_id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                            }
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn PasskeyTable() -> impl IntoView {
    use crate::auth::get_users;
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let users_resource = Resource::new(move || refresh.get(), |_| get_users());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = users_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"ID"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Username"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Created At"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(users) => users.into_iter().map(|u| view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 text-outline-variant">"#" {u.id}</td>
                                <td class="py-4 px-4 font-bold text-primary">{u.username}</td>
                                <td class="py-4 px-4 text-outline">{u.created_at}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| {
                                                let id = u.id;
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = crate::auth::delete_user(id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Revoke"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn LandingPageTable() -> impl IntoView {
    use crate::pages::dynamic_landing::{delete_landing_page, get_all_landing_pages};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let pages_resource = Resource::new(move || refresh.get(), |_| get_all_landing_pages());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = pages_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Slug"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(pages) => pages.into_iter().map(|p| {
                            let p_clone = p.clone();
                            let p_clone_2 = p.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 font-bold text-primary">"/" {p.slug}</td>
                                <td class="py-4 px-4 text-outline">{p.title}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::LandingPage(Some(p_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let target_slug = p_clone_2.slug.clone();
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = delete_landing_page(target_slug).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="3" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn NavTable() -> impl IntoView {
    use crate::components::nav::{delete_nav_item, get_all_nav_items};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let nav_resource = Resource::new(move || refresh.get(), |_| get_all_nav_items());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = nav_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Label"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Binding"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 px-4 font-medium">"#" {n.id.to_string()}</td>
                                <td class="py-4 px-4 font-bold text-primary">
                                    {if let Some(_pid) = n.parent_id { format!("↳ {}", n.label) } else { n.label.clone() }}
                                </td>
                                <td class="py-4 px-4 text-outline">{n.href.clone().unwrap_or_default()}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::NavItem(Some(n_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let id = n.id;
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = delete_nav_item(id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn FooterTable() -> impl IntoView {
    use crate::components::footer::{delete_footer_item, get_all_footer_items};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let footer_resource = Resource::new(move || refresh.get(), |_| get_all_footer_items());

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = footer_resource.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead>
                    <tr class="text-outline border-b border-outline-variant/30">
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                        <th class="py-4 px-4 font-bold text-primary">"Label"</th>
                        <th class="py-4 px-4 text-outline">"Link / Dropdown"</th>
                        <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|n| {
                            let n_clone = n.clone();
                            view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{n.display_order}</td>
                                <td class="py-4 font-bold text-primary">
                                    {n.label.clone()}
                                </td>
                                <td class="py-4 px-4 text-outline">{n.href.unwrap_or_else(|| "DROPDOWN [null]".to_string())}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button
                                            on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::FooterItem(Some(n_clone.clone())))
                                            class="text-secondary hover:underline uppercase text-xs"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| {
                                                let id = n.id;
                                                leptos::task::spawn_local(async move {
                                                    if let Ok(_) = delete_footer_item(id).await {
                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                    }
                                                });
                                            }
                                            class="text-error hover:underline uppercase text-xs"
                                        >
                                            "Drop"
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn ServiceTable() -> impl IntoView {
    use crate::b2b::{delete_service, get_services};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let data_res = Resource::new(move || refresh.get(), |_| get_services(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = data_res.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 px-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Service(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_service(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn CaseStudyTable() -> impl IntoView {
    use crate::b2b::{delete_case_study, get_case_studies};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let data_res = Resource::new(move || refresh.get(), |_| get_case_studies(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = data_res.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Client"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 font-bold text-primary">{item.client_name}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::CaseStudy(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_case_study(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}

#[component]
pub fn HighlightTable() -> impl IntoView {
    use crate::b2b::{delete_highlight, get_highlights};
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let set_modal_state =
        expect_context::<WriteSignal<crate::components::admin_modal::ModalState>>();
    let data_res = Resource::new(move || refresh.get(), |_| get_highlights(false));

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = data_res.get();
                view! {
            <table class="w-full text-left jetbrains text-sm">
                <thead><tr class="text-outline border-b border-outline-variant/30">
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Weight"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Title"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Visible"</th>
                    <th class="py-4 px-4 font-normal tracking-widest uppercase">"Actions"</th>
                </tr></thead>
                <tbody class="divide-y divide-outline-variant/20">
                    {match ResourceState::from(res) {
                        ResourceState::Ready(items) => items.into_iter().map(|item| { let c = item.clone(); view! {
                            <tr class="hover:bg-surface-container-high transition-colors group">
                                <td class="py-4 px-4 text-outline-variant">{item.display_order}</td>
                                <td class="py-4 font-bold text-primary">{item.title}</td>
                                <td class="py-4 px-4 text-outline">{if item.is_visible { "YES" } else { "NO" }}</td>
                                <td class="py-4 px-4">
                                    <div class="flex space-x-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button on:click=move |_| set_modal_state.set(crate::components::admin_modal::ModalState::Highlight(Some(c.clone()))) class="text-secondary hover:underline uppercase text-xs">"Edit"</button>
                                        <button on:click=move |_| { let id = item.id; leptos::task::spawn_local(async move { if let Ok(_) = delete_highlight(id).await { set_refresh.set(refresh.get_untracked() + 1); } }); } class="text-error hover:underline uppercase text-xs">"Drop"</button>
                                    </div>
                                </td>
                            </tr>
                        }}).collect::<Vec<_>>().into_any(),
                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                        ResourceState::Error(_) => view! { <tr><td colspan="4" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                    }}
                </tbody>
            </table>
            }.into_any()
            }}
        </Transition>
    }
}
