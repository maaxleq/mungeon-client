use crate::model;
use crate::session;

use crossterm::event;
use crossterm::execute;
use crossterm::terminal;
use tui::backend;
use tui::layout;
use tui::style;
use tui::text;
use tui::widgets;
use tui::Terminal;

use std::error;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;

static STYLE_GREEN: style::Style = style::Style {
    fg: Some(style::Color::Black),
    bg: Some(style::Color::Green),
    add_modifier: style::Modifier::empty(),
    sub_modifier: style::Modifier::empty(),
};

static STYLE_RED: style::Style = style::Style {
    fg: Some(style::Color::White),
    bg: Some(style::Color::Red),
    add_modifier: style::Modifier::empty(),
    sub_modifier: style::Modifier::empty(),
};

static STYLE_TITLE: style::Style = style::Style {
    fg: Some(style::Color::White),
    bg: Some(style::Color::Blue),
    add_modifier: style::Modifier::empty(),
    sub_modifier: style::Modifier::empty(),
};

static STYLE_DUNGEON: style::Style = style::Style {
    fg: Some(style::Color::White),
    bg: Some(style::Color::Blue),
    add_modifier: style::Modifier::empty(),
    sub_modifier: style::Modifier::empty(),
};

static STYLE_RESET: style::Style = style::Style {
    fg: Some(style::Color::Reset),
    bg: Some(style::Color::Reset),
    add_modifier: style::Modifier::empty(),
    sub_modifier: style::Modifier::empty(),
};

enum ChannelEvent<I> {
    Input(I),
    Tick,
}

#[derive(Clone)]
struct PopupManager {
    pub popup_mode: bool,
    pub title: String,
    pub infos: Vec<String>,
    pub list: widgets::ListState
}

impl PopupManager {
    pub fn new() -> PopupManager {
        PopupManager {
            popup_mode: false,
            infos: Vec::new(),
            title: String::new(),
            list: widgets::ListState::default(),
        }
    }
}

pub struct Runner {
    pub terminal: Terminal<backend::CrosstermBackend<std::io::Stdout>>,
    session: session::Session,
    receiver: mpsc::Receiver<ChannelEvent<event::KeyEvent>>,
    popup_manager: PopupManager,
}

impl Runner {
    pub fn try_new(session: session::Session) -> Result<Runner, io::Error> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let backend = backend::CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        let (sender, receiver) = mpsc::channel::<ChannelEvent<event::KeyEvent>>();
        let tick_rate = 100;

        Runner::spawn_sender_thread(sender, tick_rate);

