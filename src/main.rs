
//#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]

//use panic_halt as _;
use panic_itm as _;
use rtfm::app;

use stm32ral::gpio;
use stm32ral::write_reg;

use heapless::{
    consts::*,
    i,
    spsc::{Consumer, Producer, Queue, SingleCore},
};

#[macro_use]
mod util;

mod clocksetup;
mod dmasetup;
mod timersetup;

pub const ROWS: usize = 1; //128
pub const U16PERROW: usize = 14;
pub const BUFLEN: usize = ROWS * U16PERROW;

#[repr(align(128))]
pub struct Doublebuffer {
    pub a: [u16; BUFLEN],
    pub b: [u16; BUFLEN],
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
        dma_int_consumer: Consumer<'static, u32, U4, u8, SingleCore>,
        idle_consumer: Consumer<'static, u32, U4, u8, SingleCore>,
        dma_int_producer: Producer<'static, u32, U4, u8, SingleCore>,
        idle_producer: Producer<'static, u32, U4, u8, SingleCore>,
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

        //var gamma = new Uint16Array(256);
        //for (var i=0.0;i<256;i=i+1.0) gamma[i]=Math.round(Math.pow(i/255.0,2.8)*65535);
        //16 gamma corrected Brightness values 
        let _gamma: [u16; 16] = [0, 33, 232, 723, 1619, 3024, 5038, 7757,
                                11274, 15678, 21058, 27499, 35085, 43899, 54023, 65535];

        let mydoublebuffer = Doublebuffer {
            #[repr(align(128))]
            a: [0x945F, 0xFFFF, 
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0xFFFF
            ],
            #[repr(align(128))]
            b: [0x945F, 0xFFFF, 
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000
            ],
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

        //Set up two SPSCs to pass around ownership of the DMA buffer pointers
        //Safety: split call: in init interrupts are not yet enabled so mutating a static is still safe
        //Safety: u8_sc: safe on single core systems
        static mut IDLE2DMA_SPSC: Queue<u32, U4, u8, SingleCore> = unsafe {Queue(i::Queue::u8_sc())};
        let (idle_producer, dma_int_consumer) = unsafe {IDLE2DMA_SPSC.split()};
        static mut DMA2IDLE_SPSC: Queue<u32, U4, u8, SingleCore> = unsafe {Queue(i::Queue::u8_sc())};
        let (dma_int_producer, idle_consumer) = unsafe {DMA2IDLE_SPSC.split()};

        //Return the now initialized Late Ressources
        init::LateResources {
            myitm,
            mygpiob,
            //mytim4,
            mydma,
            mydoublebuffer,
            idle_producer, 
            dma_int_consumer,
            dma_int_producer, 
            idle_consumer,
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
