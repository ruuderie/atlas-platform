//! Align legacy invite-accept contract_type with `PmContractType::ManagementAgreement`.
//!
//! Invite accept historically wrote `property_management_agreement`; the typed
//! enum wire value is `management_agreement`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE atlas_contracts
                   SET contract_type = 'management_agreement'
                 WHERE contract_type = 'property_management_agreement';
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Irreversible rename of a typo alias — leave as management_agreement.
        let _ = manager;
        Ok(())
    }
}
