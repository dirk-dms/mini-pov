//#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]
#![allow(unused_imports)]

use cortex_m::asm;
use panic_halt as _;
//use panic_itm as _;
use rtfm::app;

use cortex_m::iprintln;
use r0::{init_data, zero_bss};
use stm32ral::{dbgmcu, flash, gpio, rcc, tim2, tim3, tim4, Interrupt};
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
        mytim2: stm32ral::tim2::Instance,
        mytim3: stm32ral::tim3::Instance,
        mytim4: stm32ral::tim4::Instance,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        //84MHz CPU/AHB/APB2, 42 MHz APB1, 48 MHz SDIO / USB clock
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

        let myitm = cx.core.ITM;
        //iprintln!(&mut myitm.stim[0], "Beginning hardware init.");

        // Stop all timers on debug halt, which makes debugging waaaaay easier.
        let mydbgmcu = cx.device.DBGMCU;
        modify_reg!(dbgmcu, mydbgmcu, APB1_FZ, DBG_TIM2_STOP: 1, DBG_TIM3_STOP: 1);

        let mygpiob = cx.device.GPIOB;
        //Find out the type
        //let () = cx.device.GPIOB;

        // Setup LEDs and timer3 CC1 input
        //enable clock for port b
        modify_reg!(rcc, myrcc, AHB1ENR, GPIOBEN: Enabled);
        //set output mode for the two leds on pin b2 and b12
        modify_reg!(gpio, mygpiob, MODER, MODER2: Output, MODER12: Output);
        //select alternate function number AF2 for pin PB4 (TIM3_CH1)
        modify_reg!(gpio, mygpiob, AFRL, AFRL4: AF2);
        //set alternate function mode for pin b4 (trigger input TIM3_CC1)
        modify_reg!(gpio, mygpiob, MODER, MODER4: Alternate);

        // Setup timers
        let mytim2 = cx.device.TIM2;
        let mytim3 = cx.device.TIM3;
        let mytim4 = cx.device.TIM4;
        //Enable Timer clocks
        modify_reg!(
            rcc,
            myrcc,
            APB1ENR,
            TIM2EN: Enabled,
            TIM3EN: Enabled,
            TIM4EN: Enabled
        );

        //TIM3 configuration
        //Prescaler for TIM3 divide by 2^16
        modify_reg!(tim3, mytim3, PSC, PSC: 0xFFFF);
        //Onepulse mode, preload, not enabled, up
        modify_reg!(
            tim3,
            mytim3,
            CR1,
            ARPE: Enabled,
            CEN: Disabled,
            CKD: NotDivided,
            CMS: EdgeAligned,
            DIR: Up,
            OPM: Enabled,
            UDIS: Enabled,
            URS: CounterOnly
        );
        //Configure TIM3 Master mode controller to send OC2REF as TRGO
        modify_reg!(tim3, mytim3, CR2, MMS: CompareOC2);
        //CC1 channel as input, IC1 mapped on TI1, Filtered by 8 CLK_INT
        modify_reg!(tim3, mytim3, CCMR1, CC1S: 1, IC1F: FCK_INT_N8);
        //TI1FP1 must detect a rising edge
        modify_reg!(tim3, mytim3, CCER, CC1P: 0, CC1NP: 0);
        //Configure TI1FP1 as trigger for the slave mode controller (TRGI)
        modify_reg!(tim3, mytim3, SMCR, TS: TI1FP1);
        //TI1FP1 is used to start the counter
        modify_reg!(tim3, mytim3, SMCR, SMS: Trigger_Mode);
        //PWM mode 2 with preload for OC2
        modify_reg!(tim3, mytim3, CCMR1, OC2M: 0b111, OC2PE: 1);
        //Period 0xFFFF
        modify_reg!(tim3, mytim3, ARR, ARR: 0xFFFF);
        //Delay afte 1/4 period go high
        modify_reg!(tim3, mytim3, CCR2, CCR: 0x3FFF);
        //Create an update event to auto reload the preload values
        write_reg!(tim3, mytim3, EGR, UG: Update);
        //Clear all TIM3 interupt Flags
        write_reg!(
            tim3,
            mytim3,
            SR,
            TIF: Clear,
            UIF: Clear,
            CC1IF: Clear,
            CC2IF: Clear,
            CC3IF: Clear,
            CC4IF: Clear
        );

        //TIM2 configuration
        //Prescaler for TIM2 divide by 2^16
        modify_reg!(tim2, mytim2, PSC, PSC: 0xFFFF);
        // preload, not enabled, up
        modify_reg!(
            tim2,
            mytim2,
            CR1,
            ARPE: Enabled,
            CEN: Disabled,
            CKD: NotDivided,
            CMS: EdgeAligned,
            DIR: Up,
            OPM: Disabled,
            UDIS: Enabled,
            URS: CounterOnly
        );
        //Configure TIM2 Master mode controller to send OC2REF as TRGO
        modify_reg!(tim2, mytim2, CR2, MMS: CompareOC2);
        //Configure ITR2 (from TIM3) as trigger for the slave mode controller (TRGI)
        modify_reg!(tim2, mytim2, SMCR, TS: ITR2);
        //ITR2 is used to gate the counter
        modify_reg!(tim2, mytim2, SMCR, SMS: Gated_Mode);
        //PWM mode 2 with preload for OC2
        modify_reg!(tim2, mytim2, CCMR1, OC2M: 0b111, OC2PE: 1);
        //Period 0x3FFF
        modify_reg!(tim2, mytim2, ARR, ARR: 0x3FFF);
        //Delay after 1/4 period go high
        modify_reg!(tim2, mytim2, CCR2, CCR: 0x0FFF);
        //Create an update event to auto reload the preload values
        write_reg!(tim2, mytim2, EGR, UG: Update);
        //Clear all TIM2 interupt Flags
        write_reg!(
            tim2,
            mytim2,
            SR,
            TIF: Clear,
            UIF: Clear,
            CC1IF: Clear,
            CC2IF: Clear,
            CC3IF: Clear,
            CC4IF: Clear
        );

        //TIM4 configuration
        //Prescaler for TIM4 divide by 2^16
        modify_reg!(tim4, mytim4, PSC, PSC: 0xFFFF);
        // preload, not enabled, up
        modify_reg!(
            tim4,
            mytim4,
            CR1,
            ARPE: Enabled,
            CEN: Disabled,
            CKD: NotDivided,
            CMS: EdgeAligned,
            DIR: Up,
            OPM: Disabled,
            UDIS: Enabled,
            URS: CounterOnly
        );
        //Configure ITR1 (from TIM2) as trigger for the slave mode controller (TRGI)
        modify_reg!(tim4, mytim4, SMCR, TS: ITR1);
        //ITR1 is used to gate the counter
        modify_reg!(tim4, mytim4, SMCR, SMS: Gated_Mode);
        //PWM mode 2 with preload for OC2
        modify_reg!(tim4, mytim4, CCMR1, OC2M: 0b111, OC2PE: 1);
        //Period 0x07FF
        modify_reg!(tim4, mytim4, ARR, ARR: 0x07FF);
        //Delay after 1/2 period go high
        modify_reg!(tim4, mytim4, CCR2, CCR: 0x03FF);
        //Create an update event to auto reload the preload values
        write_reg!(tim4, mytim4, EGR, UG: Update);
        //Clear all TIM4 interupt Flags
        write_reg!(
            tim4,
            mytim4,
            SR,
            TIF: Clear,
            UIF: Clear,
            CC1IF: Clear,
            CC2IF: Clear,
            CC3IF: Clear,
            CC4IF: Clear
        );

        //Enable TIM3 interuupts for CC2 and update events, disable all other interrupts
        modify_reg!(
            tim3,
            mytim3,
            DIER,
            CC1IE: Disabled,
            CC2IE: Enabled,
            CC3IE: Disabled,
            CC4IE: Disabled,
            TIE: Disabled,
            UIE: Enabled
        );
        //Enable TIM2 interuupts for CC2 and update events, disable all other interrupts
        modify_reg!(
            tim2,
            mytim2,
            DIER,
            CC1IE: Disabled,
            CC2IE: Enabled,
            CC3IE: Disabled,
            CC4IE: Disabled,
            TIE: Disabled,
            UIE: Enabled
        );

        //Enable TIM4 DMA requests from Update event
        modify_reg!(tim4, mytim4, DIER, UDE: Enabled);

        //Enable timer4, still inactive because gated by TIM2
        modify_reg!(tim4, mytim4, CR1, CEN: Enabled);
        //Enable timer2, still inactive because gated by TIM3
        modify_reg!(tim2, mytim2, CR1, CEN: Enabled);
        //We dont enable timer3, triggered by external pin EN set by Hardware

        //iprintln!(&mut myitm.stim[0], "Hardware init done.");

        init::LateResources {
            myitm,
            mygpiob,
            mytim2,
            mytim3,
            mytim4,
        }
    }

    #[task(binds = TIM2, priority=2, resources = [myitm, mygpiob, mytim2])]
    fn tim2_handler(mut cx: tim2_handler::Context) {
        if read_reg!(tim2, cx.resources.mytim2, SR, CC2IF == Match) {
            //we are lower priority than DMA handler
            // so we need to lock the shared ressource
            cx.resources.mygpiob.lock(|mygpiob| {
                write_reg!(gpio, mygpiob, BSRR, BS12: Set); //green on
            });
        };
        if read_reg!(tim2, cx.resources.mytim2, SR, UIF == UpdatePending) {
            //we are lower priority than DMA handler
            // so we need to lock the shared ressource
            cx.resources.mygpiob.lock(|mygpiob| {
                write_reg!(gpio, mygpiob, BSRR, BR12: Reset); //green off
            });
        };
        write_reg!(
            tim2,
            cx.resources.mytim2,
            SR,
            TIF: Clear,
            UIF: Clear,
            CC1IF: Clear,
            CC2IF: Clear,
            CC3IF: Clear,
            CC4IF: Clear
        );
        //we are lower priority than DMA handler
        // so we need to lock the shared ressource
        /*cx.resources.myitm.lock(|myitm| {
            iprintln!(&mut myitm.stim[0], "2");
        });*/
    }

    #[task(binds = TIM3, priority=1, resources = [myitm, mygpiob, mytim3])]
    fn tim3_handler(cx: tim3_handler::Context) {
        //Clear all TIM3 interupt Flags
        write_reg!(
            tim3,
            cx.resources.mytim3,
            SR,
            TIF: Clear,
            UIF: Clear,
            CC1IF: Clear,
            CC2IF: Clear,
            CC3IF: Clear,
            CC4IF: Clear
        );
        //we are lower priority than TIM2 handler
        // so we need to lock the shared ressource
        /*cx.resources.myitm.lock(|myitm| {
            iprintln!(&mut myitm.stim[0], "3");
        });*/
    }

    /*#[task(binds = TIM4, priority=3, resources = [myitm, mygpiob, mytim4])]
    fn tim4_handler(cx: tim4_handler::Context) {
        if read_reg!(tim4, cx.resources.mytim4, SR, CC2IF == Match) {
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BS2: Set); //red on
        };
        if read_reg!(tim4, cx.resources.mytim4, SR, UIF == UpdatePending) {
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BR2: Reset); //red off
        };

        //asm::bkpt();
        //Clear all TIM4 interupt Flags
        write_reg!(
            tim4,
            cx.resources.mytim4,
            SR,
            TIF: Clear,
            UIF: Clear,
            CC1IF: Clear,
            CC2IF: Clear,
            CC3IF: Clear,
            CC4IF: Clear
        );
        //iprintln!(&mut cx.resources.myitm.stim[0], "4");
    }*/

    #[task(binds = DMA1_Stream6, priority=5, resources = [myitm, mygpiob])]
    fn dma_handler(cx: dma_handler::Context) {
        asm::bkpt();

        //iprintln!(&mut cx.resources.myitm.stim[0], "4");
    }
};
