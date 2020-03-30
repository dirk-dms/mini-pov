
//#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]
//#![feature(const_mut_refs)]

//use panic_halt as _;
use panic_itm as _;
use rtfm::app;

use stm32ral::gpio;
use stm32ral::{read_reg, write_reg};

use heapless::{
    consts::*,
    i,
    spsc::{Consumer, Producer, Queue},
};

use core::sync::atomic::Ordering;
use core::sync::atomic::compiler_fence;

#[macro_use]
mod util;

mod clocksetup;
mod dmasetup;
mod timersetup;

pub const ROWS: usize = 128; //128
pub const U16PERROW: usize = 14;
pub const BUFLEN: usize = ROWS * U16PERROW;

pub const LEDCMD: [u16; 2] = [0x945F, 0xFFFF];

#[repr(align(128))]
pub struct DMAbuffer (
    [u16; BUFLEN]
);

#[app(device = stm32ral::stm32f4::stm32f401, peripherals = true)]
const APP: () = {
    struct Resources {
        //Late Ressource
        dmabufa: DMAbuffer,
        dmabufb: DMAbuffer,
        dmabufc: DMAbuffer,

        mygpiob: stm32ral::gpio::Instance,
        myitm: cortex_m::peripheral::ITM,
        //mytim4: stm32ral::tim4::Instance,
        mydma: stm32ral::dma::Instance,
        dma_int_consumer: Consumer<'static, u32, U2,>,
        idle_consumer: Consumer<'static, u32, U2,>,
        dma_int_producer: Producer<'static, u32 , U2>,
        idle_producer: Producer<'static, u32 , U2>,
    }

    #[init()]
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

        let mut dmabufa = DMAbuffer([0; BUFLEN]);
        let mut dmabufb = DMAbuffer([0; BUFLEN]);
        let mut dmabufc = DMAbuffer([0; BUFLEN]);
        //Safety these raw pointers point to static initialized Memory buffers so they are always valid
        //They get passed back and forth between idle and interrrupt handler 
        // and only the code wich has the pointer can access the buffer.
        let bufrefa = &mut dmabufa as *const _ as u32;
        let bufrefb = &mut dmabufb as *const _ as u32;
        let bufrefc = &mut dmabufc as *const _ as u32;
   

        dmasetup::dmaconfig(&myrcc, &mydma, &myspi, bufrefa, bufrefb);

        //Set up two SPSCs to pass around ownership of the DMA buffer pointers
        //Safety: split call: in init interrupts are not yet enabled so mutating a static is still safe
        static mut IDLE2DMA_SPSC: Queue<u32, U2> = Queue(i::Queue::new());
        let (idle_producer, dma_int_consumer) = unsafe {IDLE2DMA_SPSC.split()};
        static mut DMA2IDLE_SPSC: Queue<u32, U2> = Queue(i::Queue::new());
        let (mut dma_int_producer, idle_consumer) = unsafe {DMA2IDLE_SPSC.split()};

        //try to send the two buffers not owned by the dma  to idle 
        // we've just taken one element out of the queue 
        // so there should be a free slot to put the result into
        if dma_int_producer.enqueue(bufrefc).is_err() {panic!("dma to idle queue full!")};

        //Return the now initialized Late Ressources
        init::LateResources {
            myitm,
            mygpiob,
            //mytim4,
            mydma,
            idle_producer, 
            dma_int_consumer,
            dma_int_producer, 
            idle_consumer,
            dmabufa,
            dmabufb,
            dmabufc,
        }
    }

    #[idle(resources = [idle_producer, idle_consumer])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            if let Some(next_buffer) = cx.resources.idle_consumer.dequeue() {
                // Prepare the next Buffer
                
                // compiler fence we *really make sure all other threads / cores / interupt handlers / DMAs <= we need this
                // observe any changes made in the code until now.
                // i.e. This prevents reordering load/stores accross this fence and compiles down to a dmb with memory clobber
                compiler_fence(Ordering:: SeqCst);
                // try to send it to DMA 
                // there exist three buffers globally of which usually two but at least one one is always active 
                // inside the DMA unit so max 2 Buffers can be queued at any time
                // so there should always be a free slot in the queue to store the pointer
                if cx.resources.idle_producer.enqueue(next_buffer).is_err() {panic!("idle to dma queue full!")};
            // We dindn't get a buffer nothing to do but sleep
            } else {
                cortex_m::asm::wfi();
            }
        }
    }

    #[task(binds = DMA1_STREAM6, priority=3, resources = [myitm, mygpiob, mydma, dma_int_consumer, dma_int_producer])]
    fn dma_handler(cx: dma_handler::Context) {

        let (finished_buf,active_buf) = if read_reg!(stm32ral::dma, cx.resources.mydma, CR6, CT == Memory0) {
            //Memory0 active, Memory 1 just finished
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BS12: Set); //green on
            (read_reg!(stm32ral::dma, cx.resources.mydma, M1AR6) , read_reg!(stm32ral::dma, cx.resources.mydma, M0AR6))
        } else {
            //Memory1 active, Memory 0 just finished
            write_reg!(gpio, cx.resources.mygpiob, BSRR, BR12: Reset); //green off
            (read_reg!(stm32ral::dma, cx.resources.mydma, M0AR6), read_reg!(stm32ral::dma, cx.resources.mydma, M1AR6))
        };

        // enqueue finished buffer as free buffer for idle task if it is not the same buffer 
        // as the currently active one (was not re-scheduled)
        // might happen if the finished buffer was re scheduled due to no new data available
        if finished_buf != active_buf {
            if cx.resources.dma_int_producer.enqueue(finished_buf).is_err() {panic!("dma to idle queue full!")};
        };

        //Check for new data zu send
        if let Some(next_buffer) = cx.resources.dma_int_consumer.dequeue() {
            
            if read_reg!(stm32ral::dma, cx.resources.mydma, CR6, CT == Memory0) {
                //Memory 0 is active so update Memory 1 to next bufferr
                write_reg!(stm32ral::dma, cx.resources.mydma, M1AR6, next_buffer);
               
            } else {
                //Memory 1 is active so update Memory 0 to next bufferr
                write_reg!(stm32ral::dma, cx.resources.mydma, M0AR6, next_buffer);
            };    

        } else {
            // We dindn't get a new buffer re-schedule the currently active one instead
            // This meaans we didn't get new data on time and as a result we re-transmit the currently active buffer
            // In this case we only possess one pointer inside the DMA unit 
            // and two pointers are somwhere in the queues or in use in the idle task
            
            if read_reg!(stm32ral::dma, cx.resources.mydma, CR6, CT == Memory0) {
                //Memory 0 is active so update Memory 1 to active_buf (re-schedule)
                write_reg!(stm32ral::dma, cx.resources.mydma, M1AR6, active_buf);
               
            } else {
                //Memory 1 is active so update Memory 0 to to active_buf (re-schedule)
                write_reg!(stm32ral::dma, cx.resources.mydma, M0AR6, active_buf);
            };
        };
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
