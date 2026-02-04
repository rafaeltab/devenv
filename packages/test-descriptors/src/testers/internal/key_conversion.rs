use crate::testers::keys::{Key, Modifier};

/// Key to bytes/tmux format conversion utilities.
#[derive(Debug)]
pub(crate) struct KeyConversion;

impl KeyConversion {
    /// Convert a Key to bytes for direct PTY input.
    pub(crate) fn key_to_bytes(key: Key) -> Vec<u8> {
        match key {
            Key::Char(c) => c.to_string().into_bytes(),
            Key::Enter => b"\r".to_vec(),
            Key::Esc => b"\x1b".to_vec(),
            Key::Tab => b"\t".to_vec(),
            Key::Backspace => b"\x7f".to_vec(),
            Key::Up => b"\x1b[A".to_vec(),
            Key::Down => b"\x1b[B".to_vec(),
            Key::Right => b"\x1b[C".to_vec(),
            Key::Left => b"\x1b[D".to_vec(),
            Key::Home => b"\x1b[H".to_vec(),
            Key::End => b"\x1b[F".to_vec(),
            Key::PageUp => b"\x1b[5~".to_vec(),
            Key::PageDown => b"\x1b[6~".to_vec(),
            Key::Delete => b"\x1b[3~".to_vec(),
            Key::Insert => b"\x1b[2~".to_vec(),
            Key::F(n) => match n {
                1 => b"\x1bOP".to_vec(),
                2 => b"\x1bOQ".to_vec(),
                3 => b"\x1bOR".to_vec(),
                4 => b"\x1bOS".to_vec(),
                5 => b"\x1b[15~".to_vec(),
                6 => b"\x1b[17~".to_vec(),
                7 => b"\x1b[18~".to_vec(),
                8 => b"\x1b[19~".to_vec(),
                9 => b"\x1b[20~".to_vec(),
                10 => b"\x1b[21~".to_vec(),
                11 => b"\x1b[23~".to_vec(),
                12 => b"\x1b[24~".to_vec(),
                _ => vec![],
            },
            Key::Ctrl(c) => {
                // Ctrl+letter: letter - '@' (0x40) gives the control character
                let ctrl_char = (c.to_ascii_uppercase() as u8).saturating_sub(0x40);
                vec![ctrl_char]
            }
            Key::Alt(c) => {
                // Alt+key: ESC followed by the key
                let mut bytes = vec![0x1b];
                bytes.extend(c.to_string().into_bytes());
                bytes
            }
            Key::Shift(c) => {
                // Shift just produces the uppercase character
                c.to_uppercase().to_string().into_bytes()
            }
            Key::Modified { key, modifier } => Self::build_modified_key(*key, modifier),
        }
    }

    fn build_modified_key(key: Key, modifier: Modifier) -> Vec<u8> {
        match modifier {
            Modifier::Ctrl => match key {
                Key::Char(c) => {
                    let ctrl_char = (c.to_ascii_uppercase() as u8).saturating_sub(0x40);
                    vec![ctrl_char]
                }
                _ => Self::key_to_bytes(key),
            },
            Modifier::Alt => {
                let mut bytes = vec![0x1b];
                bytes.extend(Self::key_to_bytes(key));
                bytes
            }
            Modifier::Shift => match key {
                Key::Char(c) => c.to_uppercase().to_string().into_bytes(),
                _ => Self::key_to_bytes(key),
            },
            Modifier::Super => {
                // Super/Meta key - typically same as Alt in terminals
                let mut bytes = vec![0x1b];
                bytes.extend(Self::key_to_bytes(key));
                bytes
            }
        }
    }

    /// Convert a Key to tmux send-keys format.
    pub(crate) fn key_to_tmux_format(key: Key) -> String {
        match key {
            Key::Char(c) => {
                // Escape special characters for tmux
                match c {
                    ';' => "\\;".to_string(),
                    '"' => "\\\"".to_string(),
                    '\'' => "\\'".to_string(),
                    '\\' => "\\\\".to_string(),
                    '$' => "\\$".to_string(),
                    '`' => "\\`".to_string(),
                    _ => c.to_string(),
                }
            }
            Key::Enter => "Enter".to_string(),
            Key::Esc => "Escape".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Backspace => "BSpace".to_string(),
            Key::Up => "Up".to_string(),
            Key::Down => "Down".to_string(),
            Key::Right => "Right".to_string(),
            Key::Left => "Left".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::Delete => "DC".to_string(),
            Key::Insert => "IC".to_string(),
            Key::F(n) => format!("F{}", n),
            Key::Ctrl(c) => format!("C-{}", c),
            Key::Alt(c) => format!("M-{}", c),
            Key::Shift(c) => c.to_uppercase().to_string(),
            Key::Modified { key, modifier } => {
                let prefix = match modifier {
                    Modifier::Ctrl => "C-",
                    Modifier::Alt => "M-",
                    Modifier::Shift => "S-",
                    Modifier::Super => "M-", // tmux uses M- for Meta/Super
                };
                format!("{}{}", prefix, Self::key_to_tmux_format(*key))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_to_bytes() {
        assert_eq!(KeyConversion::key_to_bytes(Key::Char('a')), b"a".to_vec());
        assert_eq!(KeyConversion::key_to_bytes(Key::Char('Z')), b"Z".to_vec());
    }

    #[test]
    fn test_special_keys_to_bytes() {
        assert_eq!(KeyConversion::key_to_bytes(Key::Enter), b"\r".to_vec());
        assert_eq!(KeyConversion::key_to_bytes(Key::Esc), b"\x1b".to_vec());
        assert_eq!(KeyConversion::key_to_bytes(Key::Tab), b"\t".to_vec());
    }

    #[test]
    fn test_arrow_keys_to_bytes() {
        assert_eq!(KeyConversion::key_to_bytes(Key::Up), b"\x1b[A".to_vec());
        assert_eq!(KeyConversion::key_to_bytes(Key::Down), b"\x1b[B".to_vec());
        assert_eq!(KeyConversion::key_to_bytes(Key::Right), b"\x1b[C".to_vec());
        assert_eq!(KeyConversion::key_to_bytes(Key::Left), b"\x1b[D".to_vec());
    }

    #[test]
    fn test_ctrl_to_bytes() {
        // Ctrl+C = 0x03
        assert_eq!(KeyConversion::key_to_bytes(Key::Ctrl('c')), vec![0x03]);
        // Ctrl+A = 0x01
        assert_eq!(KeyConversion::key_to_bytes(Key::Ctrl('a')), vec![0x01]);
    }

    #[test]
    fn test_key_to_tmux_format() {
        assert_eq!(KeyConversion::key_to_tmux_format(Key::Enter), "Enter");
        assert_eq!(KeyConversion::key_to_tmux_format(Key::Esc), "Escape");
        assert_eq!(KeyConversion::key_to_tmux_format(Key::Ctrl('c')), "C-c");
        assert_eq!(KeyConversion::key_to_tmux_format(Key::Alt('x')), "M-x");
        assert_eq!(KeyConversion::key_to_tmux_format(Key::F(1)), "F1");
    }
}
