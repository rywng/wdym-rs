use clap::Parser;

use isolang::Language;
use wdym::search;
use wdym::search::parse_lang;
use wdym::search::SearchConfig;
use wdym::{translators::SearchProvider, search::LanguageParseError};

#[derive(Parser, Debug)]
#[command(version, about)]
struct CliArgs {
    input: String,

    #[arg(short, long)]
    source_lang: Option<String>,

    #[arg(short, long)]
    dest_lang: Option<String>,

    #[arg(short, long, default_value_t = SearchProvider::GoogleTranslate)]
    provider: SearchProvider,
}

impl TryInto<SearchConfig> for CliArgs {
    type Error = LanguageParseError;

    fn try_into(self) -> Result<SearchConfig, Self::Error> {
        let res: SearchConfig = SearchConfig {
            query: self.input,
            source_language: match self.source_lang {
                Some(lang) => Some(parse_lang(lang)?),
                None => None,
            },
            target_language: match self.dest_lang {
                Some(lang) => Some(parse_lang(lang)?),
                None => None,
            },
            provider: self.provider,
        };

        Ok(res)
    }
    // add code here
}

fn main() {
    let args = CliArgs::parse();

    let res = search::lookup(args.try_into().unwrap()).unwrap();

    dbg!(res);
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[should_panic]
    fn invalid_cli_args_source() {
        let args = CliArgs {
            input: "book".to_owned(),
            source_lang: Some("invalid language for test".to_string()),
            dest_lang: None,
            provider: SearchProvider::GoogleTranslate,
        };

        let _search_conf: SearchConfig = args.try_into().unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_cli_args_dest() {
        let args = CliArgs {
            input: "book".to_owned(),
            source_lang: None,
            dest_lang: Some("invalid language for test".to_string()),
            provider: SearchProvider::GoogleTranslate,
        };

        let _search_conf: search::SearchConfig = args.try_into().unwrap();
    }
}
