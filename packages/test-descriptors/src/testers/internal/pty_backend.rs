use portable_pty::{MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Error type for PtyBackend operations.
#[derive(Debug)]
pub enum PtyError {
    Io(std::io::Error),
    Lock(String),
    Pty(String),
}

impl std::fmt::Display for PtyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtyError::Io(e) => write!(f, "IO error: {}", e),
            PtyError::Lock(e) => write!(f, "Lock error: {}", e),
            PtyError::Pty(e) => write!(f, "PTY error: {}", e),
        }
    }
}

impl std::error::Error for PtyError {}

impl From<std::io::Error> for PtyError {
    fn from(e: std::io::Error) -> Self {
        PtyError::Io(e)
    }
}

/// Shared PTY reading/writing logic.
///
/// This provides a background reader thread and synchronous access
/// to PTY output for terminal emulation.
pub(crate) struct PtyBackend {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    _reader_thread: JoinHandle<()>,
    running: Arc<Mutex<bool>>,
}

impl PtyBackend {
    pub(crate) fn new(master: Box<dyn MasterPty + Send>) -> Result<Self, PtyError> {
        let read_buffer = Arc::new(Mutex::new(Vec::new()));
        let read_buffer_clone = Arc::clone(&read_buffer);
        let running = Arc::new(Mutex::new(true));
        let running_clone = Arc::clone(&running);

        // Get a reader from the master PTY
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| PtyError::Pty(e.to_string()))?;

        // Take the writer upfront - this can only be called once
        let writer = master
            .take_writer()
            .map_err(|e| PtyError::Pty(e.to_string()))?;

        let reader_thread = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                // Check if we should stop
                if let Ok(running) = running_clone.lock() {
                    if !*running {
                        break;
                    }
                }

                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut buf) = read_buffer_clone.lock() {
                            buf.extend_from_slice(&buffer[..n]);
                        }
                    }
                    Err(e) => {
                        // Check for non-blocking read errors
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self {
            master,
            writer,
            read_buffer,
            _reader_thread: reader_thread,
            running,
        })
    }

    /// Read available bytes from the PTY output buffer.
    pub(crate) fn read_available(&self) -> Option<Vec<u8>> {
        if let Ok(mut buf) = self.read_buffer.lock() {
            if !buf.is_empty() {
                let bytes = buf.clone();
                buf.clear();
                return Some(bytes);
            }
        }
        None
    }

    /// Write bytes to the PTY input.
    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), PtyError> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Get the PTY size.
    #[allow(dead_code)]
    pub(crate) fn get_size(&self) -> Result<PtySize, PtyError> {
        self.master
            .get_size()
            .map_err(|e| PtyError::Pty(e.to_string()))
    }

    /// Resize the PTY.
    #[allow(dead_code)]
    pub(crate) fn resize(&self, size: PtySize) -> Result<(), PtyError> {
        self.master
            .resize(size)
            .map_err(|e| PtyError::Pty(e.to_string()))
    }
}

impl Drop for PtyBackend {
    fn drop(&mut self) {
        // Signal the reader thread to stop
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
    }
}

impl std::fmt::Debug for PtyBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyBackend")
            .field(
                "read_buffer_len",
                &self.read_buffer.lock().map(|b| b.len()).unwrap_or(0),
            )
            .finish()
    }
}
