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
}
pub fn timerconfig(
    rcc: &stm32ral::rcc::Instance,
    tim2: &stm32ral::tim2::Instance,
    tim3: &stm32ral::tim3::Instance,
    tim4: &stm32ral::tim4::Instance,
) {
    const PRESCALER: u32 = 0x3FFF;
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
    //Prescaler for TIM3 divide by 2^16
    modify_reg!(stm32ral::tim3, tim3, PSC, PSC: PRESCALER);
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
    //Period 0xFFFF
    modify_reg!(stm32ral::tim3, tim3, ARR, ARR: 0xFFFF);
    //Delay afte 1/4 period go high
    modify_reg!(stm32ral::tim3, tim3, CCR2, CCR: 0x3FFF);
    //Create an update event to auto reload the preload values
    write_reg!(stm32ral::tim3, tim3, EGR, UG: Update);

    //TIM2 configuration
    //Prescaler for TIM2 divide by 2^16
    modify_reg!(stm32ral::tim2, tim2, PSC, PSC: PRESCALER);
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
    //Period 0x3FFF
    modify_reg!(stm32ral::tim2, tim2, ARR, ARR: 0x3FFF);
    //Delay after 1/4 period go high
    modify_reg!(stm32ral::tim2, tim2, CCR2, CCR: 0x0FFF);
    //Create an update event to auto reload the preload values
    write_reg!(stm32ral::tim2, tim2, EGR, UG: Update);

    //TIM4 configuration
    //Prescaler for TIM4 divide by 2^16
    modify_reg!(stm32ral::tim4, tim4, PSC, PSC: PRESCALER);
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
    //Period 0x07FF
    modify_reg!(stm32ral::tim4, tim4, ARR, ARR: 0x07FF);
    //Delay after 1/2 period go high
    modify_reg!(stm32ral::tim4, tim4, CCR2, CCR: 0x03FF);
    //Create an update event to auto reload the preload values
    write_reg!(stm32ral::tim4, tim4, EGR, UG: Update);
    //Clear all TIM4 interupt Flags
    write_reg!(
        stm32ral::tim4,
        tim4,
        SR,
        TIF: Clear,
        UIF: Clear,
        CC1IF: Clear,
        CC2IF: Clear,
        CC3IF: Clear,
        CC4IF: Clear
    );

    //Enable TIM4 interuupts for CC2 and update events, disable all other interrupts
    modify_reg!(
        stm32ral::tim4,
        tim4,
        DIER,
        CC1IE: Disabled,
        CC2IE: Enabled,
        CC3IE: Disabled,
        CC4IE: Disabled,
        TIE: Disabled,
        UIE: Enabled
    );

    //Enable TIM4 DMA requests from Update event
    modify_reg!(stm32ral::tim4, tim4, DIER, UDE: Enabled);
    //Enable timer4, still inactive because gated by TIM2
    modify_reg!(stm32ral::tim4, tim4, CR1, CEN: Enabled);
    //Enable timer2, still inactive because gated by TIM3
    modify_reg!(stm32ral::tim2, tim2, CR1, CEN: Enabled);
    //We dont enable timer3, triggered by external pin EN set by Hardware
}
