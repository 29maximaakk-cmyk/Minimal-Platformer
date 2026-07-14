// platformer.rs - Минимальный платформер на Rust
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use std::fs;

const WIDTH: usize = 40;
const HEIGHT: usize = 12;
const GRAVITY: f64 = 0.3;
const JUMP_SPEED: f64 = -5.5;
const PLAYER: char = '@';
const WALL: char = '#';
const COIN: char = '$';
const EXIT: char = 'X';
const RECORD_FILE: &str = "platformer_record.json";

const LEVEL_RAW: [&str; 6] = [
    "                                        ",
    "                                        ",
    "          #   #                         ",
    "   $      #   #     #   #              ",
    "   @      #   #  $  #   #   X          ",
    "########   ########   #############    ",
];

#[derive(Serialize, Deserialize)]
struct RecordData {
    record: f64,
}

struct Platformer {
    level: Vec<Vec<char>>,
    player_x: usize,
    player_y: usize,
    vx: f64,
    vy: f64,
    on_ground: bool,
    coins: u32,
    total_coins: u32,
    exit_x: usize,
    exit_y: usize,
    game_over: bool,
    won: bool,
    paused: bool,
    running: bool,
    start_time: Instant,
    record: Option<f64>,
    last_update: Instant,
}

impl Platformer {
    fn new() -> Self {
        let mut p = Self {
            level: Vec::new(),
            player_x: 0,
            player_y: 0,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            coins: 0,
            total_coins: 0,
            exit_x: 0,
            exit_y: 0,
            game_over: false,
            won: false,
            paused: false,
            running: true,
            start_time: Instant::now(),
            record: Self::load_record(),
            last_update: Instant::now(),
        };
        p.reset();
        p
    }

    fn load_record() -> Option<f64> {
        if let Ok(data) = fs::read_to_string(RECORD_FILE) {
            if let Ok(rec) = serde_json::from_str::<RecordData>(&data) {
                return Some(rec.record);
            }
        }
        None
    }

    fn save_record(time: f64) {
        let data = RecordData { record: time };
        let _ = fs::write(RECORD_FILE, serde_json::to_string(&data).unwrap());
    }

    fn find_player_and_exit(&mut self) {
        for y in 0..self.level.len() {
            for x in 0..self.level[y].len() {
                let ch = self.level[y][x];
                if ch == PLAYER {
                    self.player_x = x;
                    self.player_y = y;
                    self.level[y][x] = ' ';
                } else if ch == EXIT {
                    self.exit_x = x;
                    self.exit_y = y;
                } else if ch == COIN {
                    self.total_coins += 1;
                }
            }
        }
    }

    fn reset(&mut self) {
        // Инициализация уровня
        let rows = std::cmp::max(LEVEL_RAW.len(), HEIGHT);
        self.level = Vec::with_capacity(rows);
        for i in 0..rows {
            let row = if i < LEVEL_RAW.len() {
                LEVEL_RAW[i].to_string()
            } else {
                " ".repeat(WIDTH)
            };
            let mut chars: Vec<char> = row.chars().collect();
            while chars.len() < WIDTH {
                chars.push(' ');
            }
            self.level.push(chars);
        }
        self.player_x = 0;
        self.player_y = 0;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.coins = 0;
        self.total_coins = 0;
        self.game_over = false;
        self.won = false;
        self.paused = false;
        self.start_time = Instant::now();
        self.find_player_and_exit();
    }

    fn collides(&self, x: usize, y: usize) -> bool {
        if x >= WIDTH || y >= self.level.len() {
            return true;
        }
        self.level[y][x] == WALL
    }

    fn get_left_collision(&self, mut x: usize, y: usize) -> usize {
        while self.collides(x, y) && x > 0 { x -= 1; }
        x + 1
    }
    fn get_right_collision(&self, mut x: usize, y: usize) -> usize {
        while self.collides(x, y) && x < WIDTH - 1 { x += 1; }
        x - 1
    }
    fn get_top_collision(&self, x: usize, mut y: usize) -> usize {
        while self.collides(x, y) && y > 0 { y -= 1; }
        y + 1
    }
    fn get_bottom_collision(&self, x: usize, mut y: usize) -> usize {
        while self.collides(x, y) && y < self.level.len() - 1 { y += 1; }
        y - 1
    }

