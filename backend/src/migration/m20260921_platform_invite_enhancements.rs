use sea_orm_migration::prelude::*;

/// Adds contextual columns to `platform_invite` for the enhanced onboarding invite flow:
///   - `display_name`    — pre-fill the new user's full name
///   - `folio_role`      — which Folio persona this user will have
///   - `app_instance_id` — which specific app instance they're being invited into
///   - `target_app_url`  — where the magic link should land (overrides FRONTEND_URL)
///   - `personal_message`— optional operator note included in the invite email
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260921_platform_invite_enhancements"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("platform_invite"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("display_name")).string().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("app_role")).string().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("app_instance_id")).uuid().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("target_app_url")).string().null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("personal_message")).text().null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("platform_invite"))
                    .drop_column(Alias::new("display_name"))
                    .drop_column(Alias::new("app_role"))
                    .drop_column(Alias::new("app_instance_id"))
                    .drop_column(Alias::new("target_app_url"))
                    .drop_column(Alias::new("personal_message"))
                    .to_owned(),
            )
            .await
    }
}
