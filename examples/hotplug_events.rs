extern crate libusb;

use std::error::Error;
use std::slice;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut ctx = libusb::Context::new()?;
    let filter = libusb::HotplugFilter::new()
        .enumerate();
    ctx.register_callback(filter, |device, event| {
        eprintln!("invoked");
        println!("{:?} - {:?}", device.device_descriptor(), event);
    })?;
    loop {
        ctx.handle_events();
    }
    Ok(())
}

