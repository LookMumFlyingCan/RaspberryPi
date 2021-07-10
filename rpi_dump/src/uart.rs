use crate::adsb::Adsb;
use std::boxed;
use std::time::Duration;
use std::io;
use std::sync::mpsc;
use serialport;
use stoppable_thread;
use spmc;
use std::fs::File;
use std::io::BufRead;

const SEND_BUFFER_SIZE: usize = 32usize;

pub struct Uart {
    pub decoder: Adsb,
    handle: Option<stoppable_thread::StoppableHandle<()>>,
    port: boxed::Box<dyn serialport::SerialPort>,
    hook: mpsc::Sender<bool>
}

impl Uart {
    pub fn new(dec: Adsb, name: &str, baudrate: u32) -> Result<Self, &'static str>{
        let crx = dec.child.clone();
        let (hooktx, hookrx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();

        match serialport::new(name, baudrate)
        .timeout(Duration::from_millis(5))
        .open() {
            Ok(x) => {
                let mut rclone: boxed::Box<dyn serialport::SerialPort> = match x.try_clone(){
                    Err(_y) => {
                        Err("failed to clone serial port")
                    },
                    Ok(y) => Ok(y)
                }?;
                Ok(
                    Uart { port: x, decoder: dec, handle: Some(Uart::get_output_thread(crx, &mut rclone, hookrx)?), hook: hooktx }
                )},
                Err(_x) => {
                    Err(Box::leak(format!("failed to open serial port {}", name).into_boxed_str()))
                }
            }
        }

        pub fn reset(&mut self, path: String, gain: f32, freq: u32) -> Result<(), &'static str> {
            let (hooktx, hookrx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();

            match self.handle.take() {
                Some(t) => {t.stop(); Ok(())},
                None => { Err("failed to stop worker thread, did you initialize??") }
            }?;

            self.decoder.reset(path, gain, freq)?;

            self.hook = hooktx;
            self.handle = Some(Uart::get_output_thread(self.decoder.child.clone(), &mut self.port, hookrx)?);

            Ok(())
        }

        pub fn feed(&mut self) -> Result<(), &'static str>  {
            match self.hook.send(true){
                Err(_) => Err("failed to activate hook"),
                _ => Ok(())
            }
        }

        pub fn temp(&mut self) -> Result<(), &'static str> {
            match self.hook.send(false){
                Err(_) => Err("failed to activate hook"),
                _ => Ok(())
            }
        }

        pub fn reciever(&mut self, path: String, gain: f32, freq: u32) {
            let mut lgain = gain;
            let mut lfreq = freq;
            loop {
                let mut pbuff: [u8; crate::adsb::BUFFER_SIZE] = [0; crate::adsb::BUFFER_SIZE];
                match self.port.read(&mut pbuff) {
                    Ok(len) => {
                        match pbuff[0] {
                            85 /* U */ => {
                                let bufferu: String = String::from_utf8_lossy(&pbuff[..len-1]).to_string();
                                let command = bufferu.split(' ').collect::<Vec<&str>>();

                                if command.len() < 3 {
                                    error!("arguments not supplied");
                                    continue;
                                }

                                lgain = match command[1].parse::<f32>() {
                                    Ok(x) => x,
                                    Err(_y) => {
                                        error!("invalid float received");
                                        continue;
                                    }
                                };
                                lfreq = match command[2].parse::<u32>() {
                                    Ok(x) => x,
                                    Err(y) => {
                                        error!("invalid int received {} {}", y, command[2]);
                                        continue;
                                    }
                                }; info!("setting gain to: {} and frequency to: {}", gain, freq);
                                self.reset(path.clone(), lgain, lfreq)
                            },
                            84 /* T */ => {
                                self.temp()
                            },
                            82 /* R */ => { info!("resetting the adsb decoder"); self.reset(path.clone(), lgain, lfreq) },
                            70 /* F */ => { info!("feeding new frame"); self.feed() },
                            83 /* S */ => {
                                //reset usb
                                Ok(())
                            },
                            _ => {
                                error!("unrecognized command");
                                Ok(())
                            }
                        }.unwrap();
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => error!("serial port recive failed {:?}", e)
                }
            }
        }

        fn get_output_thread(crx: spmc::Receiver<Vec<u8>>, port: &mut boxed::Box<dyn serialport::SerialPort>, hook: mpsc::Receiver<bool>) -> Result<stoppable_thread::StoppableHandle<()>, &'static str>{
            let mut rclone: boxed::Box<dyn serialport::SerialPort> = match port.try_clone(){
                Err(_x) => {
                    Err("failed to clone serial port")
                },
                Ok(x) => Ok(x)
            }?;

            Ok(stoppable_thread::spawn(move |stop| {
                let mut alt_buffer: [u8; SEND_BUFFER_SIZE] = [0; SEND_BUFFER_SIZE];
                while !stop.get() {

                    match hook.recv() {

                        Ok(x) => {
                            let mut send_buffer: [u8; SEND_BUFFER_SIZE] = [0; SEND_BUFFER_SIZE];

                            if !x {
                                let mut stringcpu: String = format!("");
                                match io::BufReader::new(match File::open("/sys/class/thermal/thermal_zone0/temp") { Ok(x) => x, _ => continue }).read_line(&mut stringcpu) {
                                    Err(_) => continue,
                                    _ => {}
                                };

                                let cpu_temp = (match stringcpu.parse::<f32>() {
                                    Ok(x) => x,
                                    _ => continue
                                } / 1000.).to_le_bytes();

                                let scpu_temp = stringcpu.as_bytes();

                                send_buffer[0] = 'T' as u8;
                                send_buffer[1] = cpu_temp[0];
                                send_buffer[2] = cpu_temp[1];
                                send_buffer[3] = cpu_temp[2];
                                send_buffer[4] = cpu_temp[3];
                                send_buffer[5] = scpu_temp[0];
                                send_buffer[6] = scpu_temp[1];

                                match rclone.write( &send_buffer ) {
                                    Ok(_x) => {},
                                    Err(_x) => error!("{}", _x)
                                };

                                continue;
                            }

                            match crx.try_recv() {
                                Ok(x) => {
                                    for i in 0..x.len() {
                                        send_buffer[i] = x[i];
                                    }

                                    info!("read: {:?}", &send_buffer);

                                    if (send_buffer[0] == ('^' as u8)) && (send_buffer[2] == ('I' as u8)) {
                                        alt_buffer[0] = '^' as u8;
                                        alt_buffer[1] = 'N' as u8;
                                        alt_buffer[2] = 'D' as u8;
                                    } else if send_buffer[0] == ('^' as u8) {
                                        for i in 0..3 {
                                            alt_buffer[i] = send_buffer[i];
                                        }
                                    }

                                    match rclone.write( &send_buffer ) {
                                        Ok(_x) => {},
                                        Err(_x) => error!("{}", _x)
                                    };
                                },
                                Err(_) => {
                                    for i in 0..3 {
                                        send_buffer[i] = alt_buffer[i];
                                    }


                                    match rclone.write( &send_buffer ) {
                                        Ok(_x) => {},
                                        Err(_x) => error!("{}", _x)
                                    };
                                }
                            };
                        },
                        _ => {}

                    }}}))
                }
            }