        Ok(Runner {
            session: session,
            terminal: terminal,
            receiver: receiver,
            popup_manager: PopupManager::new(),
        })
    }

    fn spawn_sender_thread(sender: mpsc::Sender<ChannelEvent<event::KeyEvent>>, tick_rate: u64) {
        let tick_rate = time::Duration::from_millis(tick_rate);

        thread::spawn(move || {
            let mut last_tick = time::Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| time::Duration::from_secs(0));
                if event::poll(timeout).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        sender.send(ChannelEvent::Input(key)).unwrap();
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    sender.send(ChannelEvent::Tick).unwrap();
                    last_tick = time::Instant::now();
                }
            }
        });
    }

    fn draw(&mut self) -> Result<(), io::Error> {
        let session = self.session.clone();
        let popup_manager = self.popup_manager.clone();

        self.terminal.draw(|f| {
            let size = f.size();

            let x_chunks = layout::Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints(
                    [
                        layout::Constraint::Percentage(30),
                        layout::Constraint::Percentage(70),
                    ]
                    .as_ref(),
                )
                .split(size);

            let left_chunks = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(
                    [
                        layout::Constraint::Percentage(30),
                        layout::Constraint::Percentage(70),
                    ]
                    .as_ref(),
                )
                .split(x_chunks[0]);

            let net_block = widgets::Block::default()
                .title("Net")
                .borders(widgets::Borders::ALL);

            let status_block = widgets::Block::default()
                .title("Status")
                .borders(widgets::Borders::ALL);

            let dungeon_block = widgets::Block::default()
                .title("Dungeon")
                .style(if matches!(session.status.clone(), Some(_)) {
                    STYLE_DUNGEON
                } else {
                    STYLE_RESET
                })
                .borders(widgets::Borders::ALL);

            let net_paragraph = widgets::Paragraph::new(vec![
                text::Spans::from(text::Span::raw(session.client.base_url.clone())),
                text::Spans::from(if session.is_connected().clone() {
                    text::Span::styled("CONNECTED", STYLE_GREEN)
                } else {
                    text::Span::styled("DISCONNECTED", STYLE_RED)
                }),
            ])
            .block(net_block)
            .wrap(widgets::Wrap { trim: false });

            let status_spans = match session.status.clone() {
                Some(status) => {
                    let temp_vec = vec![
                        text::Spans::from(vec![
                            text::Span::styled("LIFE", STYLE_TITLE),
                            text::Span::raw(String::from(" ")),
                            text::Span::raw(status.total_life.clone().to_string()),
                        ]),
                        text::Spans::from(text::Span::raw(String::from("\n"))),
                        text::Spans::from(vec![
                            text::Span::styled("ROOM", STYLE_TITLE),
                            text::Span::raw(String::from(" ")),
                            text::Span::raw(status.room.description.clone()),
                        ]),
                    ];

                    temp_vec
                }
                None => Vec::new(),
            };

            let status_paragraph = widgets::Paragraph::new(status_spans)
                .block(status_block)
                .wrap(widgets::Wrap { trim: false });

            let dungeon_canvas = widgets::canvas::Canvas::default()
                .block(dungeon_block)
                .background_color(STYLE_DUNGEON.bg.unwrap())
                .paint(|ctx| {
                    if matches!(session.status.clone(), Some(_)) {
                        ctx.draw(&widgets::canvas::Rectangle {
                            x: 0.0,
                            y: 0.0,
                            width: 100.0,
                            height: 100.0,
                            color: STYLE_DUNGEON.fg.unwrap(),
                        });
                        ctx.layer();
                        match session.status.clone() {
                            Some(status) => {
                                for direction in status.room.paths.clone().iter() {
                                    match direction {
                                        model::Direction::N => {
                                            ctx.print(50.0, 100.0, "N", STYLE_DUNGEON.fg.unwrap())
                                        }
                                        model::Direction::S => {
                                            ctx.print(50.0, 0.0, "S", STYLE_DUNGEON.fg.unwrap())
                                        }
                                        model::Direction::W => {
                                            ctx.print(0.0, 50.0, "W", STYLE_DUNGEON.fg.unwrap())
                                        }
                                        model::Direction::E => {
                                            ctx.print(100.0, 50.0, "E", STYLE_DUNGEON.fg.unwrap())
                                        }
                                    }
                                }
                            }
                            None => (),
                        };
                    }
                })
                .x_bounds([00.0, 100.0])
                .y_bounds([00.0, 100.0]);

            let entities_block = widgets::Block::default().borders(widgets::Borders::NONE);
            let mut entities_string = String::new();
            for entity in session.get_entities_keys().iter() {
                entities_string += entity.to_string().as_str();
                entities_string += "   ";
            }
            let entities_paragraph =
                widgets::Paragraph::new(vec![text::Spans::from(entities_string)])
                    .block(entities_block)
                    .wrap(widgets::Wrap { trim: false });
            let entities_area = Runner::centered_rect(80, 80, x_chunks[1]);
            f.render_widget(widgets::Clear, entities_area);
            f.render_widget(entities_paragraph, entities_area);

            f.render_widget(net_paragraph, left_chunks[0]);
            f.render_widget(status_paragraph, left_chunks[1]);
            f.render_widget(dungeon_canvas, x_chunks[1]);

            if popup_manager.popup_mode {
                let mut popup_spans: Vec<text::Spans> = Vec::new();
                for info in popup_manager.infos.clone().iter() {
                    popup_spans.push(text::Spans::from(info.clone()));
                }
                let popup_block = widgets::Block::default()
                    .title(popup_manager.title.clone())
                    .borders(widgets::Borders::ALL);
                let popup_paragraph = widgets::Paragraph::new(popup_spans)
                    .block(popup_block)
                    .wrap(widgets::Wrap { trim: false });
                let area = Runner::centered_rect(60, 60, size);
                f.render_widget(widgets::Clear, area);
                f.render_widget(popup_paragraph, area);
            }
        })?;

        Ok(())
    }

    fn handle_errors(&mut self) {
        match self.session.error.clone() {
            Some(err) => {
                self.popup_manager.popup_mode = true;
                self.popup_manager.title = String::from("Error");

                let mut infos_vec: Vec<String> = Vec::new();
                match err.code {
                    Some(code) => infos_vec.push(format!("Code: {}", code)),
                    None => (),
                }
                match err.detail.r#type {
                    Some(r#type) => infos_vec.push(format!("Type: {}", r#type.to_string())),
                    None => (),
                }
                infos_vec.push(format!("Message: {}", err.detail.message));
                self.popup_manager.infos = infos_vec;
            }
            _ => (),
        }
    }

    fn display_keybinds(&mut self){
        self.popup_manager.popup_mode = true;
        self.popup_manager.title = String::from("Keybinds");
        self.popup_manager.infos = vec![
            String::from("[c]       (re)connect"),
            String::from("[d]       disconnect"),
            String::from("[l]       look around"),
            String::from("[e]       look entity"),
            String::from("[a]       attack"),
            String::from("[arrows]  move"),
            String::from("[q]       quit"),
        ];
    }

    pub fn run(&mut self) -> Result<(), Box<dyn error::Error>> {
        loop {
            self.handle_errors();

            self.draw()?;

            match self.receiver.recv()? {
                ChannelEvent::Input(event) => match event.code {
                    event::KeyCode::Char('q') => {
                        self.restore_terminal()?;
                        break;
                    }
                    e => self.handle_input(e),
                },
                _ => (),
            }
        }

        Ok(())
    }

    // method from https://github.com/fdehau/tui-rs/blob/v0.16.0/examples/popup.rs
    fn centered_rect(percent_x: u16, percent_y: u16, r: layout::Rect) -> layout::Rect {
        let popup_layout = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(
                [
                    layout::Constraint::Percentage((100 - percent_y) / 2),
                    layout::Constraint::Percentage(percent_y),
                    layout::Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        layout::Layout::default()
            .direction(layout::Direction::Horizontal)
            .constraints(
                [
                    layout::Constraint::Percentage((100 - percent_x) / 2),
                    layout::Constraint::Percentage(percent_x),
                    layout::Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    fn restore_terminal(&mut self) -> Result<(), io::Error> {
        terminal::disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }

    fn handle_input(&mut self, e: event::KeyCode) {
        match self.popup_manager.popup_mode {
            true => match e {
                event::KeyCode::Enter => {
                    self.popup_manager.popup_mode = false;
                    self.session.clear_infos();
                }
                _ => (),
            },
            false => match e {
                event::KeyCode::Char(c) => match c {
                    'c' => self.session.connect(),
                    'd' => self.session.disconnect(),
                    'l' => self.session.look_room(),
                    'h' => self.display_keybinds(),
                    _ => (),
                },
                event::KeyCode::Up => self.session.r#move(model::Direction::N),
                event::KeyCode::Down => self.session.r#move(model::Direction::S),
                event::KeyCode::Right => self.session.r#move(model::Direction::E),
                event::KeyCode::Left => self.session.r#move(model::Direction::W),
                _ => (),
            },
        }
    }
}
