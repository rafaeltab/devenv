use vte::{Params, Parser, Perform};

/// Represents a single cell in the terminal grid.
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

/// Terminal buffer for storing and querying terminal output.
/// Uses VTE parser to process ANSI escape sequences.
#[derive(Debug, Clone)]
pub(crate) struct TerminalBuffer {
    rows: u16,
    cols: u16,
    grid: Vec<Vec<Cell>>,
    cursor_row: u16,
    cursor_col: u16,
    // Current attributes for new characters
    current_fg: (u8, u8, u8),
    current_bg: (u8, u8, u8),
    // Saved cursor position
    saved_cursor: Option<(u16, u16)>,
    // Scroll region (top, bottom) - 0-indexed
    scroll_top: u16,
    scroll_bottom: u16,
}

impl TerminalBuffer {
    pub(crate) fn new(rows: u16, cols: u16) -> Self {
        let grid = (0..rows)
            .map(|_| (0..cols).map(|_| Cell::default()).collect())
            .collect();

        Self {
            rows,
            cols,
            grid,
            cursor_row: 0,
            cursor_col: 0,
            current_fg: (255, 255, 255),
            current_bg: (0, 0, 0),
            saved_cursor: None,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
        }
    }

    pub(crate) fn process_bytes(&mut self, bytes: &[u8]) {
        let mut parser = Parser::new();
        for byte in bytes {
            parser.advance(self, *byte);
        }
    }

    pub(crate) fn screen_content(&self) -> String {
        self.grid
            .iter()
            .map(|row| {
                let line: String = row.iter().map(|cell| cell.character).collect();
                line.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Find all occurrences of text in the terminal buffer.
    /// Returns positions as (row, col) tuples.
    pub(crate) fn find_all_text(&self, text: &str) -> Vec<(u16, u16)> {
        let mut positions = Vec::new();

        for (row_idx, row) in self.grid.iter().enumerate() {
            let line: String = row.iter().map(|cell| cell.character).collect();
            let mut start = 0;
            while let Some(pos) = line[start..].find(text) {
                let col = start + pos;
                positions.push((row_idx as u16, col as u16));
                start = col + 1;
            }
        }

        positions
    }

    /// Get a cell at the given position.
    pub(crate) fn get_cell(&self, row: u16, col: u16) -> Option<&Cell> {
        self.grid
            .get(row as usize)
            .and_then(|r| r.get(col as usize))
    }

    /// Render the terminal buffer as a string for debugging.
    pub(crate) fn render(&self) -> String {
        self.screen_content()
    }

    fn scroll_up(&mut self) {
        // Remove the top line of scroll region and add a new blank line at bottom
        if self.scroll_top < self.scroll_bottom && (self.scroll_bottom as usize) < self.grid.len() {
            self.grid.remove(self.scroll_top as usize);
            let new_row = (0..self.cols).map(|_| Cell::default()).collect();
            self.grid.insert(self.scroll_bottom as usize, new_row);
        }
    }

    fn newline(&mut self) {
        if self.cursor_row >= self.scroll_bottom {
            self.scroll_up();
        } else {
            self.cursor_row += 1;
        }
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            let cell = &mut self.grid[self.cursor_row as usize][self.cursor_col as usize];
            cell.character = c;
            cell.fg_r = self.current_fg.0;
            cell.fg_g = self.current_fg.1;
            cell.fg_b = self.current_fg.2;
            cell.bg_r = self.current_bg.0;
            cell.bg_g = self.current_bg.1;
            cell.bg_b = self.current_bg.2;
        }

        self.cursor_col += 1;
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.newline();
        }
    }

    fn parse_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        while let Some(param) = iter.next() {
            let code = param.first().copied().unwrap_or(0);
            match code {
                0 => {
                    // Reset
                    self.current_fg = (255, 255, 255);
                    self.current_bg = (0, 0, 0);
                }
                30..=37 => {
                    // Standard foreground colors
                    self.current_fg = Self::standard_color(code - 30);
                }
                38 => {
                    // Extended foreground color
                    if let Some(mode) = iter.next() {
                        match mode.first().copied().unwrap_or(0) {
                            2 => {
                                // RGB
                                let r =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let g =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let b =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                self.current_fg = (r, g, b);
                            }
                            5 => {
                                // 256 color
                                let idx =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                self.current_fg = Self::color_256(idx);
                            }
                            _ => {}
                        }
                    }
                }
                39 => {
                    // Default foreground
                    self.current_fg = (255, 255, 255);
                }
                40..=47 => {
                    // Standard background colors
                    self.current_bg = Self::standard_color(code - 40);
                }
                48 => {
                    // Extended background color
                    if let Some(mode) = iter.next() {
                        match mode.first().copied().unwrap_or(0) {
                            2 => {
                                // RGB
                                let r =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let g =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let b =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                self.current_bg = (r, g, b);
                            }
                            5 => {
                                // 256 color
                                let idx =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                self.current_bg = Self::color_256(idx);
                            }
                            _ => {}
                        }
                    }
                }
                49 => {
                    // Default background
                    self.current_bg = (0, 0, 0);
                }
                90..=97 => {
                    // Bright foreground colors
                    self.current_fg = Self::bright_color(code - 90);
                }
                100..=107 => {
                    // Bright background colors
                    self.current_bg = Self::bright_color(code - 100);
                }
                _ => {}
            }
        }
    }

