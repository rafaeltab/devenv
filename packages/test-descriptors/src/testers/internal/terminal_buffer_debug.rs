//! Debug wrapper for terminal buffer

use super::terminal_buffer::TerminalBuffer;

/// Debug wrapper that logs all bytes processed
pub(crate) struct DebugTerminalBuffer {
    inner: TerminalBuffer,
}

impl DebugTerminalBuffer {
    pub(crate) fn new(rows: u16, cols: u16) -> Self {
        Self {
            inner: TerminalBuffer::new(rows, cols),
        }
    }

    pub(crate) fn process_bytes(&mut self, bytes: &[u8]) {
        eprintln!("Processing {} bytes", bytes.len());
        // Print first 200 bytes as hex
        for (i, byte) in bytes.iter().take(200).enumerate() {
            if i % 32 == 0 {
                eprint!("\n{:04x}: ", i);
            }
            eprint!("{:02x} ", byte);
        }
        eprintln!();
        self.inner.process_bytes(bytes);
    }

    pub(crate) fn render(&self) -> String {
        self.inner.render()
    }
}
