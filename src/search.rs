use std::error::Error;

use isolang::Language;

use crate::translators;
use crate::translators::SearchProvider;
use color_eyre::Result;

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
    pub(crate) examples: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Literation {
    pub(crate) orig: Option<String>,
    pub(crate) translated: Option<String>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub provider: SearchProvider,
    pub translations: Option<Vec<Translation>>,
    pub definitions: Option<Vec<Definition>>,
    pub src_lang: Option<String>,
    pub literation: Option<Literation>,
}

pub fn lookup(query: &SearchConfig) -> Result<SearchResult> {
    let res: SearchResult = match query.provider {
        SearchProvider::GoogleTranslate => {
            translators::google_translate::lookup_google_translate(query)?.into()
        }
        _ => todo!("This provider is not implemented"),
    };

    Ok(res)
}

/// Parses a language string and return a Language Enum
///
/// Reference: [Wikipedia page](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes)
///
/// # Examples
/// ```rust
/// use wdym::search::parse_lang;
/// use isolang::Language;
/// assert_eq!(parse_lang("en").unwrap(), Language::Eng);
/// ```
pub fn parse_lang(lang: &str) -> Result<Language> {
    let lang = lang.to_lowercase();
    let res = lang
        .parse::<Language>() // Parses ISO 639-1, 639-3 English names and autonyms
        .or_else(|_| {
            Language::from_locale(&lang) // Parses Unix style locale: `zh_CN.utf8`
                .ok_or(LanguageParseError(lang.to_string()))
        });

    Ok(res?)
}

#[derive(Debug)]
pub struct LanguageParseError(String);

impl Error for LanguageParseError {}

impl std::fmt::Display for LanguageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse the language code: {}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parsing_lang() {
        // Unix-style locale
        let res = parse_lang("zh_CN.utf8").unwrap();
        assert_eq!(res, Language::Zho);

        // Non-standard locale
        let res = parse_lang("zh_TW").unwrap(); // The implementation of isolang currently only
                                                // reads the `zh` part, ignoring the country code.
        assert_eq!(res, Language::Zho);

        // ISO 639-1
        let res = parse_lang("ja").unwrap();
        assert_eq!(res, Language::Jpn);

        // ISO 639-3
        let res = parse_lang("jpn").unwrap();
        assert_eq!(res, Language::Jpn);

        // Lowercase English name
        let res = parse_lang("german").unwrap();
        assert_eq!(res, Language::Deu);

        // Mixed case English name
        let res = parse_lang("HinDi").unwrap();
        assert_eq!(res, Language::Hin);

        // Autonym
        let res = parse_lang("עברית").unwrap();
        assert_eq!(res, Language::Heb);
    }
}
