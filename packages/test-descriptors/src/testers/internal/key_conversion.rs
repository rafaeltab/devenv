use crate::testers::keys::Key;

/// Key to bytes/tmux format conversion utilities.
#[derive(Debug)]
pub(crate) struct KeyConversion;

impl KeyConversion {
    /// Convert a Key to bytes for direct PTY input.
    pub(crate) fn key_to_bytes(_key: Key) -> Vec<u8> {
        todo!("Phase 3+: Implement KeyConversion::key_to_bytes")
    }

    /// Convert a Key to tmux send-keys format.
    pub(crate) fn key_to_tmux_format(_key: Key) -> String {
        todo!("Phase 3+: Implement KeyConversion::key_to_tmux_format")
    }
}
