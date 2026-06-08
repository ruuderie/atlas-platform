//! # Migration: G22 `atlas_record_relationships` — Universal M:M Junction Table
//!
//! ## Design
//!
//! A single polymorphic junction table that connects any two platform entities
//! with a labeled relationship type. This is the Salesforce Junction Object
//! pattern — without it, every app builds its own per-combination join tables:
//! `campaign_assets`, `event_service_providers`, `case_contracts`, etc.
//!
//! ## Usage pattern
//!
//! ```sql
//! -- Link a campaign to multiple assets it's promoting:
//! INSERT INTO atlas_record_relationships
//!   (source_entity_type, source_entity_id, target_entity_type, target_entity_id, relationship_type)
//! VALUES
//!   ('atlas_campaigns', $campaign_id, 'atlas_assets', $asset1_id, 'promotes'),
//!   ('atlas_campaigns', $campaign_id, 'atlas_assets', $asset2_id, 'promotes');
//!
//! -- Find all assets promoted by a campaign:
//! SELECT * FROM atlas_record_relationships
//!   WHERE source_entity_type = 'atlas_campaigns'
//!     AND source_entity_id   = $campaign_id
//!     AND relationship_type  = 'promotes';
//! ```
//!
//! ## Unique constraint
//!
//! (tenant_id, source_type, source_id, target_type, target_id, relationship_type)
//! is unique — prevents duplicate relationships of the same type between the
//! same two records.
//!
//! ## Indexes
//!
//! Two directional indexes (source→target and target→source) support both
//! forward and reverse traversal — equivalent to Salesforce Related Lists on
//! both sides of a junction object.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260807_g22_atlas_record_relationships"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_record_relationships"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    // ── Source entity ─────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("source_entity_type")).string_len(100).not_null())
                    .col(ColumnDef::new(Alias::new("source_entity_id")).uuid().not_null())
                    // ── Target entity ─────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("target_entity_type")).string_len(100).not_null())
                    .col(ColumnDef::new(Alias::new("target_entity_id")).uuid().not_null())
                    // ── Relationship label ────────────────────────────────────
                    // e.g. 'promotes', 'attended_by', 'generated_from', 'referenced_in'
                    .col(ColumnDef::new(Alias::new("relationship_type")).string_len(100).not_null())
                    // Human-readable label for the inverse direction.
                    // e.g. if forward is 'promotes', inverse might be 'promoted_by'
                    .col(ColumnDef::new(Alias::new("inverse_label")).string_len(100).null())
                    // ── Metadata ──────────────────────────────────────────────
                    // Free-form context: sort_order, weight, notes, etc.
                    .col(ColumnDef::new(Alias::new("relationship_metadata")).json_binary().null())
                    // ── Audit ─────────────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("created_by_user_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_record_relationships
                 ADD CONSTRAINT atlas_record_rel_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE;

                 -- Uniqueness: a given relationship type between the same two records can only exist once.
                 CREATE UNIQUE INDEX atlas_record_rel_unique
                     ON atlas_record_relationships(
                         tenant_id,
                         source_entity_type, source_entity_id,
                         target_entity_type, target_entity_id,
                         relationship_type
                     );

                 -- Forward traversal: 'find all targets related to this source'
                 CREATE INDEX atlas_record_rel_source
                     ON atlas_record_relationships(
                         tenant_id, source_entity_type, source_entity_id, relationship_type
                     );

                 -- Reverse traversal: 'find all sources related to this target'
                 -- (Salesforce Related Lists on the other side of the junction)
                 CREATE INDEX atlas_record_rel_target
                     ON atlas_record_relationships(
                         tenant_id, target_entity_type, target_entity_id, relationship_type
                     );",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_record_relationships CASCADE;",
            )
            .await?;
        Ok(())
    }
}
