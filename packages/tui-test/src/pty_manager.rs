use crate::TuiError;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PtyManager {
    _pty_pair: portable_pty::PtyPair,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    _reader_thread: Option<thread::JoinHandle<()>>,
}

impl PtyManager {
    pub fn spawn(
        command: &str,
        args: &[String],
        envs: &HashMap<String, String>,
        rows: u16,
        cols: u16,
    ) -> Result<Self, TuiError> {
        let pty_system = native_pty_system();

        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TuiError::PtySpawn(e.to_string()))?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);

        for (key, value) in envs {
            cmd.env(key, value);
        }

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| TuiError::PtySpawn(e.to_string()))?;

        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| TuiError::PtySpawn(e.to_string()))?;

        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| TuiError::PtySpawn(e.to_string()))?;

        // Start a background thread to read from PTY into a buffer
        let read_buffer = Arc::new(Mutex::new(Vec::new()));
        let read_buffer_clone = Arc::clone(&read_buffer);

        let reader_thread = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if let Ok(mut buf) = read_buffer_clone.lock() {
                            buf.extend_from_slice(&buffer[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            _pty_pair: pty_pair,
            child,
            writer,
            read_buffer,
            _reader_thread: Some(reader_thread),
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), TuiError> {
        self.writer.write_all(data).map_err(TuiError::PtyWrite)?;
        self.writer.flush().map_err(TuiError::PtyWrite)?;
        Ok(())
    }

    pub fn read_available(&mut self) -> Result<Vec<u8>, TuiError> {
        // Read from the buffer that the background thread is filling
        let mut buf = self.read_buffer.lock().map_err(|_| {
            TuiError::PtyRead(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to lock read buffer",
            ))
        })?;

        let data = buf.clone();
        buf.clear();
        Ok(data)
    }

    pub fn check_exit(&mut self) -> Option<i32> {
        match self.child.try_wait() {
            Ok(Some(status)) => Some(status.exit_code() as i32),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    pub fn wait_exit(&mut self) -> Option<i32> {
        match self.child.wait() {
            Ok(status) => Some(status.exit_code() as i32),
            Err(_) => None,
        }
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        // Try to kill the child process if it's still running
        let _ = self.child.kill();
    }
}
