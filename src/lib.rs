pub struct SearchConfig {
    pub query: String,
    pub source_language: Option<isolang::Language>,
    pub target_language: isolang::Language,
}

#[derive(Debug, Clone)]
pub struct TranslateError(String);

impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to translate: {}", self.0)
    }
    // add code here
}

pub mod modname;
