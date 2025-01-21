use clap::Parser;
use color_eyre::eyre::Result;

use wdym::app::App;
use wdym::search::parse_lang;
use wdym::search::SearchConfig;
use wdym::{search::LanguageParseError, translators::SearchProvider};

#[derive(Parser, Debug)]
#[command(version, about)]
struct CliArgs {
    input: String,

    /// Optional name of source language
    #[arg(short, long)]
    source_lang: Option<String>,

    /// Name of target language. Only required for some search providers.
    #[arg(short, long)]
    dest_lang: Option<String>,

    /// What search provider to use.
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

fn main() -> Result<()> {
    color_eyre::install()?;

    let search_config: SearchConfig = CliArgs::parse().try_into()?;

    let mut terminal = ratatui::init();
    let mut app = App::new(search_config);
    let result = app.run(&mut terminal);

    ratatui::restore();

    result
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

        let _search_conf: SearchConfig = args.try_into().unwrap();
    }
}
