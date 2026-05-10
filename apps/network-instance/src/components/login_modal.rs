use leptos::prelude::*;
use shared_ui::components::auth::atlas_login_panel::AtlasLoginPanel;

#[component]
pub fn LoginModal(
    /// Triggered when authentication succeeds so the parent can refresh state
    #[prop(into)] on_success: Callback<(), ()>,
    /// Controls modal visibility
    #[prop(into)] is_open: Signal<bool>,
    /// Callback to close the modal
    #[prop(into)] on_close: Callback<(), ()>,
) -> impl IntoView {
    let handle_authenticated = Callback::new(move |_: ()| {
        on_success.run(());
    });

    view! {
        <Show when=move || is_open.get() fallback=move || view! { <span/> }>
            <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
                <div class="bg-white rounded-2xl shadow-premium w-full max-w-md overflow-hidden relative animate-slide-up">
                    <button
                        class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface transition-colors"
                        on:click=move |_| on_close.run(())
                    >
                        <span class="material-symbols-outlined">"close"</span>
                    </button>

                    <div class="p-8">
                        <div class="mb-6 text-center">
                            <h2 class="text-2xl font-headline font-extrabold text-[#004289] mb-2">
                                "Welcome Back"
                            </h2>
                            <p class="text-on-surface-variant text-sm">
                                "Sign in to manage your account and alerts."
                            </p>
                        </div>

                        // Shared login panel handles passkey + magic link flows
                        <AtlasLoginPanel
                            app_title="NETWORK"
                            on_authenticated=handle_authenticated
                        />
                    </div>
                </div>
            </div>
        </Show>
    }
}

