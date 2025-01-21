use isolang::Language;

use crate::translators;
use crate::translators::SearchProvider;
use crate::translators::TranslateError;

#[derive(Debug, Default)]
pub struct SearchConfig {
    pub query: String,
    pub source_language: Option<Language>,
    pub target_language: Option<Language>,
    pub provider: SearchProvider,
}

#[derive(Debug)]
pub struct Translation {
    pub(crate) orig: Option<String>,
    pub(crate) translated: Option<String>,
}

#[derive(Debug)]
pub struct Definition {
    pub(crate) meaning: String,
    pub(crate) pos: String, // Part of speech, noun verb etc.
    pub(crate) reverse_translation: Option<Vec<String>>,
    pub(crate) confidence: Option<f32>,
}

#[derive(Debug)]
pub struct Literation {
    pub(crate) orig: Option<String>,
    pub(crate) translated: Option<String>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub(crate) provider: SearchProvider,
    pub(crate) translations: Option<Vec<Translation>>,
    pub(crate) definitions: Option<Vec<Definition>>,
    pub(crate) src_lang: Option<String>,
    pub(crate) literation: Option<Literation>,
}

pub fn lookup(query: &SearchConfig) -> Result<SearchResult, TranslateError> {
    let res: SearchResult = match query.provider {
        SearchProvider::GoogleTranslate => {
            translators::google_translate::lookup_google_translate(query)?.into()
        }
        _ => todo!("This provider is not implemented"),
    };

    Ok(res)
}

pub fn parse_lang(lang: String) -> Result<Language, LanguageParseError> {
    let res: Language = Language::from_639_1(&lang)
        .ok_or(LanguageParseError("invalid language code".to_string()))?;

    Ok(res)
}

#[derive(Debug, Clone)]
pub struct LanguageParseError(String);

impl std::fmt::Display for LanguageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse the language code: {}", self.0)
    }
    // add code here
}
