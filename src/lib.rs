pub struct SearchConfig {
    pub query: String,
    pub source_language: Option<isolang::Language>,
    pub target_language: isolang::Language,
}

pub mod translators;
