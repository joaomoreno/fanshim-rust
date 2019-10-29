extern crate sysfs_gpio;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Edge, Pin, PinValueStream};

pub struct FanSHIM {
	fan: Pin,
	data: Pin,
	clock: Pin,
	button: Pin,
	// hold_time: f32,
}

impl FanSHIM {
	pub fn new() -> Result<FanSHIM, Box<dyn Error>> {
		let fan = Pin::new(18);
		fan.export()?;
		fan.set_direction(Direction::High)?;

		let data = Pin::new(15);
		data.export()?;
		data.set_direction(Direction::Low)?;

		let clock = Pin::new(14);
		clock.export()?;
		clock.set_direction(Direction::Low)?;

		let button = Pin::new(17);
		button.export()?;
		button.set_direction(Direction::In)?;
		button.set_edge(Edge::BothEdges)?;

		Ok(FanSHIM {
			fan,
			data,
			clock,
			button,
			// hold_time: 2.0,
		})
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
		self.data.set_value(0)?;
		for _ in 0..32 {
			self.clock.set_value(1)?;
			sleep(Duration::from_nanos(250));
			self.clock.set_value(0)?;
			sleep(Duration::from_nanos(250));
		}
		// LED frame
		self.write_byte(0b1110_0000 | (((31.0 * brightness) as u8) & 0b1_1111))?;
		self.write_byte(blue)?;
		self.write_byte(green)?;
		self.write_byte(red)?;
		// end frame
		self.data.set_value(1)?;
		for _ in 0..32 {
			self.clock.set_value(1)?;
			sleep(Duration::from_nanos(250));
			self.clock.set_value(0)?;
			sleep(Duration::from_nanos(250));
		}
		Ok(())
	}

	pub fn set_fan(&self, on: bool) -> Result<(), Box<dyn Error>> {
		self.fan.set_value(on as u8)?;
		Ok(())
	}

	pub fn get_button_stream(&self) -> Result<PinValueStream, Box<dyn Error>> {
		Ok(self.button.get_value_stream()?)
	}

	fn write_byte(&self, mut byte: u8) -> Result<(), Box<dyn Error>> {
		for _ in 0..8 {
			self.data.set_value(byte & 0b10000000)?;
			self.clock.set_value(1)?;
			sleep(Duration::from_nanos(250));
			byte <<= 1;
			self.clock.set_value(0)?;
			sleep(Duration::from_nanos(250));
		}
		Ok(())
	}
}
