use crate::SearchConfig;

use super::TranslateError;

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct HttpResponse {
    dict: Option<Vec<Option<HttpResponseDict>>>,
    sentences: Option<Vec<Option<HttpResponseSentence>>>,
    src: String,
    confidence: Option<f32>,
    spell: Option<serde_json::Value>,
    ld_result: Option<serde_json::Value>,
}

/// A sentence may have some of the following.
/// A trans and orig are always appearing together
/// TODO: add sentence translation
#[derive(Deserialize, Debug)]
struct HttpResponseSentence {
    src_translit: Option<String>,
    trans: Option<String>,
    orig: Option<String>,
    translit: Option<String>,
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

impl From<HttpResponseDict> for SearchResultDict {
    fn from(value: HttpResponseDict) -> Self {
        SearchResultDict {
            pos: value.pos,
            entry: value.entry,
        }
    }
}

pub struct SearchResult {
    dicts: Option<Vec<SearchResultDict>>,
    sentence_translation: Option<Vec<(String, String)>>,
    src_translit: Option<String>,
    translit: Option<String>,
    src_lang: String,
}

impl From<&(String, String)> for crate::Translation {
    fn from(value: &(String, String)) -> Self {
        Self {
            // TODO: performance: use `&str`
            orig: Some(value.0.clone()),
            translated: Some(value.1.clone()),
        }
    }
}

impl From<SearchResult> for crate::SearchResult {
    fn from(value: SearchResult) -> Self {
        let definitions: Option<Vec<crate::Definition>> = match value.dicts {
            Some(dicts) => {
                let mut res: Vec<crate::Definition> = Vec::new();
                for dict_pos in dicts {
                    let pos: String = dict_pos.pos;
                    for entry in dict_pos.entry {
                        res.push(crate::Definition {
                            meaning: entry.word,
                            pos: pos.clone(),
                            reverse_translation: Some(entry.reverse_translation),
                            confidence: entry.score,
                        });
                    }
                }
                Some(res)
            }
            None => None,
        };
        crate::SearchResult {
            provider: super::SearchProvider::GoogleTranslate,
            translation: value.sentence_translation.map(|sentences| {
                sentences
                    .iter()
                    .map(|sentence_pair| sentence_pair.into())
                    .collect()
            }),
            src_lang: Some(value.src_lang),
            literation: crate::Literation {
                orig: value.src_translit,
                translated: value.translit,
            },
            dictionary: definitions,
        }
    }
}

impl TryFrom<HttpResponse> for SearchResult {
    type Error = TranslateError;

    fn try_from(value: HttpResponse) -> Result<Self, Self::Error> {
        let mut src_translit: Option<String> = None;
        let mut translit: Option<String> = None;

        let mut translations: Vec<(String, String)> = vec![];
        if let Some(sentences) = value.sentences {
            for sentence_opt in sentences {
                if let Some(mut sentence) = sentence_opt {
                    translit = sentence.translit.take();
                    src_translit = sentence.src_translit.take();

                    if sentence.orig.is_some() && sentence.trans.is_some() {
                        translations.push((
                            sentence.orig.take().unwrap(),
                            sentence.trans.take().unwrap(),
                        ));
                    }
                }
            }
        }

        let sentence_translation: Option<Vec<(String, String)>> = match translations.len() {
            0 => None,
            _ => Some(translations),
        };

        let res = SearchResult {
            dicts: match value.dict {
                Some(mut dicts) => Some(
                    dicts
                        .iter_mut()
                        .filter_map(|dict| (*dict).take()?.try_into().ok())
                        .collect(),
                ),
                None => None,
            },
            sentence_translation,
            src_translit,
            translit,
            src_lang: value.src,
        };

        Ok(res)
    }
}

impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "src lang: {}", self.src_lang)?;

        if let Some(translations) = &self.sentence_translation {
            writeln!(f, "translations:")?;
            for line in translations {
                writeln!(f, "\t{}", line.0)?;
                writeln!(f, "\t{}", line.1)?;
            }
        }

        if let Some(dicts) = &self.dicts {
            for dict in dicts {
                writeln!(f, "pos: {}", dict.pos)?;
                for entry in &dict.entry {
                    match entry.score {
                        Some(score) => {
                            write!(f, "\t{} ({:.3}):\n\t\t", entry.word, score)?;
                        }
                        None => {
                            write!(f, "\t{}:\n\t\t", entry.word)?;
                        }
                    }
                    for reverse_translation in &entry.reverse_translation {
                        write!(f, "{} ", reverse_translation)?
                    }
                    writeln!(f, "")?
                }
            }
        }

        if let Some(src_translit) = &self.src_translit {
            writeln!(f, "src_translit:")?;
            pretty_format_section(f, src_translit)?;
        }

        if let Some(translit) = &self.translit {
            writeln!(f, "translit:")?;
            pretty_format_section(f, translit)?;
        }

        Ok(())
    }
}

