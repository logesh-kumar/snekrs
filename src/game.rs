// External crate imports for terminal manipulation and game functionality
use crossterm::{
    cursor::{Hide, Show, MoveTo},  // Terminal cursor control
    event::{self, Event, KeyCode}, // Keyboard input handling
    execute,
    terminal::{self, Clear, ClearType},
    style::Print,
};
use rand::Rng;  // Random number generation for food placement
use std::io::{stdout, Write};
use std::time::{Duration, Instant};  // Time management for game loop
use std::collections::VecDeque;  // Double-ended queue for efficient snake body management

// Game board dimensions
// Design Decision: Fixed size makes collision detection simpler
const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

// Position struct represents a point on the game board
// Design Decision: Using u16 because terminal coordinates are never negative
#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: u16,
    y: u16,
}

// Direction enum represents possible movement directions
// Design Decision: Using enum ensures type safety for direction handling
#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// Main game struct containing all game state
// Design Decision: Encapsulating all game state in one struct makes state management clearer
pub struct Game {
    snake: VecDeque<Position>,    // Using VecDeque for O(1) push/pop at both ends
    food: Position,               // Current food position
    direction: Direction,         // Current movement direction
    next_direction: Direction,    // Buffered next direction (prevents rapid 180° turns)
    score: u32,                  // Current score
    game_over: bool,             // Game state flag
    last_update: Instant,        // Time tracking for game loop
}

impl Game {
    // Creates a new game instance with initial state
    // Design Decision: Using builder pattern for clear initialization
    pub fn new() -> Self {
        let mut snake = VecDeque::new();
        let center = Position {
            x: WIDTH / 2,
            y: HEIGHT / 2,
        };
        snake.push_back(center);  // Snake starts with one segment in center

        Game {
            snake,
            food: Game::generate_food(),
            direction: Direction::Right,  // Snake starts moving right
            next_direction: Direction::Right,
            score: 0,
            game_over: false,
            last_update: Instant::now(),
        }
    }

    // Generates random coordinates for food placement
    // Design Decision: Separate function for better code organization
    fn generate_food() -> Position {
        let mut rng = rand::thread_rng();
        Position {
            // Generate position within game bounds (excluding walls)
            x: rng.gen_range(1..WIDTH-1),
            y: rng.gen_range(1..HEIGHT-1),
        }
    }

    // Places food in a valid position (not on snake)
    // Design Decision: Retry mechanism ensures valid food placement
    fn spawn_food(&mut self) {
        self.food = Game::generate_food();
        // Keep generating new positions until food doesn't overlap with snake
        while self.snake.iter().any(|pos| pos.x == self.food.x && pos.y == self.food.y) {
            self.food = Game::generate_food();
        }
    }

    // Updates game state (snake movement, collisions, food collection)
    // Design Decision: Single function for all state updates maintains consistency
    fn update(&mut self) {
        if self.game_over {
            return;
        }

        // Apply buffered direction change
        self.direction = self.next_direction;

        // Calculate new head position based on current direction
        let head = self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        // Check wall collisions
        // Design Decision: Early returns for game-ending conditions
        if new_head.x == 0 || new_head.x == WIDTH - 1 || new_head.y == 0 || new_head.y == HEIGHT - 1 {
            self.game_over = true;
            return;
        }

        // Check self-collision
        if self.snake.iter().any(|pos| pos.x == new_head.x && pos.y == new_head.y) {
            self.game_over = true;
            return;
        }

        // Move snake by adding new head
        self.snake.push_front(new_head);

        // Handle food collection
        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.score += 1;
            self.spawn_food();
        } else {
            // Remove tail if no food was eaten
            self.snake.pop_back();
        }
    }

    // Renders the game state to the terminal
    // Design Decision: Using crossterm for cross-platform terminal manipulation
    fn draw(&self) -> std::io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        
        // Draw game border
        for x in 0..WIDTH {
            execute!(stdout, MoveTo(x, 0), Print("#"))?;
        }

        // Draw game area and entities
        for y in 1..HEIGHT-1 {
            execute!(stdout, MoveTo(0, y), Print("#"))?;
            for x in 1..WIDTH-1 {
                let pos = Position { x, y };
                let char = if self.snake.front() == Some(&pos) {
                    'O'  // Snake head (distinct from body)
                } else if self.snake.contains(&pos) {
                    'o'  // Snake body
                } else if self.food.x == x && self.food.y == y {
                    '*'  // Food
                } else {
                    ' '  // Empty space
                };
                execute!(stdout, MoveTo(x, y), Print(char))?;
            }
            execute!(stdout, MoveTo(WIDTH-1, y), Print("#"))?;
        }

        // Draw bottom border
        for x in 0..WIDTH {
            execute!(stdout, MoveTo(x, HEIGHT-1), Print("#"))?;
        }

        // Draw UI elements (score and controls)
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

    // Main game loop
    // Design Decision: Using Result for error handling
    pub fn run(&mut self) -> std::io::Result<()> {
        // Set up terminal for game display
        terminal::enable_raw_mode()?;
        execute!(stdout(), Hide)?;

        self.draw()?;

        while !self.game_over {
            // Input handling with non-blocking poll
            // Design Decision: 50ms poll rate for responsive controls
            if let Ok(true) = event::poll(Duration::from_millis(50)) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    match key_event.code {
                        // Prevent 180° turns by checking opposite direction
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

            // Game state update at fixed time intervals
            // Design Decision: 100ms update rate for smooth movement
            if self.last_update.elapsed() >= Duration::from_millis(100) {
                self.update();
                self.draw()?;
                self.last_update = Instant::now();
            }
        }

        // Clean up terminal state
        terminal::disable_raw_mode()?;
        execute!(stdout(), Show)?;
        println!("\nGame Over! Final score: {}", self.score);
        Ok(())
    }
}
