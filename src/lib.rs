extern crate sysfs_gpio;
use std::error::Error;
use std::sync::mpsc;
use std::thread::{sleep, spawn};
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

pub struct FanSHIM {
	fan: Pin,
	data: Pin,
	clock: Pin,
	button: Pin,
	on_press: Option<Box<dyn Fn() + Send>>,
	on_release: Option<Box<dyn Fn() + Send>>,
	on_hold: Option<Box<dyn Fn() + Send>>,
	hold_time: f32,
	polling_handle: Option<mpsc::Sender<()>>,
	polling_delay: Duration,
}

impl FanSHIM {
	pub fn new() -> Result<FanSHIM, Box<dyn Error>> {
		let fan = Pin::new(18);
		fan.set_direction(Direction::Out)?;
		fan.export()?;

		let data = Pin::new(15);
		data.set_direction(Direction::Low)?;
		data.export()?;

		let clock = Pin::new(14);
		clock.set_direction(Direction::Low)?;
		clock.export()?;

		let button = Pin::new(17);
		button.set_direction(Direction::Out)?;
		button.export()?;

		Ok(FanSHIM {
			fan,
			data,
			clock,
			button,
			on_press: None,
			on_release: None,
			on_hold: None,
			hold_time: 2.0,
			polling_handle: None,
			polling_delay: Duration::from_millis(50),
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

	pub fn set_on_press<CB: 'static + Fn() + Send>(
		&mut self,
		callback: CB,
	) -> Result<(), Box<dyn Error>> {
		self.on_press = Some(Box::new(callback));
		self.on_did_change_callbacks()
	}

	pub fn unset_on_press<CB: 'static + Fn() + Send>(&mut self) -> Result<(), Box<dyn Error>> {
		self.on_press = None;
		self.on_did_change_callbacks()
	}

	pub fn set_on_release<CB: 'static + Fn() + Send>(
		&mut self,
		callback: CB,
	) -> Result<(), Box<dyn Error>> {
		self.on_release = Some(Box::new(callback));
		self.on_did_change_callbacks()
	}

	pub fn unset_on_release<CB: 'static + Fn() + Send>(&mut self) -> Result<(), Box<dyn Error>> {
		self.on_release = None;
		self.on_did_change_callbacks()
	}

	pub fn set_on_hold<CB: 'static + Fn() + Send>(
		&mut self,
		callback: CB,
	) -> Result<(), Box<dyn Error>> {
		self.on_hold = Some(Box::new(callback));
		self.on_did_change_callbacks()
	}

	pub fn unset_on_hold<CB: 'static + Fn() + Send>(&mut self) -> Result<(), Box<dyn Error>> {
		self.on_hold = None;
		self.on_did_change_callbacks()
	}

	fn on_did_change_callbacks(&mut self) -> Result<(), Box<dyn Error>> {
		if let (None, None, None) = (
			self.on_press.as_ref(),
			self.on_release.as_ref(),
			self.on_hold.as_ref(),
		) {
			if let Some(tx) = &self.polling_handle {
				tx.send(())?;
			}
		} else {
			if let None = self.polling_handle {
				let (tx, rx) = mpsc::channel();
				let delay = self.polling_delay;
				let button = self.button;
				// let on_press = self.on_press;
				// let on_release = self.on_release;
				// let on_hold = self.on_hold;

				spawn(move || loop {
					let last = 1u8;

					loop {
						if let Ok(_) = rx.try_recv() {
							return;
						}

						let current = button.get_value().unwrap();

						if last > current {
							// if let Some(a) = self.on_press {}
						}

						sleep(delay);
					}
				});
				self.polling_handle = Some(tx);
			}
		}

		Ok(())
	}

	fn start_polling(&self) {}

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

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_set_led() -> Result<(), Box<dyn Error>> {
		let fanshim = FanSHIM::new()?;
		fanshim.set_led(255, 0, 0, 1.0)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(0, 255, 0, 1.0)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(0, 0, 255, 1.0)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(255, 255, 255, 1.0)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(255, 0, 0, 1.0)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(255, 0, 0, 0.75)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(255, 0, 0, 0.5)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(255, 0, 0, 0.25)?;
		sleep(Duration::from_millis(250));
		fanshim.set_led(0, 0, 0, 1.0)?;
		sleep(Duration::from_millis(250));
		fanshim.set_fan(true)?;
		sleep(Duration::from_millis(5000));
		fanshim.set_fan(false)?;
		sleep(Duration::from_millis(5000));
		fanshim.set_fan(true)?;
		sleep(Duration::from_millis(5000));
		fanshim.set_fan(false)?;
		Ok(())
	}
}
