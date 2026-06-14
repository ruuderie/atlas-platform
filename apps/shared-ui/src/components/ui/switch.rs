use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::tw_merge;

mod components {
    use super::*;
    clx! {SwitchLabel, span, "text-sm font-medium"}
}

pub use components::*;

#[component]
pub fn Switch(
    #[prop(optional, into)] id: String,
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] on_checked_change: Option<Callback<bool>>,
    #[prop(into, optional, default = "Toggle switch".to_string())] aria_label: String,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] thumb_class: String,
    #[prop(into, optional, default = "default".to_string())] variant: String,
) -> impl IntoView {
    // Internal state for self-controlled fallback when no callback is present.
    let is_checked = RwSignal::new(checked.get_untracked());

    // Sync external changes into internal state
    Effect::new(move |_| {
        let val = checked.get();
        is_checked.set(val);
    });

    let state = move || if is_checked.get() { "checked" } else { "unchecked" };

    let variant_for_class = variant.clone();
    let variant_for_thumb = variant.clone();

    let track_class = move || {
        let base_class = if variant_for_class == "compact" {
            "inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-[var(--green)] data-[state=unchecked]:bg-[rgba(255,255,255,0.1)]"
        } else {
            "inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input"
        };
        tw_merge!(base_class, class.clone())
    };

    let inner_thumb_class = move || {
        let base_thumb = if variant_for_thumb == "compact" {
            "block rounded-full ring-0 shadow-lg transition-transform pointer-events-none size-3.5 bg-white data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
        } else {
            "block rounded-full ring-0 shadow-lg transition-transform pointer-events-none size-5 bg-background data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0"
        };
        tw_merge!(base_thumb, thumb_class.clone())
    };

    view! {
        <button
            data-name="Switch"
            id=id
            type="button"
            role="switch"
            aria-checked=move || is_checked.get().to_string()
            aria-label=aria_label
            data-state=state
            class=track_class
            disabled=move || disabled.get()
            on:click=move |_| {
                if !disabled.get() {
                    let next_val = !is_checked.get();
                    if let Some(callback) = on_checked_change {
                        callback.run(next_val);
                    } else {
                        // self-controlled fallback
                        is_checked.set(next_val);
                    }
                }
            }
        >
            <span
                data-state=state
                class=inner_thumb_class
            />
        </button>
    }
}