#[derive(Debug, Clone)]
pub struct TranslateError(String);

impl std::fmt::Display for TranslateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to translate: {}", self.0)
    }
    // add code here
}

pub mod google_translate;
