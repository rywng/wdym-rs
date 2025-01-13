use super::SearchConfig;
use super::TranslateError;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct HttpResponse {
    dict: Option<Vec<Option<HttpResponseDict>>>,
    src: String,
    confidence: Option<f32>,
    spell: Option<serde_json::Value>,
    ld_result: Option<serde_json::Value>,
}

impl TryInto<SearchResult> for HttpResponse {
    type Error = TranslateError;

    fn try_into(self) -> Result<SearchResult, Self::Error> {
        match self.dict {
            Some(mut dicts) => Ok(SearchResult {
                dicts: dicts
                    .iter_mut()
                    .filter_map(|dict| dict.take().try_into().ok())
                    .collect(),
                src_lang: self.src,
            }),
            None => Err(TranslateError("no answer possible".to_string())),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct HttpResponseDict {
    pos: String,
    terms: Vec<String>,
    entry: Vec<SearchResultEntry>,
    base_form: String,
    pos_enum: i32,
}

impl TryInto<SearchResultDict> for Option<HttpResponseDict> {
    type Error = TranslateError;

    fn try_into(self) -> Result<SearchResultDict, Self::Error> {
        match self {
            Some(dict) => Ok(SearchResultDict {
                pos: dict.pos,
                entry: dict.entry,
            }),
            None => Err(TranslateError("No entry".to_string())),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct SearchResultEntry {
    word: String,
    reverse_translation: Vec<String>,
    score: Option<f32>,
}

struct SearchResultDict {
    pos: String,
    entry: Vec<SearchResultEntry>,
}

struct SearchResult {
    dicts: Vec<SearchResultDict>,
    src_lang: String,
}

impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "src lang: {}", self.src_lang)?;
        for dict in &self.dicts {
            writeln!(f, "pos: {}", dict.pos)?;
            for entry in &dict.entry {
                write!(
                    f,
                    "\t{} ({:.3}):\n\t\t",
                    entry.word,
                    entry.score.unwrap_or(0.0)
                )?;
                for reverse_translation in &entry.reverse_translation {
                    write!(f, "{} ", reverse_translation)?
                }
                writeln!(f, "")?
            }
        }

        Ok(())
    }
}

/// Looks up the translation on google translate, using the endpoint by:
/// <https://github.com/ssut/py-googletrans/issues/268#issuecomment-1146554742>
/// This will only success for a small number of words
/// TODO: add support for multiple definitions
pub fn lookup_google_translate(search_options: SearchConfig) -> Result<String, TranslateError> {
    let url = reqwest::Url::parse_with_params(
        "https://clients5.google.com/translate_a/single",
        &[
            ("dj", "1"),
            ("dt", "t"),
            ("dt", "sp"),
            ("dt", "ld"),
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
    let body: HttpResponse = response.json().unwrap();
    let search_result: SearchResult = body.try_into()?;
    Ok(format!("{}", search_result))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_google_translate_derive_deserialize() {
        let jsondata = r#"
        {"dict":[{"pos":"interjection","terms":["もしもし！","今日は!"],"entry":[{"word":"もしもし！","reverse_translation":["Hello!"],"score":0.004559123},{"word":"今日は!","reverse_translation":["Hi!","Hello!","Good afternoon!","Good day!"]}],"base_form":"Hello!","pos_enum":9}],"src":"en","confidence":1.0,"spell":{},"ld_result":{"srclangs":["en"],"srclangs_confidences":[1.0],"extended_srclangs":["en"]}}
        "#;
        let response: HttpResponse = serde_json::from_str(jsondata).unwrap();
        assert_eq!(response.src, "en");
    }

    #[test]
    fn test_google_translate_lookup() {
        let search_options = SearchConfig {
            query: "Good Morning".to_string(),
            source_language: Some(isolang::Language::Eng),
            target_language: isolang::Language::from_name("Japanese").unwrap(),
        };

        assert!(lookup_google_translate(search_options)
            .unwrap()
            .contains("お早う"));
    }
}