/// Formats the section
///
/// # Arguments
///
/// * f - The `Formatter` to use
///
/// # Returns
///
/// Error if the formatting fail
fn pretty_format_section(
    f: &mut std::fmt::Formatter<'_>,
    translit: &String,
) -> Result<(), std::fmt::Error> {
    Ok(
        for translit_line in translit.split_inclusive(|c: char| ".?!".contains(c)) {
            writeln!(
                f,
                "\t{}",
                translit_line.strip_prefix(" ").unwrap_or(translit_line)
            )?;
        },
    )
}

/// Looks up the translation on google translate, using the endpoint by:
/// <https://github.com/ssut/py-googletrans/issues/268#issuecomment-1146554742>
/// This will only success for a small number of words
/// TODO: add support for multiple definitions
pub fn lookup_google_translate(
    search_options: SearchConfig,
) -> Result<SearchResult, TranslateError> {
    let url = reqwest::Url::parse_with_params(
        "https://clients5.google.com/translate_a/single",
        &[
            ("dj", "1"),
            ("dt", "at"),
            ("dt", "bd"),
            ("dt", "rm"), // Transliteration
            ("dt", "rw"),
            ("dt", "sp"),
            ("dt", "ss"),
            ("dt", "t"),
            ("client", "dict-chrome-ex"),
            (
                "sl",
                match search_options.source_language {
                    Some(lang) => lang.to_639_1().unwrap_or("auto"),
                    None => "auto",
                },
            ),
            (
                "tl",
                search_options
                    .target_language
                    .ok_or_else(|| {
                        TranslateError(
                            "google translate requires a destination language".to_string(),
                        )
                    })?
                    .to_639_1()
                    .expect("Should have a corresponding language"),
            ),
            ("q", &search_options.query),
        ],
    )
    .unwrap();
    let response: reqwest::blocking::Response = reqwest::blocking::get(url).unwrap();
    let body: HttpResponse = response.json().unwrap();
    let search_result: SearchResult = body.try_into()?;
    Ok(search_result)
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    #[test]
    fn deserialize_json() {
        let jsondata = r#"
        {"dict":[{"pos":"interjection","terms":["もしもし！","今日は!"],"entry":[{"word":"もしもし！","reverse_translation":["Hello!"],"score":0.004559123},{"word":"今日は!","reverse_translation":["Hi!","Hello!","Good afternoon!","Good day!"]}],"base_form":"Hello!","pos_enum":9}],"src":"en","confidence":1.0,"spell":{},"ld_result":{"srclangs":["en"],"srclangs_confidences":[1.0],"extended_srclangs":["en"]}}
        "#;
        let response: HttpResponse = serde_json::from_str(jsondata).unwrap();
        assert_eq!(response.src, "en");
    }

    #[test]
    fn simple_lookup() {
        let search_options = SearchConfig {
            provider: crate::translators::SearchProvider::GoogleTranslate,
            query: "Good Morning".to_string(),
            source_language: Some(isolang::Language::Eng),
            target_language: Some(isolang::Language::from_name("Japanese").unwrap()),
        };

        assert!(lookup_google_translate(search_options)
            .unwrap()
            .to_string()
            .contains("お早う"));
    }

    /// Test the transliteration and translation
    ///
    /// # Examples
    /// ```rust
    /// write me later
    /// ```
    #[test]
    fn translit_translate() {
        let search_options = SearchConfig {
            query: "Typer is a library for building CLI applications that users will love using and developers will love creating. Based on Python type hints. It's also a command line tool to run scripts, automatically converting them to CLI applications. The key features are: Intuitive to write: Great editor support. Completion everywhere. Less time debugging. Designed to be easy to use and learn. Less time reading docs. Easy to use: It's easy to use for the final users. Automatic help, and automatic completion for all shells. Short: Minimize code duplication. Multiple features from each parameter declaration. Fewer bugs. Start simple: The simplest example adds only 2 lines of code to your app: 1 import, 1 function call. Grow large: Grow in complexity as much as you want, create arbitrarily complex trees of commands and groups of subcommands, with options and arguments. Run scripts: Typer includes a typer command/program that you can use to run scripts, automatically converting them to CLIs, even if they don't use Typer internally. ".to_string(),
            source_language: Some(isolang::Language::Eng),
            target_language: Some(isolang::Language::Jpn),
            provider: crate::translators::SearchProvider::GoogleTranslate,
        };

        let res = lookup_google_translate(search_options).unwrap().to_string();

        // sentence translation
        assert!(res.contains("Typerは、ユーザーが使用するのが大好きなCLIアプリケーションを構築するライブラリであり、開発者が作成するのが大好きです。 "));

        // Transliteration
        assert!(res.contains("Typer wa, yūzā ga shiyō suru no ga daisukina CLI apurikēshon o kōchiku suru raiburarideari, kaihatsu-sha ga sakusei suru no ga daisukidesu."))
    }

    #[test]
    fn test_source_translit() {
        let search_options = SearchConfig {
            query: "計算".to_string(),
            source_language: Some(isolang::Language::Jpn),
            target_language: Some(isolang::Language::Eng),
            provider: crate::translators::SearchProvider::GoogleTranslate,
        };

        assert!(lookup_google_translate(search_options)
            .unwrap()
            .to_string()
            .contains("Keisan"));
    }
}
