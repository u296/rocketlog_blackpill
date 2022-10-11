use cortex_m::delay::Delay;
use stm32_hal2::gpio::Pin;

pub fn blink_on(delay: &mut Delay, led: &mut Pin, millis: u32) {
    led.set_low();
    delay.delay_ms(millis);
    led.set_high();
}

/// The pattern will be displayed from LSB to MSB
pub fn led_message_loop(delay: &mut Delay, led: &mut Pin, pattern: u32, len: usize) -> ! {
    let long_delay = 500;
    let short_delay = 200;
    let pulse_interval = 50;
    let repeat_interval = 3000;

    // setting the pin to low will actually activate the LED, and
    // vice-versa

    loop {
        for i in 0..len {
            led.set_low(); // turn on LED
            if ((pattern >> i) & 1) == 1 {
                delay.delay_ms(long_delay);
            } else {
                delay.delay_ms(short_delay);
            }
            led.set_high(); // turn off LED
            delay.delay_ms(pulse_interval);
        }
        delay.delay_ms(repeat_interval);
    }
}
