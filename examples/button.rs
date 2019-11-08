extern crate fanshim;
extern crate futures;
extern crate tokio;

use fanshim::FanSHIM;
use futures::{Future, Stream};
use std::error::Error;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
  let mut fanshim = FanSHIM::new()?;
  fanshim.hold_time = Duration::from_secs(1);
  println!("Listening to button events...");
  tokio::run(
    fanshim
      .get_button_stream()?
      .for_each(move |val| {
        println!("{}", val);
        Ok(())
      })
      .map_err(|_| {}),
  );

  Ok(())
}
