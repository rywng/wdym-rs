//! Implementations of different translators and dictionaries
#[derive(Debug, Clone)]
pub struct TranslateError(String);

impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to translate: {}", self.0)
    }
    // add code here
}

#[derive(Debug)]
pub enum SearchProvider {
    GoogleTranslate,
}

pub mod google_translate;
