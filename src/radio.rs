//! functions for dealing with sx1276 LoRa radio chip, specifically
//! a XL1276-PO1, marked as 868 MHz
//! 
use cortex_m::delay::Delay;
use stm32_hal2::{pac::SPI1, gpio::{Pin, Port, PinMode}, spi::{BaudRate, Spi}};
use sx127x_lora::LoRa;


/// ## Expects the sx1276 to be connected as follows
/// * sclk to A5
/// * miso to A6
/// * mosi to A7
/// * nss as output
/// * reset as output
/// ## Notes
/// 868 (MHz) is probably a good frequency
pub fn setup_radio(regs: SPI1, nss: Pin, reset: Pin, frequency: i64, led: &mut Pin, delay: &mut Delay) -> LoRa<Spi<SPI1>, Pin, Pin> {
    // we want mode 0, software slave select, 8 bits per word
    // and full duplex, which is default
    let spi_cfg = Default::default();

    // we need to setup pins because this stupid HAL lets us
    // enable things in the wrong context

    // see datasheet page 47 for alt modes
    let _sclk = Pin::new(Port::A, 5, PinMode::Alt(5));
    let _miso = Pin::new(Port::A, 6, PinMode::Alt(5));
    let _mosi = Pin::new(Port::A, 7, PinMode::Alt(5));

    // apb bus runs at 100MHz by default it seems, we want to
    // run at roughly 8MHz judging by examples, so we're gonna
    // prescale by dividing by 16 to achieve 6.25 MHz
    let spi = Spi::new(regs, spi_cfg, BaudRate::Div16);

    // 868 is marked in the table on the radio
    match LoRa::new(spi, nss, reset, frequency, delay) {
        Ok(radio) => radio,
        Err(_e) => crate::com::led::led_message_loop(delay, led, 0b0110, 4)
    }
}