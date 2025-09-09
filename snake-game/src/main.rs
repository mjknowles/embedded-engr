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

    // Get core peripherals - CPU-level stuff
    // let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Configure system clocks - your chip needs to know how fast to run
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Get GPIO ports
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
    let (mut tx, mut _rx) = uart.split();

    // Your LED for visual feedback
    let mut led = gpioa.pa5.into_push_pull_output();

    loop {
        // Infinite loop - embedded programs never exit

        // STEP 8: Send text to your computer terminal
        // b"Hello..." creates a byte string (array of u8)
        // \r\n = carriage return + newline (proper line ending)
        for byte in b"Hello from STM32!\r\n" {
            // block!() waits until UART hardware is ready
            // tx.write() sends one byte at a time
            block!(tx.write(*byte)).unwrap();
        }

        // STEP 9: Visual feedback with LED
        led.set_high(); // Turn LED on
        cortex_m::asm::delay(8_000_000); // Wait ~1 second
        led.set_low(); // Turn LED off
        cortex_m::asm::delay(8_000_000); // Wait ~1 second
    }
}
