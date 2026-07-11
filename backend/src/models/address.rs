use sea_orm::FromJsonQueryResult;
use sea_orm::Value as Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct Address {
    #[validate(length(max = 255))]
    pub street_address: Option<String>,
    #[validate(length(max = 255))]
    pub street_address2: Option<String>,
    #[validate(length(max = 100))]
    pub city: Option<String>,
    #[validate(length(max = 100))]
    pub state_province: Option<String>,
    #[validate(length(max = 20))]
    pub postal_code: Option<String>,
    #[validate(length(max = 100))]
    pub country: Option<String>,
    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: Option<f64>,
    #[validate(range(min = -180.0, max = 180.0))]
    pub longitude: Option<f64>,
    #[validate(length(max = 500))]
    pub formatted_address: Option<String>,
    #[validate(length(max = 100))]
    pub place_id: Option<String>,
}

impl Address {
    pub fn get_full_address(&self) -> Option<String> {
        let parts: Vec<&str> = vec![
            self.street_address.as_deref(),
            self.street_address2.as_deref(),
            self.city.as_deref(),
            self.state_province.as_deref(),
            self.postal_code.as_deref(),
            self.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }

    pub fn get_coordinates(&self) -> Option<(f64, f64)> {
        match (self.latitude, self.longitude) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult, Serialize, Deserialize)]
pub struct AddressJson(pub Address);

impl Validate for AddressJson {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        self.0.validate()
    }
}

impl From<Address> for Json {
    fn from(address: Address) -> Self {
        Json::from(serde_json::to_value(address).unwrap())
    }
}

impl From<AddressJson> for Address {
    fn from(json: AddressJson) -> Self {
        json.0
    }
}
