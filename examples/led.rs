extern crate fanshim;

use fanshim::FanSHIM;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
  let fanshim = FanSHIM::new()?;
  fanshim.set_led(255, 0, 0, 1.0)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(0, 255, 0, 1.0)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(0, 0, 255, 1.0)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(255, 255, 255, 1.0)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(255, 0, 0, 1.0)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(255, 0, 0, 0.75)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(255, 0, 0, 0.5)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(255, 0, 0, 0.25)?;
  thread::sleep(Duration::from_millis(250));
  fanshim.set_led(0, 0, 0, 1.0)?;
  Ok(())
}
