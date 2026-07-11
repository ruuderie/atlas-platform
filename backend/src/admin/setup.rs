use crate::auth::hash_password;
use crate::entities::{user, user_account};
use chrono::Utc;
use dotenv::dotenv;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
pub async fn create_admin_user_if_not_exists(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing::info!("Email: {:?}", email);
    tracing::info!("Password: {:?}", password);
    tracing::info!(
        "Admin first name: {:?}",
        std::env::var("ADMIN_FIRST_NAME").unwrap()
    );
    tracing::info!(
        "Admin last name: {:?}",
        std::env::var("ADMIN_LAST_NAME").unwrap()
    );
    tracing::info!("Admin phone: {:?}", std::env::var("ADMIN_PHONE").unwrap());
    // Check if the admin user already exists
    let existing_admin = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await?;
    tracing::info!("Existing admin: {:?}", existing_admin);

    if existing_admin.is_none() {
        // Create the admin user
        let hashed_password = hash_password(password)?;
        let new_admin = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set("admin".to_string()),
            first_name: Set(std::env::var("ADMIN_FIRST_NAME").unwrap()),
            last_name: Set(std::env::var("ADMIN_LAST_NAME").unwrap()),
            email: Set(email.to_string()),
            password_hash: Set(hashed_password),
            is_active: Set(true),
            phone: Set(std::env::var("ADMIN_PHONE").unwrap()),
            last_login: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let insert_res = user::Entity::insert(new_admin).exec(db).await?;

        let account = user_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(insert_res.last_insert_id),
            account_id: Set(Uuid::nil()), // Platform-level sentinel; no real account for super admin
            role: Set(user_account::UserRole::PlatformSuperAdmin),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        user_account::Entity::insert(account).exec(db).await?;
        tracing::info!("Admin user created successfully");
    } else {
        println!("Admin Found");
        tracing::info!("Admin user already exists");
    }

    Ok(())
}
