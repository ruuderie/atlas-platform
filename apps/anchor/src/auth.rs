use leptos::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
pub mod ssr {
    pub use webauthn_rs::prelude::*;

    pub fn get_webauthn() -> Webauthn {
        let origin_str =
            std::env::var("RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let id_str = std::env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let rp_origin = Url::parse(&origin_str).expect("Invalid RP_ORIGIN URL");
        let builder = WebauthnBuilder::new(&id_str, &rp_origin)
            .expect("Invalid RP_ID or RP_ORIGIN configuration")
            .rp_name("Anchor");
        builder.build().unwrap()
    }
}

#[server(IsSystemInitialized, "/api")]
pub async fn is_system_initialized() -> Result<bool, ServerFnError> {
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let app_state = match extract::<Extension<AppState>>().await {
        Ok(state) => state,
        Err(_) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let Extension(tenant) = match extract::<Extension<crate::state::TenantContext>>().await {
        Ok(t) => t,
        Err(_) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE tenant_id IS NOT DISTINCT FROM $1")
        .bind(tenant.0)
        .fetch_one(&app_state.pool)
        .await
        .unwrap_or(0);

    Ok(count > 0)
}

#[server(RegisterStart, "/api")]
pub async fn register_start(
    username: String,
    setup_token: Option<String>,
) -> Result<String, ServerFnError> {
    use self::ssr::*;
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let app_state = match extract::<Extension<AppState>>().await {
        Ok(state) => state,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let Extension(tenant) = match extract::<Extension<crate::state::TenantContext>>().await {
        Ok(t) => t,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let user_exists: bool =
        match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND tenant_id IS NOT DISTINCT FROM $2)")
            .bind(&username)
            .bind(tenant.0)
            .fetch_one(&app_state.pool)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
        };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE tenant_id IS NOT DISTINCT FROM $1")
        .bind(tenant.0)
        .fetch_one(&app_state.pool)
        .await
        .unwrap_or(0);

    if count == 0 {
        let expected_token =
            std::env::var("SETUP_TOKEN").unwrap_or_else(|_| "CHANGEME".to_string());
        if setup_token.unwrap_or_default() != expected_token {
            return Err(ServerFnError::ServerError("Invalid setup token.".into()));
        }
    } else if !user_exists {
        if !crate::auth::check_session().await.unwrap_or(false) {
            return Err(ServerFnError::ServerError(
                "Registration locked. Admin already exists.".into(),
            ));
        }
    }

    let user_unique_id = Uuid::new_v4();
    let webauthn = get_webauthn();
    let res = match webauthn.start_passkey_registration(
        user_unique_id.clone(),
        &username,
        &username,
        None,
    ) {
        Ok(r) => r,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let (challenge, reg_state) = res;
    let reg_json = serde_json::to_value(&reg_state).unwrap();

    if let Err(e) = sqlx::query("INSERT INTO auth_challenges (tenant_id, id, challenge_data) VALUES ($1, $2, $3)")
        .bind(tenant.0)
        .bind(user_unique_id)
        .bind(reg_json)
        .execute(&app_state.pool)
        .await
    {
        return Err(ServerFnError::ServerError("Internal System Error".into()));
    }

    let challenge_json = serde_json::to_string(&challenge).unwrap();
    let payload = serde_json::json!({
        "challenge_id": user_unique_id,
        "options": challenge_json
    });

    Ok(payload.to_string())
}

#[server(RegisterFinish, "/api")]
pub async fn register_finish(
    username: String,
    challenge_id: Uuid,
    credential_json: String,
) -> Result<String, ServerFnError> {
    use self::ssr::*;
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let app_state = match extract::<Extension<AppState>>().await {
        Ok(state) => state,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let Extension(tenant) = match extract::<Extension<crate::state::TenantContext>>().await {
        Ok(t) => t,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let challenge_row: (serde_json::Value,) =
        match sqlx::query_as("SELECT challenge_data FROM auth_challenges WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
            .bind(challenge_id)
            .bind(tenant.0)
            .fetch_one(&app_state.pool)
            .await
        {
            Ok(row) => row,
            Err(_) => {
                return Err(ServerFnError::ServerError(
                    "Challenge expired or invalid".into(),
                ))
            }
        };

    let reg_state: PasskeyRegistration = match serde_json::from_value(challenge_row.0) {
        Ok(s) => s,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let credential: RegisterPublicKeyCredential = match serde_json::from_str(&credential_json) {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let webauthn = get_webauthn();
    let passkey = match webauthn.finish_passkey_registration(&credential, &reg_state) {
        Ok(p) => p,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let session_token = Uuid::new_v4().to_string();
    let passkey_json = serde_json::to_value(&passkey).unwrap();

    if let Err(e) =
        sqlx::query("INSERT INTO users (tenant_id, username, passkey, session_token) VALUES ($1, $2, $3, $4)")
            .bind(tenant.0)
            .bind(&username)
            .bind(passkey_json)
            .bind(&session_token)
            .execute(&app_state.pool)
            .await
    {
        return Err(ServerFnError::ServerError("Internal System Error".into()));
    }

    sqlx::query("DELETE FROM auth_challenges WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(challenge_id)
        .bind(tenant.0)
        .execute(&app_state.pool)
        .await
        .ok();

    use leptos_axum::ResponseOptions;
    let response = leptos::expect_context::<ResponseOptions>();
    let header_val = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict",
        session_token
    );
    response.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&header_val).unwrap(),
    );

    Ok("SUCCESS".to_string())
}

#[server(LoginStart, "/api")]
pub async fn login_start(username: String) -> Result<String, ServerFnError> {
    use self::ssr::*;
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let app_state = match extract::<Extension<AppState>>().await {
        Ok(state) => state,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let Extension(tenant) = match extract::<Extension<crate::state::TenantContext>>().await {
        Ok(t) => t,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let user_row: (serde_json::Value,) =
        match sqlx::query_as("SELECT passkey FROM users WHERE username = $1 AND tenant_id IS NOT DISTINCT FROM $2")
            .bind(&username)
            .bind(tenant.0)
            .fetch_one(&app_state.pool)
            .await
        {
            Ok(row) => row,
            Err(_) => return Err(ServerFnError::ServerError("User not found".into())),
        };

    let passkey: Passkey = match serde_json::from_value(user_row.0) {
        Ok(k) => k,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let webauthn = get_webauthn();
    let res = match webauthn.start_passkey_authentication(&[passkey]) {
        Ok(r) => r,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let (challenge, auth_state) = res;
    let auth_id = Uuid::new_v4();

    let auth_json = serde_json::to_value(&auth_state).unwrap();
    if let Err(e) = sqlx::query("INSERT INTO auth_challenges (tenant_id, id, challenge_data) VALUES ($1, $2, $3)")
        .bind(tenant.0)
        .bind(auth_id)
        .bind(auth_json)
        .execute(&app_state.pool)
        .await
    {
        return Err(ServerFnError::ServerError("Internal System Error".into()));
    }

    let challenge_json = serde_json::to_string(&challenge).unwrap();
    let payload = serde_json::json!({
        "challenge_id": auth_id,
        "options": challenge_json
    });

    Ok(payload.to_string())
}

#[server(LoginFinish, "/api")]
pub async fn login_finish(
    username: String,
    challenge_id: Uuid,
    auth_json: String,
) -> Result<String, ServerFnError> {
    use self::ssr::*;
    use crate::state::AppState;
    use axum::Extension;
    use leptos_axum::extract;

    let app_state = match extract::<Extension<AppState>>().await {
        Ok(state) => state,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let Extension(tenant) = match extract::<Extension<crate::state::TenantContext>>().await {
        Ok(t) => t,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let _user_row: (serde_json::Value,) =
        match sqlx::query_as("SELECT passkey FROM users WHERE username = $1 AND tenant_id IS NOT DISTINCT FROM $2")
            .bind(&username)
            .bind(tenant.0)
            .fetch_one(&app_state.pool)
            .await
        {
            Ok(row) => row,
            Err(_) => return Err(ServerFnError::ServerError("User not found".into())),
        };

    let challenge_row: (serde_json::Value,) =
        match sqlx::query_as("SELECT challenge_data FROM auth_challenges WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
            .bind(challenge_id)
            .bind(tenant.0)
            .fetch_one(&app_state.pool)
            .await
        {
            Ok(row) => row,
            Err(_) => {
                return Err(ServerFnError::ServerError(
                    "Challenge expired or invalid".into(),
                ))
            }
        };

    let auth_state: PasskeyAuthentication = match serde_json::from_value(challenge_row.0) {
        Ok(s) => s,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };
    let credential: PublicKeyCredential = match serde_json::from_str(&auth_json) {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let webauthn = get_webauthn();
    let _auth_res = match webauthn.finish_passkey_authentication(&credential, &auth_state) {
        Ok(r) => r,
        Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
    };

    let session_token = Uuid::new_v4().to_string();

    if let Err(e) = sqlx::query("UPDATE users SET session_token = $1 WHERE username = $2 AND tenant_id IS NOT DISTINCT FROM $3")
        .bind(&session_token)
        .bind(&username)
        .bind(tenant.0)
        .execute(&app_state.pool)
        .await
    {
        return Err(ServerFnError::ServerError("Internal System Error".into()));
    }

    sqlx::query("DELETE FROM auth_challenges WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(challenge_id)
        .bind(tenant.0)
        .execute(&app_state.pool)
        .await
        .ok();

    use leptos_axum::ResponseOptions;
    let response = leptos::expect_context::<ResponseOptions>();
    let header_val = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict",
        session_token
    );
    response.append_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&header_val).unwrap(),
    );

    Ok("SUCCESS".to_string())
}

#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<bool, ServerFnError> {
    use crate::state::AppState;
    use axum::http::HeaderMap;
    use axum::Extension;
    use axum_extra::extract::cookie::CookieJar;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let cookies = CookieJar::from_headers(&headers);
    let session_cookie = cookies.get("session");

    if let Some(cookie) = session_cookie {
        let app_state = match extract::<Extension<AppState>>().await {
            Ok(state) => state,
            Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
        };
        let Extension(tenant) = match extract::<Extension<crate::state::TenantContext>>().await {
            Ok(t) => t,
            Err(e) => return Err(ServerFnError::ServerError("Internal System Error".into())),
        };
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE session_token = $1 AND tenant_id IS NOT DISTINCT FROM $2)")
                .bind(cookie.value())
                .bind(tenant.0)
                .fetch_one(&app_state.pool)
                .await
                .unwrap_or(false);
        Ok(exists)
    } else {
        Ok(false)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserRecord {
    pub id: i32,
    pub username: String,
    pub created_at: String,
}

#[server(GetUsers, "/api")]
pub async fn get_users() -> Result<Vec<UserRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query("SELECT id, username, to_char(created_at, 'YYYY.MM.DD HH24:MI:SS') as created_at FROM users WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY id ASC")
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;
    let users = rows
        .into_iter()
        .map(|row| UserRecord {
            id: row.get("id"),
            username: row.get("username"),
            created_at: row.get("created_at"),
        })
        .collect();
    Ok(users)
}

#[server(DeleteUser, "/api")]
pub async fn delete_user(id: i32) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM users WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn test_webauthn_builder_initialization_defaults() {
        std::env::remove_var("RP_ORIGIN");
        std::env::remove_var("RP_ID");
        let webauthn = ssr::get_webauthn();
        // Fallbacks are localhost:3000
        assert_eq!(webauthn.get_rp_id(), "localhost");
    }

    #[test]
    fn test_webauthn_builder_initialization_env() {
        std::env::set_var("RP_ORIGIN", "https://anchor.com");
        std::env::set_var("RP_ID", "anchor.com");
        let webauthn = ssr::get_webauthn();
        assert_eq!(webauthn.get_rp_id(), "anchor.com");
    }
}
