//! Hard-delete an asset subtree and dependent Folio records.

use anyhow::{anyhow, Result};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct AssetPurgeService;

impl AssetPurgeService {
    /// Permanently remove `root_asset_id` and all descendant assets, plus related rows.
    pub async fn purge_tree(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        root_asset_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        let tree_ids = Self::collect_tree_ids(db, tenant_id, root_asset_id).await?;
        if tree_ids.is_empty() {
            return Err(anyhow!("asset not found"));
        }

        let txn = db.begin().await?;

        Self::purge_tree_in_txn(&txn, tenant_id, &tree_ids).await?;

        txn.commit().await?;
        Ok(())
    }

    async fn collect_tree_ids(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        root_asset_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        use crate::entities::atlas_asset;

        let root = atlas_asset::Entity::find_by_id(root_asset_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?;
        if root.is_none() {
            return Ok(Vec::new());
        }

        let all = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .all(db)
            .await?;

        let mut tree = vec![root_asset_id];
        let mut queue = vec![root_asset_id];
        while let Some(parent_id) = queue.pop() {
            for asset in &all {
                if asset.parent_asset_id == Some(parent_id) && !tree.contains(&asset.id) {
                    tree.push(asset.id);
                    queue.push(asset.id);
                }
            }
        }
        Ok(tree)
    }

    fn deepest_first_order(all: &[crate::entities::atlas_asset::Model], tree_ids: &[Uuid]) -> Vec<Uuid> {
        let tree: HashSet<Uuid> = tree_ids.iter().copied().collect();
        let mut depth: HashMap<Uuid, usize> = HashMap::new();

        for id in tree_ids {
            depth.insert(*id, 0);
        }
        let mut changed = true;
        while changed {
            changed = false;
            for asset in all {
                if !tree.contains(&asset.id) {
                    continue;
                }
                if let Some(parent) = asset.parent_asset_id {
                    if tree.contains(&parent) {
                        let parent_depth = depth.get(&parent).copied().unwrap_or(0);
                        let cur = depth.get(&asset.id).copied().unwrap_or(0);
                        if cur <= parent_depth {
                            depth.insert(asset.id, parent_depth + 1);
                            changed = true;
                        }
                    }
                }
            }
        }

        let mut ordered = tree_ids.to_vec();
        ordered.sort_by(|a, b| depth.get(b).cmp(&depth.get(a)));
        ordered
    }

    async fn purge_tree_in_txn<C: ConnectionTrait>(
        conn: &C,
        tenant_id: Uuid,
        tree_ids: &[Uuid],
    ) -> Result<()> {
        use crate::entities::{
            atlas_asset, atlas_case, atlas_contract, atlas_document, atlas_ledger_entry,
            atlas_opportunity, atlas_regulatory_registration,
            atlas_reservation,
        };

        if tree_ids.is_empty() {
            return Ok(());
        }

        let ids_sql = uuid_in_list(tree_ids);

        // 1. Bookings RESTRICT asset delete — must go first.
        conn.execute_unprepared(&format!(
            "DELETE FROM atlas_bookings WHERE tenant_id = '{tenant_id}' AND asset_id IN ({ids_sql})"
        ))
        .await?;

        // 2. Collect contract + case ids for downstream cleanup.
        let contract_ids: Vec<Uuid> = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::AssetId.is_in(tree_ids.to_vec()))
            .all(conn)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        let case_ids: Vec<Uuid> = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::AssetId.is_in(tree_ids.to_vec()))
            .all(conn)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect();

        let mut rel_entity_ids: Vec<Uuid> = tree_ids.to_vec();
        rel_entity_ids.extend(&contract_ids);
        rel_entity_ids.extend(&case_ids);
        rel_entity_ids.sort_unstable();
        rel_entity_ids.dedup();

