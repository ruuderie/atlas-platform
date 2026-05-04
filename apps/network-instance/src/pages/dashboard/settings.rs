use leptos::prelude::*;
use crate::auth::AuthContext;
use shared_ui::components::auth::passkey_manager::ManagePasskeys;

#[component]
pub fn DashboardSettings() -> impl IntoView {
    let _auth = use_context::<AuthContext>().expect("AuthContext missing");

    view! {
        <div class="px-8 py-10 max-w-4xl mx-auto">
            <h1 class="text-3xl font-extrabold text-on-surface font-headline mb-6">"Account Settings"</h1>
            <p class="text-on-surface-variant font-medium mb-10">
                "Manage your authentication methods and security settings."
            </p>
            
            {move || {
                // Settings page is gated, so we can render ManagePasskeys
                view! {
                    <ManagePasskeys 
                        api_base_url=Signal::derive(|| format!("{}/api/passkeys", crate::get_api_base_url()))
                        auth_token="".to_string()
                    />
                }.into_any()
            }}
        </div>
    }
}
