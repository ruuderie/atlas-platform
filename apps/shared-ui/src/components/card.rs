use leptos::prelude::*;

#[component]
pub fn Card(
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <div class=format!("card {}", class)>
            {children()}
        </div>
    }
}
