use sea_orm::{Database};
use sea_orm_migration::prelude::*;
use dotenv::dotenv;
use std::env;
use atlas_backend::migration::Migrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("LOCAL_DATABASE_URL").unwrap_or("postgresql://postgres:postgres@localhost:5433/oplydb".to_string());
    let db = Database::connect(&db_url).await?;
    Migrator::up(&db, None).await?;
    println!("Migrations complete.");
    Ok(())
}
