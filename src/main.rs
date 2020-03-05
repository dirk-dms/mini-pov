#![no_main]
#![no_std]
//#![deny(warnings)]

//use panic_halt as _;  stm32f4::stm32f401,
use panic_itm as _;
use rtfm::app;

use stm32ral::{gpio, tim4};
use stm32ral::{read_reg, write_reg};

#[macro_use]
mod util;

mod clocksetup;
mod dmasetup;
mod timersetup;

pub const BUFLEN: usize = 3 * 6;

pub struct Doublebuffer {
    pub a: [u8; BUFLEN],
    pub b: [u8; BUFLEN],
}

#[app(device = stm32ral::stm32f4::stm32f401, peripherals = true)]
const APP: () = {
    struct Resources {
        //Late Ressource
        mydoublebuffer: Doublebuffer,
        mygpiob: stm32ral::gpio::Instance,
        myitm: cortex_m::peripheral::ITM,
        mytim4: stm32ral::tim4::Instance,
        mydma: stm32ral::dma::Instance,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Configure our clocks
        let myrcc = cx.device.RCC;
        let myflash = cx.device.FLASH;
        let mydbgmcu = cx.device.DBGMCU;
        let mygpiob = cx.device.GPIOB;
        let mytim2 = cx.device.TIM2;
        let mytim3 = cx.device.TIM3;
        let mytim4 = cx.device.TIM4;
        let myitm = cx.core.ITM;
        let mydma = cx.device.DMA1;
        let myuart = cx.device.USART1;
        let mydoublebuffer = Doublebuffer {
            a: [0xC3; BUFLEN],
            b: [0xA5; BUFLEN],
        };

        // Configure our clocks
        clocksetup::clocksetup(&myrcc, &myflash);
        // Stop all timers on debug halt for better debugging
        timersetup::timer234debugstop(&mydbgmcu);
        // Setup GPIOB (LEDs and timer3 CC1 input)
        timersetup::portconfig(&myrcc, &mygpiob);
        // Setup timers
        timersetup::timerconfig(&myrcc, &mytim2, &mytim3, &mytim4);
        // Setup dma
        dmasetup::dmaconfig(&myrcc, &mydma, &myuart, &mydoublebuffer);

        //Return the now initialized Late Ressources
        init::LateResources {
            myitm,
            mygpiob,
            mytim4,
            mydma,
            mydoublebuffer,
        }
    }

    #[task(binds = TIM4, priority=2, resources = [myitm, mygpiob, mytim4])]
    fn tim4_handler(mut cx: tim4_handler::Context) {
        if read_reg!(tim4, cx.resources.mytim4, SR, CC2IF == Match) {
            cx.resources.mygpiob.lock(|gpiob| {
                write_reg!(gpio, gpiob, BSRR, BS2: Set); //red on
            });
        };
        if read_reg!(tim4, cx.resources.mytim4, SR, UIF == UpdatePending) {
            cx.resources.mygpiob.lock(|gpiob| {
                write_reg!(gpio, gpiob, BSRR, BR2: Reset); //red off
            });
        };

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
        //we are lower priority than DMA handler
        //so we need to lock the shared ressource
        /*cx.resources.myitm.lock(|myitm| {
            cortex_m::iprintln!(&mut myitm.stim[0], "4");
        });*/
    }

    #[task(binds = DMA1_STREAM6, priority=3, resources = [myitm, mygpiob, mydma, mydoublebuffer])]
    fn dma_handler(cx: dma_handler::Context) {
        static mut A_DONE_USING_B: bool = true;

        if *A_DONE_USING_B {
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BS12: Set); //green on
        } else {
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BR12: Reset); //green off
        }
        *A_DONE_USING_B = !*A_DONE_USING_B;
        //cortex_m::iprintln!(&mut cx.resources.myitm.stim[0], "D");
    }
};

//Find out the type
//let () = cx.device.GPIOB;
