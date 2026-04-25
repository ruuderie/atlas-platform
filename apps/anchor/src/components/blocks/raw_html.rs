use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RawHtmlData {
    pub content: String,
}

#[component]
pub fn RawHtmlBlock(data: RawHtmlData) -> impl IntoView {
    view! {
        <div inner_html=data.content></div>
    }
}
