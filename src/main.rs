#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;

mod dht11;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut led = pins.d13.into_output();
    led.set_high();

    let mut dht11 = dht11::Dht11::new(pins.d2.into_opendrain());

    loop {
        match dht11.measure() {
            Ok((temp, humidity)) => {
                ufmt::uwriteln!(&mut serial, "{} - {}", temp, humidity).unwrap_infallible();
            }
            Err(_err) => {
                led.set_low();
                ufmt::uwriteln!(&mut serial, "Failed to read temp/humidity").unwrap_infallible();
            }
        }

        arduino_hal::delay_ms(5000);
    }
}
