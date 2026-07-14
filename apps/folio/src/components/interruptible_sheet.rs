//! Interruptible right sheet — Escape / overlay dismiss.

use leptos::prelude::*;

#[component]
pub fn InterruptibleSheet(
    open: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(optional, into)] subtitle: Option<Signal<String>>,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional)] footer: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if open.get() {
                    "folio-sheet-overlay folio-sheet-overlay--open"
                } else {
                    "folio-sheet-overlay"
                }
            }
            role="presentation"
            on:click=move |_| open.set(false)
        />
        <aside
            class=move || {
                if open.get() {
                    "folio-sheet folio-sheet--open"
                } else {
                    "folio-sheet"
                }
            }
            role="dialog"
            aria-modal="true"
            aria-labelledby="folio-sheet-title"
            tabindex="0"
            on:click=move |ev| ev.stop_propagation()
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    open.set(false);
                }
            }
        >
            <header class="folio-sheet__header">
                <div class="folio-sheet__titles">
                    <h2 id="folio-sheet-title" class="folio-sheet__title">{move || title.get()}</h2>
                    {subtitle.map(|sub| {
                        view! { <p class="folio-sheet__subtitle">{move || sub.get()}</p> }
                    })}
                </div>
                <button
                    type="button"
                    class="folio-sheet__close press"
                    aria-label="Close"
                    on:click=move |_| open.set(false)
                >
                    <span class="material-symbols-outlined" aria-hidden="true">"close"</span>
                </button>
            </header>
            <div class="folio-sheet__body">
                {children.map(|c| c())}
            </div>
            {footer.map(|f| view! { <footer class="folio-sheet__footer">{f()}</footer> })}
        </aside>
    }
}
