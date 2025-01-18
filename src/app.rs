use ratatui::backend::Backend;
use ratatui::crossterm::event;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{self, Block, Paragraph};

use crate::search::{self, SearchConfig, SearchResult};

#[derive(Debug)]
pub struct App {
    search_config: Option<SearchConfig>,
    results: Option<search::SearchResult>,
    running_state: RunningState,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Start,
    Searching,
    Result,
    Finished,
}

#[derive(Debug)]
enum Message {
    QueryReceived(SearchConfig),
    Searching(SearchConfig),
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
        // Search the user-given query first
        let mut cur_message: Option<Message> =
            Some(Message::QueryReceived(self.search_config.clone().unwrap()));

        while self.running_state != RunningState::Finished {
            terminal.draw(|f| self.view(f)).unwrap();

            if cur_message.is_none() {
                cur_message = self.handle_event();
            }

            if let Some(msg) = cur_message.take() {
                cur_message = self.update(msg);
            }
        }
    }

    fn handle_event(&self) -> Option<Message> {
        let event = event::read().expect("Failed to read event");
        match event {
            event::Event::FocusGained => todo!(),
            event::Event::FocusLost => todo!(),
            event::Event::Key(key_event) => {
                if key_event.kind == event::KeyEventKind::Press {
                    return handle_key(key_event);
                }
            }
            event::Event::Mouse(_mouse_event) => todo!(),
            event::Event::Paste(_) => todo!(),
            event::Event::Resize(_, _) => todo!(),
        }

        None
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::ResultReceived(search_result) => {
                self.results = Some(search_result);
                self.running_state = RunningState::Result;
                None
            }
            Message::Quit => {
                self.running_state = RunningState::Finished;
                None
            }
            Message::QueryReceived(search_config) => {
                self.running_state = RunningState::Searching;
                Some(Message::Searching(search_config))
            }
            Message::Searching(search_config) => {
                Some(Message::ResultReceived(search(&search_config).unwrap()))
            }
        }
    }

    fn view(&self, frame: &mut ratatui::Frame) {
        let title = Line::from("What Do You Mean?".bold());
        let bottom_title = Line::from("Press <q> to quit");
        let block = Block::bordered().title(title.centered()).title_bottom(bottom_title.centered());

        let content: Paragraph = match self.running_state {
            RunningState::Searching => Paragraph::new(Line::from(vec![
                "Searching with: ".italic().into(),
                self.search_config
                    .as_ref()
                    .expect("Should have a config")
                    .provider
                    .to_string()
                    .italic()
                    .bold()
                    .into(),
            ])),
            RunningState::Result => {
                format_result(&self.results.as_ref().expect("Should have a result"))
            }
            RunningState::Finished => Paragraph::new(""),
            RunningState::Start => Paragraph::new("Started".italic()),
        };

        frame.render_widget(
            Paragraph::from(content)
                .left_aligned()
                .block(block)
                .wrap(widgets::Wrap { trim: false }),
            frame.area(),
        );
    }
}

fn format_result(result: &SearchResult) -> Paragraph {
    let provider: Line = result.provider.to_string().bold().into();
    let mut res: Vec<Line> = vec![provider];

    if let Some(translations) = &result.translation {
        let mut lines: Vec<Line> = Vec::new();
        for translation in translations {
            lines.push(Line::from(
                translation.orig.clone().unwrap_or("".to_string()).italic(),
            ));
            lines.push(Line::from(
                translation
                    .translated
                    .clone()
                    .unwrap_or("".to_string())
                    .bold(),
            ));
        }
        res.append(&mut lines);
    }

    Paragraph::new(res)
}

fn search(search_config: &SearchConfig) -> Option<SearchResult> {
    search::lookup(search_config).ok()
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        event::KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}
