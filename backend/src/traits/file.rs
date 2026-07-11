use sea_orm::EntityTrait;

pub trait FileAssociable: EntityTrait {
    fn entity_type() -> &'static str;
}
