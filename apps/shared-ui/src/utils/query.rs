use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[allow(non_snake_case)]
pub mod QUERY {
    pub const PAGE: &str = "page";
    pub const START_DATE: &str = "start_date";
    pub const END_DATE: &str = "end_date";
}

pub struct QueryUtils;

impl QueryUtils {
    pub fn extract(key: String) -> Memo<Option<String>> {
        Memo::new(move |_| {
            let query = use_query_map();
            query.with(|q| q.get(&key).clone())
        })
    }

    pub fn update_dates_url(_start: Option<time::Date>, _end: Option<time::Date>) {
        // Stub for now. Ideally this would navigate and update the URL params.
    }
}
