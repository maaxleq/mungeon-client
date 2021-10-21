use crate::session;

use console_engine::crossterm::terminal;
use console_engine::*;

static PIXEL_BORDER_VERTICAL: pixel::Pixel = pixel::Pixel {
    fg: Color::Reset,
    bg: Color::Reset,
    chr: '｜',
};

static PIXEL_BORDER_HORIZONTAL: pixel::Pixel = pixel::Pixel {
    fg: Color::Reset,
    bg: Color::Reset,
    chr: '-',
};

static COLOR_INFO_BG: Color = Color::White;
static COLOR_INFO_SUCESS: Color = Color::DarkGreen;
static COLOR_INFO_FAILURE: Color = Color::DarkRed;

struct ScreenChunk {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

type ScreenCut = (ScreenChunk, ScreenChunk, ScreenChunk, ScreenChunk);

trait GetScreenCut {
    fn get(width: u32, height: u32) -> ScreenCut;
}

impl GetScreenCut for ScreenCut {
    fn get(width: u32, height: u32) -> ScreenCut {
        (
            ScreenChunk {
                x: 0,
                y: 0,
                width: width / 2,
                height: height / 2,
            },
            ScreenChunk {
                x: width / 2 + 1,
                y: 0,
                width: width - width / 2,
                height: height / 2,
            },
            ScreenChunk {
                x: 0,
                y: height / 2 + 1,
                width: width / 2,
                height: height - height / 2,
            },
            ScreenChunk {
                x: width / 2 + 1,
                y: height / 2 + 1,
                width: width - width / 2,
                height: height - height / 2,
            },
        )
    }
}

pub struct Runner<'a> {
    session: session::Session<'a>,
    ui_engine: ConsoleEngine,
    game_screen: screen::Screen,
    status_screen: screen::Screen,
    info_screen: screen::Screen,
    keybinds_screen: screen::Screen,
    screen_cut: ScreenCut,
}

impl<'a> Runner<'a> {
    pub fn new(url: String) -> Runner<'a> {
        let (width, height) = terminal::size().unwrap();
        let cut = ScreenCut::get(width as u32, height as u32);

        Runner {
            session: session::Session::new(url),
            ui_engine: ConsoleEngine::init(width as u32, height as u32, 30).unwrap(),
            game_screen: screen::Screen::new(cut.0.width, cut.0.height),
            status_screen: screen::Screen::new(cut.1.width, cut.1.height),
            keybinds_screen: screen::Screen::new(cut.2.width, cut.2.height),
            info_screen: screen::Screen::new(cut.3.width, cut.3.height),
            screen_cut: cut,
        }
    }

    fn adapt_size(&mut self) {
        let (width, height) = terminal::size().unwrap();

        self.ui_engine.resize(width as u32, height as u32);

        self.game_screen
            .resize(self.screen_cut.0.width, self.screen_cut.0.height);
        self.status_screen
            .resize(self.screen_cut.0.width, self.screen_cut.0.height);
        self.keybinds_screen
            .resize(self.screen_cut.0.width, self.screen_cut.0.height);
        self.info_screen
            .resize(self.screen_cut.0.width, self.screen_cut.0.height);
    }

    fn clear(&mut self) {
        self.ui_engine.clear_screen();
        self.game_screen.clear();
        self.status_screen.clear();
        self.keybinds_screen.clear();
        self.info_screen.clear();
    }

    fn display(&mut self) {
        self.ui_engine.draw();
    }

    fn will_quit(&self) -> bool {
        self.ui_engine.is_key_pressed(KeyCode::Esc)
    }

    fn draw_screens(&mut self) {
        self.ui_engine.print_screen(
            self.screen_cut.0.x as i32,
            self.screen_cut.0.y as i32,
            &self.game_screen,
        );
        self.ui_engine.print_screen(
            self.screen_cut.1.x as i32,
            self.screen_cut.1.y as i32,
            &self.status_screen,
        );
        self.ui_engine.print_screen(
            self.screen_cut.2.x as i32,
            self.screen_cut.2.y as i32,
            &self.keybinds_screen,
        );
        self.ui_engine.print_screen(
            self.screen_cut.3.x as i32,
            self.screen_cut.3.y as i32,
            &self.info_screen,
        );
    }

    fn draw_border(&mut self) {
        let (width, height) = terminal::size().unwrap();

        let break_x = width / 2;
        let break_y = height / 2;

        for y in 0..height {
            self.ui_engine
                .set_pxl(break_x as i32, y as i32, PIXEL_BORDER_VERTICAL);
        }

        for x in 0..width {
            self.ui_engine
                .set_pxl(x as i32, break_y as i32, PIXEL_BORDER_HORIZONTAL);
        }
    }

    fn cut_screen(&mut self) {
        let (width, height) = terminal::size().unwrap();

        self.screen_cut = ScreenCut::get(width as u32, height as u32);
    }

    fn handle_input(&mut self) {
        if self.ui_engine.is_key_pressed(KeyCode::Char('c')){
            self.session.connect();
        }
        if self.ui_engine.is_key_pressed(KeyCode::Char('d')){
            self.session.disconnect();
        }
    }

    fn fill_game_screen(&mut self) {
        self.game_screen.print(0, 0, "MUNGEON");
        self.game_screen.print(0, 1, format!("{}", self.session.client.base_url).as_str());

        if self.session.is_connected() {
            self.game_screen.print_fbg(0, 2, "CONNECTED", COLOR_INFO_SUCESS, COLOR_INFO_BG);
        }
        else {
            self.game_screen.print_fbg(0, 2, "DISCONNECTED", COLOR_INFO_FAILURE, COLOR_INFO_BG);
        }
    }

    fn fill_info_screen(&mut self) {}

    fn fill_keybinds_screen(&mut self) {
        self.keybinds_screen.print(0, 0, "[c] > (re)connect");
        self.keybinds_screen.print(0, 1, "[d] > disconnect");
    }

    fn fill_status_screen(&mut self) {
        Runner::print_strs_to_screen(&mut self.status_screen, &self.session.status_info);
    }

    fn fill_screens(&mut self){
        self.fill_game_screen();
        self.fill_status_screen();
        self.fill_keybinds_screen();
        self.fill_info_screen();
    }

    fn print_strs_to_screen(screen: &mut screen::Screen, strs: &Vec<&'a str>){
        for (i, s) in strs.iter().enumerate() {
            screen.print(0, i as i32, s);
        }
    }

    pub fn run(&mut self) {
        loop {
            self.ui_engine.wait_frame();

            if self.will_quit() {
                break;
            }

            self.handle_input();

            self.clear();
            self.session.clean();

            self.cut_screen();
            self.adapt_size();

            self.fill_screens();
            self.draw_border();
            self.draw_screens();

            self.display();
        }
    }
}
