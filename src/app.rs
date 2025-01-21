use ratatui::backend::Backend;
use ratatui::text::{Line, Span};
use ratatui::widgets::{self, Block, Paragraph, Widget};
use ratatui::{crossterm::event, style::Stylize};

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

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("What Do You Mean?".bold());
        let bottom_title = Line::from("Press <q> to quit");
        let block = Block::bordered()
            .border_type(widgets::BorderType::Rounded)
            .title(title.centered())
            .title_bottom(bottom_title.centered())
            .padding(widgets::Padding::horizontal(1));

        let inner_area = block.inner(area);

        block.render(area, buf);

        match self.running_state {
            RunningState::Start => {
                "Starting".italic().render(inner_area, buf);
            }
            RunningState::Searching => {
                Line::from(vec![
                    "Searching for: ".italic(),
                    self.search_config
                        .as_ref()
                        .expect("Should have a search config")
                        .query
                        .to_string()
                        .italic()
                        .bold(),
                ])
                .render(inner_area, buf);
            }
            RunningState::Result => {
                render_result(
                    &self.results.as_ref().expect("Should have a result"),
                    &self
                        .search_config
                        .as_ref()
                        .expect("Should have a search config"),
                    inner_area,
                    buf,
                );
            }
            _ => {}
        };
    }
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
        frame.render_widget(self, frame.area());
    }
}

fn render_result(
    result: &SearchResult,
    config: &SearchConfig,
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
) {
    let provider = result.provider.to_string().bold().cyan();
    let language = Line::from(vec![
        config
            .source_language
            .unwrap_or(isolang::Language::Und)
            .to_string().italic(),
        " -> ".into(),
        config
            .target_language
            .unwrap_or(isolang::Language::Und)
            .to_string().italic(),
    ])
    .cyan().right_aligned();
    let block: Block = Block::bordered()
        .title(provider)
        .title(language)
        .padding(widgets::Padding::horizontal(1));
    let mut res: Vec<Line> = Vec::new();

    if let Some(definitions) = &result.definitions {
        make_title(&mut res, "Definitions");
        for definition in definitions {
            let mut line: Vec<Span> = vec![
                definition.meaning.clone().underlined().cyan(),
                format!(" ({})", definition.pos).italic().dim(),
            ];
            if let Some(reverse_translation) = &definition.reverse_translation {
                let mut translations: Vec<Span> = reverse_translation
                    .iter()
                    .map(|s| format!("{} ", s).italic())
                    .collect();
                line.push(": ".into());
                line.append(&mut translations);
            }
            res.push(Line::from(line));
        }
    }

    if let Some(translations) = &result.translations {
        make_title(&mut res, "Translations");
        for translation in translations {
            res.push(Line::from(
                translation.orig.clone().unwrap_or("".to_string()).italic(),
            ));
            res.push(Line::from(
                translation
                    .translated
                    .clone()
                    .unwrap_or("".to_string())
                    .bold(),
            ));
        }
    }

    if let Some(literation) = &result.literation {
        make_title(&mut res, "Literations");
        if let Some(original) = &literation.orig {
            res.push(vec!["Original: ".bold().dim(), original.clone().italic().into()].into());
        }
        if let Some(translated) = &literation.translated {
            res.push(vec!["Translated: ".bold().dim(), translated.clone().into()].into());
        }
    }

    Paragraph::new(res)
        .wrap(widgets::Wrap { trim: true })
        .block(block)
        .render(area, buf);
}

fn make_title<'a>(res: &mut Vec<Line<'a>>, title: &'a str) {
    res.push("".into());
    res.push(title.bold().blue().into());
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
