extern crate futures;
extern crate sysfs_gpio;

use futures::prelude::*;
use std::error::Error;
use std::fmt;
use std::thread::sleep;
use std::time::{Duration, Instant};
use sysfs_gpio::{Direction, Edge, Pin, PinValueStream};
use tokio::timer::Delay;

pub struct FanSHIM {
	fan: Pin,
	data: Pin,
	clock: Pin,
	button: Pin,
	// hold_time: f32,
}

#[derive(Debug)]
pub enum ButtonEvent {
	Press,
	Release,
	Hold,
}

impl fmt::Display for ButtonEvent {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ButtonEvent::Press => write!(f, "press"),
			ButtonEvent::Release => write!(f, "release"),
			ButtonEvent::Hold => write!(f, "hold"),
		}
	}
}

pub struct ButtonStream {
	stream: PinValueStream,
	delay: Option<Delay>,
}

impl ButtonStream {
	fn new(stream: PinValueStream) -> ButtonStream {
		ButtonStream {
			stream,
			delay: None,
		}
	}
}

impl Stream for ButtonStream {
	type Item = ButtonEvent;
	type Error = sysfs_gpio::Error;

	fn poll(&mut self) -> Poll<Option<ButtonEvent>, Self::Error> {
		match self.delay.take() {
			Some(mut d) => match d
				.poll()
				.map_err(|_| sysfs_gpio::Error::Unexpected(String::from("timer error")))?
			{
				Async::Ready(_) => Ok(Some(ButtonEvent::Hold).into()),
				Async::NotReady => match self.stream.poll()? {
					Async::Ready(Some(0)) => {
						let later = Instant::now() + Duration::from_secs(2);
						self.delay = Some(Delay::new(later));
						Ok(Some(ButtonEvent::Press).into())
					}
					Async::Ready(Some(_)) => {
						self.delay = None;
						Ok(Some(ButtonEvent::Release).into())
					}
					Async::Ready(None) => Ok(Async::Ready(None)),
					Async::NotReady => {
						self.delay = Some(d);
						Ok(Async::NotReady)
					}
				},
			},
			None => match self.stream.poll()? {
				Async::Ready(Some(0)) => {
					let later = Instant::now() + Duration::from_secs(2);
					self.delay = Some(Delay::new(later));
					Ok(Some(ButtonEvent::Press).into())
				}
				Async::Ready(Some(_)) => {
					self.delay = None;
					Ok(Some(ButtonEvent::Release).into())
				}
				Async::Ready(None) => Ok(Async::Ready(None)),
				Async::NotReady => Ok(Async::NotReady),
			},
		}
	}
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

	pub fn get_button_stream(&self) -> Result<ButtonStream, Box<dyn Error>> {
		let stream = self.button.get_value_stream()?;
		Ok(ButtonStream::new(stream))
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
