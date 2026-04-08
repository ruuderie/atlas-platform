use std::collections::BTreeMap;
use crate::api::models::PlatformAppModel;

/// Groups a flat list of PlatformAppModels by their `tenant_id`.
/// Returns a BTreeMap where:
/// - Key is the `tenant_id`.
/// - Value is a tuple containing the `tenant_name` (extracted from the first seen app)
///   and the `Vec<PlatformAppModel>` natively bound to that tenant.
pub fn group_apps_by_tenant(
    apps: Vec<PlatformAppModel>,
) -> BTreeMap<String, (String, Vec<PlatformAppModel>)> {
    let mut grouped = BTreeMap::new();
    for app in apps {
        let entry = grouped
            .entry(app.tenant_id.clone())
            .or_insert_with(|| (app.name.clone(), Vec::new()));
        entry.1.push(app);
    }
    grouped
}
