use stm32ral::{modify_reg, write_reg};

pub fn timer234debugstop(dbgmcu: &stm32ral::dbgmcu::Instance) {
    // Stop timer 2,3,4 on debug halt for better debugging
    modify_reg!(stm32ral::dbgmcu, dbgmcu, APB1_FZ, DBG_TIM2_STOP: 1, DBG_TIM3_STOP: 1, DBG_TIM4_STOP: 1);
}
pub fn portconfig(rcc: &stm32ral::rcc::Instance, gpio: &stm32ral::gpio::Instance) {
    //enable clock for port b
    modify_reg!(stm32ral::rcc, rcc, AHB1ENR, GPIOBEN: Enabled);
    //set output mode for the two leds on pin b2 and b12
    modify_reg!(stm32ral::gpio, gpio, MODER, MODER2: Output, MODER12: Output);
    //select alternate function number AF2 for pin PB4 (TIM3_CH1)
    modify_reg!(stm32ral::gpio, gpio, AFRL, AFRL4: AF2);
    //set alternate function mode for pin b4 (trigger input TIM3_CC1)
    modify_reg!(stm32ral::gpio, gpio, MODER, MODER4: Alternate);

    //set open drain output mode for pin B0 (FET control) for the FAN
    // drive '0' for off, high Z is on.
    //write_reg!(stm32ral::gpio, gpio, BSRR, BR0: Reset); //0 is 0 = FAN on

    write_reg!(stm32ral::gpio, gpio, BSRR, BS0: Set); //1 is High Z = FAN off
    modify_reg!(stm32ral::gpio, gpio, PUPDR, PUPDR0: PullUp);
    modify_reg!(stm32ral::gpio, gpio, OTYPER, OT0: OpenDrain);
    modify_reg!(stm32ral::gpio, gpio, MODER, MODER0: Output);
}
pub fn timerconfig(
    rcc: &stm32ral::rcc::Instance,
    tim2: &stm32ral::tim2::Instance,
    tim3: &stm32ral::tim3::Instance,
    tim4: &stm32ral::tim4::Instance,
) {
    const SPIDIV: u32 = 16;
    const BYTEDIV: u32 = SPIDIV * 8;

    const TIM4PERIOD: u32 = 2; //period is 2, generate 1 DMA strobe per Byte / Update event
    const TIM4DIV: u32 = BYTEDIV / TIM4PERIOD; //count at double the bytefrequency

    const TIM2PERIOD: u32 = 16; //16*4 = 64 Bytes intervall with 28 Bytes data
    const TIM2DIV: u32 = BYTEDIV * 4; //counts in 4 Byte steps

    const TIM3PERIOD: u32 = 129; //128 collums image and one collumn off
    const TIM3DIV: u32 = TIM2DIV * TIM2PERIOD; //counts image collumns

    //Enable Timer clocks
    modify_reg!(
        stm32ral::rcc,
        rcc,
        APB1ENR,
        TIM2EN: Enabled,
        TIM3EN: Enabled,
        TIM4EN: Enabled
    );
    //TIM3 configuration
    //Prescaler for TIM3
    modify_reg!(stm32ral::tim3, tim3, PSC, PSC: TIM3DIV);
    //Onepulse mode, preload, not enabled, up
    modify_reg!(
        stm32ral::tim3,
        tim3,
        CR1,
        ARPE: Enabled,
        CEN: Disabled,
        CKD: Div1,
        CMS: EdgeAligned,
        DIR: Up,
        OPM: Enabled,
        UDIS: Enabled,
        URS: CounterOnly
    );
    //Configure TIM3 Master mode controller to send OC2REF as TRGO
    modify_reg!(stm32ral::tim3, tim3, CR2, MMS: CompareOC2);
    //CC1 channel as input, IC1 mapped on TI1, Filtered by 8 CLK_INT
    modify_reg!(stm32ral::tim3, tim3, CCMR1, CC1S: 1, IC1F: FCK_INT_N8);
    //TI1FP1 must detect a rising edge
    modify_reg!(stm32ral::tim3, tim3, CCER, CC1P: 0, CC1NP: 0);
    //Configure TI1FP1 as trigger for the slave mode controller (TRGI)
    modify_reg!(stm32ral::tim3, tim3, SMCR, TS: TI1FP1);
    //TI1FP1 is used to start the counter
    modify_reg!(stm32ral::tim3, tim3, SMCR, SMS: Trigger_Mode);
    //PWM mode 2 with preload for OC2
    modify_reg!(stm32ral::tim3, tim3, CCMR1, OC2M: 0b111, OC2PE: 1);
    //Period 129
    modify_reg!(stm32ral::tim3, tim3, ARR, ARR: TIM3PERIOD - 1);
    //Delay afte 1 off, 128 on period go high
    modify_reg!(stm32ral::tim3, tim3, CCR2, CCR: 1);
    //Create an update event to auto reload the preload values
    write_reg!(stm32ral::tim3, tim3, EGR, UG: Update);

    //TIM2 configuration
    //Prescaler for TIM2
    modify_reg!(stm32ral::tim2, tim2, PSC, PSC: TIM2DIV);
    // preload, not enabled, up
    modify_reg!(
        stm32ral::tim2,
        tim2,
        CR1,
        ARPE: Enabled,
        CEN: Disabled,
        CKD: Div1,
        CMS: EdgeAligned,
        DIR: Up,
        OPM: Disabled,
        UDIS: Enabled,
        URS: CounterOnly
    );
    //Configure TIM2 Master mode controller to send OC2REF as TRGO
    modify_reg!(stm32ral::tim2, tim2, CR2, MMS: CompareOC2);
    //Configure ITR2 (from TIM3) as trigger for the slave mode controller (TRGI)
    modify_reg!(stm32ral::tim2, tim2, SMCR, TS: ITR2);
    //ITR2 is used to gate the counter
    modify_reg!(stm32ral::tim2, tim2, SMCR, SMS: Gated_Mode);
    //PWM mode 2 with preload for OC2
    modify_reg!(stm32ral::tim2, tim2, CCMR1, OC2M: 0b111, OC2PE: 1);
    //Period is 16
    modify_reg!(stm32ral::tim2, tim2, ARR, ARR: TIM2PERIOD - 1);
    //16-9 = 7 counts high (a 4 bytes = 28 Bytes)
    modify_reg!(stm32ral::tim2, tim2, CCR2, CCR: 9);
    //Create an update event to auto reload the preload values
    write_reg!(stm32ral::tim2, tim2, EGR, UG: Update);

    //TIM4 configuration
    //Prescaler for TIM4
    modify_reg!(stm32ral::tim4, tim4, PSC, PSC: TIM4DIV);
    // preload, not enabled, up
    modify_reg!(
        stm32ral::tim4,
        tim4,
        CR1,
        ARPE: Enabled,
        CEN: Disabled,
        CKD: Div1,
        CMS: EdgeAligned,
        DIR: Up,
        OPM: Disabled,
        UDIS: Enabled,
        URS: CounterOnly
    );
    //Configure ITR1 (from TIM2) as trigger for the slave mode controller (TRGI)
    modify_reg!(stm32ral::tim4, tim4, SMCR, TS: ITR1);
    //ITR1 is used to gate the counter
    modify_reg!(stm32ral::tim4, tim4, SMCR, SMS: Gated_Mode);
    //PWM mode 2 with preload for OC2
    modify_reg!(stm32ral::tim4, tim4, CCMR1, OC2M: 0b111, OC2PE: 1);
    //Period 0x01
    modify_reg!(stm32ral::tim4, tim4, ARR, ARR: TIM4PERIOD - 1);
    //Delay after 1/2 period go high
    modify_reg!(stm32ral::tim4, tim4, CCR2, CCR: 0x1);
    //Create an update event to auto reload the preload values
    write_reg!(stm32ral::tim4, tim4, EGR, UG: Update);

    //Enable TIM4 DMA requests from Update event
    modify_reg!(stm32ral::tim4, tim4, DIER, UDE: Enabled);
    //Enable timer4, still inactive because gated by TIM2
    modify_reg!(stm32ral::tim4, tim4, CR1, CEN: Enabled);
    //Enable timer2, still inactive because gated by TIM3
    modify_reg!(stm32ral::tim2, tim2, CR1, CEN: Enabled);
    //We dont enable timer3, triggered by external pin EN set by Hardware
}
