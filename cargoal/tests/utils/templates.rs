use cargoal::renderer::Context;
use cargoal::routes::http::Request;
use std::collections::HashMap;

#[cfg(test)]
pub fn home_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();
    context.insert("title".to_string(), "Home Page".into());
    context.insert("message".to_string(), "Welcome to the Home Page!".into());
    context
}

#[cfg(test)]
pub fn about_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();
    context.insert("title".to_string(), "About Us".into());
    context.insert("message".to_string(), "Learn more about us here.".into());
    context
}

#[cfg(test)]
pub fn conditional_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();
    context.insert("is_logged_in".to_string(), true.into());
    context
}

#[cfg(test)]
pub fn list_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();
    let items = vec!["Item 1", "Item 2", "Item 3"];
    context.insert("items".to_string(), items.into());
    context
}

#[cfg(test)]
pub fn filters_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();
    context.insert("uppercase_text".to_string(), "uppercase text".into());
    context.insert("lowercase_text".to_string(), "LOWERCASE TEXT".into());
    context.insert("trimmed_text".to_string(), "   Trimmed Text   ".into());
    context
}

#[cfg(test)]
pub fn include_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();
    context.insert("title".to_string(), "Include Test".into());
    context.insert("content".to_string(), "Main Content Section".into());
    context
}

#[cfg(test)]
pub fn escaping_handler(_req: &Request) -> Context {
    let mut context = HashMap::new();

    context.insert(
        "user_input".to_string(),
        "<script>alert('XSS')</script>".into(),
    );

    context
}
