#[derive(Clone)]
pub struct Scope {
    pub module: String,
    pub test: String,
    pub lang: String,
    pub normalized: String,
}

impl Scope {
    pub fn new(scope: &str, lang: &str) -> Self {
        let normalized = Self::normalize(scope, lang);
        let parts = normalized
            .split("::")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        Self {
            module: parts[0].clone(),
            test: parts[1].clone(),
            lang: lang.to_string(),
            normalized,
        }
    }

    fn normalize(scope: &str, lang: &str) -> String {
        [
            &scope
                .split("::")
                .filter(|s| !s.contains("test"))
                .filter(|s| !s.contains(env!("CARGO_CRATE_NAME")))
                .filter(|s| *s != lang)
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("::"),
            lang,
        ]
        .join("::")
    }
}
