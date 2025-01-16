pub mod translators;
use isolang::Language;
use translators::SearchProvider;
use translators::TranslateError;

pub struct SearchConfig {
    pub query: String,
    pub source_language: Option<Language>,
    pub target_language: Option<Language>,
    pub provider: SearchProvider,
}

#[derive(Debug)]
pub struct Translation {
    orig: Option<String>,
    translated: Option<String>,
}

#[derive(Debug)]
pub struct Definition {
    meaning: String,
    pos: String, // Part of speech, noun verb etc.
    reverse_translation: Option<Vec<String>>,
    confidence: Option<f32>,
}

#[derive(Debug)]
pub struct Literation {
    orig: Option<String>,
    translated: Option<String>,
}

#[derive(Debug)]
pub struct SearchResult {
    provider: SearchProvider,
    translation: Option<Vec<Translation>>,
    dictionary: Option<Vec<Definition>>,
    src_lang: Option<String>,
    literation: Literation,
}

pub fn lookup(query: SearchConfig) -> Result<SearchResult, TranslateError> {
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
