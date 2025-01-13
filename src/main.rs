use clap::Parser;

use wdym::modname::lookup_google_translate;
use wdym::SearchConfig;

#[derive(Parser, Debug)]
#[command(version, about)]
struct CliArgs {
    input: String,

    #[arg(short, long)]
    source_lang: Option<String>,

    #[arg(short, long)]
    dest_lang: String,
}

impl TryInto<SearchConfig> for CliArgs {
    type Error = String;

    fn try_into(self) -> Result<SearchConfig, Self::Error> {
        let res: SearchConfig = SearchConfig {
            query: self.input,
            source_language: match self.source_lang {
                Some(lang) => isolang::Language::from_639_1(&lang),
                None => None,
            },
            target_language: match isolang::Language::from_639_1(&self.dest_lang) {
                Some(lang) => lang,
                None => return Err("Failed to match the language".to_string()),
            },
        };

        Ok(res)
    }
    // add code here
}

fn main() {
    let args = CliArgs::parse();

    println!("{}", lookup_google_translate(args.try_into().unwrap()).unwrap());
}
