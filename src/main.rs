extern crate sysfs_gpio;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::Pin;

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

	for _ in 0..36 {
		clk.set_value(1)?;
		sleep(Duration::from_nanos(500));
		clk.set_value(0)?;
		sleep(Duration::from_nanos(500));
	}

	Ok(())
}

fn eof(dat: &Pin, clk: &Pin) -> Result<(), Box<dyn Error>> {
	dat.set_value(0)?;

	for _ in 0..32 {
		clk.set_value(1)?;
		sleep(Duration::from_nanos(500));
		clk.set_value(0)?;
		sleep(Duration::from_nanos(500));
	}

	Ok(())
}

fn write_byte(dat: &Pin, clk: &Pin, mut byte: u8) -> Result<(), Box<dyn Error>> {
	for _ in 0..8 {
		dat.set_value(byte & 0b10000000)?;
		clk.set_value(1)?;
		sleep(Duration::from_nanos(500));
		byte <<= 1;
		clk.set_value(0)?;
		sleep(Duration::from_nanos(500));
	}

	Ok(())
}

fn show_pixel(dat: &Pin, clk: &Pin, pixel: &Pixel) -> Result<(), Box<dyn Error>> {
	sof(dat, clk)?;
	write_byte(
		dat,
		clk,
		0b11100000 | (((3.0 * pixel.brightness) as u8) & 0b11111),
	)?;
	write_byte(dat, clk, pixel.blue)?;
	write_byte(dat, clk, pixel.green)?;
	write_byte(dat, clk, pixel.red)?;
	eof(dat, clk)?;
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let dat = Pin::new(DAT);
	let clk = Pin::new(CLK);
	dat.export()?;
	clk.export()?;

	let pixel = Pixel {
		red: 255,
		green: 0,
		blue: 0,
		brightness: 0.2,
	};

	show_pixel(&dat, &clk, &pixel)?;

	Ok(())
}
