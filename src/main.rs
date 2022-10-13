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

use mpu6050::{Mpu6050, Mpu6050Error};
use stm32_hal2::{
    self, access_global,
    clocks::{self, Clocks},
    gpio::{Pin, PinMode, Port},
    pac::{self},
};
use usb_device::UsbError;

mod accelerometer;
mod com;

fn write_vec_to_slice(x: [f32; 3], slice: &mut [u8]) {
    if slice.len() < 12 {
        return // slice not long enough
    }
    slice[0..4].copy_from_slice(&x[0].to_le_bytes());
    slice[4..8].copy_from_slice(&x[1].to_le_bytes());
    slice[8..12].copy_from_slice(&x[2].to_le_bytes());
}

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

    com::usb::setup(
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        clocks.hclk(),
        &mut dp.RCC,
    );

    let mut accelerometer =
        accelerometer::setup_accelerometer(dp.I2C1, &clocks, &mut delay, &mut led);

    let mut loops = 0;

    loop {
        if loops % 50 == 0 {
            com::led::toggle(&mut led);
        }

        //delay.delay_ms(1);

        let acc = accelerometer.get_acc().unwrap();

        let mut message_buffer = [0u8; 64];

        write_vec_to_slice(acc, &mut message_buffer);
        message_buffer[12] = b'\n';

        // message length is 13, 12 bytes of vector and 1 newline

        free(|cs| {
            com::usb::get_reply().borrow(cs).replace(Some((message_buffer, 13)))
        });

        loops += 1;
    }
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf() // abort
}
