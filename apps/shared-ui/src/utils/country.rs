use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Display, EnumIter)]
pub enum Country {
    UnitedStatesOfAmerica,
    UnitedKingdom,
    France,
    Germany,
    Canada,
    Australia,
    Spain,
    Italy,
    Japan,
    China,
    India,
    Brazil,
    Mexico,
    Unknown,
}

impl Country {
    pub fn all() -> &'static [Country] {
        &[
            Country::UnitedStatesOfAmerica,
            Country::UnitedKingdom,
            Country::France,
            Country::Germany,
            Country::Canada,
            Country::Australia,
            Country::Spain,
            Country::Italy,
            Country::Japan,
            Country::China,
            Country::India,
            Country::Brazil,
            Country::Mexico,
        ]
    }

    pub fn name(&self) -> &'static str { "Unknown" }
    pub fn alpha2(&self) -> &'static str { "UN" }
    pub fn dial_code_formatted(&self) -> &'static str { "+00" }
    pub fn flag_emoji(&self) -> &'static str { "🏳️" }
}
