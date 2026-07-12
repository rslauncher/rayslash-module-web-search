#[allow(warnings)]
mod bindings;

use bindings::exports::rayslash::module::provider::Guest;
use bindings::rayslash::module::types::{
    Action, Icon, ModuleError, QueryContext, QueryResponse, ResultItem,
};
use serde::Deserialize;

struct Component;

#[derive(Deserialize)]
struct Settings {
    #[serde(default = "default_searches")]
    searches: Vec<Search>,
}

#[derive(Clone, Deserialize)]
struct Search {
    name: String,
    keyword: String,
    url: String,
    #[serde(default = "enabled")]
    enabled: bool,
}

impl Guest for Component {
    fn query(context: QueryContext) -> Result<QueryResponse, ModuleError> {
        let settings: Settings =
            serde_json::from_str(&context.settings_json).unwrap_or_else(|_| Settings {
                searches: default_searches(),
            });
        let input = context.query.trim();
        let mut results = Vec::new();
        for search in settings
            .searches
            .into_iter()
            .filter(|search| search.enabled)
        {
            let Some(terms) = triggered_terms(input, &search.keyword) else {
                continue;
            };
            if !valid_template(&search.url) {
                continue;
            }
            let url = search.url.replace("%s", &encode(terms));
            results.push(ResultItem {
                id: format!(
                    "web-search:{}:{}",
                    search.keyword.to_ascii_lowercase(),
                    terms.to_ascii_lowercase()
                ),
                title: format!("Search {} for {terms}", search.name),
                subtitle: url.clone(),
                icon: Icon::Text(search.keyword),
                score: None,
                action: Action::OpenUrl(url),
            });
        }
        Ok(QueryResponse {
            exclusive: !results.is_empty(),
            results,
        })
    }
}

fn triggered_terms<'a>(input: &'a str, keyword: &str) -> Option<&'a str> {
    let input = input.trim();
    let (trigger, terms) = input.split_once(char::is_whitespace)?;
    (trigger.eq_ignore_ascii_case(keyword) && !terms.trim().is_empty()).then(|| terms.trim())
}
fn valid_template(value: &str) -> bool {
    value.starts_with("https://") && value.matches("%s").count() == 1
}
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}
fn enabled() -> bool {
    true
}
fn default_searches() -> Vec<Search> {
    vec![Search {
        name: "Web".into(),
        keyword: "search".into(),
        url: "https://www.google.com/search?q=%s".into(),
        enabled: true,
    }]
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_trigger_and_terms() {
        assert_eq!(triggered_terms("search rust", "search"), Some("rust"));
        assert_eq!(triggered_terms("search", "search"), None);
    }
    #[test]
    fn encodes_query_values() {
        assert_eq!(encode("rust wasm"), "rust%20wasm");
    }
}
