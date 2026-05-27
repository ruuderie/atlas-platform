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
                    .table(AtlasDocuments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasDocuments::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasDocuments::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasDocuments::AttachmentId).uuid().not_null())
                    .col(ColumnDef::new(AtlasDocuments::ShareTokenId).uuid().null())
                    .col(ColumnDef::new(AtlasDocuments::DocumentCategory).string().not_null())
                    .col(ColumnDef::new(AtlasDocuments::AppNamespace).string_len(30).not_null())
                    .col(ColumnDef::new(AtlasDocuments::RelatedEntityType).string().null())
                    .col(ColumnDef::new(AtlasDocuments::RelatedEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasDocuments::IsCounterpartyVisible).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasDocuments::RequiresSignature).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasDocuments::IsSigned).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasDocuments::SignedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasDocuments::SignedByUserId).uuid().null())
                    .col(ColumnDef::new(AtlasDocuments::SignatureBlob).text().null())
                    .col(ColumnDef::new(AtlasDocuments::VersionNumber).integer().not_null().default(1))
                    .col(ColumnDef::new(AtlasDocuments::SupersedesDocumentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasDocuments::CreatedAt)
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
                    .table(AtlasDocuments::Table)
                    .col(AtlasDocuments::TenantId)
                    .col(AtlasDocuments::RelatedEntityType)
                    .col(AtlasDocuments::RelatedEntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_documents_namespace")
                    .table(AtlasDocuments::Table)
                    .col(AtlasDocuments::TenantId)
                    .col(AtlasDocuments::AppNamespace)
                    .col(AtlasDocuments::DocumentCategory)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasDocuments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasDocuments {
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