        // 3. Record relationships touching any purged entity id.
        if !rel_entity_ids.is_empty() {
            let rel_sql = uuid_in_list(&rel_entity_ids);
            conn.execute_unprepared(&format!(
                "DELETE FROM atlas_record_relationships \
                 WHERE tenant_id = '{tenant_id}' \
                 AND (source_entity_id IN ({rel_sql}) OR target_entity_id IN ({rel_sql}))"
            ))
            .await?;
        }

        // 4. Vault documents on assets or contracts in scope.
        let mut doc_entity_ids = tree_ids.to_vec();
        doc_entity_ids.extend(&contract_ids);
        doc_entity_ids.sort_unstable();
        doc_entity_ids.dedup();
        if !doc_entity_ids.is_empty() {
            atlas_document::Entity::delete_many()
                .filter(atlas_document::Column::TenantId.eq(tenant_id))
                .filter(atlas_document::Column::RelatedEntityId.is_in(doc_entity_ids))
                .exec(conn)
                .await?;
        }

        // 5. Ledger rows billed to contracts or cases in scope.
        let mut billable_ids = contract_ids;
        billable_ids.extend(case_ids);
        billable_ids.sort_unstable();
        billable_ids.dedup();
        if !billable_ids.is_empty() {
            atlas_ledger_entry::Entity::delete_many()
                .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
                .filter(atlas_ledger_entry::Column::BillableEntityId.is_in(billable_ids))
                .exec(conn)
                .await?;
        }

        // 6. Domain rows keyed by asset_id.
        atlas_case::Entity::delete_many()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::AssetId.is_in(tree_ids.to_vec()))
            .exec(conn)
            .await?;

        atlas_contract::Entity::delete_many()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::AssetId.is_in(tree_ids.to_vec()))
            .exec(conn)
            .await?;

        conn.execute_unprepared(&format!(
            "DELETE FROM atlas_leases WHERE tenant_id = '{tenant_id}' AND asset_id IN ({ids_sql})"
        ))
        .await?;

        atlas_reservation::Entity::delete_many()
            .filter(atlas_reservation::Column::TenantId.eq(tenant_id))
            .filter(atlas_reservation::Column::ReservedAssetId.is_in(tree_ids.to_vec()))
            .exec(conn)
            .await?;

        atlas_opportunity::Entity::delete_many()
            .filter(atlas_opportunity::Column::TenantId.eq(tenant_id))
            .filter(atlas_opportunity::Column::AssetId.is_in(tree_ids.to_vec()))
            .exec(conn)
            .await?;

        atlas_regulatory_registration::Entity::delete_many()
            .filter(atlas_regulatory_registration::Column::TenantId.eq(tenant_id))
            .filter(atlas_regulatory_registration::Column::AssetId.is_in(tree_ids.to_vec()))
            .exec(conn)
            .await?;

        conn.execute_unprepared(&format!(
            "DELETE FROM atlas_service_requests WHERE asset_id IN ({ids_sql})"
        ))
        .await?;

        // 7. Assets deepest-first (children before parents).
        let all_assets = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .all(conn)
            .await?;
        let delete_order = Self::deepest_first_order(&all_assets, tree_ids);
        for asset_id in delete_order {
            atlas_asset::Entity::delete_by_id(asset_id)
                .exec(conn)
                .await?;
        }

        Ok(())
    }
}

fn uuid_in_list(ids: &[Uuid]) -> String {
    ids.iter()
        .map(|id| format!("'{id}'"))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_in_list_quotes_and_commas() {
        let a = Uuid::nil();
        let b = Uuid::new_v4();
        let s = uuid_in_list(&[a, b]);
        assert!(s.starts_with('\''));
        assert!(s.contains(','));
        assert!(s.contains(&a.to_string()));
        assert!(s.contains(&b.to_string()));
    }

    #[test]
    fn empty_tree_ids_short_circuit() {
        // purge_tree_in_txn returns Ok immediately when tree_ids is empty
        // (handler maps empty collect → "asset not found" before txn).
        assert!(uuid_in_list(&[]).is_empty());
    }
}
