#![no_main]
#![no_std]

/*
Hardwired:

mpu6050 on i2c1, scl = B6, sda = B7

*/


use cortex_m::{
    self,
    delay::Delay,
    interrupt::{free, CriticalSection, Mutex},
};
use cortex_m_rt::{entry};

use mpu6050::{Mpu6050, Mpu6050Error};
use stm32_hal2::{
    self, access_global,
    clocks::{self, Clocks},
    gpio::{OutputType, Pin, PinMode, Port},
    i2c_f4::{I2c, I2cDevice},
    make_globals,
    pac::{self, gpioa, i2c1, GPIOA, I2C1, interrupt},
    usart::{Usart, UsartConfig},
    usb_otg::{self, Usb1, Usb1BusType},
};

use usb_device::{
    class_prelude::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

mod accelerometer;
mod com;

use core::{cell::*, borrow::BorrowMut};

/* This is quite bad, but the problem is that the buffer
 * must have a static lifetime, and the allocator must
 * be static because the serial and device contain a
 * reference to it, meaning we would need to hold a mutex
 * guard forever, preventing interrupts.
 */

// this might need to have a size of 1280 bytes
// because that's how many bytes the f411 has in its
// otg contrtoller FIFO
static mut EP_BUFFER: [u32; 1024] = [0; 1024];
static mut GLOB_BUS_ALLOCATOR: Option<UsbBusAllocator<usb_otg::Usb1BusType>> = None;

make_globals!(
    (GLOB_USB_SERIAL, SerialPort<usb_otg::Usb1BusType>),
    (GLOB_USB_DEVICE, UsbDevice<usb_otg::Usb1BusType>),
    (MESSAGE, u8),
    (ERROR, UsbError),
    (ERROR_SET, ())
);

#[entry]
fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();

    let dp = pac::Peripherals::take().unwrap();

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

    // turn on usb power
    dp.RCC.apb1enr.modify(|_, w| w.pwren().set_bit());

    //dp.PWR.csr.modify(|_,w| w.)

    

    // setup pins, check datasheet page 47
    let mut usb_dm = Pin::new(Port::A, 11, PinMode::Alt(10));
    let mut usb_dp = Pin::new(Port::A, 12, PinMode::Alt(10));
    usb_dm.output_type(OutputType::PushPull);
    usb_dp.output_type(OutputType::PushPull);

    //com::led::led_message_loop(&mut delay, &mut led, 0b1, 1);
    free(|cs| {
        let usb = usb_otg::Usb1::new(
            dp.OTG_FS_GLOBAL,
            dp.OTG_FS_DEVICE,
            dp.OTG_FS_PWRCLK,
            clocks.hclk(),
        );

        //com::led::led_message_loop(&mut delay, &mut led, 0b101, 3);
        unsafe {
            GLOB_BUS_ALLOCATOR = Some(usb_otg::Usb1BusType::new(usb, &mut EP_BUFFER));
            GLOB_USB_SERIAL
                .borrow(cs)
                .replace(Some(SerialPort::new(GLOB_BUS_ALLOCATOR.as_ref().unwrap())));
            GLOB_USB_DEVICE.borrow(cs).replace(Some(
                UsbDeviceBuilder::new(
                    GLOB_BUS_ALLOCATOR.as_ref().unwrap(),
                    UsbVidPid(0x16c0, 0x27dd),
                )
                .manufacturer("u296")
                .product("usart")
                .serial_number("sn")
                .device_class(USB_CLASS_CDC)
                .build(),
            ));
        }

        MESSAGE.borrow(cs).replace(Some(1));
    });

    // enable interrupts
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::interrupt::OTG_FS);
        cp.NVIC.set_priority(pac::interrupt::OTG_FS, 1); // set high priority
    };


    loop {
        com::led::blink_on(&mut delay, &mut led, 100);
        delay.delay_ms(100);

        

        free(|cs| {
            access_global!(MESSAGE, message, cs);

            *message += 1;
        });

        

    }


}



#[interrupt]
fn OTG_FS() {
    free(|cs| {
        access_global!(GLOB_USB_SERIAL, usb_serial, cs);
        access_global!(GLOB_USB_DEVICE, usb_device, cs);
        access_global!(MESSAGE, message, cs);

        /* poll only returns true when some event has happened,
         * like a new packet coming in. This means that if poll
         * isn't returning true, nothing is left to read from the
         * host, and nothing is left to send i presume
         */
        if !usb_device.poll(&mut [usb_serial]) {
            return
        }

        let mut buf = [0; 128];

        

        // if we don't read it seems like some buffer overflows and the
        // board panics
        match usb_serial.read(&mut buf) {
            Ok(count) => {

            },
            Err(UsbError::WouldBlock) => {},
            Err(e) => {}
        }

        if *message % 5 == 0 { // every 5 blinks, every second
            match usb_serial.write(b"A\n") {
                Ok(count) => {

                },
                Err(UsbError::WouldBlock) => {},
                Err(e) => {}
            }
        }
        
    });
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf() // abort
}
