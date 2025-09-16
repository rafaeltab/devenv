use std::cmp::max;

use clap::Parser;
use colored::{ColoredString, Colorize};
use figlet_rs::FIGfont;
use terminal_size::terminal_size;

use crate::font::ANSI_REGULAR;

mod font;

const BACKGROUND_CHAR: &str = "╱";

#[derive(Debug, Parser)]
#[command(name = "banner")]
#[command(about = "Generate a banner")]
struct Cli {
    #[arg(long)]
    width: Option<usize>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    colorchoice::ColorChoice::Always.write_global();

    let width = cli
        .width
        .or_else(|| terminal_size().map(|y| y.0.0.into()))
        .expect("Couldn't determine width");

    let font = FIGfont::from_content(ANSI_REGULAR).unwrap();
    let figure = font.convert("Rafaeltab").unwrap();

    let figure_height: usize = figure.height.try_into().unwrap();
    let figure_text = figure.to_string();
    let figure_width = figure_text.lines().next().unwrap().chars().count();
    let mut figure_lines = figure_text.lines();
    let start_offset_i: i32 = (width as i32 / 4) - (figure_width as i32 / 2);
    let start_offset: usize = if start_offset_i < 5 {
        5
    } else {
        start_offset_i as usize
    };

    let mut s = String::with_capacity((width + 1) * (figure_height + 4));
    let col_start = (163, 0, 0);
    let col_end = (242, 186, 189);

    s.push('\n');
    s.push('\n');

    for line_nr in 0..figure_height - 2 {
        for i in 0..start_offset {
            let percentage = (i as f32 + line_nr as f32 * 30_f32) / width as f32;
            let col = color_at(percentage, col_start, col_end);
            s.push_str(&BACKGROUND_CHAR.truecolor(col.0, col.1, col.2).to_string());
        }
        s.push(' ');
        let figure_line = figure_lines.next().unwrap();
        s.push_str(&gradient_text(
            figure_line,
            col_end,
            col_start,
            100,
            line_nr * 5,
        ));

        for i in (start_offset + figure_width + 1)..width {
            let percentage = i as f32 / width as f32;
            let col = color_at(percentage, col_start, col_end);
            s.push_str(&BACKGROUND_CHAR.truecolor(col.0, col.1, col.2).to_string());
        }
        s.push('\n');
    }

    println!("{}", s);
    Ok(())
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn color_at(t: f32, start: (u8, u8, u8), end: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        lerp(start.0, end.0, t),
        lerp(start.1, end.1, t),
        lerp(start.2, end.2, t),
    )
}

fn gradient_text(
    s: &str,
    start: (u8, u8, u8),
    end: (u8, u8, u8),
    extra_len_gradient: usize,
    offset: usize,
) -> String {
    let graphemes: Vec<char> = s.chars().collect(); // simple; use unicode-segmentation for true graphemes
    let n = max(graphemes.len(), 1);
    let mut out = String::with_capacity(s.len() + n * 10);

    for (i, ch) in graphemes.into_iter().enumerate() {
        let t = if n == 1 {
            0.0
        } else {
            (i as f32 + offset as f32) / (n as f32 - 1.0 + extra_len_gradient as f32)
        };
        let (r, g, b) = color_at(t, start, end);
        let part: ColoredString = ch.to_string().truecolor(r, g, b);
        out.push_str(&part.to_string());
    }
    out
}
