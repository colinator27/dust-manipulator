use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use defer_rs::defer;

use crate::config::Config;

const MESSAGE_HEADER_SIZE: usize = 24;

const MESSAGE_CODE_HELLO: u32 = 0x55541000;
const MESSAGE_CODE_HELLO_ACK: u32 = 0x55541001;
const MESSAGE_CODE_GOODBYE: u32 = 0x55542000;
const MESSAGE_CODE_SCREENSHOT: u32 = 0x55544000;
const MESSAGE_CODE_SCREENSHOT_START_DELAY: u32 = 0x55545000;
const MESSAGE_CODE_SCREENSHOT_MODE: u32 = 0x55546000;
const MESSAGE_CODE_HOTKEY_1: u32 = 0x55548000;
const MESSAGE_CODE_HOTKEY_2: u32 = 0x55548001;
const MESSAGE_CODE_HOTKEY_3: u32 = 0x55548002;
const MESSAGE_CODE_HOTKEY_4: u32 = 0x55548003;
const MESSAGE_CODE_HOTKEY_5: u32 = 0x55548004;

#[derive(Clone)]
pub struct ScreenshotData {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub data: Vec<u8>
}

impl ScreenshotData {
    // Note: only works for grayscale color comparisons right now.
    pub fn get_brightest_pixel(&self, x: u32, y: u32, size: u32) -> u32 {
        assert!(x + size <= self.width);
        assert!(y + size <= self.height);
        let mut brightest = 0xFF000000;
        for y in y..(y+size) {
            for x in x..(x+size) {
                let pos = ((y * self.stride) + (x * 4)) as usize;
                let color = u32::from_ne_bytes(self.data[pos..pos+4].try_into().unwrap());
                if color > brightest {
                    brightest = color;
                }
            }
        }
        return brightest;
    }
    fn is_same_as(&self, other: &ScreenshotData) -> bool {
        if self.width != other.width || self.height != other.height {
            return false;
        }
        let mut y = 0;
        while y < self.height {
            let mut x = 0;
            while x < self.width {
                let pos = ((y * self.stride) + (x * 4)) as usize;
                let other_pos = ((y * other.stride) + (x * 4)) as usize;
                let color = u32::from_ne_bytes(self.data[pos..pos+4].try_into().unwrap());
                let other_color = u32::from_ne_bytes(other.data[other_pos..other_pos+4].try_into().unwrap());
                if color != other_color {
                    return false;
                }
                x += 1;
            }
            y += 1;
        }
        true
    }
}

// Message headers received from the OBS plugin
struct MessageHeader {
	pub message_code: u32,
	pub screenshot_has_more: u32,
	pub screenshot_width: u32,
	pub screenshot_height: u32,
	pub screenshot_stride: u32,
	pub screenshot_bits_per_pixel: u32
}

// Messages to be routed to the OBS plugin
pub struct MessageToSend {
    message_code: u32,
    data: u32
} 
impl MessageToSend {
    pub fn new_screenshot_start_delay(delay_ms: u32) -> Self {
        MessageToSend {
            message_code: MESSAGE_CODE_SCREENSHOT_START_DELAY,
            data: delay_ms
        }
    }
    pub fn new_screenshot_mode(is_single_only: bool) -> Self {
        MessageToSend {
            message_code: MESSAGE_CODE_SCREENSHOT_MODE,
            data: if is_single_only { 1 } else { 0 }
        }
    }
}

