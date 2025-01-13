use super::SearchConfig;
use super::TranslateError;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GoogleTranslateResponse {
    pub(crate) dict: Option<Vec<GoogleTranslateResponseDict>>,
    pub(crate) src: String,
    pub(crate) confidence: Option<f32>,
    pub(crate) spell: Option<serde_json::Value>,
    pub(crate) ld_result: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct GoogleTranslateResponseDict {
    pub(crate) pos: String,
    pub(crate) terms: Vec<String>,
    pub(crate) entry: Vec<GoogleTranslateResponseEntry>,
    pub(crate) base_form: String,
    pub(crate) pos_enum: i32,
}

#[derive(Deserialize, Debug)]
struct GoogleTranslateResponseEntry {
    pub(crate) word: String,
    pub(crate) reverse_translation: Vec<String>,
    pub(crate) score: Option<f32>,
}

/// Looks up the translation on google translate, using the endpoint by:
/// <https://github.com/ssut/py-googletrans/issues/268#issuecomment-1146554742>
/// This will only successs for a small number of words
pub fn lookup_google_translate(
    search_options: SearchConfig,
) -> Result<std::string::String, TranslateError> {
    let url = reqwest::Url::parse_with_params(
        "https://clients5.google.com/translate_a/single",
        &[
            ("dj", "1"),
            ("dt", "bd"),
            ("client", "dict-chrome-ex"),
            (
                "sl",
                match search_options.source_language {
                    Some(lang) => lang.to_639_1().unwrap_or("auto"),
                    None => "auto",
                },
            ),
            ("tl", search_options.target_language.to_639_1().unwrap()),
            ("q", &search_options.query),
        ],
    )
    .unwrap();
    let response: reqwest::blocking::Response = reqwest::blocking::get(url).unwrap();
    let body: GoogleTranslateResponse = dbg!(response.json().unwrap());
    if body.dict.is_some() {
        Ok(body.dict.unwrap()[0].entry[0].word.clone())
    } else {
        Err(TranslateError(search_options.query))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_google_translate_derive_deserialize() {
        let jsondata = r#"
        {"dict":[{"pos":"interjection","terms":["もしもし！","今日は!"],"entry":[{"word":"もしもし！","reverse_translation":["Hello!"],"score":0.004559123},{"word":"今日は!","reverse_translation":["Hi!","Hello!","Good afternoon!","Good day!"]}],"base_form":"Hello!","pos_enum":9}],"src":"en","confidence":1.0,"spell":{},"ld_result":{"srclangs":["en"],"srclangs_confidences":[1.0],"extended_srclangs":["en"]}}
        "#;
        let response: GoogleTranslateResponse = serde_json::from_str(jsondata).unwrap();
        assert_eq!(response.src, "en");
    }

    #[test]
    fn test_google_translate_lookup() {
        let search_options = SearchConfig {
            query: "Good Morning".to_string(),
            source_language: Some(isolang::Language::Eng),
            target_language: isolang::Language::from_name("Japanese").unwrap(),
        };

        assert_eq!("お早う", lookup_google_translate(search_options).unwrap())
    }
}
