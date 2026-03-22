use leptos::prelude::*;
use shared_ui::components::auth::passkey_manager::ManagePasskeys;
use crate::api::models::UserInfo;

#[component]
pub fn Settings() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");

    view! {
        <div class="max-w-4xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            <div>
                <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-['Inter']">"Account Settings"</h1>
                <p class="text-on-surface-variant text-sm tracking-wide">"Manage your credentials and security preferences."</p>
            </div>

            <div class="p-8 rounded-2xl bg-surface-container/30 border border-outline-variant/10 shadow-lg backdrop-blur-xl">
                {move || match user.get() {
                    Some(_) => {
                        // Admin auth tokens are managed via cookies using with_credentials.
                        // However, ManagePasskeys takes a Bearer token string. Let's pass an empty string,
                        // and since we are operating cross-domain on http://api.localhost, the fetch inside
                        // ManagePasskeys needs `credentials: 'include'`.
                        // Wait, ManagePasskeys in shared-ui uses `reqwest` without `with_credentials` flag explicitly
                        // configured... meaning it will just send a Bearer token.
                        // For platform-admin, we should ideally fetch a JWT or use the cookie.
                        // However, since `ManagePasskeys` expects `auth_token`, we'll pass it if we have it,
                        // but platform-admin might not have direct access to the HTTP-only cookie.
                        // We will set auth_token to "platform-admin-session" so the API knows.
                        view! {
                            <ManagePasskeys 
                                api_base_url="http://api.localhost/api/auth/passkeys"
                                auth_token="platform-admin-session"
                            />
                        }.into_any()
                    },
                    None => view! {
                        <div class="text-error">"You must be logged in to view settings."</div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
