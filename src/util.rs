//pub mod armv7m;
//pub mod copy_words;
//pub mod startup;
//pub mod stm32;
use stm32ral::{modify_reg, read_reg, reset_reg, stm32f4::stm32f401, write_reg};

pub struct ClockConfig {
    pub crystal_hz: f32,
    pub crystal_divisor: u32,
    pub pll_multiplier: u32,
    pub general_divisor: u32,
    pub pll48_divisor: u32,

    pub ahb_divisor: u32,
    pub apb1_divisor: u32,
    pub apb2_divisor: u32,

    pub flash_latency: u32,
}

macro_rules! block_while {
    ($condition:expr) => {
        while $condition {}
    };
}

macro_rules! block_until {
    ($condition:expr) => {
        block_while!(!$condition)
    };
}

pub fn configure_clocks(
    rcc: &stm32ral::rcc::Instance,
    flash: &stm32ral::flash::Instance,
    cfg: &ClockConfig,
) {
    // Switch to the internal 16MHz oscillator while messing with the PLL.
    modify_reg!(stm32ral::rcc, rcc, CR, HSION: On);
    block_until! { read_reg!(stm32ral::rcc, rcc, CR, HSIRDY == Ready) }

    // Make the switch.
    modify_reg!(stm32ral::rcc, rcc, CFGR, SW: HSI);
    block_until! { read_reg!(stm32ral::rcc, rcc, CFGR, SWS == HSI) }

    // Turn off the PLL.
    modify_reg!(stm32ral::rcc, rcc, CR, PLLON: Off);
    block_while! { read_reg!(stm32ral::rcc, rcc, CR, PLLRDY == Ready) }

    // Apply divisors before boosting frequency.
    modify_reg!(stm32ral::rcc, rcc, CFGR, HPRE: cfg.ahb_divisor, PPRE1: cfg.apb1_divisor, PPRE2: cfg.apb2_divisor);

    // Configure the flash latency and enable flash cache and prefetching
    modify_reg!(stm32ral::flash, flash, ACR, LATENCY: cfg.flash_latency, DCEN: 1, ICEN: 1, PRFTEN: 1);

    // Switch on the crystal oscillator.
    modify_reg!(stm32ral::rcc, rcc, CR, HSEON: On);
    block_until! { read_reg!(stm32ral::rcc, rcc, CR, HSERDY == Ready) }

    // Configure the PLL.
    modify_reg!(stm32ral::rcc, rcc, PLLCFGR, 
        PLLM: cfg.crystal_divisor, 
        PLLN: cfg.pll_multiplier, 
        PLLQ: cfg.pll48_divisor, 
        PLLP: cfg.general_divisor, 
        PLLSRC: HSE);

    // Turn it on.
    modify_reg!(stm32ral::rcc, rcc, CR, PLLON: On);
    block_until! { read_reg!(stm32ral::rcc, rcc, CR, PLLRDY == Ready) }

    // Select PLL as clock source.
    modify_reg!(stm32ral::rcc, rcc, CFGR, SW: PLL);
    block_until! { read_reg!(stm32ral::rcc, rcc, CFGR, SWS == PLL) }
}