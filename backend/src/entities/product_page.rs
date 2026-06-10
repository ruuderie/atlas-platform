//! Sea-ORM entities for product_page_templates and product_page_variants.

pub mod template {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "product_page_templates")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id:                 Uuid,
        pub product_id:         Uuid,
        pub hero_payload:       Value,
        pub blocks_payload:     Value,
        pub meta_title:         Option<String>,
        pub meta_description:   Option<String>,
        pub og_image_url:       Option<String>,
        pub structured_data:    Value,
        pub cta_label:          String,
        pub cta_action:         String,
        pub created_at:         DateTimeWithTimeZone,
        pub updated_at:         DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod variant {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "product_page_variants")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id:                  Uuid,
        pub product_id:          Uuid,
        pub template_id:         Uuid,
        pub variant_slug:        String,
        pub locale:              String,
        pub country_code:        Option<String>,
        pub region:              Option<String>,
        pub city:                Option<String>,
        pub geo_lat:             Option<f64>,
        pub geo_lng:             Option<f64>,
        pub hero_overrides:      Value,
        pub block_overrides:     Value,
        pub meta_title:          Option<String>,
        pub meta_description:    Option<String>,
        pub og_image_url:        Option<String>,
        pub canonical_url:       Option<String>,
        pub structured_data:     Option<Value>,
        pub launch_mode:         String,
        pub is_published:        bool,
        pub cta_label:           Option<String>,
        pub cta_action:          Option<String>,
        pub pre_order_cap:       Option<i32>,
        pub pre_order_sold:      i32,
        pub lead_count:          i32,
        pub view_count:          i32,

        // ── Localization fields (m20260905) ───────────────────────────────────
        /// "manual" | "city_inject" | "ai_localize"
        pub copy_strategy:           String,
        /// "not_started" | "pending" | "complete" | "failed"
        pub localization_status:     String,
        pub localization_task_id:    Option<Uuid>,
        /// If set, serve this variant at {subdomain_override}.{apex_domain}
        pub subdomain_override:      Option<String>,

        pub created_at:          DateTimeWithTimeZone,
        pub updated_at:          DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
