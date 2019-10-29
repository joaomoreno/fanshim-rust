extern crate fanshim;
extern crate futures;
extern crate tokio;

use fanshim::FanSHIM;
use futures::{Future, Stream};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let fanshim = FanSHIM::new()?;
  let stream = fanshim.get_button_stream()?;

  tokio::run(
    stream
      .for_each(move |val| {
        println!("Pin changed value to {}", val);
        Ok(())
      })
      .map_err(|_| {}),
  );

  Ok(())
}
