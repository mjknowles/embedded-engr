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
            snake_length: 3, // Start with 3 segments
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
        game.setup_board();

        game
    }

    fn setup_board(&mut self) {
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
}

// Helper function to send a string over UART
fn send_string(tx: &mut stm32f4xx_hal::serial::Tx<stm32f4xx_hal::pac::USART2>, text: &[u8]) {
    for byte in text {
        block!(tx.write(*byte)).unwrap();
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
    // For now, just show a placeholder score
    send_string(tx, b"000\r\n");
    send_string(tx, b"Controls: w/a/s/d to move, q to quit\r\n");

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
    send_string(&mut tx, b"Welcome to STM32 Snake!\r\n");
    send_string(&mut tx, b"Use w/a/s/d to control the snake.\r\n");
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
        match rx.read() {
            Ok(received_byte) => {
                match received_byte {
                    b'w' => game.snake_direction = Direction::Up,
                    b'a' => game.snake_direction = Direction::Left,
                    b's' => game.snake_direction = Direction::Down,
                    b'd' => game.snake_direction = Direction::Right,
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
                cortex_m::asm::delay(1_000_000);
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
        cortex_m::asm::delay(8_000_000); // ~1 second per frame for now
    }
}
