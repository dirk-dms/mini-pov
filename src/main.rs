#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]

//use panic_halt as _;  stm32f4::stm32f401,
use panic_itm as _;
use rtfm::app;

use stm32ral::gpio;
use stm32ral::write_reg;

#[macro_use]
mod util;

mod clocksetup;
mod dmasetup;
mod timersetup;

pub const ROWS: usize = 3; //128
pub const BYTESPERROW: usize = 28;
pub const BUFLEN: usize = ROWS * BYTESPERROW;

#[repr(align(128))]
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
        //mytim4: stm32ral::tim4::Instance,
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
        let myspi = cx.device.SPI2;

        let mydoublebuffer = Doublebuffer {
            #[repr(align(128))]
            a: [0xC3; BUFLEN],
            #[repr(align(128))]
            b: [0xA5; BUFLEN],
        };

        // Configure our clocks
        cortex_m::asm::bkpt();
        clocksetup::clocksetup(&myrcc, &myflash);
        cortex_m::asm::bkpt();
        // Stop all timers on debug halt for better debugging
        timersetup::timer234debugstop(&mydbgmcu);
        // Setup GPIOB (LEDs and timer3 CC1 input)
        timersetup::portconfig(&myrcc, &mygpiob);
        // Setup timers
        timersetup::timerconfig(&myrcc, &mytim2, &mytim3, &mytim4);
        // Setup dma
        dmasetup::dmaconfig(&myrcc, &mydma, &myspi, &mydoublebuffer);

        //Return the now initialized Late Ressources
        init::LateResources {
            myitm,
            mygpiob,
            //mytim4,
            mydma,
            mydoublebuffer,
        }
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
        //cortex_m::iprintln!(&mut cx.resources.myitm.stim[0], "I");
        //Clear all DMA interupt Flags
        write_reg!(
            stm32ral::dma,
            cx.resources.mydma,
            HIFCR,
            CDMEIF6: Clear,
            CFEIF6: Clear,
            CHTIF6: Clear,
            CTCIF6: Clear,
            CTEIF6: Clear
        );
    }
};

//Find out the type
//let () = cx.device.GPIOB;
