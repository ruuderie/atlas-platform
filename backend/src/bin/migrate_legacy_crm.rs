//! One-time data migration binary: Legacy CRM → Platform Generics (dev + uat only).
//!
//! Usage examples (from backend directory):
//!   cargo run --bin migrate_legacy_crm -- --dry-run --all
//!   cargo run --bin migrate_legacy_crm -- --tenant 35f95f2a-db97-4166-be66-5215654cac84
//!
//! This binary is **not** intended for production. It is the tooling to get dev and uat
//! cleanly onto the new unified model so we can remove legacy CRM dependencies.

use dotenv::dotenv;
use sea_orm::Database;
use std::env;
use uuid::Uuid;

use atlas_backend::services::unification_data_migration::{
    find_tenants_with_legacy_data, migrate_buildwithruud_dev_sample, migrate_known_tenants,
    migrate_tenant_legacy_crm,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();

    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let all_flag = args
        .iter()
        .any(|a| a == "--all" || a == "--all-with-legacy");

    let db_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("LOCAL_DATABASE_URL"))
        .unwrap_or_else(|_| "postgresql://ruud:bD3@ULYzhFcNm@10.42.0.1:5432/atlas_dev".to_string());

    println!("Connecting to database...");
    let db = Database::connect(&db_url).await?;

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "--buildwithruud-sample") {
        let report = migrate_buildwithruud_dev_sample(&db, dry_run).await?;
        println!("{}", report);
        return Ok(());
    }

    if let Some(tenant_arg) = args.iter().find(|a| a.starts_with("--tenant=")) {
        let tenant_str = tenant_arg.split('=').nth(1).unwrap();
        let tenant_id = Uuid::parse_str(tenant_str)?;
        // We don't have the name easily, so use a placeholder
        let report = migrate_tenant_legacy_crm(&db, tenant_id, "unknown-tenant", dry_run).await?;
        println!("{}", report);
        return Ok(());
    }

    if all_flag {
        println!("Discovering tenants with legacy CRM data...");
        let discovered = find_tenants_with_legacy_data(&db).await?;
        println!("Found {} tenants with legacy data.", discovered.len());

        if discovered.is_empty() {
            println!("Nothing to migrate.");
            return Ok(());
        }

        // Convert to the expected slice format
        let tenant_refs: Vec<(Uuid, &str)> = discovered
            .iter()
            .map(|(id, name)| (*id, name.as_str()))
            .collect();

        let reports = migrate_known_tenants(&db, &tenant_refs, dry_run).await?;
        for r in reports {
            println!("{}", r);
            println!("---");
        }
        return Ok(());
    }

    // Default helpful message
    print_help();
    Ok(())
}

fn print_help() {
    println!(
        r#"
Legacy CRM → Platform Generics One-Time Migration (dev/uat only)

Options:
  --dry-run, -n          Simulate only, do not write data
  --all                  Migrate every tenant that still has legacy rows
  --tenant=UUID          Migrate a single specific tenant
  --buildwithruud-sample Quick dev exercise using the known buildwithruud tenant
  --help, -h

Examples:
  cargo run --bin migrate_legacy_crm -- --dry-run --all
  cargo run --bin migrate_legacy_crm -- --all
  cargo run --bin migrate_legacy_crm -- --tenant 35f95f2a-db97-4166-be66-5215654cac84 --dry-run

This tool has no production path and creates no dependency on any specific tenant.
"#
    );
}
