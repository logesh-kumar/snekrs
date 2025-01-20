use crossterm::{
    cursor::{Hide, Show, MoveTo},
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, Clear, ClearType},
    style::Print,
};
use rand::Rng;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: u16,
    y: u16,
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Game {
    snake: VecDeque<Position>,
    food: Position,
    direction: Direction,
    next_direction: Direction,
    score: u32,
    game_over: bool,
    last_update: Instant,
}

impl Game {
    pub fn new() -> Self {
        let mut snake = VecDeque::new();
        let center = Position {
            x: WIDTH / 2,
            y: HEIGHT / 2,
        };
        snake.push_back(center);

        Game {
            snake,
            food: Game::generate_food(),
            direction: Direction::Right,
            next_direction: Direction::Right,
            score: 0,
            game_over: false,
            last_update: Instant::now(),
        }
    }

    fn generate_food() -> Position {
        let mut rng = rand::thread_rng();
        Position {
            x: rng.gen_range(1..WIDTH-1),
            y: rng.gen_range(1..HEIGHT-1),
        }
    }

    fn spawn_food(&mut self) {
        self.food = Game::generate_food();
        // Make sure food doesn't spawn on snake
        while self.snake.iter().any(|pos| pos.x == self.food.x && pos.y == self.food.y) {
            self.food = Game::generate_food();
        }
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        // Update direction
        self.direction = self.next_direction;

        // Calculate new head position
        let head = self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        // Check for collisions with walls
        if new_head.x == 0 || new_head.x == WIDTH - 1 || new_head.y == 0 || new_head.y == HEIGHT - 1 {
            self.game_over = true;
            return;
        }

        // Check for collisions with self
        if self.snake.iter().any(|pos| pos.x == new_head.x && pos.y == new_head.y) {
            self.game_over = true;
            return;
        }

        // Move snake
        self.snake.push_front(new_head);

        // Check if food was eaten
        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.score += 1;
            self.spawn_food();
        } else {
            self.snake.pop_back();
        }
    }

    fn draw(&self) -> std::io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        
        // Draw top border
        for x in 0..WIDTH {
            execute!(stdout, MoveTo(x, 0), Print("#"))?;
        }

        // Draw middle section
        for y in 1..HEIGHT-1 {
            execute!(stdout, MoveTo(0, y), Print("#"))?;
            for x in 1..WIDTH-1 {
                let pos = Position { x, y };
                let char = if self.snake.front() == Some(&pos) {
                    'O' // Snake head
                } else if self.snake.contains(&pos) {
                    'o' // Snake body
                } else if self.food.x == x && self.food.y == y {
                    '*' // Food
                } else {
                    ' '
                };
                execute!(stdout, MoveTo(x, y), Print(char))?;
            }
            execute!(stdout, MoveTo(WIDTH-1, y), Print("#"))?;
        }

        // Draw bottom border
        for x in 0..WIDTH {
            execute!(stdout, MoveTo(x, HEIGHT-1), Print("#"))?;
        }

        execute!(
            stdout,
            MoveTo(0, HEIGHT),
            Print(format!("Score: {}", self.score)),
            MoveTo(0, HEIGHT+1),
            Print("Use arrow keys to move, 'q' to quit")
        )?;
        
        stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), Hide)?;

        self.draw()?;

        while !self.game_over {
            // Handle input
            if let Ok(true) = event::poll(Duration::from_millis(50)) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    match key_event.code {
                        KeyCode::Left if self.direction != Direction::Right => {
                            self.next_direction = Direction::Left;
                        },
                        KeyCode::Right if self.direction != Direction::Left => {
                            self.next_direction = Direction::Right;
                        },
                        KeyCode::Up if self.direction != Direction::Down => {
                            self.next_direction = Direction::Up;
                        },
                        KeyCode::Down if self.direction != Direction::Up => {
                            self.next_direction = Direction::Down;
                        },
                        KeyCode::Char('q') => self.game_over = true,
                        _ => {}
                    }
                }
            }

            // Update game state every 100ms
            if self.last_update.elapsed() >= Duration::from_millis(100) {
                self.update();
                self.draw()?;
                self.last_update = Instant::now();
            }
        }

        terminal::disable_raw_mode()?;
        execute!(stdout(), Show)?;
        println!("\nGame Over! Final score: {}", self.score);
        Ok(())
    }
}
