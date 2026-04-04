use leptos::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SmtpConfig {
    pub smtp_host: String,
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
    pub use lettre::message::{header, MultiPart, SinglePart};
    pub use lettre::transport::smtp::authentication::Credentials;
    pub use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
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
        smtp_host: "".into(),
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
                "smtp_host" => config.smtp_host = value,
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

    sqlx::query("INSERT INTO system_secrets (key, value) VALUES ('smtp_host', $1) ON CONFLICT (key) DO UPDATE SET value = $1").bind(host).execute(&state.pool).await?;
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

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let mut host = String::new();
    let mut port = 587;
    let mut username = String::new();
    let mut token = String::new();
    let mut from = String::new();

    if let Ok(rows) = sqlx::query("SELECT key, value FROM system_secrets WHERE key LIKE 'smtp_%'")
        .fetch_all(&state.pool)
        .await
    {
        for row in rows {
            let key: String = row.get("key");
            let value: String = row.get("value");
            match key.as_str() {
                "smtp_host" => host = value,
                "smtp_port" => port = value.parse().unwrap_or(587),
                "smtp_username" => username = value,
                "smtp_token" => token = value,
                "smtp_from" => from = value,
                _ => {}
            }
        }
    }

    if host.is_empty() || token.is_empty() {
        println!(
            "SMTP is not fully configured in system_secrets. Email to {} aborted.",
            to_email
        );
        return Ok(());
    }

    let email = Message::builder()
        .from(
            from.parse()
                .unwrap_or_else(|_| "admin@ruuderie.com".parse().unwrap()),
        )
        .to(to_email
            .parse()
            .unwrap_or_else(|_| "admin@ruuderie.com".parse().unwrap()))
        .subject(&subject)
        .multipart(
            MultiPart::alternative().singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(body_html),
            ),
        )
        .unwrap();

    let creds = Credentials::new(username, token);

    let mailer: AsyncSmtpTransport<Tokio1Executor> = if port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .unwrap()
            .port(port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .unwrap()
            .port(port)
            .credentials(creds)
            .build()
    };

    match mailer.send(email).await {
        Ok(_) => {
            println!("Email successfully sent to {}", to_email);
            Ok(())
        }
        Err(e) => {
            println!("Failed to send email to {}: {:?}", to_email, e);
            Err(ServerFnError::ServerError(
                "Failed to send email over SMTP.".into(),
            ))
        }
    }
}
