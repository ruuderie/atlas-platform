use crate::entities::customer::{Model as CustomerEntity, CustomerAttributes};
use crate::models::address::AddressJson;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub customer_type: String,
    pub attributes: CustomerAttributes,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub tin: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub website: Option<String>,
    pub annual_revenue: Option<f64>,
    pub employee_count: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[validate(nested)]
    pub billing_address: Option<AddressJson>,
    #[validate(nested)]
    pub shipping_address: Option<AddressJson>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomerInput {
    pub name: String,
    pub customer_type: String,
    pub attributes: CustomerAttributes,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub tin: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub website: Option<String>,
    pub annual_revenue: Option<f64>,
    pub employee_count: Option<i32>,
    pub billing_address: Option<AddressJson>,
    pub shipping_address: Option<AddressJson>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerInput {
    pub name: Option<String>,
    pub customer_type: Option<String>,
    pub attributes: Option<CustomerAttributes>,
    pub cpf: Option<String>,
    pub cnpj: Option<String>,
    pub tin: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub website: Option<String>,
    pub annual_revenue: Option<f64>,
    pub employee_count: Option<i32>,
    pub is_active: Option<bool>,
    pub billing_address: Option<AddressJson>,
    pub shipping_address: Option<AddressJson>,
    pub primary_contact_id: Option<Uuid>,
}

impl From<CustomerEntity> for Customer {
    fn from(entity: CustomerEntity) -> Self {
        Customer {
            id: entity.id,
            name: entity.name,
            customer_type: entity.customer_type.to_string(),
            attributes: entity.attributes,
            cpf: entity.cpf,
            cnpj: entity.cnpj,
            tin: entity.tin,
            email: entity.email,
            phone: entity.phone,
            whatsapp: entity.whatsapp,
            telegram: entity.telegram,
            twitter: entity.twitter,
            instagram: entity.instagram,
            facebook: entity.facebook,
            website: entity.website,
            annual_revenue: entity.annual_revenue,
            employee_count: entity.employee_count,
            is_active: entity.is_active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            billing_address: entity.billing_address,
            shipping_address: entity.shipping_address,
        }
    }
}


