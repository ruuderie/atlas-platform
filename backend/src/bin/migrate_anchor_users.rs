use sea_orm::{Database, ConnectionTrait, Statement, DatabaseBackend, EntityTrait, ActiveModelTrait, Set, ColumnTrait, QueryFilter};
use uuid::Uuid;
use chrono::Utc;
use std::env;
use atlas_backend::entities::{user, user_account, profile, account, passkey};
use atlas_backend::auth::hash_password;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Connect to the generic DB which contains both Anchor schema (users) and Platform schema right now for UAT
    // Note: Assuming they share the standard database pool or we use the platform db url
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await?;

    println!("Migrating Anchor users to Platform users...");

    // Query from raw anchor users table
    let query_res = db.query_all(Statement::from_string(
        DatabaseBackend::Postgres,
        "SELECT id, username, passkey, created_at, session_token, tenant_id FROM users WHERE passkey IS NOT NULL AND tenant_id IS NOT NULL".to_owned()
    )).await?;

    for row in query_res {
        let username: String = row.try_get("", "username").unwrap_or_default();
        let passkey_json: serde_json::Value = row.try_get("", "passkey").unwrap_or_default();
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("", "created_at").unwrap_or_else(|_| Utc::now());
        let tenant_id_opt: Option<Uuid> = row.try_get("", "tenant_id").unwrap_or(None);

        if tenant_id_opt.is_none() {
            println!("Skipping user {} as they have no tenant_id", username);
            continue;
        }
        let tenant_id = tenant_id_opt.unwrap();

        println!("Migrating user: {} for tenant {}", username, tenant_id);

        // 1. Create or Find User
        let existing_user: Option<user::Model> = user::Entity::find()
            .filter(user::Column::Email.eq(&username))
            .one(&db)
            .await?;

        let user_id = if let Some(u) = existing_user {
            u.id
        } else {
            let u_id = Uuid::new_v4();
            let random_pass = Uuid::new_v4().to_string();
            let hashed_pass = hash_password(&random_pass)?;

            let new_user = user::ActiveModel {
                id: Set(u_id),
                username: Set(username.clone()),
                email: Set(username.clone()), // Anchor usernames act as identifier/email usually
                first_name: Set("Migrated".to_string()),
                last_name: Set("User".to_string()),
                phone: Set("".to_string()),
                password_hash: Set(hashed_pass),
                is_admin: Set(false),
                is_active: Set(true),
                last_login: Set(None),
                created_at: Set(created_at),
                updated_at: Set(Utc::now()),
            };
            new_user.insert(&db).await?;
            u_id
        };

        // 2. Create Passkey record
        let cred_id: Vec<u8> = passkey_json.get("cred_id").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
        let pub_key: Vec<u8> = passkey_json.get("cred").and_then(|v| serde_json::to_vec(v).ok()).unwrap_or_default();
        let sign_count: i32 = passkey_json.get("counter").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let new_passkey = passkey::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            credential_id: Set(cred_id),
            public_key: Set(pub_key),
            sign_count: Set(sign_count),
            name: Set("Migrated Anchor Passkey".to_string()),
            last_used_at: Set(None),
            created_at: Set(created_at),
            updated_at: Set(Utc::now()),
        };
        // ignore errors on duplicate if run multiple times
        let _ = new_passkey.insert(&db).await;

        // 3. Ensure Account Exists for Tenant
        let existing_account: Option<account::Model> = account::Entity::find()
            .filter(account::Column::TenantId.eq(tenant_id))
            .one(&db)
            .await?;
        
        let account_id = if let Some(a) = existing_account {
            a.id
        } else {
            let a_id = Uuid::new_v4();
            let new_account = account::ActiveModel {
                id: Set(a_id),
                tenant_id: Set(tenant_id),
                name: Set(username.clone()),
                is_active: Set(true),
                stripe_customer_id: sea_orm::NotSet,
                stripe_payment_method_id: sea_orm::NotSet,
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            new_account.insert(&db).await?;
            a_id
        };

        // 4. Ensure Profile Exists
        let existing_profile: Option<profile::Model> = profile::Entity::find()
            .filter(profile::Column::TenantId.eq(tenant_id))
            .one(&db)
            .await?;

        let _profile_id = if let Some(p) = existing_profile {
            p.id
        } else {
            let p_id = Uuid::new_v4();
            let new_profile = profile::ActiveModel {
                id: Set(p_id),
                account_id: Set(account_id),
                tenant_id: Set(tenant_id),
                profile_type: Set(profile::ProfileType::Individual),
                display_name: Set(username.clone()),
                contact_info: Set(username.clone()),
                is_active: Set(true),
                service_area_zips: sea_orm::NotSet,
                business_name: Set(None),
                business_address: Set(None),
                business_phone: Set(None),
                business_website: Set(None),
                properties: Set(None),
                additional_info: Set(None),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            new_profile.insert(&db).await?;
            p_id
        };

        // 5. Ensure UserAccount Association
         // Step 6: Create the UserAccount to link user and account
        let existing_assoc: Option<user_account::Model> = user_account::Entity::find()
            .filter(user_account::Column::UserId.eq(user_id))
            .filter(user_account::Column::AccountId.eq(account_id))
            .one(&db)
            .await?;
        
        if existing_assoc.is_none() {
            let new_user_account = user_account::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                account_id: Set(account_id),
                role: Set(user_account::UserRole::Owner),
                created_at: Set(Utc::now()),
                is_active: Set(true),
                updated_at: Set(Utc::now()),
            };
            new_user_account.insert(&db).await?;
        }

        println!("Successfully migrated user {}", username);
    }

    println!("Migration complete.");
    Ok(())
}
