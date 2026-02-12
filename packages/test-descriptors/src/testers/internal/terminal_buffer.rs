//! TerminalBuffer implementation using wezterm-term
//!
//! This is a drop-in replacement for the custom TerminalBuffer,
//! using the battle-tested wezterm-term library.

use std::io::{self, Write};
use std::sync::Arc;
use wezterm_term::color::{ColorAttribute, ColorPalette};
use wezterm_term::{Terminal as WezTerminal, TerminalConfiguration, TerminalSize};

/// Simple configuration implementation
#[derive(Debug)]
struct SimpleConfig {
    generation: usize,
}

impl SimpleConfig {
    fn new() -> Self {
        Self { generation: 0 }
    }
}

impl TerminalConfiguration for SimpleConfig {
    fn generation(&self) -> usize {
        self.generation
    }

    fn scrollback_size(&self) -> usize {
        0 // No scrollback to match current behavior
    }

    fn color_palette(&self) -> ColorPalette {
        ColorPalette::default()
    }
}

/// Represents a single cell in the terminal grid.
/// This matches the existing Cell struct for API compatibility.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cell {
    pub character: char,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            fg_r: 255,
            fg_g: 255,
            fg_b: 255,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
        }
    }
}

/// Dummy writer that discards all data (we don't need to write to PTY in tests)
struct DummyWriter;

impl Write for DummyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Terminal buffer using wezterm-term for VTE processing
pub(crate) struct TerminalBuffer {
    terminal: WezTerminal,
    rows: u16,
    cols: u16,
}

impl Clone for TerminalBuffer {
    fn clone(&self) -> Self {
        // Create a new terminal with same dimensions
        let size = TerminalSize {
            rows: self.rows as usize,
            cols: self.cols as usize,
            pixel_width: 0,
            pixel_height: 0,
            dpi: 96,
        };
        let config: Arc<dyn TerminalConfiguration + Send + Sync> = Arc::new(SimpleConfig::new());
        let new_terminal = WezTerminal::new(
            size,
            config,
            "test-terminal",
            "0.1.0",
            Box::new(DummyWriter),
        );

        Self {
            terminal: new_terminal,
            rows: self.rows,
            cols: self.cols,
        }
    }
}

impl PartialEq for TerminalBuffer {
    fn eq(&self, other: &Self) -> bool {
        // Compare by rendering both terminals
        self.render() == other.render()
    }
}

impl std::fmt::Debug for TerminalBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalBuffer")
            .field("rows", &self.rows)
            .field("cols", &self.cols)
            .field("render", &self.render())
            .finish()
    }
}

impl TerminalBuffer {
    pub(crate) fn new(rows: u16, cols: u16) -> Self {
        let size = TerminalSize {
            rows: rows as usize,
            cols: cols as usize,
            pixel_width: 0,
            pixel_height: 0,
            dpi: 96,
        };

        let config: Arc<dyn TerminalConfiguration + Send + Sync> = Arc::new(SimpleConfig::new());

        let terminal = WezTerminal::new(
            size,
            config,
            "test-terminal",
            "0.1.0",
            Box::new(DummyWriter),
        );

        Self {
            terminal,
            rows,
            cols,
        }
    }

    pub(crate) fn process_bytes(&mut self, bytes: &[u8]) {
        let canonicalized = Self::canonicalize_bytes(bytes);
        self.terminal.advance_bytes(canonicalized);
    }

    fn canonicalize_bytes(bytes: &[u8]) -> Box<[u8]> {
        let mut res = Vec::<u8>::with_capacity(bytes.len());

        let mut iter = bytes.iter().peekable();

        while let Some(&byte) = iter.next() {
            if byte == b'\r' {
                res.push(byte);
                if let Some(next_char) = iter.peek()
                    && **next_char == b'\n'
                {
                    res.push(b'\n');
                    iter.next();
                }
                continue;
            }
            if byte == b'\n' {
                // This one was not preceded by a \r so we must add it
                res.push(b'\r');
                res.push(byte);
                continue;
            }

            res.push(byte);
        }

        res.into_boxed_slice()
    }

    pub(crate) fn render(&self) -> String {
        let screen = self.terminal.screen();
        let mut lines = Vec::new();

        // Get visible lines - screen stores lines in VecDeque
        // Last N lines are the visible ones
        let visible_rows = self.rows as usize;
        let all_lines = screen.lines_in_phys_range(0..screen.scrollback_rows());

        // Get the last visible_rows lines
        let start_idx = all_lines.len().saturating_sub(visible_rows);
        let visible_lines = &all_lines[start_idx..];

        for line in visible_lines {
            let text: String = line
                .visible_cells()
                .map(|cell| {
                    // Handle multi-char cells (wide characters)
                    cell.str().chars().next().unwrap_or(' ')
                })
                .collect();
            lines.push(text.trim_end().to_string());
        }

        lines.join("\n")
    }

    pub(crate) fn screen_content(&self) -> String {
        self.render()
    }

