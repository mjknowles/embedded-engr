#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nb::block;
use panic_halt as _;
use stm32f4xx_hal::{
    pac,
    prelude::*,
    serial::{config::Config, Serial},
};

// Game constants
const BOARD_WIDTH: usize = 20;
const BOARD_HEIGHT: usize = 15;
const MAX_SNAKE_LENGTH: usize = 100;

// Game cell types
#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Wall,
    Snake,
    Food,
}

// Position on the game board
#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

// Snake movement direction
#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// Main game state
struct GameState {
    // Game board - 2D array of cells
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],

    // Snake data
    snake_body: [Position; MAX_SNAKE_LENGTH], // Snake segments
    snake_length: usize,                      // Current snake length
    snake_direction: Direction,               // Current movement direction

    // Food position
    food_position: Position,

    // Game status
    score: u32,
    game_over: bool,
}

impl GameState {
    fn new() -> Self {
        let mut game = GameState {
            board: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            snake_body: [Position { x: 0, y: 0 }; MAX_SNAKE_LENGTH],
            snake_length: 3,
            snake_direction: Direction::Right,
            food_position: Position { x: 15, y: 7 },
            score: 0,
            game_over: false,
        };

        // Initialize snake in the middle of the board
        game.snake_body[0] = Position { x: 10, y: 7 }; // Head
        game.snake_body[1] = Position { x: 9, y: 7 }; // Body
        game.snake_body[2] = Position { x: 8, y: 7 }; // Tail

        // Set up the board borders
        game.update_board();

        game
    }

    fn update_board(&mut self) {
        // Clear the board
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                self.board[row][col] = Cell::Empty;
            }
        }

        // Add walls around the border
        for col in 0..BOARD_WIDTH {
            self.board[0][col] = Cell::Wall; // Top wall
            self.board[BOARD_HEIGHT - 1][col] = Cell::Wall; // Bottom wall
        }

        for row in 0..BOARD_HEIGHT {
            self.board[row][0] = Cell::Wall; // Left wall
            self.board[row][BOARD_WIDTH - 1] = Cell::Wall; // Right wall
        }

        // Place snake on board
        for i in 0..self.snake_length {
            let pos = self.snake_body[i];
            self.board[pos.y][pos.x] = Cell::Snake;
        }

        // Place food on board
        self.board[self.food_position.y][self.food_position.x] = Cell::Food;
    }

    // Move the snake forward one step
    fn move_snake(&mut self) {
        if self.game_over {
            return; // Don't move if game is over
        }

        // Calculate new head position based on current direction
        let current_head = self.snake_body[0];
        let new_head = match self.snake_direction {
            Direction::Up => Position {
                x: current_head.x,
                y: current_head.y.saturating_sub(1), // Prevent underflow
            },
            Direction::Down => Position {
                x: current_head.x,
                y: current_head.y + 1,
            },
            Direction::Left => Position {
                x: current_head.x.saturating_sub(1), // Prevent underflow
                y: current_head.y,
            },
            Direction::Right => Position {
                x: current_head.x + 1,
                y: current_head.y,
            },
        };

        // Check for collisions BEFORE moving
        if self.check_collision(new_head) {
            self.game_over = true;
            return;
        }

        // Check if we're eating food
        let eating_food = new_head == self.food_position;

        if eating_food {
            // Grow the snake by NOT removing the tail
            self.score += 10;
            self.snake_length += 1;

            // Place new food (simple approach - just move it)
            self.place_new_food();
        } else {
            // Move the snake by shifting all segments
            // Move tail segments forward (from back to front)
            for i in (1..self.snake_length).rev() {
                self.snake_body[i] = self.snake_body[i - 1];
            }
        }

        // Place new head
        self.snake_body[0] = new_head;

        // Update the board representation
        self.update_board();
    }

    // Check if a position would cause a collision
    fn check_collision(&self, pos: Position) -> bool {
        // Check bounds (walls)
        if pos.x == 0 || pos.x >= BOARD_WIDTH - 1 || pos.y == 0 || pos.y >= BOARD_HEIGHT - 1 {
            return true;
        }

        // Check self-collision (hitting snake body)
        for i in 0..self.snake_length {
            if pos == self.snake_body[i] {
                return true;
            }
        }

        false
    }

    // Place food in a new location
    fn place_new_food(&mut self) {
        // Simple approach: just move food to a fixed location for now
        // Later we can make this random
        self.food_position = Position {
            x: (self.food_position.x + 3) % (BOARD_WIDTH - 2) + 1,
            y: (self.food_position.y + 2) % (BOARD_HEIGHT - 2) + 1,
        };

        // Make sure food doesn't spawn on snake (basic check)
        for i in 0..self.snake_length {
            if self.food_position == self.snake_body[i] {
                // Move food one more position if it conflicts
                self.food_position.x = (self.food_position.x + 1) % (BOARD_WIDTH - 2) + 1;
                break;
            }
        }
    }

    // NEW: Change direction (with validation)
    fn change_direction(&mut self, new_direction: Direction) {
        // Prevent snake from reversing into itself
        let opposite = match self.snake_direction {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };

        if new_direction != opposite {
            self.snake_direction = new_direction;
        }
    }

    // NEW: Reset the game
    fn reset(&mut self) {
        self.snake_length = 3;
        self.snake_direction = Direction::Right;
        self.score = 0;
        self.game_over = false;

        // Reset snake position
        self.snake_body[0] = Position { x: 10, y: 7 };
        self.snake_body[1] = Position { x: 9, y: 7 };
        self.snake_body[2] = Position { x: 8, y: 7 };

        // Reset food position
        self.food_position = Position { x: 15, y: 7 };

        self.update_board();
    }
}

