//! Status pill — typed tone variants for WO / project / occupancy chips.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusPillTone {
    Ok,
    Warn,
    Danger,
    Neutral,
    Info,
}

impl StatusPillTone {
    pub const fn class(self) -> &'static str {
        match self {
            Self::Ok => "status-pill status-pill--ok",
            Self::Warn => "status-pill status-pill--warn",
            Self::Danger => "status-pill status-pill--danger",
            Self::Neutral => "status-pill status-pill--neutral",
            Self::Info => "status-pill status-pill--info",
        }
    }
}

#[component]
pub fn StatusPill(
    #[prop(into)] label: String,
    #[prop(default = StatusPillTone::Neutral)] tone: StatusPillTone,
) -> impl IntoView {
    view! {
        <span class=tone.class()>{label}</span>
    }
}
