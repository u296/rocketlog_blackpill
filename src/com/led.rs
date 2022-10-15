use cortex_m::delay::Delay;
use stm32_hal2::gpio::{Pin, Port};

// port doesn't implement partialeq

// setting the pin low will turn on the LED

pub fn turn_on(led: &mut Pin) {
    if led.pin != 13 /*|| led.port != Port::C*/ {
        panic!("not an led")
    }
    led.set_low();
}

pub fn turn_off(led: &mut Pin) {
    if led.pin != 13 /*|| led.port != Port::C*/ {
        panic!("not an led")
    }
    led.set_high();
}

pub fn toggle(led: &mut Pin) {
    if led.is_high() {
        turn_on(led);
    } else {
        turn_off(led);
    }
}

pub fn blink_on(delay: &mut Delay, led: &mut Pin, millis: u32) {
    turn_on(led);
    delay.delay_ms(millis);
    turn_off(led);
}

/// The pattern will be displayed from LSB to MSB
pub fn led_message_loop(delay: &mut Delay, led: &mut Pin, pattern: u32, len: usize) -> ! {
    let long_delay = 1000;
    let short_delay = 500;
    let pulse_interval = 100;
    let repeat_interval = 3000;


    loop {
        for i in 0..len {
            turn_on(led); // turn on LED
            if ((pattern >> i) & 1) == 1 {
                delay.delay_ms(long_delay);
            } else {
                delay.delay_ms(short_delay);
            }
            turn_off(led); // turn off LED
            delay.delay_ms(pulse_interval);
        }
        delay.delay_ms(repeat_interval);
    }
}
