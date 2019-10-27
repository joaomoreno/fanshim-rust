extern crate sysfs_gpio;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

const DAT: u64 = 15;
const CLK: u64 = 14;

struct HSL {
	hue: u32,
	saturation: f32,
	lightness: f32,
}

#[derive(Debug)]
struct RGB {
	red: u8,
	green: u8,
	blue: u8,
}

fn hsl_to_rgb(hsl: HSL) -> RGB {
	let a = hsl.saturation * hsl.lightness.min(1.0 - hsl.lightness);
	let f = |n: f32| {
		let k = (n + (hsl.hue as f32) / 30.0) % 12.0;
		(hsl.lightness - a * (k - 3.0).min(9.0 - k).min(1.0).max(-1.0))
	};
	RGB {
		red: (f(0.0) * 255.0) as u8,
		green: (f(8.0) * 255.0) as u8,
		blue: (f(4.0) * 255.0) as u8,
	}
}

struct Pixel {
	rgb: RGB,
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
	write_byte(dat, clk, pixel.rgb.blue)?;
	write_byte(dat, clk, pixel.rgb.green)?;
	write_byte(dat, clk, pixel.rgb.red)?;
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

	for i in 0u32..360 * 5 {
		let hsl = HSL {
			hue: i % 360,
			saturation: 1.0,
			lightness: 0.5,
		};
		let rgb = hsl_to_rgb(hsl);
		// println!("{:?}", rgb);
		let pixel = Pixel {
			rgb,
			brightness: 0.5,
			// brightness: 0.1 * (10 - ((i % 20) - 10).abs()) as f32,
		};
		show_pixel(&dat, &clk, &pixel)?;
		sleep(Duration::from_millis(1));
	}

	Ok(())
}
