extern crate colorsys;
extern crate fanshim;

use colorsys::{Hsl, Rgb};
use fanshim::FanSHIM;
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn Error>> {
  let fanshim = FanSHIM::new()?;
  let mut h: u16 = 0;

  loop {
    println!("{}", h);
    let hsl = Hsl::new(h as f64, 100.0, 50.0, Some(1.0));
    let rgb = Rgb::from(&hsl);

    // println!("{:?}", hsl);
    println!(
      "{} {} {}",
      rgb.get_red() as u8,
      rgb.get_green() as u8,
      rgb.get_blue() as u8
    );
    fanshim.set_led(
      rgb.get_red() as u8,
      rgb.get_green() as u8,
      rgb.get_blue() as u8,
      1.0,
    )?;

    thread::sleep(Duration::from_millis(100));

    h = (h + 1) % 360;
  }

  Ok(())
}
