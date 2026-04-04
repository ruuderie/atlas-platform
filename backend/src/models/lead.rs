use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::entities::lead;
use crate::models::address::AddressJson;

#[derive(Debug, Serialize, Deserialize)]
pub struct LeadModel {
    pub id: Uuid,
    pub name: String,
    pub listing_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
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
    pub message: Option<String>,
    pub source: Option<String>,
    pub is_converted: bool,
    pub converted_to_contact: bool,
    pub associated_deal_id: Option<Uuid>,
    pub converted_customer_id: Option<Uuid>,
    pub converted_contact_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadInput {
    pub name: String,
    pub listing_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
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
    pub message: Option<String>,
    pub source: Option<String>,
    pub _bot_check: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadInput {
    pub name: Option<String>,
    pub listing_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
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
    pub message: Option<String>,
    pub source: Option<String>,
    pub is_converted: Option<bool>,
    pub converted_to_contact: Option<bool>,
    pub associated_deal_id: Option<Uuid>,
    pub converted_customer_id: Option<Uuid>,
    pub converted_contact_id: Option<Uuid>,
}

impl From<lead::Model> for LeadModel {
    fn from(lead: lead::Model) -> Self {
        Self {
            id: lead.id,
            name: lead.name,
            listing_id: lead.listing_id,
            account_id: lead.account_id,
            first_name: lead.first_name,
            last_name: lead.last_name,
            email: lead.email,
            phone: lead.phone,
            whatsapp: lead.whatsapp,
            telegram: lead.telegram,
            twitter: lead.twitter,
            instagram: lead.instagram,
            facebook: lead.facebook,
            billing_address: lead.billing_address,
            shipping_address: lead.shipping_address,
            message: lead.message,
            source: lead.source,
            is_converted: lead.is_converted,
            converted_to_contact: lead.converted_to_contact,
            associated_deal_id: lead.associated_deal_id,
            converted_customer_id: lead.converted_customer_id,
            converted_contact_id: lead.converted_contact_id,
            created_at: lead.created_at,
            updated_at: lead.updated_at,
        }
    }
}
