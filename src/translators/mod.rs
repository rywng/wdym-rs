//! Implementations of different translators and dictionaries
use clap::ValueEnum;

#[derive(Debug, Clone)]
pub struct TranslateError(String);

impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to translate: {}", self.0)
    }
}

impl std::error::Error for TranslateError {}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SearchProvider {
    #[default]
    GoogleTranslate,
    Jisho,
}

impl std::fmt::Display for SearchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("No variables should be skipped")
            .get_name()
            .fmt(f)
    }
}

pub mod google_translate;
