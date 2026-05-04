use shared_ui::components::auth::passkey_login::PasskeyLoginButton;
fn test() {
    let email = leptos::RwSignal::new("".to_string());
    PasskeyLoginButton {
        api_base_url: "".to_string(),
        email,
        on_success: |_| {},
        on_error: |_| {},
    };
}