    fn standard_color(idx: u16) -> (u8, u8, u8) {
        match idx {
            0 => (0, 0, 0),       // Black
            1 => (205, 0, 0),     // Red
            2 => (0, 205, 0),     // Green
            3 => (205, 205, 0),   // Yellow
            4 => (0, 0, 238),     // Blue
            5 => (205, 0, 205),   // Magenta
            6 => (0, 205, 205),   // Cyan
            7 => (229, 229, 229), // White
            _ => (255, 255, 255),
        }
    }

    fn bright_color(idx: u16) -> (u8, u8, u8) {
        match idx {
            0 => (127, 127, 127), // Bright Black (Gray)
            1 => (255, 0, 0),     // Bright Red
            2 => (0, 255, 0),     // Bright Green
            3 => (255, 255, 0),   // Bright Yellow
            4 => (92, 92, 255),   // Bright Blue
            5 => (255, 0, 255),   // Bright Magenta
            6 => (0, 255, 255),   // Bright Cyan
            7 => (255, 255, 255), // Bright White
            _ => (255, 255, 255),
        }
    }

    fn color_256(idx: u8) -> (u8, u8, u8) {
        match idx {
            0..=15 => {
                // Standard + bright colors
                if idx < 8 {
                    Self::standard_color(idx as u16)
                } else {
                    Self::bright_color((idx - 8) as u16)
                }
            }
            16..=231 => {
                // 216 color cube: 6x6x6
                let idx = idx - 16;
                let r = (idx / 36) % 6;
                let g = (idx / 6) % 6;
                let b = idx % 6;
                let to_rgb = |c: u8| if c == 0 { 0 } else { 55 + c * 40 };
                (to_rgb(r), to_rgb(g), to_rgb(b))
            }
            232..=255 => {
                // Grayscale: 24 shades
                let gray = 8 + (idx - 232) * 10;
                (gray, gray, gray)
            }
        }
    }
}