pub fn run_server(config: &Config, end_signal: Arc<AtomicBool>, connected: Arc<AtomicBool>, screenshot_data: Arc<Mutex<Vec<ScreenshotData>>>, 
                  hotkey_sender: Sender<u32>, messages_to_send_receiver: Receiver<MessageToSend>) {
    println!("Server thread started");
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], config.server_port))).expect("Failed to bind server to port");
    listener.set_nonblocking(true).unwrap();
    'listener_loop: for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                defer!(connected.store(false, Ordering::Relaxed));
                
                println!("New connection: {}", stream.peer_addr().unwrap());

                stream.set_nonblocking(true).unwrap();

                let mut local_screenshot_data: Vec<ScreenshotData> = Vec::with_capacity(100);
                let mut data = [0 as u8; MESSAGE_HEADER_SIZE];
                let mut got_hello = false;
                while match stream.read(&mut data) {
                    Ok(size) => {
                        if end_signal.load(Ordering::Relaxed) {
                            stream.shutdown(Shutdown::Both).unwrap();
                            return;
                        }
                        if size == 0 {
                            println!("Connection closed");
                            continue 'listener_loop;
                        }
                        if size != MESSAGE_HEADER_SIZE {
                            println!("Received unexpected size ({})", size);
                            _ = stream.shutdown(Shutdown::Both);
                            continue 'listener_loop;
                        }

                        let header: MessageHeader = MessageHeader { 
                            message_code: u32::from_be_bytes(data[0..4].try_into().unwrap()), 
                            screenshot_has_more: u32::from_be_bytes(data[4..8].try_into().unwrap()),
                            screenshot_width: u32::from_be_bytes(data[8..12].try_into().unwrap()), 
                            screenshot_height: u32::from_be_bytes(data[12..16].try_into().unwrap()), 
                            screenshot_stride: u32::from_be_bytes(data[16..20].try_into().unwrap()), 
                            screenshot_bits_per_pixel: u32::from_be_bytes(data[20..24].try_into().unwrap()),
                        };

                        match header.message_code {
                            MESSAGE_CODE_HELLO => {
                                println!("Received hello message, sending acknowledgment back");
                                connected.store(true, Ordering::Relaxed);
                                got_hello = true;
                                if let Err(e) = stream.write(&u32::to_be_bytes(MESSAGE_CODE_HELLO_ACK)) {
                                    println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                                    println!("Error: {e}");
                                    _ = stream.shutdown(Shutdown::Both);
                                }
                            },
                            MESSAGE_CODE_GOODBYE => {
                                println!("Received goodbye message, disconnecting");
                                connected.store(false, Ordering::Relaxed);
                                _ = stream.shutdown(Shutdown::Both);
                            },
                            MESSAGE_CODE_HOTKEY_1 => {
                                _ = hotkey_sender.send(0);
                            },
                            MESSAGE_CODE_HOTKEY_2 => {
                                _ = hotkey_sender.send(1);
                            },
                            MESSAGE_CODE_HOTKEY_3 => {
                                _ = hotkey_sender.send(2);
                            },
                            MESSAGE_CODE_HOTKEY_4 => {
                                _ = hotkey_sender.send(3);
                            },
                            MESSAGE_CODE_HOTKEY_5 => {
                                _ = hotkey_sender.send(4);
                            },
                            MESSAGE_CODE_SCREENSHOT => {
                                if !got_hello {
                                    println!("Received unexpected message");
                                    stream.shutdown(Shutdown::Both).unwrap();
                                    continue 'listener_loop;
                                }

                                // Verify that this is an image format we can handle
                                if header.screenshot_bits_per_pixel != 32 ||
                                   header.screenshot_width != 640 ||
                                   header.screenshot_height != 480 {
                                    println!("Received unexpected screenshot params, ignoring");
                                    match stream.read_to_end(&mut Vec::new()) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                                            println!("Error: {e}");
                                            _ = stream.shutdown(Shutdown::Both);
                                        }
                                    }
                                    continue 'listener_loop;
                                }

                                // Read all of the screenshot data
                                let screenshot_length = (header.screenshot_stride * header.screenshot_height) as usize;
                                let mut recv_screenshot_data: Vec<u8> = vec![0; screenshot_length];
                                let mut recv_data_pos = 0;
                                let mut recv_data_remaining = screenshot_length;

                                while recv_data_remaining > 0 {
                                    match stream.read(&mut recv_screenshot_data[recv_data_pos..]) {
                                        Ok(size) => {
                                            if size == 0 {
                                                println!("Connection closed");
                                                continue 'listener_loop;
                                            }
                                            if size > recv_data_remaining {
                                                println!("Received unexpected extra screenshot data, ignoring (received {} instead of maximum {})", size, recv_data_remaining);
                                                continue 'listener_loop;
                                            }

                                            recv_data_pos += size;
                                            recv_data_remaining -= size;

                                            if recv_data_remaining == 0 {
                                                let new_screenshot_data = ScreenshotData { 
                                                    width: header.screenshot_width, 
                                                    height: header.screenshot_height, 
                                                    stride: header.screenshot_stride, 
                                                    data: recv_screenshot_data 
                                                };

                                                // If the screenshot data isn't a duplicate of the previous screenshot, add it to the vector
                                                if local_screenshot_data.len() == 0 || !new_screenshot_data.is_same_as(local_screenshot_data.last().unwrap()) {
                                                    local_screenshot_data.push(new_screenshot_data);
                                                }
    
                                                if header.screenshot_has_more == 0 {
                                                    // Flush local screenshots to main thread
                                                    screenshot_data.lock().unwrap().append(&mut local_screenshot_data);
                                                }
                                                break;
                                            }
                                        },
                                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                            if end_signal.load(Ordering::Relaxed) {
                                                _ = stream.shutdown(Shutdown::Both);
                                                return;
                                            }
                                            thread::sleep(Duration::from_millis(5));
                                        },
                                        Err(e) => {
                                            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                                            println!("Error: {e}");
                                            _ = stream.shutdown(Shutdown::Both);
                                        }
                                    }
                                    thread::sleep(Duration::from_millis(5));
                                }
                            },
                            _ => {
                                if !got_hello {
                                    println!("Received unexpected message");
                                    _ = stream.shutdown(Shutdown::Both);
                                    continue 'listener_loop;
                                }
                                println!("Unknown message code");
                            }
                        }

                        true
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Shut down server if the signal was received
                        if end_signal.load(Ordering::Relaxed) {
                            _ = stream.shutdown(Shutdown::Both);
                            return;
                        }

                        // Send any pending messages
                        if got_hello {
                            for message_to_send in messages_to_send_receiver.try_iter() {
                                if let Err(e) = stream.write(&[u32::to_be_bytes(message_to_send.message_code), u32::to_be_bytes(message_to_send.data)].concat()) {
                                    println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                                    println!("Error: {e}");
                                    _ = stream.shutdown(Shutdown::Both);
                                }
                            }
                        }

                        // Nothing to do; sleep for a bit to not waste CPU
                        thread::sleep(Duration::from_millis(10));
                        true
                    },
                    Err(e) => {
                        println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                        println!("Error: {e}");
                        _ = stream.shutdown(Shutdown::Both);
                        false
                    }
                } {}
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Shut down server if the signal was received
                if end_signal.load(Ordering::Relaxed) {
                    return;
                }

                // Nothing to do; sleep for a bit to not waste CPU
                thread::sleep(Duration::from_millis(10));
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}