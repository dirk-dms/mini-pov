//#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]
//#![allow(unused_imports)]

use cortex_m::asm;
//use panic_halt as _;
use panic_itm as _;
use rtfm::app;

use cortex_m::iprintln;
use stm32ral::{gpio, tim2, tim3, tim4};
use stm32ral::{read_reg, stm32f4::stm32f401, write_reg};

mod hwsetup;
mod util;

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
        // Configure our clocks
        let myrcc = cx.device.RCC;
        let myflash = cx.device.FLASH;
        hwsetup::clocksetup(&myrcc, &myflash);

        //Now that the clockspeed is nominal print a staus message via itm
        let mut myitm = cx.core.ITM;
        iprintln!(&mut myitm.stim[0], "Beginning hardware init.");

        // Stop all timers on debug halt for better debugging
        let mydbgmcu = cx.device.DBGMCU;
        hwsetup::timer234debugstop(&mydbgmcu);

        // Setup GPIOB (LEDs and timer3 CC1 input)
        let mygpiob = cx.device.GPIOB;
        hwsetup::portconfig(&myrcc, &mygpiob);

        // Setup timers
        let mytim2 = cx.device.TIM2;
        let mytim3 = cx.device.TIM3;
        let mytim4 = cx.device.TIM4;
        hwsetup::timerconfig(&myrcc, &mytim2, &mytim3, &mytim4);

        iprintln!(&mut myitm.stim[0], "Hardware init done.");

        //Return the now initialized Ressources
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
        //we are lower priority than TIM4 handler
        //so we need to lock the shared ressource
        cx.resources.myitm.lock(|myitm| {
            iprintln!(&mut myitm.stim[0], "2");
        });
    }

    #[task(binds = TIM3, priority=1, resources = [myitm, mytim3])]
    fn tim3_handler(mut cx: tim3_handler::Context) {
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
        cx.resources.myitm.lock(|myitm| {
            iprintln!(&mut myitm.stim[0], "3");
        });
    }

    #[task(binds = TIM4, priority=3, resources = [myitm, mygpiob, mytim4])]
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
        iprintln!(&mut cx.resources.myitm.stim[0], "4");
    }

    #[task(binds = DMA1_STREAM6, priority=3, resources = [myitm, mygpiob])]
    fn dma_handler(cx: dma_handler::Context) {
        asm::bkpt();

        iprintln!(&mut cx.resources.myitm.stim[0], "D");
    }
};

//Find out the type
//let () = cx.device.GPIOB;