impl Perform for TerminalBuffer {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                // Backspace
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            0x09 => {
                // Tab - move to next tab stop (every 8 columns)
                self.cursor_col = ((self.cursor_col / 8) + 1) * 8;
                if self.cursor_col >= self.cols {
                    self.cursor_col = self.cols - 1;
                }
            }
            0x0A | 0x0B | 0x0C => {
                // LF, VT, FF - newline
                self.newline();
            }
            0x0D => {
                // CR
                self.carriage_return();
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let first_param = params.iter().next().and_then(|p| p.first().copied());
        let second_param = params.iter().nth(1).and_then(|p| p.first().copied());

        match action {
            'A' => {
                // Cursor Up
                let n = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                // Cursor Down
                let n = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
            }
            'C' => {
                // Cursor Forward
                let n = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
            }
            'D' => {
                // Cursor Back
                let n = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'E' => {
                // Cursor Next Line
                let n = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
                self.cursor_col = 0;
            }
            'F' => {
                // Cursor Previous Line
                let n = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_row = self.cursor_row.saturating_sub(n);
                self.cursor_col = 0;
            }
            'G' => {
                // Cursor Horizontal Absolute
                let col = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_col = (col - 1).min(self.cols - 1);
            }
            'H' | 'f' => {
                // Cursor Position
                let row = first_param.unwrap_or(1).max(1) as u16;
                let col = second_param.unwrap_or(1).max(1) as u16;
                self.cursor_row = (row - 1).min(self.rows - 1);
                self.cursor_col = (col - 1).min(self.cols - 1);
            }
            'J' => {
                // Erase in Display
                let mode = first_param.unwrap_or(0);
                match mode {
                    0 => {
                        // Clear from cursor to end of screen
                        for col in self.cursor_col..self.cols {
                            self.grid[self.cursor_row as usize][col as usize] = Cell::default();
                        }
                        for row in (self.cursor_row + 1)..self.rows {
                            for col in 0..self.cols {
                                self.grid[row as usize][col as usize] = Cell::default();
                            }
                        }
                    }
                    1 => {
                        // Clear from start to cursor
                        for row in 0..self.cursor_row {
                            for col in 0..self.cols {
                                self.grid[row as usize][col as usize] = Cell::default();
                            }
                        }
                        for col in 0..=self.cursor_col {
                            self.grid[self.cursor_row as usize][col as usize] = Cell::default();
                        }
                    }
                    2 | 3 => {
                        // Clear entire screen
                        for row in 0..self.rows {
                            for col in 0..self.cols {
                                self.grid[row as usize][col as usize] = Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                let mode = first_param.unwrap_or(0);
                match mode {
                    0 => {
                        // Clear from cursor to end of line
                        for col in self.cursor_col..self.cols {
                            self.grid[self.cursor_row as usize][col as usize] = Cell::default();
                        }
                    }
                    1 => {
                        // Clear from start to cursor
                        for col in 0..=self.cursor_col {
                            self.grid[self.cursor_row as usize][col as usize] = Cell::default();
                        }
                    }
                    2 => {
                        // Clear entire line
                        for col in 0..self.cols {
                            self.grid[self.cursor_row as usize][col as usize] = Cell::default();
                        }
                    }
                    _ => {}
                }
            }
            'L' => {
                // Insert Lines
                let n = first_param.unwrap_or(1).max(1) as usize;
                let row = self.cursor_row as usize;
                for _ in 0..n {
                    if row < self.grid.len() {
                        self.grid
                            .insert(row, (0..self.cols).map(|_| Cell::default()).collect());
                        if self.grid.len() > self.rows as usize {
                            self.grid.pop();
                        }
                    }
                }
            }
            'M' => {
                // Delete Lines
                let n = first_param.unwrap_or(1).max(1) as usize;
                let row = self.cursor_row as usize;
                for _ in 0..n {
                    if row < self.grid.len() {
                        self.grid.remove(row);
                        self.grid
                            .push((0..self.cols).map(|_| Cell::default()).collect());
                    }
                }
            }
            'P' => {
                // Delete Characters
                let n = first_param.unwrap_or(1).max(1) as usize;
                let row = &mut self.grid[self.cursor_row as usize];
                let col = self.cursor_col as usize;
                for _ in 0..n {
                    if col < row.len() {
                        row.remove(col);
                        row.push(Cell::default());
                    }
                }
            }
            '@' => {
                // Insert Characters
                let n = first_param.unwrap_or(1).max(1) as usize;
                let row = &mut self.grid[self.cursor_row as usize];
                let col = self.cursor_col as usize;
                for _ in 0..n {
                    if col < row.len() {
                        row.insert(col, Cell::default());
                        row.pop();
                    }
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition
                self.parse_sgr(params);
            }
            'r' => {
                // Set Scrolling Region
                let top = first_param.unwrap_or(1).max(1) as u16;
                let bottom = second_param.unwrap_or(self.rows as u16).max(1) as u16;
                self.scroll_top = (top - 1).min(self.rows - 1);
                self.scroll_bottom = (bottom - 1).min(self.rows - 1);
            }
            's' => {
                // Save Cursor Position
                self.saved_cursor = Some((self.cursor_row, self.cursor_col));
            }
            'u' => {
                // Restore Cursor Position
                if let Some((row, col)) = self.saved_cursor {
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
            }
            'd' => {
                // Cursor Vertical Absolute
                let row = first_param.unwrap_or(1).max(1) as u16;
                self.cursor_row = (row - 1).min(self.rows - 1);
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'7' => {
                // Save Cursor (DECSC)
                self.saved_cursor = Some((self.cursor_row, self.cursor_col));
            }
            b'8' => {
                // Restore Cursor (DECRC)
                if let Some((row, col)) = self.saved_cursor {
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
            }
            b'c' => {
                // Full Reset (RIS)
                *self = Self::new(self.rows, self.cols);
            }
            b'D' => {
                // Index (IND)
                self.newline();
            }
            b'E' => {
                // Next Line (NEL)
                self.carriage_return();
                self.newline();
            }
            b'M' => {
                // Reverse Index (RI)
                if self.cursor_row > self.scroll_top {
                    self.cursor_row -= 1;
                } else {
                    // Scroll down
                    if self.scroll_bottom > self.scroll_top {
                        self.grid.remove(self.scroll_bottom as usize);
                        self.grid.insert(
                            self.scroll_top as usize,
                            (0..self.cols).map(|_| Cell::default()).collect(),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
}

impl PartialEq for TerminalBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid
    }
}
