# `mini-pov`

> A bare metal embedded rust persistance of vision project on an esperuino pico board with an Stm32F401CDU mcu

This is a work in progress and by no means finished yet...

## The basic idea 

> We strap the pcb and LEDs and powersupply to the back of the fan and fix the blades on the ground. 
> The fan will turn with ~50Hz and generate 2 tacho pulses per revolution to synchronize the video. 
> The leds are mounted vertically so we get a cylindrical display. 
> Since the PCB rotates we'll auto shut down the fan after a minute or so.
> The tacho pulses connect to a CC input of Timer3 to (re-)enable the one shot timer (trigger it).
> Timer 3 generates a CC output during the whole width of the image, which gates timer2.
> Timer 2 runs continiously in gated mode generating a CC output 
> which is active high during each collumn update to gate Timer4.
> Timer 4 runs continiously in gated mode generating a DMA request 
> for each Byte to send in a collumn upon update.
> The DMA operates in Double Buffered Mode for continious operation.
> Each column data contains the whole SPI command sequence to update a whole collumn.
> Command bytes plus 12*2 Byes PWM values get sent to the SPI Port.
> 
> Each single DMA transfer writes the whole image to the spi port. 
> The DMA transfer complete interrupt can then advance the buffer pointer 
> from the just finished buffer to next framebuffer. 
> In the background task we will actually have tripple buffering, 
> one Buffer being displayed, a next Buffer that is already prepared,
> and a third one under construction by the background task.
> 
> 12 * 60 Pixels resolution will be ok for some scrolling text,
> so we'll need some Character Fonts stored somewhere too.
> Internally we'll work with 8 Bit brightness values and 
> gamma correction will map them to 16 bit pwm values.

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

## Debug with VSCode, CodeLLDB, OpenOCD, itmdump and STLINK V2-1

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

> I picked an openocd board config file which fits our setup too.
> It has an onboard STLink V2-1 and also an Stm32F4xx family chip.

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
