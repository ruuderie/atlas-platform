use leptos::prelude::*;
use shared_ui::components::auth::passkey_manager::ManagePasskeys;
use crate::api::models::UserInfo;
use crate::api::profile::{update_email, update_password};
use crate::app::GlobalToast;

#[component]
pub fn Settings() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set user context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    let new_email = RwSignal::new(String::new());
    let current_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Update email action
    let save_email_action = Action::new_local(move |_: &()| {
        let email = new_email.get();
        let t = toast.clone();
        let u = user.get();
        let su = set_user.clone();
        
        async move {
            if email.is_empty() {
                t.show_toast("Error", "Email is required", "error");
                return;
            }
            if let Some(mut user_info) = u {
                if email == user_info.email {
                    t.show_toast("Info", "Email is already set to this value", "info");
                    return;
                }
                match update_email(email.clone()).await {
                    Ok(_) => {
                        user_info.email = email;
                        su.set(Some(user_info));
                        t.show_toast("Success", "Email updated successfully.", "success");
                    }
                    Err(e) => t.show_toast("Error", &format!("Failed to update email: {}", e), "error"),
                }
            }
        }
    });

    // Update password action
    let save_password_action = Action::new_local(move |_: &()| {
        let current = current_password.get();
        let new_pass = new_password.get();
        let confirm = confirm_password.get();
        let t = toast.clone();

        async move {
            if current.is_empty() || new_pass.is_empty() || confirm.is_empty() {
                t.show_toast("Error", "All password fields are required", "error");
                return;
            }
            if new_pass != confirm {
                t.show_toast("Error", "New passwords do not match", "error");
                return;
            }
            match update_password(current, new_pass).await {
                Ok(_) => {
                    t.show_toast("Success", "Password updated successfully.", "success");
                    current_password.set(String::new());
                    new_password.set(String::new());
                    confirm_password.set(String::new());
                }
                Err(e) => t.show_toast("Error", &format!("Failed to update password: {}", e), "error"),
            }
        }
    });

    view! {
        <div class="max-w-4xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            <header>
                <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-['Inter']">"Account Settings"</h1>
                <p class="text-on-surface-variant text-sm tracking-wide">"Manage your credentials and security preferences."</p>
            </header>

            <Show when=move || user.get().is_some() fallback=move || view! {
                <div class="p-8 rounded-2xl bg-surface-container border border-outline-variant/10 text-error">
                    "You must be logged in to view settings."
                </div>
            }>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    // Left Column: Profile & Password
                    <div class="space-y-8">
                        // Profile Info Section (Email)
                        <section class="p-6 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm">
                            <h2 class="text-lg font-semibold text-on-surface mb-4">"Profile Information"</h2>
                            
                            <div class="mb-4">
                                <label class="block text-sm font-medium text-on-surface-variant mb-1">"Current Email"</label>
                                <div class="p-3 bg-surface-container-highest rounded-lg text-on-surface/70 text-sm">
                                    {move || user.get().map(|u| u.email).unwrap_or_default()}
                                </div>
                            </div>

                            <form on:submit=move |e| { e.prevent_default(); save_email_action.dispatch(()); } class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-on-surface mb-1">"New Email Address"</label>
                                    <input type="email" 
                                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                                        placeholder="Enter new email address"
                                        prop:value=move || new_email.get()
                                        on:input=move |ev| new_email.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div class="flex justify-end">
                                    <button type="submit" class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-bold text-on-primary shadow-lg shadow-primary/20 hover:scale-105 transition-all">
                                        "Update Email"
                                    </button>
                                </div>
                            </form>
                        </section>

                        // Password Section
                        <section class="p-6 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm">
                            <h2 class="text-lg font-semibold text-on-surface mb-4">"Change Password"</h2>
                            
                            <form on:submit=move |e| { e.prevent_default(); save_password_action.dispatch(()); } class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-on-surface mb-1">"Current Password"</label>
                                    <input type="password" 
                                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                                        placeholder="••••••••"
                                        prop:value=move || current_password.get()
                                        on:input=move |ev| current_password.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-on-surface mb-1">"New Password"</label>
                                    <input type="password" 
                                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                                        placeholder="••••••••"
                                        prop:value=move || new_password.get()
                                        on:input=move |ev| new_password.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-on-surface mb-1">"Confirm New Password"</label>
                                    <input type="password" 
                                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                                        placeholder="••••••••"
                                        prop:value=move || confirm_password.get()
                                        on:input=move |ev| confirm_password.set(event_target_value(&ev))
                                        required
                                    />
                                </div>
                                <div class="flex justify-end pt-2">
                                    <button type="submit" class="bg-surface-container-high border border-outline/20 px-4 py-2 rounded-lg text-sm font-bold text-on-surface shadow-sm hover:bg-surface-bright/20 transition-all">
                                        "Update Password"
                                    </button>
                                </div>
                            </form>
                        </section>
                    </div>

                    // Right Column: Passkeys
                    <div>
                        <section class="p-6 rounded-2xl bg-surface-container border border-outline-variant/10 shadow-sm h-full max-h-[600px] overflow-y-auto">
                            <h2 class="text-lg font-semibold text-on-surface mb-4">"Passkeys & Biometrics"</h2>
                            <p class="text-sm text-on-surface-variant mb-6">"Setup hardware keys or device biometrics (like Touch ID or Windows Hello) to sign in without a password."</p>
                            
                            <div class="bg-surface-container-highest rounded-xl border border-outline/10 p-4">
                                <ManagePasskeys 
                                    api_base_url=Signal::derive(move || crate::api::client::api_url("/api/passkeys")) 
                                    auth_token=crate::api::client::get_auth_token().unwrap_or_default()
                                />
                            </div>
                        </section>
                    </div>
                </div>
            </Show>
        </div>
    }
}
