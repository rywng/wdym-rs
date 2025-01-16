pub mod translators;
use translators::SearchProvider;
use translators::TranslateError;

pub struct SearchConfig {
    pub query: String,
    pub source_language: Option<isolang::Language>,
    pub target_language: isolang::Language,
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

pub fn lookup(provider: SearchProvider, query: SearchConfig) -> Result<SearchResult, TranslateError> {
    let res: SearchResult = match provider {
        SearchProvider::GoogleTranslate => translators::google_translate::lookup_google_translate(query)?.into(),
    };

    Ok(res)
}
