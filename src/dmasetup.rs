use stm32ral::{modify_reg, read_reg, write_reg};

pub fn dmaconfig(
    rcc: &stm32ral::rcc::Instance,
    dma: &stm32ral::dma::Instance,
    spi: &stm32ral::spi::Instance,
    bufa: u32,
    bufb: u32
) {
    //const PRESCALER: u32 = 0x3FFF;
    //Enable DMA1 clocks
    modify_reg!(stm32ral::rcc, rcc, AHB1ENR, DMA1EN: Enabled);
    cortex_m::asm::dmb(); // ensure DMA is powered on before we write to it
    write_reg!(stm32ral::dma, dma, CR6, EN: Disabled); //Disable stream 6

    //clear all interrupt Flags for stream_6
    block_until! { read_reg!(stm32ral::dma, dma, CR6, EN == Disabled) }
    write_reg!(
        stm32ral::dma,
        dma,
        HIFCR,
        CDMEIF6: Clear,
        CFEIF6: Clear,
        CHTIF6: Clear,
        CTCIF6: Clear,
        CTEIF6: Clear
    );
    //set the USART Data register as DMA destination
    write_reg!(stm32ral::dma, dma, PAR6, &spi.DR as *const _ as u32);
    //set the Buffer A as DMA Doublebuffer 0 source
    write_reg!(stm32ral::dma, dma, M0AR6, bufa);
    //set the Buffer B as DMA Doublebuffer 1 source
    write_reg!(stm32ral::dma, dma, M1AR6, bufb);
    //set the Number of bytes to transfer
    write_reg!(stm32ral::dma, dma, NDTR6, super::BUFLEN as u32);
    //select the DMA channel 2 for stream 6, DMA Flow controller, Prio=high
    //Circular mode, double buffered, Mem to Peripheral,
    // Memory is byte incremented periferal is fixed byte
    //All interrupts exept Half complete and Direct Mode Error are enabled
    // Transfer error: Bus error during DMA read/write or
    // modification of Memory adress register of active doublebuffer
    // FFIO Error: Over/Underrun or incompatible MBurst setting for FIFO threshold
    modify_reg!(
        stm32ral::dma,
        dma,
        CR6,
        CHSEL: 2,
        PFCTRL: DMA,
        PL: High,
        CIRC: Enabled,
        DBM: Enabled,
        DIR: MemoryToPeripheral,
        MBURST: Single,
        MINC: Incremented,
        MSIZE: Bits8,
        PBURST: Single,
        PINC: Fixed,
        PINCOS: PSIZE,
        PSIZE: Bits8,
        DMEIE: Disabled,
        HTIE: Disabled,
        TCIE: Enabled,
        TEIE: Enabled
    );
    //Fifo direct mode, fifo error interrupt enabled, FIFO Threshold Half full
    write_reg!(
        stm32ral::dma,
        dma,
        FCR6,
        DMDIS: Disabled,
        FEIE: Enabled,
        FTH: Half
    );
    modify_reg!(stm32ral::dma, dma, CR6, EN: Enabled); //Enable stream 6
    block_until! { read_reg!(stm32ral::dma, dma, CR6, EN == Enabled) };
}
