# Tetris on Pine64 Pinecil

Ever wanted to play tetris on a soldering iron? Me neither. Now you can do so anyways.

## Usage

### Setup the environment
Install `rustup`, `dfu-utils`.\
\
Add the target platform: 
```
rustup target add riscv32imac-unknown-none-elf
```
Install `cargo-binutils` for `cargo objcopy`:
```
cargo install cargo-binutils
```

### Build
```
cargo build --release
```

### Run
Convert ELF to binary:
```
cargo objcopy --release -- -O binary app.bin
```
Flash:
```
sudo dfu-util -d 28e9:0189 -a 0 -D app.bin -s 0x08000000:leave
```

## Some references and crates

- Pinecil GD32VF103 RISC-V Rust Demos: https://github.com/alvinhochun/gd32vf103-pinecil-demo-rs
- Longan Nano Rust examples: https://github.com/riscv-rust/longan-nano/tree/master/examples
- The Embedded Rust Book: https://rust-embedded.github.io/book
- EXTI interrupt example: https://github.com/andelf/longan-nano-playground-rs/blob/master/examples/button.rs
- `embedded_graphics`: https://docs.rs/embedded-graphics/latest/embedded_graphics
- `ssd1306`: https://docs.rs/ssd1306/latest/ssd1306
- `gd32vf103-pac`: https://docs.rs/gd32vf103-pac/0.4.0/gd32vf103_pac
- `gd32vf103xx-hal`: https://docs.rs/gd32vf103xx-hal/0.4.0/gd32vf103xx_hal
- `embedded-hal`: https://docs.rs/embedded-hal/0.2.4/embedded_hal
- `bitvec`: https://docs.rs/bitvec/latest/bitvec
- `oorandom`: https://docs.rs/oorandom/latest/oorandom
- `heapless`: https://docs.rs/heapless/latest/heapless
- `numtoa`: https://docs.rs/numtoa/latest/numtoa
