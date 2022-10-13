use core::ops::Deref;
use cortex_m::delay::Delay;
use mpu6050::Mpu6050;
use stm32_hal2::{
    clocks::Clocks,
    gpio::{OutputType, Pin, PinMode, Port},
    i2c::{I2c, I2cDevice},
    pac::{i2c1, I2C1},
};

pub fn setup_accelerometer(
    regs: I2C1,
    clocks: &Clocks,
    delay: &mut Delay,
    led: &mut Pin,
) -> Mpu6050<I2c<I2C1>> {
    let mut scl = Pin::new(Port::B, 6, PinMode::Alt(4));
    let mut sda = Pin::new(Port::B, 7, PinMode::Alt(4));
    scl.output_type(OutputType::OpenDrain);
    sda.output_type(OutputType::OpenDrain);

    let i2c = I2c::new(regs, I2cDevice::One, 100000, clocks);

    let mut mpu = Mpu6050::new(i2c);

    match mpu.init(delay) {
        Ok(_) => mpu,
        Err(_) => {
            crate::com::led::led_message_loop(delay, led, 0b0001, 4);
        }
    }
}
