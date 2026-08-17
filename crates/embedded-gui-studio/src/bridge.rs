//! Live Hardware Bridge server for hot-reloading real silicon over TCP / Serial-CDC.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct HardwareBridge {
    pub is_running: bool,
    pub listen_addr: String,
    pub client_count: Arc<Mutex<usize>>,
    pub active_streams: Arc<Mutex<Vec<TcpStream>>>,
}

impl HardwareBridge {
    pub fn new(port: u16) -> Self {
        Self {
            is_running: false,
            listen_addr: format!("127.0.0.1:{}", port),
            client_count: Arc::new(Mutex::new(0)),
            active_streams: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Ok(());
        }

        let listener = TcpListener::bind(&self.listen_addr)
            .map_err(|e| format!("Failed to bind {}: {}", self.listen_addr, e))?;
        listener.set_nonblocking(true).ok();

        let client_count = Arc::clone(&self.client_count);
        let active_streams = Arc::clone(&self.active_streams);

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(s) = stream {
                    s.set_nonblocking(true).ok();
                    if let Ok(mut streams) = active_streams.lock() {
                        streams.push(s);
                        if let Ok(mut count) = client_count.lock() {
                            *count = streams.len();
                        }
                    }
                }
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        self.is_running = true;
        Ok(())
    }

    /// Broadcasts live KDL screen payload to all connected physical hardware devices.
    pub fn broadcast_kdl(&self, kdl_payload: &str) -> usize {
        if !self.is_running {
            return 0;
        }

        let message = format!("SYNC_KDL:{}\n---END_KDL---\n", kdl_payload);
        let mut streams = match self.active_streams.lock() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut sent = 0;
        streams.retain_mut(|stream| {
            if stream.write_all(message.as_bytes()).is_ok() && stream.flush().is_ok() {
                sent += 1;
                true
            } else {
                false
            }
        });

        if let Ok(mut count) = self.client_count.lock() {
            *count = streams.len();
        }

        sent
    }
}
