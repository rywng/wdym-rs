use ratatui::backend::Backend;

use crate::search::{self, SearchConfig, SearchResult};

#[derive(Debug)]
pub struct App {
    results: Option<search::SearchResult>,
    running_state: RunningState,
    search_config: Option<SearchConfig>,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Finished,
}

#[derive(Debug)]
enum Message {
    QueryReceived(SearchConfig),
    ResultReceived(SearchResult),
    Quit,
}

impl App {
    pub fn new<'a>(search_config: search::SearchConfig) -> App {
        App {
            results: None,
            running_state: Default::default(),
            search_config: Some(search_config),
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::Terminal<impl Backend>) {
        let mut message: Option<Message> =
            Some(Message::QueryReceived(self.search_config.take().unwrap()));
        while self.running_state != RunningState::Finished {
            while let Some(msg) = message.take() {
                message = self.update(msg);
            }

            terminal.draw(|f| self.view(f)).unwrap();
        }
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::ResultReceived(search_result) => {
                self.results = Some(search_result);
                Some(Message::Quit)
            }
            Message::Quit => {
                self.running_state = RunningState::Finished;
                None
            }
            Message::QueryReceived(search_config) => {
                Some(Message::ResultReceived(search(&search_config).unwrap()))
            }
        }
    }

    fn view(&mut self, frame: &mut ratatui::Frame) {
        frame.render_widget(
            ratatui::text::Text::from(format!("{:?}", self.results)),
            frame.area(),
        );
    }
}

fn search(search_config: &SearchConfig) -> Option<SearchResult> {
    search::lookup(&search_config).ok()
}
