extern crate sysfs_gpio;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

const DAT: u64 = 15;
const CLK: u64 = 14;

struct Pixel {
	red: u8,
	green: u8,
	blue: u8,
	brightness: f32,
}

// Emit exactly enough clock pulses to latch the small dark die APA102s which are weird
// for some reason it takes 36 clocks, the other IC takes just 4 (number of pixels/2)
fn sof(dat: &Pin, clk: &Pin) -> Result<(), Box<dyn Error>> {
	dat.set_value(0)?;

	for _ in 0..32 {
		clk.set_value(1)?;
		sleep(Duration::from_nanos(250));
		clk.set_value(0)?;
		sleep(Duration::from_nanos(250));
	}

	Ok(())
}

fn eof(dat: &Pin, clk: &Pin) -> Result<(), Box<dyn Error>> {
	dat.set_value(1)?;

	for _ in 0..32 {
		clk.set_value(1)?;
		sleep(Duration::from_nanos(250));
		clk.set_value(0)?;
		sleep(Duration::from_nanos(250));
	}

	Ok(())
}

fn write_byte(dat: &Pin, clk: &Pin, mut byte: u8) -> Result<(), Box<dyn Error>> {
	for _ in 0..8 {
		dat.set_value(byte & 0b10000000)?;
		clk.set_value(1)?;
		sleep(Duration::from_nanos(250));
		byte <<= 1;
		clk.set_value(0)?;
		sleep(Duration::from_nanos(250));
	}

	Ok(())
}

fn show_pixel(dat: &Pin, clk: &Pin, pixel: &Pixel) -> Result<(), Box<dyn Error>> {
	sof(dat, clk)?;
	write_byte(
		dat,
		clk,
		0b1110_0000 | (((31.0 * pixel.brightness) as u8) & 0b1_1111),
	)?;
	write_byte(dat, clk, pixel.blue)?;
	write_byte(dat, clk, pixel.green)?;
	write_byte(dat, clk, pixel.red)?;
	eof(dat, clk)?;
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let dat = Pin::new(DAT);
	dat.set_direction(Direction::Low)?;
	dat.export()?;
	let clk = Pin::new(CLK);
	clk.set_direction(Direction::Low)?;
	clk.export()?;

	for i in 0i32..100 {
		let pixel = Pixel {
			red: 255,
			green: 0,
			blue: 0,
			brightness: 0.1 * (10 - ((i % 20) - 10).abs()) as f32,
		};
		show_pixel(&dat, &clk, &pixel)?;
		sleep(Duration::from_millis(5));
	}

	Ok(())
}
