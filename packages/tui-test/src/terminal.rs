// Simple terminal buffer implementation
// TODO: Integrate alacritty_terminal for proper VT100/ANSI parsing

pub struct TerminalBuffer {
    rows: u16,
    cols: u16,
    content: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug)]
struct Cell {
    c: char,
    fg: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
}

impl Cell {
    fn empty() -> Self {
        Self {
            c: ' ',
            fg: None,
            bg: None,
        }
    }
}

impl TerminalBuffer {
    pub fn new(rows: u16, cols: u16) -> Self {
        let content = vec![vec![Cell::empty(); cols as usize]; rows as usize];
        Self {
            rows,
            cols,
            content,
        }
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes);

        // Find the last non-empty row
        let mut row = 0;
        for (i, line) in self.content.iter().enumerate() {
            if line.iter().any(|c| c.c != ' ') {
                row = i + 1;
            }
        }
        if row >= self.rows as usize {
            row = 0;
        }

        let mut col = 0;
        let mut current_fg: Option<(u8, u8, u8)> = None;
        let mut current_bg: Option<(u8, u8, u8)> = None;

        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // ANSI escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let mut code = String::new();

                    // Read until we hit a letter (the command)
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_alphabetic() {
                            let cmd = chars.next().unwrap();
                            if cmd == 'm' {
                                // SGR (Select Graphic Rendition) - color codes
                                self.parse_sgr(&code, &mut current_fg, &mut current_bg);
                            }
                            break;
                        } else {
                            code.push(chars.next().unwrap());
                        }
                    }
                }
                continue;
            }

            if ch == '\n' || ch == '\r' {
                row += 1;
                col = 0;
                if row >= self.rows as usize {
                    break;
                }
            } else if ch.is_control() {
                continue;
            } else {
                if col < self.cols as usize && row < self.rows as usize {
                    self.content[row][col].c = ch;
                    self.content[row][col].fg = current_fg;
                    self.content[row][col].bg = current_bg;
                    col += 1;
                }
            }
        }
    }

    fn parse_sgr(&self, code: &str, fg: &mut Option<(u8, u8, u8)>, bg: &mut Option<(u8, u8, u8)>) {
        let parts: Vec<&str> = if code.is_empty() {
            vec!["0"]
        } else {
            code.split(';').collect()
        };

        for part in parts {
            match part {
                "0" => {
                    // Reset
                    *fg = None;
                    *bg = None;
                }
                // Foreground colors (30-37)
                "30" => *fg = Some((0, 0, 0)),       // Black
                "31" => *fg = Some((255, 0, 0)),     // Red
                "32" => *fg = Some((0, 255, 0)),     // Green
                "33" => *fg = Some((255, 255, 0)),   // Yellow
                "34" => *fg = Some((0, 0, 255)),     // Blue
                "35" => *fg = Some((255, 0, 255)),   // Magenta
                "36" => *fg = Some((0, 255, 255)),   // Cyan
                "37" => *fg = Some((255, 255, 255)), // White

                // Background colors (40-47)
                "40" => *bg = Some((0, 0, 0)),       // Black
                "41" => *bg = Some((255, 0, 0)),     // Red
                "42" => *bg = Some((0, 255, 0)),     // Green
                "43" => *bg = Some((255, 255, 0)),   // Yellow
                "44" => *bg = Some((0, 0, 255)),     // Blue
                "45" => *bg = Some((255, 0, 255)),   // Magenta
                "46" => *bg = Some((0, 255, 255)),   // Cyan
                "47" => *bg = Some((255, 255, 255)), // White

                _ => {
                    // TODO: Handle 256-color and RGB color codes
                }
            }
        }
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

        for (row_idx, row) in self.content.iter().enumerate() {
            let row_text: String = row.iter().map(|cell| cell.c).collect();

            let mut col = 0;
            while let Some(offset) = row_text[col..].find(text) {
                positions.push((row_idx as u16, (col + offset) as u16));
                col += offset + 1;
            }
        }

        positions
    }

    pub fn render(&self) -> String {
        self.content
            .iter()
            .map(|row| row.iter().map(|cell| cell.c).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn get_cell_color(&self, row: u16, col: u16) -> Option<CellColors> {
        if row >= self.rows || col >= self.cols {
            return None;
        }

        let cell = &self.content[row as usize][col as usize];
        Some(CellColors {
            fg: cell.fg,
            bg: cell.bg,
        })
    }
}

impl Clone for TerminalBuffer {
    fn clone(&self) -> Self {
        Self {
            rows: self.rows,
            cols: self.cols,
            content: self.content.clone(),
        }
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
