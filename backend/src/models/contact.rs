use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::entities::contact;
use crate::models::address::{AddressJson};
use crate::models::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Contact {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    #[validate(nested)]
    pub billing_address: Option<AddressJson>,
    #[validate(nested)]
    pub shipping_address: Option<AddressJson>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactInput {
    pub customer_id: Option<Uuid>,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub billing_address: Option<AddressJson>,
    pub shipping_address: Option<AddressJson>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactInput {
    pub customer_id: Option<Uuid>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub billing_address: Option<AddressJson>,
    pub shipping_address: Option<AddressJson>,
}

impl From<contact::Model> for Contact {
    fn from(model: contact::Model) -> Self {
        Contact {
            id: model.id.clone(),
            customer_id: model.customer_id.clone(),
            name: model.name.clone(),
            first_name: model.first_name.clone(),
            last_name: model.last_name.clone(),
            email: model.email.clone(),
            phone: model.phone.clone(),
            whatsapp: model.whatsapp.clone(),
            telegram: model.telegram.clone(),
            twitter: model.twitter.clone(),
            instagram: model.instagram.clone(),
            facebook: model.facebook.clone(),
            billing_address: model.billing_address,
            shipping_address: model.shipping_address,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
