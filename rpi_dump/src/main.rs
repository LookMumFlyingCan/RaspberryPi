mod adsb;
mod config;
mod uart;

use crate::config::Config;
use crate::adsb::Adsb;
use crate::uart::Uart;

extern crate pretty_env_logger;
#[macro_use] extern crate log;
use pretty_env_logger::env_logger;

fn main() {
  // set the log level to be maximal and init logger
  pretty_env_logger::env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

  // load the config
  let config = Config::load("config.json");

  // memcpy the dump1090 path
  let path = config.path.clone();

  // create a adsb handler
  let adsb = match Adsb::new(path, config.gain, config.freq) {
    Ok(x) => x,
    Err(x) => {error!("adsb decoder start failed: {}", x); return;}
  };

  // create and start a serial transmitter
  let mut serial = match Uart::new(adsb, &config.terminal[..], config.baudrate) {
    Ok(x) => x,
    Err(x) => {error!("serial handler start failed: {}", x); return;}
  };

  // make a serial reciever from the main thread since we dont need it anymore
  serial.reciever(config.path, config.gain, config.freq);
}
