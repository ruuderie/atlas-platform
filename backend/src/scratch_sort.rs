use sea_orm_migration::prelude::*;
fn sort_migrations(mut migrations: Vec<Box<dyn MigrationTrait>>) -> Vec<Box<dyn MigrationTrait>> {
    migrations.sort_by(|a, b| a.name().cmp(b.name()));
    migrations
}
