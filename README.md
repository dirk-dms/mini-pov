# `mini-pov`

> A bare metal embedded rust persistance of vision project on an esperuino pico board with an Stm32F401CDU mcu

## Required Software

- Rust 2018 edition 

## Required Hardware

To build this project you'll need:

- Esperuino Pico

- Adafruit 12 Channel 16 Bit PWM led driver

- A 5V DC Fan with tacho signal (For example Sanyo Denki DC Fan 52x15 PN: 109P0505M701)

- 12 Leds <= the important blinky bit of the project

- Adafruit PowerBoost 500 Battery charging and LDO circuit

- A Big rechargable LIPO Battery

## Debug with Visual Studio Code, CodeLLDB, OpenOCD, itmdump and STLINK V2-1

- This repo contains a launch.json and tasks.json vor Visual Studio Code.

- You'll need the Rust-Analyzer and CodeLLDB Plugins vor VSCode

- Have a recent (!) OpenOCD installation (0.10.0)

- CodeLLDB now ships its own LLDB wich is recent enough for our purposes

- Itmdump - install with 

``` console
$ cargo install itm
```

## Debugging

1. Creat a Fifo for itm logging

``` console
$ mkfifo .vscode/itm.log
```

2. Start itmdump to listen to itm messages in a separate terminal

``` console
$ itmdump -f .vscode/itm.log -F
```

2. Start OpenOCD in a separate terminal

I picked an openocd board config file which fits our setup too.
It has an onboard STLink V2-1 and also an Stm32F4xx family chip.

``` console
$ openocd -c "gdb_port 3333" -s "/home/dirk/rust/projects/mini-pov/" -f ../scripts/board/stm32f429disc1.cfg
```

4. Debug your programm from vscode

``` console
$ cargo build
```

# License

This code is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
