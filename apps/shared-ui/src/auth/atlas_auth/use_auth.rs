use leptos::prelude::*;
use super::state::*;
use super::server_fns::*;

#[derive(Clone)]
pub struct AtlasAuthState {
    pub auth_state:       RwSignal<AuthState>,
    pub use_email:        RwSignal<bool>,
    pub email:            RwSignal<String>,
    pub is_loading:       RwSignal<bool>,
    /// True after a magic link has been successfully dispatched.
    /// Distinct from `error` — this is a positive outcome signal.
    pub magic_link_sent:  RwSignal<bool>,
    /// Set when an error occurs. Never set on success.
    pub error:            RwSignal<Option<String>>,
    pub countdown:        RwSignal<i32>,
    pub from_magic_link:  RwSignal<bool>,
    pub dispatch_login:   Action<(), ()>,
    pub dispatch_logout:  Action<(), ()>,
    #[cfg(any(feature = "ssr", feature = "hydrate"))]
    pub auth_resource:    Resource<Result<bool, ServerFnError>>,
    #[cfg(not(any(feature = "ssr", feature = "hydrate")))]
    pub auth_resource:    LocalResource<Result<bool, ServerFnError>>,
}

pub fn use_atlas_auth() -> AtlasAuthState {
    let auth_state = RwSignal::new(AuthState::Authenticating);
    let use_email = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let is_loading = RwSignal::new(false);
    let error = RwSignal::new(None);
    let magic_link_sent = RwSignal::new(false);
    let countdown = RwSignal::new(0i32);
    let from_magic_link = RwSignal::new(false);

    #[cfg(any(feature = "ssr", feature = "hydrate"))]
    let auth_resource = Resource::new(|| (), |_| check_session());
    #[cfg(not(any(feature = "ssr", feature = "hydrate")))]
    let auth_resource = LocalResource::new(|| check_session());

    let dispatch_login_fn = move |_: &()| {
        let email_val = email.get_untracked();
        // Validate and set loading state SYNCHRONOUSLY before the async block.
        // This disables the button on the very next reactive tick, closing
        // the race window that caused two emails to be sent on a slow network.
        let valid = !email_val.trim().is_empty();
        if valid {
            is_loading.set(true);
            error.set(None);
        } else {
            error.set(Some("Email is required.".to_string()));
        }
        async move {
            if !valid { return; }
            match request_magic_link(email_val).await {
                Ok(_) => {
                    magic_link_sent.set(true);
                    error.set(None);
                    countdown.set(60);
                    #[cfg(feature = "hydrate")]
                    leptos::task::spawn_local(async move {
                        use std::time::Duration;
                        while countdown.get_untracked() > 0 {
                            let (tx, rx) = futures::channel::oneshot::channel::<()>();
                            set_timeout_with_handle(
                                move || { let _ = tx.send(()); },
                                Duration::from_secs(1),
                            ).expect("failed to set timeout");
                            rx.await.unwrap();
                            countdown.update(|c| *c -= 1);
                        }
                    });
                }
                Err(e) => {
                    leptos::logging::error!("Magic link request error: {:?}", e);
                    error.set(Some("Unable to send login link. Please try again.".to_string()));
                }
            }
            is_loading.set(false);
        }
    };

    #[cfg(any(feature = "ssr", feature = "hydrate"))]
    let dispatch_login = Action::new(dispatch_login_fn);
    #[cfg(not(any(feature = "ssr", feature = "hydrate")))]
    let dispatch_login = Action::new_local(dispatch_login_fn);

    let dispatch_logout_fn = move |_: &()| {
        async move {
            let _ = revoke_session().await;
            auth_state.set(AuthState::Unauthenticated);
            // In a real app we might redirect to login, but setting unauthenticated is a good start.
        }
    };

    #[cfg(any(feature = "ssr", feature = "hydrate"))]
    let dispatch_logout = Action::new(dispatch_logout_fn);
    #[cfg(not(any(feature = "ssr", feature = "hydrate")))]
    let dispatch_logout = Action::new_local(dispatch_logout_fn);

    AtlasAuthState {
        auth_state,
        use_email,
        email,
        is_loading,
        magic_link_sent,
        error,
        countdown,
        from_magic_link,
        dispatch_login,
        dispatch_logout,
        auth_resource,
    }
}
