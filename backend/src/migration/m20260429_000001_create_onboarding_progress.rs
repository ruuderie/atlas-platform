use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260429_000001_create_onboarding_progress"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OnboardingProgress::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OnboardingProgress::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OnboardingProgress::TenantId).uuid().not_null())
                    .col(ColumnDef::new(OnboardingProgress::AppInstanceId).uuid().not_null())
                    .col(ColumnDef::new(OnboardingProgress::StepId).string().not_null())
                    .col(
                        ColumnDef::new(OnboardingProgress::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(OnboardingProgress::Skipped)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(OnboardingProgress::DismissedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(OnboardingProgress::Metadata)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(OnboardingProgress::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OnboardingProgress::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_onboarding_progress_tenant")
                            .from(OnboardingProgress::Table, OnboardingProgress::TenantId)
                            .to(Alias::new("tenant"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_onboarding_progress_app_instance")
                            .from(OnboardingProgress::Table, OnboardingProgress::AppInstanceId)
                            .to(Alias::new("app_instances"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one progress record per (app_instance, step)
        manager
            .create_index(
                Index::create()
                    .name("idx_onboarding_progress_unique_step")
                    .table(OnboardingProgress::Table)
                    .col(OnboardingProgress::AppInstanceId)
                    .col(OnboardingProgress::StepId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OnboardingProgress::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum OnboardingProgress {
    Table,
    Id,
    TenantId,
    AppInstanceId,
    StepId,
    CompletedAt,
    Skipped,
    DismissedAt,
    Metadata,
    CreatedAt,
    UpdatedAt,
}
