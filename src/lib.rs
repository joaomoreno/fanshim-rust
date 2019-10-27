extern crate sysfs_gpio;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

pub struct RPi {
	dat: Pin,
	clk: Pin,
}

impl RPi {
	pub fn new() -> Result<RPi, Box<dyn Error>> {
		let dat = Pin::new(15);
		dat.set_direction(Direction::Low)?;
		dat.export()?;
		let clk = Pin::new(14);
		clk.set_direction(Direction::Low)?;
		clk.export()?;
		Ok(RPi { dat, clk })
	}
	// https://cdn-shop.adafruit.com/datasheets/APA102.pdf
	pub fn set_led(
		&self,
		red: u8,
		green: u8,
		blue: u8,
		brightness: f32,
	) -> Result<(), Box<dyn Error>> {
		// start frame
		self.dat.set_value(0)?;
		for _ in 0..32 {
			self.clk.set_value(1)?;
			sleep(Duration::from_nanos(250));
			self.clk.set_value(0)?;
			sleep(Duration::from_nanos(250));
		}
		// LED frame
		self.write_byte(0b1110_0000 | (((31.0 * brightness) as u8) & 0b1_1111))?;
		self.write_byte(blue)?;
		self.write_byte(green)?;
		self.write_byte(red)?;
		// end frame
		self.dat.set_value(1)?;
		for _ in 0..32 {
			self.clk.set_value(1)?;
			sleep(Duration::from_nanos(250));
			self.clk.set_value(0)?;
			sleep(Duration::from_nanos(250));
		}
		Ok(())
	}

	fn write_byte(&self, mut byte: u8) -> Result<(), Box<dyn Error>> {
		for _ in 0..8 {
			self.dat.set_value(byte & 0b10000000)?;
			self.clk.set_value(1)?;
			sleep(Duration::from_nanos(250));
			byte <<= 1;
			self.clk.set_value(0)?;
			sleep(Duration::from_nanos(250));
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_set_led() -> Result<(), Box<dyn Error>> {
		let pi = RPi::new()?;
		pi.set_led(255, 0, 0, 1.0)?;
		sleep(Duration::from_millis(500));
		pi.set_led(0, 255, 0, 1.0)?;
		sleep(Duration::from_millis(500));
		pi.set_led(0, 0, 255, 1.0)?;
		sleep(Duration::from_millis(500));
		pi.set_led(255, 255, 255, 1.0)?;
		sleep(Duration::from_millis(500));
		pi.set_led(0, 0, 0, 1.0)?;
		Ok(())
	}
}
