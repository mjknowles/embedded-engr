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

    // Send welcome message
    for byte in b"STM32 Snake Game Ready!\r\n" {
        block!(tx.write(*byte)).unwrap();
    }
    for byte in b"Type 'w', 'a', 's', 'd' to move:\r\n" {
        block!(tx.write(*byte)).unwrap();
    }

    loop {
        // Infinite loop - embedded programs never exit

        // Check if we received any keyboard input
        // nb::Error::WouldBlock means "no data available right now"
        match rx.read() {
            Ok(received_byte) => {
                // We got a character! Let's respond to it

                // Echo back what we received (so you can see what you typed)
                for byte in b"You pressed: " {
                    block!(tx.write(*byte)).unwrap();
                }
                block!(tx.write(received_byte)).unwrap(); // The actual key
                for byte in b"\r\n" {
                    block!(tx.write(*byte)).unwrap();
                }

                // React to specific keys (future Snake controls!)
                match received_byte {
                    b'w' => {
                        for byte in b"Moving UP!\r\n" {
                            block!(tx.write(*byte)).unwrap();
                        }
                        led.set_high(); // Turn LED on for up
                    }
                    b'a' => {
                        for byte in b"Moving LEFT!\r\n" {
                            block!(tx.write(*byte)).unwrap();
                        }
                        led.set_low(); // Turn LED off for left
                    }
                    b's' => {
                        for byte in b"Moving DOWN!\r\n" {
                            block!(tx.write(*byte)).unwrap();
                        }
                        led.set_high(); // Turn LED on for down
                    }
                    b'd' => {
                        for byte in b"Moving RIGHT!\r\n" {
                            block!(tx.write(*byte)).unwrap();
                        }
                        led.set_low(); // Turn LED off for right
                    }
                    b'q' => {
                        for byte in b"Quit command received!\r\n" {
                            block!(tx.write(*byte)).unwrap();
                        }
                    }
                    _ => {
                        // Any other key
                        for byte in b"Unknown command. Use w/a/s/d to move.\r\n" {
                            block!(tx.write(*byte)).unwrap();
                        }
                    }
                }
            }
            Err(nb::Error::WouldBlock) => {
                // No data available - this is normal!
                // Don't do anything, just continue the loop
            }
            Err(_) => {
                // Some other error occurred
                for byte in b"UART Error!\r\n" {
                    block!(tx.write(*byte)).unwrap();
                }
            }
        }
    }
}
