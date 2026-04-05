use leptos::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SmtpConfig {
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_username: String,
    pub smtp_token: String,
    pub smtp_from: String,
}

#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use crate::auth::check_session;
    pub use axum::Extension;
    pub use leptos_axum::extract;
    pub use sqlx::Row;
}

#[server(GetSmtpConfig, "/api")]
pub async fn get_smtp_config() -> Result<SmtpConfig, ServerFnError> {
    use self::ssr_imports::*;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let mut config = SmtpConfig {
        smtp_server: "".into(),
        smtp_port: "587".into(),
        smtp_username: "".into(),
        smtp_token: "".into(),
        smtp_from: "".into(),
    };

    if let Ok(rows) = sqlx::query("SELECT key, value FROM system_secrets WHERE key LIKE 'smtp_%'")
        .fetch_all(&state.pool)
        .await
    {
        for row in rows {
            let key: String = row.get("key");
            let value: String = row.get("value");
            match key.as_str() {
                "smtp_server" => config.smtp_server = value,
                "smtp_port" => config.smtp_port = value,
                "smtp_username" => config.smtp_username = value,
                "smtp_token" => config.smtp_token = value,
                "smtp_from" => config.smtp_from = value,
                _ => {}
            }
        }
    }

    // Mask the token when returning to the UI to avoid reading real passwords plainly
    if !config.smtp_token.is_empty() {
        config.smtp_token = "********".into();
    }

    Ok(config)
}

#[server(UpdateSmtpConfig, "/api")]
pub async fn update_smtp_config(
    host: String,
    port: String,
    username: String,
    token: String,
    from: String,
) -> Result<(), ServerFnError> {
    use self::ssr_imports::*;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query("INSERT INTO system_secrets (key, value) VALUES ('smtp_server', $1) ON CONFLICT (key) DO UPDATE SET value = $1").bind(host).execute(&state.pool).await?;
    sqlx::query("INSERT INTO system_secrets (key, value) VALUES ('smtp_port', $1) ON CONFLICT (key) DO UPDATE SET value = $1").bind(port).execute(&state.pool).await?;
    sqlx::query("INSERT INTO system_secrets (key, value) VALUES ('smtp_username', $1) ON CONFLICT (key) DO UPDATE SET value = $1").bind(username).execute(&state.pool).await?;

    if !token.is_empty() && token != "********" {
        sqlx::query("INSERT INTO system_secrets (key, value) VALUES ('smtp_token', $1) ON CONFLICT (key) DO UPDATE SET value = $1").bind(token).execute(&state.pool).await?;
    }

    sqlx::query("INSERT INTO system_secrets (key, value) VALUES ('smtp_from', $1) ON CONFLICT (key) DO UPDATE SET value = $1").bind(from).execute(&state.pool).await?;

    Ok(())
}

#[server(SendEmail, "/api")]
pub async fn send_email(
    to_email: String,
    subject: String,
    body_html: String,
) -> Result<(), ServerFnError> {
    use self::ssr_imports::*;

    let tenant_id = std::env::var("TENANT_ID").ok().and_then(|t| uuid::Uuid::parse_str(&t).ok());

    let payload = serde_json::json!({
        "tenant_id": tenant_id,
        "to_email": to_email,
        "subject": subject,
        "body_html": body_html,
    });

    let url = format!("{}/api/communications/email", crate::atlas_client::get_atlas_api_url());
    let client = reqwest::Client::new();
    let res = client.post(&url).json(&payload).send().await;

    match res {
        Ok(r) if r.status().is_success() => {
            println!("Email successfully proxied to platform for {}", to_email);
            Ok(())
        }
        Ok(r) => {
            println!("Failed to proxy email to {}: status {}", to_email, r.status());
            Err(ServerFnError::ServerError(
                "Platform failed to send email.".into(),
            ))
        }
        Err(e) => {
            println!("Failed to proxy email to {}: {:?}", to_email, e);
            Err(ServerFnError::ServerError(
                "Failed to communicate with Platform email API.".into(),
            ))
        }
    }
}