// Helper function to send a string over UART
fn send_string(tx: &mut stm32f4xx_hal::serial::Tx<stm32f4xx_hal::pac::USART2>, text: &[u8]) {
    for byte in text {
        block!(tx.write(*byte)).unwrap();
    }
}

// Function to send a number as text
fn send_number(tx: &mut stm32f4xx_hal::serial::Tx<stm32f4xx_hal::pac::USART2>, mut num: u32) {
    if num == 0 {
        block!(tx.write(b'0')).unwrap();
        return;
    }

    // Convert number to string (simple approach)
    let mut digits = [0u8; 10]; // Max 10 digits for u32
    let mut digit_count = 0;

    while num > 0 {
        digits[digit_count] = (num % 10) as u8 + b'0';
        num /= 10;
        digit_count += 1;
    }

    // Send digits in reverse order (most significant first)
    for i in (0..digit_count).rev() {
        block!(tx.write(digits[i])).unwrap();
    }
}

// Function to render the game board to terminal
fn render_game(tx: &mut stm32f4xx_hal::serial::Tx<stm32f4xx_hal::pac::USART2>, game: &GameState) {
    // Clear screen (ANSI escape code)
    send_string(tx, b"\x1b[2J\x1b[H");

    // Render the board
    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            let character = match game.board[row][col] {
                Cell::Empty => b' ',
                Cell::Wall => b'#',
                Cell::Snake => b'o',
                Cell::Food => b'*',
            };
            block!(tx.write(character)).unwrap();
        }
        send_string(tx, b"\r\n"); // End of row
    }

    // Show game info
    send_string(tx, b"Score: ");
    send_number(tx, game.score);
    send_string(tx, b"   Length: ");
    send_number(tx, game.snake_length as u32);
    send_string(tx, b"\r\n");

    send_string(tx, b"Controls: w/a/s/d to move, r to restart\r\n");

    if game.game_over {
        send_string(tx, b"GAME OVER! Press any key to restart.\r\n");
    }
}

#[entry]
fn main() -> ! {
    // Get device peripherals - hardware access
    let dp = pac::Peripherals::take().unwrap();

    // Configure system clocks - your chip needs to know how fast to run
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Get GPIO (general purpose IO) ports
    let gpioa = dp.GPIOA.split();

    // Configure UART pins
    // PA2 = TX (transmit to computer)
    // PA3 = RX (receive from computer)
    let tx_pin = gpioa.pa2.into_alternate();
    let rx_pin = gpioa.pa3.into_alternate();

    // Create UART interface
    // USART2 connects to the USB port on your Nucleo board
    let uart = Serial::new(
        dp.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(115200.bps()),
        &clocks,
    )
    .unwrap();

    // Split UART into transmit and receive parts
    let (mut tx, mut rx) = uart.split();

    // Your LED for visual feedback
    let mut led = gpioa.pa5.into_push_pull_output();

    let mut game = GameState::new();

    // Welcome message
    send_string(&mut tx, b"STM32 Snake Game!\r\n");
    send_string(&mut tx, b"Use w/a/s/d to control the snake.\r\n");
    send_string(&mut tx, b"Collect food (*) to grow and score points!\r\n");
    send_string(&mut tx, b"Press any key to start...\r\n");

    // Wait for first keypress to start
    loop {
        if rx.read().is_ok() {
            break;
        }
    }

    loop {
        // Render the current game state
        render_game(&mut tx, &game);

        // Handle input (non-blocking)
        for _ in 0..10 {
            match rx.read() {
                Ok(received_byte) => {
                    match received_byte {
                        b'w' => game.snake_direction = Direction::Up,
                        b'a' => game.snake_direction = Direction::Left,
                        b's' => game.snake_direction = Direction::Down,
                        b'd' => game.snake_direction = Direction::Right,
                        b'r' => {
                            game.reset();
                            send_string(&mut tx, b"Game restarted!\r\n");
                        }
                        b'q' => {
                            send_string(&mut tx, b"Thanks for playing!\r\n");
                            // In a real game, we might reset or quit
                        }
                        _ => {
                            // Unknown key - ignore
                        }
                    }

                    // Visual feedback - blink LED when key pressed
                    led.set_high();
                    cortex_m::asm::delay(500_000);
                    led.set_low();
                }
                Err(nb::Error::WouldBlock) => {
                    // No input available - that's fine
                }
                Err(_) => {
                    // Some error occurred
                }
            }

            // Game timing - delay between frames
            cortex_m::asm::delay(800_000); // ~1 second per frame for now
        }

        // Move the snake forward one step
        game.move_snake();
    }
}
