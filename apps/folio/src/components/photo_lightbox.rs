//! Photo strip + lightbox for job evidence (ratings, WO detail, project rollup).

use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhotoItem {
    pub id: String,
    pub src: String,
    pub caption: String,
}

#[component]
pub fn PhotoStrip(
    #[prop(into)] photos: Signal<Vec<PhotoItem>>,
    #[prop(optional)] on_add: Option<Callback<()>>,
) -> impl IntoView {
    let lightbox_open = RwSignal::new(false);
    let lightbox_src = RwSignal::new(String::new());
    let lightbox_cap = RwSignal::new(String::new());

    view! {
        <div class="photo-strip">
            <For
                each=move || photos.get()
                key=|p| p.id.clone()
                children=move |p| {
                    let src = p.src.clone();
                    let cap = p.caption.clone();
                    let src_click = src.clone();
                    let cap_click = cap.clone();
                    view! {
                        <button
                            type="button"
                            class="photo-strip__tile press"
                            on:click=move |_| {
                                lightbox_src.set(src_click.clone());
                                lightbox_cap.set(cap_click.clone());
                                lightbox_open.set(true);
                            }
                        >
                            <img src=src alt=cap.clone() loading="lazy"/>
                            <span class="photo-strip__cap">{cap}</span>
                        </button>
                    }
                }
            />
            {on_add.map(|cb| {
                view! {
                    <button
                        type="button"
                        class="photo-strip__add press"
                        on:click=move |_| cb.run(())
                    >
                        <span class="material-symbols-outlined" aria-hidden="true">"add_a_photo"</span>
                        "Add"
                    </button>
                }
            })}
        </div>
        <PhotoLightbox
            open=lightbox_open
            src=Signal::derive(move || lightbox_src.get())
            caption=Signal::derive(move || lightbox_cap.get())
        />
    }
}

#[component]
pub fn PhotoLightbox(
    open: RwSignal<bool>,
    #[prop(into)] src: Signal<String>,
    #[prop(into)] caption: Signal<String>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if open.get() {
                    "photo-lightbox photo-lightbox--open"
                } else {
                    "photo-lightbox"
                }
            }
            role="dialog"
            aria-modal="true"
            aria-label="Photo"
            tabindex="0"
            on:click=move |_| open.set(false)
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    open.set(false);
                }
            }
        >
            <button
                type="button"
                class="photo-lightbox__close press"
                aria-label="Close"
                on:click=move |_| open.set(false)
            >
                <span class="material-symbols-outlined" aria-hidden="true">"close"</span>
            </button>
            <figure class="photo-lightbox__figure" on:click=move |ev| ev.stop_propagation()>
                <img src=move || src.get() alt=move || caption.get()/>
                <figcaption>{move || caption.get()}</figcaption>
            </figure>
        </div>
    }
}
