use std::fmt::Display;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct PhoneNumber {
    pub value: String,
}

impl PhoneNumber {
    pub fn new(val: &str, _max_digits: usize) -> Self {
        Self { value: val.to_string() }
    }
    
    pub fn format(&self, _country: crate::utils::country::Country) -> String {
        self.value.clone()
    }
    
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct PhoneFormat {
    pub max_digits: usize,
}

impl PhoneFormat {
    pub fn for_country(_country: crate::utils::country::Country) -> Self {
        Self { max_digits: 15 }
    }
    
    pub fn placeholder(&self) -> &'static str {
        "+1 (555) 555-5555"
    }
}
