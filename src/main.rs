#![no_main]
#![no_std]

/*
blink error codes

long  = _
short = .

_... accelerometer setup failed
._.. usb write error
__.. usb write would block
.._. usb flush would block
_._. usb flush error
.__. radio setup failure
*/

/*
Hardwired:

mpu6050 on i2c1, scl = B6, sda = B7
usb on pins A11 and A12


*/

use cortex_m::{
    self,
    delay::Delay,
    interrupt::{free, CriticalSection, Mutex},
};
use cortex_m_rt::entry;

use embedded_hal::spi::{Phase, Mode, Polarity};
use mpu6050::{Mpu6050, Mpu6050Error};
use stm32_hal2::{
    self, access_global,
    clocks::{self, Clocks},
    gpio::{Pin, PinMode, Port, OutputType},
    pac::{self}, spi::{Spi, SpiConfig, BaudRate, SlaveSelect, SpiCommMode},
};


mod accelerometer;
mod radio;
mod com;


#[entry]
fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();

    let mut dp = pac::Peripherals::take().unwrap();

    let clocks = Clocks {
        ..Default::default()
    };

    match clocks.setup() {
        Ok(_) => (),
        Err(_) => {
            panic!("")
            // we actually need the clocks to be able to blink, so we have to panic here
        }
    }
    let mut delay = Delay::new(cp.SYST, clocks.systick());
    let mut led = Pin::new(Port::C, 13, PinMode::Output);

  
    let nss = Pin::new(Port::A, 4, PinMode::Output);
    let reset = Pin::new(Port::A, 3, PinMode::Output);

    let radio = radio::setup_radio(dp.SPI1, nss, reset, 868, &mut led, &mut delay);
    

    com::led::led_message_loop(&mut delay, &mut led, 1, 1);
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf() // abort
}
