use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into)] open: Signal<bool>,
    on_close: Callback<()>,
    #[prop(optional, into)] title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="modal-backdrop" style:display=move || if open.get() { "flex" } else { "none" }>
            <div class="modal-content card">
                <div class="modal-header">
                    <h3>{title.clone()}</h3>
                    <button class="close-btn" on:click=move |_| on_close.run(())>"✕"</button>
                </div>
                <div class="modal-body">
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Slideover(
    #[prop(into)] open: Signal<bool>,
    on_close: Callback<()>,
    #[prop(optional, into)] title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="modal-backdrop" style:display=move || if open.get() { "flex" } else { "none" }>
            <div class="slideover-content">
                <div class="slideover-header">
                    <h3>{title.clone()}</h3>
                    <button class="close-btn" on:click=move |_| on_close.run(())>"✕"</button>
                </div>
                <div class="slideover-body">
                    {children()}
                </div>
            </div>
        </div>
    }
}
