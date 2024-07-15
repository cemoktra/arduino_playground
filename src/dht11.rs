//! module for reading DHT11 temperature and humidity

use arduino_hal::{
    delay_ms, delay_us,
    port::{mode::OpenDrain, Pin, PinOps},
};
use ufmt::uDisplay;

/// DHT11 reading error
pub enum Error {
    Crc,
    Timeout,
}

/// Wrapper for temperature
pub struct Temperature(i16);

/// Wrapper for humidity
pub struct Humidity(u16);

/// DHT11 sensor
pub struct Dht11<PD> {
    pin: Pin<OpenDrain, PD>,
}

const TIMEOUT: u32 = 1000;

impl<PD> Dht11<PD>
where
    PD: PinOps,
{
    /// create a new DHT11 struct reading from `pin`
    pub fn new(pin: Pin<OpenDrain, PD>) -> Self {
        Self { pin }
    }

    /// perform a measurement
    pub fn measure(&mut self) -> Result<(Temperature, Humidity), Error> {
        self.handshake()?;

        let mut data = [0u8; 5];
        for i in 0..40 {
            data[i / 8] <<= 1;
            if self.read_bit()? {
                data[i / 8] |= 1;
            }
        }

        let crc = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if crc != data[4] {
            return Err(Error::Crc);
        }

        let mut temp = i16::from(data[2] & 0x7f) * 10 + i16::from(data[3]);
        if data[2] & 0x80 != 0 {
            temp = -temp;
        }
        let humidity = u16::from(data[0]) * 10 + u16::from(data[1]);

        Ok((Temperature(temp), Humidity(humidity)))
    }

    /// prepare reading
    fn handshake(&mut self) -> Result<(), Error> {
        self.pin.set_low();
        delay_ms(20);

        self.pin.set_high();
        delay_us(40);

        self.read_bit()?;

        Ok(())
    }

    /// read a single bit
    fn read_bit(&self) -> Result<bool, Error> {
        let low = self.wait_for_pulse::<true>()?;
        let high = self.wait_for_pulse::<false>()?;
        Ok(high > low)
    }

    /// wait for pulse
    fn wait_for_pulse<const KIND: bool>(&self) -> Result<u32, Error> {
        let mut count = 0u32;

        while self.read_line() != KIND {
            count += 1;
            if count >= TIMEOUT {
                return Err(Error::Timeout);
            }
            delay_us(1);
        }

        Ok(count)
    }

    fn read_line(&self) -> bool {
        self.pin.is_high()
    }
}

impl uDisplay for Temperature {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let i = self.0 / 10;
        let d = self.0 % 10;

        <i16 as uDisplay>::fmt(&i, f)?;
        f.write_char('.')?;
        <i16 as uDisplay>::fmt(&d, f)?;
        f.write_str("°C")
    }
}

impl uDisplay for Humidity {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let i = self.0 / 10;
        let d = self.0 % 10;

        <u16 as uDisplay>::fmt(&i, f)?;
        f.write_char('.')?;
        <u16 as uDisplay>::fmt(&d, f)?;
        f.write_char('%')
    }
}
