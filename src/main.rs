//#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]
#![allow(unused_imports)]

//use cortex_m::asm;
//use panic_halt as _;
use panic_itm as _;
use rtfm::app;

use cortex_m::iprintln;
use r0::{init_data, zero_bss};
use stm32ral::{dbgmcu, flash, gpio, rcc, tim2, tim3, Interrupt};
use stm32ral::{modify_reg, read_reg, reset_reg, stm32f4::stm32f401, write_reg};

pub mod util;

#[app(device = stm32ral::stm32f4::stm32f401, peripherals = true)]
const APP: () = {
    struct Resources {
        // A resource
        #[init(0)]
        shared: u32,

        //Late Ressource
        mygpiob: stm32ral::gpio::Instance,
        myitm: cortex_m::peripheral::ITM,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        //8/2*168/4 = 84MHz CPU
        //8/2*168/4/1 = 84MHz AHB
        //8/2*168/4/1/1 = 84MHz APB2
        //8/2*168/4/1/2 = 42 MHz APB1
        //8/2*168/7 = 48 MHz SDIO / USB clock
        let myclock_config: util::ClockConfig = util::ClockConfig {
            crystal_hz: 8000000.0,
            crystal_divisor: 4,
            pll_multiplier: 168,
            general_divisor: stm32f401::rcc::PLLCFGR::PLLP::RW::Div4,
            pll48_divisor: 7,
            ahb_divisor: stm32f401::rcc::CFGR::HPRE::RW::Div1,
            apb1_divisor: stm32f401::rcc::CFGR::PPRE1::RW::Div2,
            apb2_divisor: stm32f401::rcc::CFGR::PPRE2::RW::Div1,
            flash_latency: 2, //2 wait states for 84MHz at 3.3V.
        };

        let myrcc = cx.device.RCC;
        let myflash = cx.device.FLASH;
        util::configure_clocks(&myrcc, &myflash, &myclock_config);

        let mut myitm = cx.core.ITM;
        iprintln!(&mut myitm.stim[0], "Beginning hardware init.");

        // Stop all timers on debug halt, which makes debugging waaaaay easier.
        let mydbgmcu = cx.device.DBGMCU;
        modify_reg!(dbgmcu, mydbgmcu, APB1_FZ, DBG_TIM2_STOP: 1, DBG_TIM3_STOP: 1);

        let mygpiob = cx.device.GPIOB;
        //Find out the type
        //let () = cx.device.GPIOB;

        // Setup LEDs
        modify_reg!(rcc, myrcc, AHB1ENR, GPIOBEN: Enabled);
        modify_reg!(gpio, mygpiob, MODER, MODER2: Output, MODER12: Output);

        // Setup timers
        let _mytim2 = cx.device.TIM2;
        let mytim3 = cx.device.TIM3;
        //Enable Timer clocks
        modify_reg!(rcc, myrcc, APB1ENR, TIM2EN: Enabled, TIM3EN: Enabled);
        //Prescaler divide by 2^16
        modify_reg!(tim3, mytim3, PSC, PSC: 0xFFFF);
        //Onepulse mode, no preload, not enabled, up
        modify_reg!(
            tim3,
            mytim3,
            CR1,
            ARPE: Disabled,
            CEN: Disabled,
            CKD: NotDivided,
            CMS: EdgeAligned,
            DIR: Up,
            OPM: Enabled,
            UDIS: Enabled,
            URS: CounterOnly
        );
        //CC1 channel as input, IC1 mapped on TI1, Filtered by 8 CLK_INT
        modify_reg!(tim3, mytim3, CCMR1, CC1S: 1, IC1F: FCK_INT_N8);
        //TI1FP1 must detect a rising edge
        modify_reg!(tim3, mytim3, CCER, CC1P: 0, CC1NP: 0);
        //Configure TI1FP1 as trigger for the slave mode controller (TRGI)
        modify_reg!(tim3, mytim3, SMCR, TS: TI1FP1);
        //TI1FP1 is used to start the counter
        modify_reg!(tim3, mytim3, SMCR, SMS: Trigger_Mode);
        //PWM mode 2
        modify_reg!(tim3, mytim3, CCMR1, OC1M: 0b111);
        //Period
        modify_reg!(tim3, mytim3, ARR, ARR: 8);
        //Delay
        modify_reg!(tim3, mytim3, CCR1, CCR: 1);

        //enable Timer3 Global Interrupt
        rtfm::pend(Interrupt::TIM3);

        iprintln!(&mut myitm.stim[0], "Hardware init done.");

        init::LateResources { myitm, mygpiob }
    }

    /*#[idle(resources = [myitm, mygpiob])]
    fn idle(cx: idle::Context) -> ! {
        iprintln!(&mut cx.resources.myitm.stim[0], "Entering idle loop.");
        loop {
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BS2: Set, BR12: Reset);
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BR2: Reset, BS12: Set);
        }
    }*/

    #[task(binds = TIM3, priority=1, resources = [myitm, mygpiob])]
    fn tim3_handler(cx: tim3_handler::Context) {
        iprintln!(&mut cx.resources.myitm.stim[0], "3");
    }
};
