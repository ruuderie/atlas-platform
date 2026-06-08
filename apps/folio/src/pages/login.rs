use leptos::prelude::*;

/// Magic-link login page.
#[component]
pub fn Login() -> impl IntoView {
    let email    = RwSignal::new(String::new());
    let pending  = RwSignal::new(false);
    let sent     = RwSignal::new(false);
    let error    = RwSignal::new(Option::<String>::None);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if pending.get() { return; }
        pending.set(true);
        error.set(None);
        let e = email.get();
        leptos::task::spawn_local(async move {
            match crate::auth::request_magic_link(e).await {
                Ok(_)  => { sent.set(true); }
                Err(e) => { error.set(Some(e.to_string())); }
            }
            pending.set(false);
        });
    };

    view! {
        <div class="login-page">
            <div class="login-card">
                <div class="login-logo">
                    <span class="logo-text">"Folio"</span>
                    <span class="logo-sub">"Property Management"</span>
                </div>

                {move || sent.get().then(|| view! {
                    <div class="login-sent">
                        <p>"Check your inbox — a login link is on its way."</p>
                    </div>
                })}

                {move || (!sent.get()).then(|| view! {
                    <form on:submit=submit class="login-form">
                        <label class="login-label" for="email">"Email address"</label>
                        <input
                            id="email"
                            type="email"
                            class="login-input"
                            placeholder="you@example.com"
                            required
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))
                        />
                        {move || error.get().map(|e| view! {
                            <p class="login-error">{e}</p>
                        })}
                        <button
                            type="submit"
                            class="login-btn"
                            disabled=move || pending.get()
                        >
                            {move || if pending.get() { "Sending…" } else { "Send login link" }}
                        </button>
                    </form>
                })}
            </div>
        </div>
    }
}
