use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::io::Read;
use std::result::Result;
use std::process::Child;
use std::str;
use stoppable_thread;
use spmc;

extern crate hex;

pub const BUFFER_SIZE: usize = 128;

pub struct Adsb {
  pub child: spmc::Receiver<Vec<u8>>,
  handle: Option<stoppable_thread::StoppableHandle<()>>,
  killer: Child
}

impl Adsb{

  pub fn new(path: String, gain: f32, freq: u32) -> Result<Adsb, &'static str> {
    let dump = Adsb::get_thread(path, gain, freq)?;

    Ok(Adsb { child: dump.2, handle: Some(dump.1), killer: dump.0 })
  }

  pub fn reset(&mut self, path: String, gain: f32, freq: u32) -> Result<(), &'static str> {
    match self.killer.kill() {
      Ok(x) => Ok(x),
      Err(_x) => Err("Failed to kill the old worker thread :(")
    }?;
    match self.handle.take() {
      Some(t) => {t.stop(); ()},
      None => {}
    };
    
    let dump = Adsb::get_thread(path, gain, freq)?;
    self.child = dump.2;
    self.handle = Some(dump.1);
    self.killer = dump.0;

    Ok(())
  }

  fn get_thread(path: String, gain: f32, freq: u32) -> Result<(Child, stoppable_thread::StoppableHandle<()>, spmc::Receiver<Vec<u8>>), &'static str> {
    let (mut tx, rx): (spmc::Sender<Vec<u8>>, spmc::Receiver<Vec<u8>>) = spmc::channel();
    let (ctx, crx): (mpsc::Sender<Child>, mpsc::Receiver<Child>) = mpsc::channel();

    let child_handle = stoppable_thread::spawn(move |stop| {
      let mut child = match Command::new(path)
        .arg("--raw")
        .arg("--gain")
        .arg(format!("{:.2}", gain))
        .arg("--freq")
        .arg(format!("{}", freq))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(x) => x,
        Err(x) => {error!("could not start dump1090: {}", x); return; }
      };

      let mut childout = match child.stdout.take() {
        Some(x) => x,
        None => {error!("was not able to capture the output of dump1090"); return; }
      };
      
      let mut childerr = match child.stderr.take() {
        Some(x) => x,
        None => {error!("was not able to capture the output of dump1090"); return; }
      };
       
      let mut errbuf = [0u8; 1];
      match childerr.read(&mut errbuf) {
        Ok(_x) => if errbuf[0] == 70 {
          info!("dump1090 successfully found the device");
          
          match tx.send(vec!['^' as u8, 'D' as u8, 'I' as u8]) {
            Ok(x) => x,
            Err(x) => {error!("failed to pass dump1090 data over mpsc: {}", x); return; }
          };
        } else {
          error!("dump1090 did not find the device, {}", errbuf[0]);
          
          match tx.send(vec!['^' as u8, 'D' as u8, 'E' as u8]) {
            Ok(x) => x,
            Err(x) => {error!("failed to pass dump1090 data over mpsc: {}", x); return; }
          };
        },
        Err(x) => {error!("cannot read from dump1090: {}", x);}
      };

      match ctx.send(child) {
        Ok(x) => x,
        Err(x) => {error!("failed to pass thread handle over mpsc: {}", x); return; }
      };

      loop {
        let mut buffer = [0; BUFFER_SIZE];

        if stop.get() { 
          break; 
        }

        match childout.read(&mut buffer) {
          Ok(x) => x,
          Err(x) => {error!("cannot read from dump1090: {}", x); return;}
        };

        if buffer[0] == 0 {
          continue;
        }

        for line in String::from_utf8_lossy(&buffer[..]).split('\n') {
          if line.len() == 0 || line.len() % 2 == 1 || line.chars().nth(0usize).unwrap() == '\u{0}' {
            continue;
          }

          let mut send_buffer =
                      match hex::decode(
                        &line[1..line.len()-1]
                      ) { Ok(x) => x, Err(x) => { error!("hex decode failed: {:?}", x); vec![] } };

          send_buffer.insert(0, '*' as u8);

          match tx.send(send_buffer) {
            Ok(x) => x,
            Err(x) => {error!("failed to pass dump1090 data over mpsc: {}", x); return; }
          }; 
        }
    }});

    match crx.recv() {
      Ok(x) => Ok((x, child_handle, rx)),
      Err(x) => { 
        Err(Box::leak(x.to_string().into_boxed_str()))
      }
    }
  }
}
