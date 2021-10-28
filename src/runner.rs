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

enum ChannelEvent<I> {
    Input(I),
    Tick,
}

struct InputManager {
    pub text_mode: bool,
    current_text: String,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            text_mode: false,
            current_text: String::new(),
        }
    }

    pub fn append_char(&mut self, c: char) {
        self.current_text.push(c);
    }

    pub fn clear(&mut self) {
        self.current_text.clear();
    }
}

pub struct Runner {
    pub terminal: Terminal<backend::CrosstermBackend<std::io::Stdout>>,
    session: session::Session,
    receiver: mpsc::Receiver<ChannelEvent<event::KeyEvent>>,
    input_manager: InputManager,
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
            input_manager: InputManager::new(),
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

            let status_spans = match session.status {
                Some(status) => {
                    let temp_vec = vec![
                        text::Spans::from(vec![
                            text::Span::styled("GUID", STYLE_TITLE),
                            text::Span::raw(String::from(" ")),
                            text::Span::raw(status.guid.clone()),
                        ]),
                        text::Spans::from(text::Span::raw(String::from("\n"))),
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

            f.render_widget(net_paragraph, left_chunks[0]);
            f.render_widget(status_paragraph, left_chunks[1]);
            f.render_widget(dungeon_block, x_chunks[1]);
        })?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn error::Error>> {
        loop {
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

    fn restore_terminal(&mut self) -> Result<(), io::Error> {
        terminal::disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }

    fn handle_input_text_mode(&mut self, e: event::KeyCode) -> bool {
        match e {
            event::KeyCode::Enter => true,
            event::KeyCode::Char(c) => {
                self.input_manager.append_char(c);

                false
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, e: event::KeyCode) {
        match e {
            event::KeyCode::Char(c) => match c {
                'c' => self.session.connect(),
                'd' => self.session.disconnect(),
                'l' => self.session.look_room(),
                _ => (),
            },
            _ => (),
        }
    }
}
