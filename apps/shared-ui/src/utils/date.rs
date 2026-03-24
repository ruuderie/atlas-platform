use time::{Date, Month};

pub struct DateUtils;

impl DateUtils {
    pub fn parse_from_url(_s: Option<String>) -> Option<Date> {
        None
    }
    
    pub fn prev_month_year(_month: Month, year: i32) -> (Month, i32) {
        (Month::January, year)
    }
    
    pub fn next_month_year(_month: Month, year: i32) -> (Month, i32) {
        (Month::February, year)
    }

    pub fn days_in_month(month: Month, year: i32) -> u8 {
        time::util::days_in_month(month, year)
    }
}
