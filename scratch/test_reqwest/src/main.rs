use leptos::ServerFnError;

fn test_fn(id: &str) -> Result<(), ServerFnError> {
    let _uuid_id = uuid::Uuid::parse_str(id)?;
    Ok(())
}

fn main() {}
