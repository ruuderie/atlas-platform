use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-14: atlas_documents — Generic Document Registry
/// Polymorphic document storage with metadata, versioning, e-signature, and access control.
/// Works on top of atlas_vault (attachment + share tokens).
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasDocument::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasDocument::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasDocument::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasDocument::AttachmentId).uuid().not_null())
                    .col(ColumnDef::new(AtlasDocument::ShareTokenId).uuid().null())
                    .col(ColumnDef::new(AtlasDocument::DocumentCategory).string().not_null())
                    .col(ColumnDef::new(AtlasDocument::AppNamespace).string_len(30).not_null())
                    .col(ColumnDef::new(AtlasDocument::RelatedEntityType).string().null())
                    .col(ColumnDef::new(AtlasDocument::RelatedEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasDocument::IsCounterpartyVisible).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasDocument::RequiresSignature).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasDocument::IsSigned).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasDocument::SignedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasDocument::SignedByUserId).uuid().null())
                    .col(ColumnDef::new(AtlasDocument::SignatureBlob).text().null())
                    .col(ColumnDef::new(AtlasDocument::VersionNumber).integer().not_null().default(1))
                    .col(ColumnDef::new(AtlasDocument::SupersedesDocumentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasDocument::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_documents_entity")
                    .table(AtlasDocument::Table)
                    .col(AtlasDocument::TenantId)
                    .col(AtlasDocument::RelatedEntityType)
                    .col(AtlasDocument::RelatedEntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_documents_namespace")
                    .table(AtlasDocument::Table)
                    .col(AtlasDocument::TenantId)
                    .col(AtlasDocument::AppNamespace)
                    .col(AtlasDocument::DocumentCategory)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasDocument::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasDocument {
    Table,
    Id,
    TenantId,
    AttachmentId,
    ShareTokenId,
    DocumentCategory,
    AppNamespace,
    RelatedEntityType,
    RelatedEntityId,
    IsCounterpartyVisible,
    RequiresSignature,
    IsSigned,
    SignedAt,
    SignedByUserId,
    SignatureBlob,
    VersionNumber,
    SupersedesDocumentId,
    CreatedAt,
}
