use std::fs::File;
use std::io::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub terminal: String,
    pub baudrate: u32,
    pub gain: f32,
    pub freq: u32,
    pub path: String
}

impl Config {
  pub fn load(path: &str) -> Self {
    let mut file = match File::open(path) {
      Ok(x) => Ok(x),
      Err(x) => { error!("could not open the config file {}", path); Err(x) }
    }.unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();
    
    let v: Config = match serde_json::from_str(&contents[0..]) {
      Ok(x) => Ok(x),
      Err(x) => { error!("json invalid {}", x); Err(x) }
    }.unwrap();


    v    
  }
}
