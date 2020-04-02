use stm32ral::{modify_reg, write_reg};

pub fn spiconfig(
    rcc: &stm32ral::rcc::Instance,
    spi: &stm32ral::spi::Instance,
) {
    //Enable SPI2 clock
    modify_reg!(stm32ral::rcc, rcc, APB1ENR, SPI2EN: Enabled);
    cortex_m::asm::dmb(); // ensure SPI is powered on before we write to it

    // disable all interrupts and dma 
    // we clock the dma by timer (not too fast)
    write_reg!(
        stm32ral::spi,
        spi,
        CR2,
        ERRIE: Masked,	
        FRF: Motorola,	
        RXDMAEN: Disabled,	
        RXNEIE: Masked,	
        SSOE: Disabled,	
        TXDMAEN: Disabled,	
        TXEIE: Masked	
    );

    // we only need tx on first rising edge data is already stable and latched
    // since APB1 is already half the clock speed compared to AHB1 
    // we only divide by 8 and not by 16 to get 6MBit SPI Line speed
    write_reg!(
        stm32ral::spi,
        spi,
        CR1,
        BIDIMODE: Unidirectional,
        BIDIOE: OutputEnabled,
        BR: Div8,
        CPHA: FirstEdge,
        CPOL: IdleLow,
        CRCEN: Disabled,
        CRCNEXT: TxBuffer,
        DFF: EightBit,
        LSBFIRST: MSBFirst,
        MSTR: Master,
        RXONLY: FullDuplex,
        SPE: Enabled,
        SSI: SlaveSelected,
        SSM: Disabled
    ); 
}