    pub(crate) fn find_all_text(&self, text: &str) -> Vec<(u16, u16)> {
        let mut positions = Vec::new();
        let screen = self.terminal.screen();

        let visible_rows = self.rows as usize;
        let all_lines = screen.lines_in_phys_range(0..screen.scrollback_rows());
        let start_idx = all_lines.len().saturating_sub(visible_rows);
        let visible_lines = &all_lines[start_idx..];

        for (row, line) in visible_lines.iter().enumerate() {
            let cells: Vec<_> = line.visible_cells().collect();
            let mut line_text = String::new();
            let mut byte_to_cell_idx = Vec::new(); // Map each byte position to cell index
            
            for (col, cell) in cells.iter().enumerate() {
                let cell_str = cell.str();
                for _char in cell_str.chars() {
                    line_text.push(_char);
                    // For each byte in the UTF-8 encoding of this char, record the cell index
                    let char_bytes = _char.len_utf8();
                    for _ in 0..char_bytes {
                        byte_to_cell_idx.push(col as u16);
                    }
                }
            }

            // Search for all occurrences in this line
            // NOTE: find() returns BYTE position, not character position
            let mut start_byte = 0;
            while let Some(pos) = line_text[start_byte..].find(text) {
                let byte_pos = start_byte + pos;
                // Convert byte position to cell index
                let cell_idx = byte_to_cell_idx[byte_pos] as usize;
                positions.push((row as u16, cell_idx as u16));
                start_byte = byte_pos + text.len(); // Skip past this occurrence (in bytes)
            }
        }

        positions
    }

    pub(crate) fn get_cell(&self, row: u16, col: u16) -> Option<Cell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }

        let screen = self.terminal.screen();
        let visible_rows = self.rows as usize;
        let all_lines = screen.lines_in_phys_range(0..screen.scrollback_rows());
        let start_idx = all_lines.len().saturating_sub(visible_rows);

        let line = all_lines.get(start_idx + row as usize)?;

        // Get cell at column
        let cells: Vec<_> = line.visible_cells().collect();
        if col as usize >= cells.len() {
            return None;
        }

        let wez_cell = &cells[col as usize];

        // Convert wezterm cell to our Cell type
        let character = wez_cell.str().chars().next().unwrap_or(' ');

        // Convert colors from wezterm's format
        let (fg_r, fg_g, fg_b) = convert_color(wez_cell.attrs().foreground());
        let (bg_r, bg_g, bg_b) = convert_color(wez_cell.attrs().background());

        Some(Cell {
            character,
            fg_r,
            fg_g,
            fg_b,
            bg_r,
            bg_g,
            bg_b,
        })
    }

    pub(crate) fn clear(&mut self) {
        // Reset the terminal to initial state
        let size = TerminalSize {
            rows: self.rows as usize,
            cols: self.cols as usize,
            pixel_width: 0,
            pixel_height: 0,
            dpi: 96,
        };
        let config: Arc<dyn TerminalConfiguration + Send + Sync> = Arc::new(SimpleConfig::new());
        self.terminal = WezTerminal::new(
            size,
            config,
            "test-terminal",
            "0.1.0",
            Box::new(DummyWriter),
        );
    }
}

/// Convert wezterm color to RGB
fn convert_color(color: ColorAttribute) -> (u8, u8, u8) {
    match color {
        ColorAttribute::TrueColorWithPaletteFallback(rgb, _) => {
            // rgb is SrgbaTuple(f32, f32, f32, f32) - convert to u8
            (
                (rgb.0 * 255.0) as u8,
                (rgb.1 * 255.0) as u8,
                (rgb.2 * 255.0) as u8,
            )
        }
        ColorAttribute::TrueColorWithDefaultFallback(rgb) => {
            // rgb is SrgbaTuple(f32, f32, f32, f32) - convert to u8
            (
                (rgb.0 * 255.0) as u8,
                (rgb.1 * 255.0) as u8,
                (rgb.2 * 255.0) as u8,
            )
        }
        ColorAttribute::PaletteIndex(idx) => ansi_to_rgb(idx),
        ColorAttribute::Default => (255, 255, 255), // Default to white
    }
}

/// Convert ANSI 256 color index to RGB
fn ansi_to_rgb(idx: u8) -> (u8, u8, u8) {
    match idx {
        0..=7 => {
            // Standard colors
            let intensity = if idx == 0 { 0 } else { 128 };
            match idx {
                0 => (0, 0, 0),
                1 => (intensity, 0, 0),
                2 => (0, intensity, 0),
                3 => (intensity, intensity, 0),
                4 => (0, 0, intensity),
                5 => (intensity, 0, intensity),
                6 => (0, intensity, intensity),
                7 => (intensity, intensity, intensity),
                _ => unreachable!(),
            }
        }
        8..=15 => {
            // High intensity colors
            let val = 255;
            match idx {
                8 => (64, 64, 64),
                9 => (val, 64, 64),
                10 => (64, val, 64),
                11 => (val, val, 64),
                12 => (64, 64, val),
                13 => (val, 64, val),
                14 => (64, val, val),
                15 => (val, val, val),
                _ => unreachable!(),
            }
        }
        16..=231 => {
            // 6x6x6 color cube
            let idx = idx - 16;
            let r = idx / 36;
            let g = (idx % 36) / 6;
            let b = idx % 6;
            (
                if r == 0 { 0 } else { r * 40 + 55 },
                if g == 0 { 0 } else { g * 40 + 55 },
                if b == 0 { 0 } else { b * 40 + 55 },
            )
        }
        232..=255 => {
            // Grayscale
            let gray = (idx - 232) * 10 + 8;
            (gray, gray, gray)
        }
    }
}
