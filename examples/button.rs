extern crate fanshim;
extern crate futures;
extern crate tokio;

use fanshim::{ButtonEvent, FanSHIM};
use futures::{Future, Stream};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let fanshim = FanSHIM::new()?;
  let stream = fanshim.get_button_stream()?;
  println!("Listening to button events...");
  tokio::run(
    stream
      .for_each(move |val| {
        println!("Button event: {}", val);

        match val {
          ButtonEvent::Press => fanshim.set_led(255, 0, 0, 1.0).unwrap(),
          ButtonEvent::Hold => fanshim.set_led(0, 255, 0, 1.0).unwrap(),
          ButtonEvent::Release => fanshim.set_led(0, 0, 0, 0.0).unwrap(),
        }

        Ok(())
      })
      .map_err(|_| {}),
  );

  Ok(())
}
