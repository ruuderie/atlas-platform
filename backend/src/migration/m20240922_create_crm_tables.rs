use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create Customer table first (no foreign key dependencies)
        manager
            .create_table(
                Table::create()
                    .table(Customer::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Customer::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Customer::Name).string().not_null())
                    .col(ColumnDef::new(Customer::PrimaryContactId).uuid().null())
                    .col(ColumnDef::new(Customer::CustomerType).string().not_null())
                    .col(ColumnDef::new(Customer::Attributes).json().not_null())
                    .col(ColumnDef::new(Customer::Cpf).string().null())
                    .col(ColumnDef::new(Customer::Cnpj).string().null())
                    .col(ColumnDef::new(Customer::Tin).string().null())
                    .col(ColumnDef::new(Customer::Email).string().null())
                    .col(ColumnDef::new(Customer::Phone).string().null())
                    .col(ColumnDef::new(Customer::Whatsapp).string().null())
                    .col(ColumnDef::new(Customer::Telegram).string().null())
                    .col(ColumnDef::new(Customer::Twitter).string().null())
                    .col(ColumnDef::new(Customer::Instagram).string().null())
                    .col(ColumnDef::new(Customer::Facebook).string().null())
                    .col(ColumnDef::new(Customer::Website).string().null())
                    .col(ColumnDef::new(Customer::AnnualRevenue).double().null())
                    .col(ColumnDef::new(Customer::EmployeeCount).integer().null())
                    .col(ColumnDef::new(Customer::IsActive).boolean().not_null())
                    .col(ColumnDef::new(Customer::BillingAddress).json().null())
                    .col(ColumnDef::new(Customer::ShippingAddress).json().null())
                    .col(ColumnDef::new(Customer::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Customer::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create Deal table second (depends on Customer)
        manager
            .create_table(
                Table::create()
                    .table(Deal::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Deal::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Deal::CustomerId).uuid().not_null())
                    .col(ColumnDef::new(Deal::Name).string().not_null())
                    .col(ColumnDef::new(Deal::Amount).double().not_null())
                    .col(ColumnDef::new(Deal::Status).string().not_null())
                    .col(ColumnDef::new(Deal::Stage).string().not_null())
                    .col(ColumnDef::new(Deal::CloseDate).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Deal::IsActive).boolean().not_null())
                    .col(ColumnDef::new(Deal::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Deal::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deal-customer_id")
                            .from(Deal::Table, Deal::CustomerId)
                            .to(Customer::Table, Customer::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create Lead table third (depends on Deal)
        manager
            .create_table(
                Table::create()
                    .table(Lead::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Lead::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Lead::AssociatedDealId).uuid().null())
                    .col(ColumnDef::new(Lead::Name).string().not_null())
                    .col(ColumnDef::new(Lead::ListingId).uuid().null())
                    .col(ColumnDef::new(Lead::AccountId).uuid().null())
                    .col(ColumnDef::new(Lead::FirstName).string().null())
                    .col(ColumnDef::new(Lead::LastName).string().null())
                    .col(ColumnDef::new(Lead::Email).string().null())
                    .col(ColumnDef::new(Lead::Phone).string().null())
                    .col(ColumnDef::new(Lead::Whatsapp).string().null())
                    .col(ColumnDef::new(Lead::Telegram).string().null())
                    .col(ColumnDef::new(Lead::Twitter).string().null())
                    .col(ColumnDef::new(Lead::Instagram).string().null())
                    .col(ColumnDef::new(Lead::Facebook).string().null())
                    .col(ColumnDef::new(Lead::BillingAddress).json().null())
                    .col(ColumnDef::new(Lead::ShippingAddress).json().null())
                    .col(ColumnDef::new(Lead::Message).string().null())
                    .col(ColumnDef::new(Lead::Source).string().null())
                    .col(ColumnDef::new(Lead::IsConverted).boolean().not_null())
                    .col(ColumnDef::new(Lead::ConvertedToContact).boolean().not_null().default(false))
                    .col(ColumnDef::new(Lead::ConvertedCustomerId).uuid().null())
                    .col(ColumnDef::new(Lead::ConvertedContactId).uuid().null())
                    .col(ColumnDef::new(Lead::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Lead::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-lead-deal_id")
                            .from(Lead::Table, Lead::AssociatedDealId)
                            .to(Deal::Table, Deal::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await?;

        // Create Contact table
        manager
            .create_table(
                Table::create()
                    .table(Contact::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Contact::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Contact::CustomerId).uuid().null())
                    .col(ColumnDef::new(Contact::Name).string().not_null())
                    .col(ColumnDef::new(Contact::FirstName).string().null())
                    .col(ColumnDef::new(Contact::LastName).string().null())
                    .col(ColumnDef::new(Contact::Email).string().null())
                    .col(ColumnDef::new(Contact::Phone).string().null())
                    .col(ColumnDef::new(Contact::Whatsapp).string().null())
                    .col(ColumnDef::new(Contact::Telegram).string().null())
                    .col(ColumnDef::new(Contact::Twitter).string().null())
                    .col(ColumnDef::new(Contact::Instagram).string().null())
                    .col(ColumnDef::new(Contact::Facebook).string().null())
                    .col(ColumnDef::new(Contact::BillingAddress).json().null())
                    .col(ColumnDef::new(Contact::ShippingAddress).json().null())
                    .col(ColumnDef::new(Contact::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Contact::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-contact-customer_id")
                            .from(Contact::Table, Contact::CustomerId)
                            .to(Customer::Table, Customer::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await?;

        // Create DealContact table (junction table)
        manager
            .create_table(
                Table::create()
                    .table(DealContact::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DealContact::DealId).uuid().not_null())
                    .col(ColumnDef::new(DealContact::ContactId).uuid().not_null())
                    .primary_key(Index::create().col(DealContact::DealId).col(DealContact::ContactId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deal_contact-deal_id")
                            .from(DealContact::Table, DealContact::DealId)
                            .to(Deal::Table, Deal::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-deal_contact-contact_id")
                            .from(DealContact::Table, DealContact::ContactId)
                            .to(Contact::Table, Contact::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create Case table
        manager
            .create_table(
                Table::create()
                    .table(Case::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Case::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Case::CustomerId).uuid().not_null())
                    .col(ColumnDef::new(Case::Title).string().not_null())
                    .col(ColumnDef::new(Case::Description).string().not_null())
                    .col(ColumnDef::new(Case::Status).string().not_null())
                    .col(ColumnDef::new(Case::Priority).string().not_null())
                    .col(ColumnDef::new(Case::AssignedTo).uuid().null())
                    .col(ColumnDef::new(Case::ClosedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Case::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Case::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-case-customer_id")
                            .from(Case::Table, Case::CustomerId)
                            .to(Customer::Table, Customer::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-case-assigned_to")
                            .from(Case::Table, Case::AssignedTo)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await?;

        // Create Activity table
        manager
            .create_table(
                Table::create()
                    .table(Activity::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Activity::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Activity::DealId).uuid().null())
                    .col(ColumnDef::new(Activity::CustomerId).uuid().null())
                    .col(ColumnDef::new(Activity::LeadId).uuid().null())
                    .col(ColumnDef::new(Activity::ContactId).uuid().null())
                    .col(ColumnDef::new(Activity::CaseId).uuid().null())
                    .col(ColumnDef::new(Activity::ActivityType).string().not_null())
                    .col(ColumnDef::new(Activity::Title).string().not_null())
                    .col(ColumnDef::new(Activity::Description).string().not_null())
                    .col(ColumnDef::new(Activity::Status).string().not_null())
                    .col(ColumnDef::new(Activity::DueDate).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Activity::CompletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Activity::CreatedBy).uuid().not_null())
                    .col(ColumnDef::new(Activity::AssignedTo).uuid().null())
                    .col(ColumnDef::new(Activity::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Activity::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-deal_id")
                            .from(Activity::Table, Activity::DealId)
                            .to(Deal::Table, Deal::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-customer_id")
                            .from(Activity::Table, Activity::CustomerId)
                            .to(Customer::Table, Customer::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-lead_id")
                            .from(Activity::Table, Activity::LeadId)
                            .to(Lead::Table, Lead::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-contact_id")
                            .from(Activity::Table, Activity::ContactId)
                            .to(Contact::Table, Contact::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-case_id")
                            .from(Activity::Table, Activity::CaseId)
                            .to(Case::Table, Case::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-created_by")
                            .from(Activity::Table, Activity::CreatedBy)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activity-assigned_to")
                            .from(Activity::Table, Activity::AssignedTo)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await?;

        // Create Note table
        manager
            .create_table(
                Table::create()
                    .table(Note::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Note::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Note::Content).string().not_null())
                    .col(ColumnDef::new(Note::CreatedBy).uuid().not_null())
                    .col(ColumnDef::new(Note::EntityType).string().not_null())
                    .col(ColumnDef::new(Note::EntityId).uuid().not_null())
                    .col(ColumnDef::new(Note::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Note::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-note-created_by")
                            .from(Note::Table, Note::CreatedBy)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create File table
        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(File::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(File::Name).string().not_null())
                    .col(ColumnDef::new(File::FileType).string().not_null())
                    .col(ColumnDef::new(File::StoragePath).string().not_null())
                    .col(ColumnDef::new(File::Size).big_integer().not_null())
                    .col(ColumnDef::new(File::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(File::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create FileAssociation table
        manager
            .create_table(
                Table::create()
                    .table(FileAssociation::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FileAssociation::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(FileAssociation::FileId).uuid().not_null())
                    .col(ColumnDef::new(FileAssociation::AssociatedEntityType).string().not_null())
                    .col(ColumnDef::new(FileAssociation::AssociatedEntityId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-file_association-file_id")
                            .from(FileAssociation::Table, FileAssociation::FileId)
                            .to(File::Table, File::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        manager
            .drop_table(Table::drop().table(FileAssociation::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Note::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Activity::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Case::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DealContact::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Contact::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Deal::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Lead::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Customer::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Add Iden enums for all tables
#[derive(Iden)]
enum Customer {
    Table,
    Id,
    Name,
    PrimaryContactId,
    CustomerType,
    Attributes,
    Cpf,
    Cnpj,
    Tin,
    Email,
    Phone,
    Whatsapp,
    Telegram,
    Twitter,
    Instagram,
    Facebook,
    Website,
    AnnualRevenue,
    EmployeeCount,
    IsActive,
    BillingAddress,
    ShippingAddress,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Lead {
    Table,
    Id,
    AssociatedDealId,
    Name,
    ListingId,
    AccountId,
    FirstName,
    LastName,
    Email,
    Phone,
    Whatsapp,
    Telegram,
    Twitter,
    Instagram,
    Facebook,
    BillingAddress,
    ShippingAddress,
    Message,
    Source,
    IsConverted,
    ConvertedToContact,
    ConvertedCustomerId,
    ConvertedContactId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Deal {
    Table,
    Id,
    CustomerId,
    Name,
    Amount,
    Status,
    Stage,
    CloseDate,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Contact {
    Table,
    Id,
    CustomerId,
    Name,
    FirstName,
    LastName,
    Email,
    Phone,
    Whatsapp,
    Telegram,
    Twitter,
    Instagram,
    Facebook,
    BillingAddress,
    ShippingAddress,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum DealContact {
    Table,
    DealId,
    ContactId,
}

#[derive(Iden)]
enum Case {
    Table,
    Id,
    CustomerId,
    Title,
    Description,
    Status,
    Priority,
    AssignedTo,
    ClosedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Activity {
    Table,
    Id,
    DealId,
    CustomerId,
    LeadId,
    ContactId,
    CaseId,
    ActivityType,
    Title,
    Description,
    Status,
    DueDate,
    CompletedAt,
    CreatedBy,
    AssignedTo,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Note {
    Table,
    Id,
    Content,
    CreatedBy,
    EntityType,
    EntityId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum File {
    Table,
    Id,
    Name,
    FileType,
    StoragePath,
    Size,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum FileAssociation {
    Table,
    Id,
    FileId,
    AssociatedEntityType,
    AssociatedEntityId,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