    fn update_physics(&mut self) {
        self.vy += GRAVITY;
        if self.vy > 10.0 { self.vy = 10.0; }

        // Горизонталь
        let new_x = self.player_x as f64 + self.vx;
        let new_x_int = new_x.round() as usize;
        if !self.collides(new_x_int, self.player_y) {
            self.player_x = new_x_int;
        } else {
            if self.vx > 0.0 {
                self.player_x = self.get_left_collision(self.player_x, self.player_y);
            } else if self.vx < 0.0 {
                self.player_x = self.get_right_collision(self.player_x, self.player_y);
            }
            self.vx = 0.0;
        }

        // Вертикаль
        let new_y = self.player_y as f64 + self.vy;
        let new_y_int = new_y.round() as usize;
        if !self.collides(self.player_x, new_y_int) {
            self.player_y = new_y_int;
        } else {
            if self.vy > 0.0 {
                self.player_y = self.get_top_collision(self.player_x, self.player_y);
                self.vy = 0.0;
                self.on_ground = true;
            } else if self.vy < 0.0 {
                self.player_y = self.get_bottom_collision(self.player_x, self.player_y);
                self.vy = 0.0;
            }
        }

        // Монеты
        if self.level[self.player_y][self.player_x] == COIN {
            self.level[self.player_y][self.player_x] = ' ';
            self.coins += 1;
        }

        // Выход
        if self.player_x == self.exit_x && self.player_y == self.exit_y {
            self.won = true;
            self.game_over = true;
            let elapsed = self.start_time.elapsed().as_secs_f64();
            if self.record.is_none() || elapsed < self.record.unwrap() {
                self.record = Some(elapsed);
                Self::save_record(elapsed);
            }
        }
    }

    fn update(&mut self) {
        if self.game_over || self.paused { return; }
        self.update_physics();
    }

    fn draw(&self) {
        execute!(stdout(), Clear(ClearType::All)).unwrap();
        let mut out = stdout();
        let elapsed = self.start_time.elapsed().as_secs_f64();
        writeln!(out, "{}", "═".repeat(WIDTH)).unwrap();
        writeln!(out, "  Монеты: {}/{}   Время: {:.1} сек", self.coins, self.total_coins, elapsed).unwrap();
        if let Some(rec) = self.record {
            writeln!(out, "  Рекорд: {:.1} сек", rec).unwrap();
        }
        writeln!(out, "{}", "═".repeat(WIDTH)).unwrap();

        let mut grid = self.level.clone();
        grid[self.player_y][self.player_x] = PLAYER;
        grid[self.exit_y][self.exit_x] = EXIT;

        for row in grid {
            write!(out, "│").unwrap();
            for ch in row {
                match ch {
                    COIN => write!(out, "\x1b[35m$\x1b[0m").unwrap(),
                    WALL => write!(out, "\x1b[34m#\x1b[0m").unwrap(),
                    PLAYER => write!(out, "\x1b[32m@\x1b[0m").unwrap(),
                    EXIT => write!(out, "\x1b[33mX\x1b[0m").unwrap(),
                    _ => write!(out, "{}", ch).unwrap(),
                }
            }
            writeln!(out, "│").unwrap();
        }
        writeln!(out, "{}", "═".repeat(WIDTH)).unwrap();
        let status = if self.paused { "ПАУЗА" } else if self.won { "ПОБЕДА!" } else if self.game_over { "ИГРА ОКОНЧЕНА" } else { "ИГРА" };
        writeln!(out, "  {}  |  ← → - движение  |  Пробел/↑ - прыжок  |  P - пауза  |  R - рестарт  |  Q - выход", status).unwrap();
        out.flush().unwrap();
    }

    fn run(&mut self) {
        terminal::enable_raw_mode().unwrap();
        self.last_update = Instant::now();

        while self.running {
            // Обработка ввода
            if event::poll(Duration::from_millis(50)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Left | KeyCode::Char('a') => {
                            if !self.game_over && !self.paused { self.vx = -1.5; }
                        }
                        KeyCode::Right | KeyCode::Char('d') => {
                            if !self.game_over && !self.paused { self.vx = 1.5; }
                        }
                        KeyCode::Up | KeyCode::Char(' ') => {
                            if self.on_ground && !self.game_over && !self.paused {
                                self.vy = JUMP_SPEED;
                                self.on_ground = false;
                            }
                        }
                        KeyCode::Char('p') => self.paused = !self.paused,
                        KeyCode::Char('r') => self.reset(),
                        KeyCode::Char('q') => {
                            self.running = false;
                            break;
                        }
                        _ => {
                            if !self.game_over && !self.paused {
                                self.vx = 0.0; // отпускание
                            }
                        }
                    }
                }
            }

            let now = Instant::now();
            if now - self.last_update >= Duration::from_secs_f64(1.0 / 60.0) {
                self.update();
                self.draw();
                self.last_update = now;
            }
        }
        terminal::disable_raw_mode().unwrap();
    }
}

fn main() {
    let mut game = Platformer::new();
    game.run();
    println!("Игра завершена.");
}
