use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::entities::profile;

#[derive(Debug, Deserialize)]
pub struct ProfileSearch {
    pub q: String,
    // Add other fields as needed
}
#[derive(Deserialize)]
pub struct CreateProfileInput {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub profile_type: profile::ProfileType,
    pub display_name: String,
    pub contact_info: String,
    pub business_details: Option<profile::BusinessDetails>,
}

#[derive(Serialize)]
pub struct ProfileModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub profile_type: profile::ProfileType,
    pub display_name: String,
    pub contact_info: String,
    pub business_details: Option<profile::BusinessDetails>,
}

#[derive(Deserialize)]
pub struct UpdateProfileInput {
    pub display_name: Option<String>,
    pub contact_info: Option<String>,
    pub business_details: Option<profile::BusinessDetails>,
}