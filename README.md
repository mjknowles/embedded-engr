# Helpful Commands

```zsh
# flash the board
cargo run

# connect to board
screen /dev/tty.usbmodem* 115200

# show size of program
cargo size --bin snake-game --release -- -A > memory-size.txt
```

# Manual Setup Stuff

```zsh
# Add ARM target for STM32F4
rustup target add thumbv7em-none-eabi

# Add source code for cross-compilation
rustup component add rust-src

# Install flashing and debugging tools
cargo install probe-rs --features cli

# for getting size of programs
cargo install cargo-binutils
```
