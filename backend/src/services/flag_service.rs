//! Feature flag resolution service.
//!
//! Priority (first decisive match wins):
//! 1. Per-app-instance enablement (`atlas_flag_instance_enablements`)
//! 2. Per-tenant override (`flag_overrides`)
//! 3. Global catalog (`feature_flags` kill-switch + rollout)
//! 4. Default: disabled

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::{
    atlas_flag_instance_enablement, feature_flag, flag_override,
};
use crate::types::flags::FlagEffect;

pub struct FlagService;

impl FlagService {
    /// Resolve whether `key` is enabled for a tenant, optionally scoped to an app instance.
    pub async fn is_enabled(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        app_instance_id: Option<Uuid>,
        key: &str,
    ) -> Result<bool, DbErr> {
        let flag = feature_flag::Entity::find()
            .filter(feature_flag::Column::Key.eq(key))
            .one(db)
            .await?;

        let Some(flag) = flag else {
            return Ok(false);
        };

        // 1. Instance enablement
        if let Some(instance_id) = app_instance_id {
            let enablement = atlas_flag_instance_enablement::Entity::find()
                .filter(atlas_flag_instance_enablement::Column::FlagKey.eq(key))
                .filter(atlas_flag_instance_enablement::Column::AppInstanceId.eq(instance_id))
                .one(db)
                .await?;

            if let Some(row) = enablement {
                return Ok(match FlagEffect::try_from(row.effect.as_str()) {
                    Ok(FlagEffect::Grant) => in_rollout(row.rollout_pct, instance_id),
                    Ok(FlagEffect::Deny) => false,
                    Err(_) => false,
                });
            }
        }

        // 2. Tenant override
        let override_row = flag_override::Entity::find()
            .filter(flag_override::Column::FlagId.eq(flag.id))
            .filter(flag_override::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?;

        if let Some(ovr) = override_row {
            return Ok(match FlagEffect::try_from(ovr.override_type.as_str()) {
                Ok(FlagEffect::Grant) => in_rollout(ovr.rollout_pct, tenant_id),
                Ok(FlagEffect::Deny) => false,
                Err(_) => false,
            });
        }

        // 3. Global catalog kill / rollout
        if !flag.is_enabled {
            return Ok(false);
        }
        if flag.has_global {
            return Ok(in_rollout(flag.global_rollout_pct, tenant_id));
        }

        // 4. Default
        Ok(false)
    }

    /// List catalog flag keys that are currently enabled for the given scope.
    pub async fn list_enabled_keys(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        app_instance_id: Option<Uuid>,
    ) -> Result<Vec<String>, DbErr> {
        let flags = feature_flag::Entity::find().all(db).await?;
        let mut keys = Vec::new();
        for flag in flags {
            if Self::is_enabled(db, tenant_id, app_instance_id, &flag.key).await? {
                keys.push(flag.key);
            }
        }
        keys.sort();
        Ok(keys)
    }
}

/// Deterministic sticky bucket in `[0, 100)` derived from a UUID salt.
fn in_rollout(pct: i32, salt: Uuid) -> bool {
    let pct = pct.clamp(0, 100);
    if pct >= 100 {
        return true;
    }
    if pct <= 0 {
        return false;
    }
    (salt.as_u128() % 100) < pct as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollout_bounds() {
        let salt = Uuid::nil();
        assert!(in_rollout(100, salt));
        assert!(!in_rollout(0, salt));
    }
}
