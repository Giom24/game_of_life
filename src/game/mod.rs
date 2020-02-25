use pancurses::{endwin, initscr, noecho, Input, Window, ACS_CKBOARD, ACS_HLINE, ACS_VLINE};
use std::thread;
use std::time::{Duration, Instant};

mod cursor;
use cursor::Cursor;

enum GameState {
    STOPED,
    PAUSED,
    RUNNING,
}

impl PartialEq for GameState {
    fn eq(&self, other: &GameState) -> bool {
        match self {
            GameState::RUNNING => match other {
                GameState::RUNNING => true,
                _ => false,
            },
            GameState::PAUSED => match other {
                GameState::PAUSED => true,
                _ => false,
            },
            GameState::STOPED => match other {
                GameState::STOPED => true,
                _ => false,
            },
        }
    }
}

pub struct Game {
    width: i32,
    height: i32,
    x_offset: i32,
    y_offset: i32,
    fields: Vec<Vec<bool>>,
    terminal: Window,
    state: GameState,
    cursor: Cursor,
}

impl Game {
    pub fn new(width: i32, height: i32) -> Game {
        let terminal = initscr();
        terminal.keypad(true);
        terminal.nodelay(true);
        noecho();
        Game {
            width: width,
            height: height,
            x_offset: (terminal.get_max_x() / 2) - (width / 2),
            y_offset: (terminal.get_max_y() / 2) - (height / 2),
            fields: vec![vec![false; height as usize]; width as usize],
            terminal: terminal,
            state: GameState::STOPED,
            cursor: Cursor::new(0, 0),
        }
    }

    fn get_field(&self, x: i32, y: i32) -> bool {
        self.fields[x as usize][y as usize]
    }
    fn set_field(&mut self, state: bool, x: i32, y: i32) {
        self.fields[x as usize][y as usize] = state;
    }

    fn handle_input(&mut self) {
        match self.terminal.getch() {
            Some(Input::Character('q')) => self.state = GameState::STOPED,
            Some(Input::KeyResize) => self.resize(),
            Some(Input::KeyUp) => {
                if self.cursor.get_y() > 0 {
                    self.cursor.up();
                }
            }
            Some(Input::KeyDown) => {
                if self.cursor.get_y() < self.height - 1 {
                    self.cursor.down();
                }
            }
            Some(Input::KeyLeft) => {
                if self.cursor.get_x() > 0 {
                    self.cursor.left();
                }
            }
            Some(Input::KeyRight) => {
                if self.cursor.get_x() < self.width - 1 {
                    self.cursor.right();
                }
            }
            Some(Input::Character(' ')) => {
                let x = self.cursor.get_x();
                let y = self.cursor.get_y();
                let state = self.get_field(x, y);
                self.set_field(!state, x, y);
            }
            Some(Input::Character('\n')) => match self.state {
                GameState::RUNNING => self.state = GameState::PAUSED,
                GameState::PAUSED => self.state = GameState::RUNNING,
                _ => (),
            },
            _ => (),
        }
    }

    fn set_char(&self, character: char, x: i32, y: i32) {
        self.terminal.mv(y, x);
        self.terminal.addch(character);
    }

    fn draw_border_horizontal(&self) {
        let character = '-';
        for i in 0..=1 {
            let y = i * (self.height + 1);
            self.terminal.mv(y + self.y_offset, 1 + self.x_offset);
            println!("{:?}", y);
            for _ in 1..=self.width {
                self.terminal.addch(character);
            }
        }
    }

    fn draw_border_vertical(&self) {
        let character = '|';
        for i in 0..=1 {
            let x = i * (self.width + 1);
            for y in 1..=self.height {
                self.set_char(character, x + self.x_offset, y + self.y_offset)
            }
        }
    }

    fn draw_border_edges(&self) {
        let character = '*';
        self.set_char(character, self.x_offset, self.y_offset);
        self.set_char(character, self.x_offset + self.width + 1, self.y_offset);
        self.set_char(character, self.x_offset, self.y_offset + self.height + 1);
        self.set_char(
            character,
            self.x_offset + self.width + 1,
            self.y_offset + self.height + 1,
        );
    }

    fn draw_border(&self) {
        self.draw_border_horizontal();
        self.draw_border_vertical();
        self.draw_border_edges();
        self.terminal.refresh();
    }

    fn draw_board(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.terminal
                    .mv(y + self.y_offset + 1, x + self.x_offset + 1);
                if self.get_field(x, y) {
                    self.terminal.addch(ACS_CKBOARD());
                } else {
                    self.terminal.addch(' ');
                }
            }
        }
        self.terminal.mv(
            self.cursor.get_y() + self.y_offset + 1,
            self.cursor.get_x() + self.x_offset + 1,
        );
        self.terminal.refresh();
    }

    fn get_living_fileds(&self, x: i32, y: i32) -> usize {
        let mut fields = vec![false; 8];

        // Horizontal Top
        if x > 0 && y > 0 {
            fields.push(self.fields[x as usize - 1][y as usize - 1]);
        }
        if y > 0 {
            fields.push(self.fields[x as usize][y as usize - 1]);
        }
        if x < self.width - 1 && y < 0 {
            fields.push(self.fields[x as usize + 1][y as usize - 1]);
        }

        // Horizontal Buttom
        if x < 0 && y < self.height - 1 {
            fields.push(self.fields[x as usize - 1][y as usize + 1]);
        }
        if y < self.height - 1 {
            fields.push(self.fields[x as usize][y as usize + 1]);
        }
        if x < self.width - 1 && y < self.height - 1 {
            fields.push(self.fields[x as usize + 1][y as usize + 1]);
        }

        // Left and Right
        if x < 0 {
            fields.push(self.fields[x as usize - 1][y as usize]);
        }
        if x > self.width {
            fields.push(self.fields[x as usize + 1][y as usize]);
        }

        fields.iter().filter(|field| **field).count()
    }

    fn step(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let state = self.get_living_fileds(x, y) == 3;
                self.set_field(state, x, y);
            }
        }
    }
    

    fn resize(&mut self) {
        self.x_offset = (self.terminal.get_max_x() / 2) - (self.width / 2);
        self.y_offset = (self.terminal.get_max_y() / 2) - (self.height / 2);
        self.terminal.clear();
        self.terminal.draw_box(ACS_VLINE(), ACS_HLINE());
        self.draw_border();
    }

    pub fn start(&mut self) {
        self.state = GameState::RUNNING;
        self.terminal.draw_box(ACS_VLINE(), ACS_HLINE());
        self.draw_border();
        let mut past = Instant::now();
        while self.state == GameState::RUNNING || self.state == GameState::PAUSED {
            self.handle_input();
            let duration = Instant::now().saturating_duration_since(past);
            if self.state != GameState::PAUSED && duration.as_secs() >= 1 {
                self.step();
                past = Instant::now();
            }
            self.draw_board();
            thread::sleep(Duration::from_millis(100));
        }

        endwin();
    }
}
