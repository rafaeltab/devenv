// Terminal buffer implementation using alacritty_terminal
use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte;

// Simple size info that implements Dimensions
struct TermSize {
    columns: usize,
    lines: usize,
}

impl TermSize {
    fn new(columns: usize, lines: usize) -> Self {
        Self { columns, lines }
    }
}

impl Dimensions for TermSize {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }

    fn last_column(&self) -> Column {
        Column(self.columns.saturating_sub(1))
    }

    fn bottommost_line(&self) -> Line {
        Line(self.lines as i32 - 1)
    }

    fn topmost_line(&self) -> Line {
        Line(0)
    }
}

pub struct TerminalBuffer {
    term: Term<VoidListener>,
    processor: vte::ansi::Processor,
    rows: u16,
    cols: u16,
}

impl TerminalBuffer {
    pub fn new(rows: u16, cols: u16) -> Self {
        let config = Config::default();
        let size = TermSize::new(cols as usize, rows as usize);

        let term = Term::new(config, &size, VoidListener);
        let processor = vte::ansi::Processor::new();

        Self {
            term,
            processor,
            rows,
            cols,
        }
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
    }

    pub fn find_text(&self, text: &str) -> Option<(u16, u16)> {
        let all = self.find_all_text(text);

        match all.len() {
            0 => None,
            1 => Some(all[0]),
            n => panic!(
                "find_text() found multiple occurrences ({}) of {:?}. Use find_all_text() if multiple matches are expected",
                n, text
            ),
        }
    }

    pub fn find_all_text(&self, text: &str) -> Vec<(u16, u16)> {
        let mut positions = Vec::new();

        for row_idx in 0..self.rows {
            let row_text = self.get_row_text(row_idx);

            let mut col = 0;
            while let Some(offset) = row_text[col..].find(text) {
                positions.push((row_idx, (col + offset) as u16));
                col += offset + 1;
            }
        }

        positions
    }

    fn get_row_text(&self, row: u16) -> String {
        let line = Line(row as i32);
        let mut text = String::new();

        for col_idx in 0..self.cols {
            let point = Point::new(line, Column(col_idx as usize));
            let cell = &self.term.grid()[point];
            text.push(cell.c);
        }

        text
    }

    pub fn render(&self) -> String {
        let mut lines = Vec::new();

        for row in 0..self.rows {
            lines.push(self.get_row_text(row));
        }

        lines.join("\n")
    }

    fn get_default_indexed_color(idx: u8) -> Option<(u8, u8, u8)> {
        // Standard 256-color palette
        match idx {
            // 16 standard colors (0-15)
            0 => Some((0, 0, 0)),       // Black
            1 => Some((205, 0, 0)),     // Red
            2 => Some((0, 205, 0)),     // Green
            3 => Some((205, 205, 0)),   // Yellow
            4 => Some((0, 0, 238)),     // Blue
            5 => Some((205, 0, 205)),   // Magenta
            6 => Some((0, 205, 205)),   // Cyan
            7 => Some((229, 229, 229)), // White

            8 => Some((127, 127, 127)),  // Bright Black
            9 => Some((255, 0, 0)),      // Bright Red
            10 => Some((0, 255, 0)),     // Bright Green
            11 => Some((255, 255, 0)),   // Bright Yellow
            12 => Some((92, 92, 255)),   // Bright Blue
            13 => Some((255, 0, 255)),   // Bright Magenta
            14 => Some((0, 255, 255)),   // Bright Cyan
            15 => Some((255, 255, 255)), // Bright White

            // 216 color cube (16-231)
            16..=231 => {
                let idx = idx - 16;
                let r = ((idx / 36) * 51) as u8;
                let g = (((idx % 36) / 6) * 51) as u8;
                let b = ((idx % 6) * 51) as u8;
                Some((r, g, b))
            }

            // 24 grayscale (232-255)
            232..=255 => {
                let gray = (8 + (idx - 232) * 10) as u8;
                Some((gray, gray, gray))
            }
        }
    }

    pub fn get_cell_color(&self, row: u16, col: u16) -> Option<CellColors> {
        if row >= self.rows || col >= self.cols {
            return None;
        }

        let point = Point::new(Line(row as i32), Column(col as usize));
        let cell = &self.term.grid()[point];

        // Get the color palette/config from term
        let colors = self.term.colors();

        // Extract foreground color
        let fg = match cell.fg {
            alacritty_terminal::vte::ansi::Color::Named(named) => match colors[named] {
                Some(rgb) => Some((rgb.r, rgb.g, rgb.b)),
                None => Self::get_default_indexed_color(named as u8),
            },
            alacritty_terminal::vte::ansi::Color::Spec(rgb) => Some((rgb.r, rgb.g, rgb.b)),
            alacritty_terminal::vte::ansi::Color::Indexed(idx) => match colors[idx as usize] {
                Some(rgb) => Some((rgb.r, rgb.g, rgb.b)),
                None => Self::get_default_indexed_color(idx),
            },
        };

        // Extract background color
        let bg = match cell.bg {
            alacritty_terminal::vte::ansi::Color::Named(named) => match colors[named] {
                Some(rgb) => Some((rgb.r, rgb.g, rgb.b)),
                None => Self::get_default_indexed_color(named as u8),
            },
            alacritty_terminal::vte::ansi::Color::Spec(rgb) => Some((rgb.r, rgb.g, rgb.b)),
            alacritty_terminal::vte::ansi::Color::Indexed(idx) => match colors[idx as usize] {
                Some(rgb) => Some((rgb.r, rgb.g, rgb.b)),
                None => Self::get_default_indexed_color(idx),
            },
        };

        // Handle reverse video flag
        let (final_fg, final_bg) = if cell.flags.contains(Flags::INVERSE) {
            (bg, fg)
        } else {
            (fg, bg)
        };

        Some(CellColors {
            fg: final_fg,
            bg: final_bg,
        })
    }
}

impl Clone for TerminalBuffer {
    fn clone(&self) -> Self {
        // Create a new terminal with the same content
        let mut new_buffer = Self::new(self.rows, self.cols);

        // Copy the screen content by rendering and re-processing
        // This is not perfect but works for our use case
        let content = self.render();
        new_buffer.process_bytes(content.as_bytes());

        new_buffer
    }
}

impl PartialEq for TerminalBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.render() == other.render()
    }
}

pub struct CellColors {
    pub fg: Option<(u8, u8, u8)>,
    pub bg: Option<(u8, u8, u8)>,
}
