use core::{borrow::Borrow, cell::RefCell};
use cortex_m::{
    interrupt::{free, CriticalSection, Mutex},
    peripheral::NVIC,
};

use stm32_hal2::{
    access_global,
    gpio::{OutputType, Pin, PinMode, Port},
    make_globals,
    pac::{self, interrupt, OTG_FS_DEVICE, OTG_FS_GLOBAL, OTG_FS_PWRCLK, RCC},
    usb_otg::{self, Usb1BusType},
};

use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

pub type SerialPortType<'a> = SerialPort<'a, Usb1BusType>;
pub type UsbDeviceType<'a> = UsbDevice<'a, Usb1BusType>;

/* This is quite bad, but the problem is that the buffer
 * must have a static lifetime, and the allocator must
 * be static because the serial and device contain a
 * reference to it, meaning we would need to hold a mutex
 * guard forever, preventing interrupts.
 */

// this might need to have a size of 1280 bytes (320 32-bit words)
// because that's how many bytes the f411 has in its
// otg contrtoller FIFO
static mut EP_BUFFER: [u32; 1024] = [0; 1024];
static mut GLOB_BUS_ALLOCATOR: Option<UsbBusAllocator<Usb1BusType>> = None;

make_globals! {
    (GLOB_USB_SERIAL, SerialPort<Usb1BusType>),
    (GLOB_USB_DEVICE, UsbDevice<Usb1BusType>),
    (GLOB_REPLY, ([u8; 64], usize))
}

#[interrupt]
fn OTG_FS() {
    free(|cs| {
        access_global!(GLOB_USB_SERIAL, usb_serial, cs);
        access_global!(GLOB_USB_DEVICE, usb_device, cs);

        /* poll only returns true when some event has happened,
         * like a new packet coming in. This means that if poll
         * isn't returning true, nothing is left to read from the
         * host, and nothing is left to send i presume
         *
         * is poll required for data to be transmitted to the host?
         */
        if !usb_device.poll(&mut [usb_serial]) {
            return;
        }

        let mut buf = [0; 128];

        // if we don't read it seems like some buffer overflows and the
        // board panics
        match usb_serial.read(&mut buf) {
            Ok(count) => {
                if count == 4 && &buf[0..4] == b"poll" {
                    match GLOB_REPLY.borrow(cs).borrow().as_ref() {
                        Some((a, b)) => match usb_serial.write(&a[0..*b]) {
                            Ok(_) => (),
                            Err(UsbError::WouldBlock) => (),
                            Err(_e) => panic!("usb error"),
                        },
                        None => (),
                    }
                }
            }
            Err(UsbError::WouldBlock) => {}
            Err(_e) => {
                panic!("usb error")
            }
        }
    });
}

pub fn setup(
    global: OTG_FS_GLOBAL,
    device: OTG_FS_DEVICE,
    pwrclk: OTG_FS_PWRCLK,
    hclk: u32,
    rcc: &mut RCC,
) {
    // turn on usb power
    rcc.apb1enr.modify(|_, w| w.pwren().set_bit());

    // setup pins, check datasheet page 47
    let mut usb_dm = Pin::new(Port::A, 11, PinMode::Alt(10));
    let mut usb_dp = Pin::new(Port::A, 12, PinMode::Alt(10));
    usb_dm.output_type(OutputType::PushPull);
    usb_dp.output_type(OutputType::PushPull);

    let usb = usb_otg::Usb1::new(global, device, pwrclk, hclk);
    free(|cs| unsafe {
        GLOB_BUS_ALLOCATOR = Some(Usb1BusType::new(usb, &mut EP_BUFFER));
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
    });

    // enable interrupts
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::interrupt::OTG_FS);
        // set nvic priority?
    };
}

pub fn get_serial() -> &'static Mutex<RefCell<Option<SerialPortType<'static>>>> {
    &GLOB_USB_SERIAL
}

pub fn get_device() -> &'static Mutex<RefCell<Option<UsbDeviceType<'static>>>> {
    &GLOB_USB_DEVICE
}

pub fn get_reply() -> &'static Mutex<RefCell<Option<([u8; 64], usize)>>> {
    &GLOB_REPLY
}

pub fn is_initialized() -> bool {
    free(|cs| {
        GLOB_USB_SERIAL.borrow(cs).borrow().is_some()
            && GLOB_USB_DEVICE.borrow(cs).borrow().is_some()
    })
}

pub fn request_interrupt(nvic: &mut NVIC) {
    nvic.request(pac::interrupt::OTG_FS);
}
