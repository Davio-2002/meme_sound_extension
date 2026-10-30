use anstyle::{Color, RgbColor, Style};

use crate::ui;

const LENS: [&str; 10] = [
    ".W.WWWW.W.",
    ".WWNNNNWW.",
    "..BBBBBB..",
    ".BBBBGGGB.",
    ".BBEBGEGB.",
    ".BCEEGGGB.",
    "BBBBBBBBGH",
    "BBBBBBBBBH",
    "H.DDDDDD..",
    "H.DD..DD..",
];

const IDLE: [&str; 10] = [
    ".W.WWWW.W.",
    ".WWNNNNWW.",
    "H.BBBBBB..",
    "HBBBBBBBB.",
    "BBBEBBEBB.",
    "BBCBEEBCB.",
    ".BBBBBBBBB",
    ".BBBBBBBBB",
    "..DDDDDD.H",
    "..DD..DD.H",
];

const SIX: [&str; 7] = [
    ".BBB.", "B....", "B....", "BBBB.", "B...B", "B...B", ".BBB.",
];

const SEVEN: [&str; 7] = [
    "hhhhh", "....h", "...h.", "..h..", ".h...", ".h...", ".h...",
];

const CAR: [&str; 10] = [
    "...........W.WWWW.W........",
    "...........WWNNNNWW........",
    "............BBBBBB.........",
    "...........BBEBBEBB........",
    "...........BCBEEBCB........",
    "...RRR....rrrrrrrrrr.......",
    "....r.RRRRRRRRRRRRRRRRR....",
    "...TTTTRRRSSSSSRRRRTTTTRRR.",
    "...ThTTrrrrrrrrrrrrThTTRRRR",
    "...TTTT............TTTT.RRR",
];

pub struct Sprite {
    lines: Vec<String>,
    width: usize,
}

impl Sprite {
    fn new(grid: &[Vec<char>]) -> Self {
        let lines = grid
            .chunks(2)
            .map(|rows| {
                rows[0]
                    .iter()
                    .zip(&rows[1])
                    .map(|(&top, &bottom)| cell(color(top), color(bottom)))
                    .collect()
            })
            .collect();
        Self {
            lines,
            width: grid[0].len(),
        }
    }

    /// Puts `text` to the right, both centered vertically.
    pub fn beside(&self, text: &[String]) -> Vec<String> {
        let height = self.lines.len().max(text.len());
        let sprite_top = (height - self.lines.len()) / 2;
        let text_top = (height - text.len()) / 2;
        let blank = " ".repeat(self.width);
        (0..height)
            .map(|i| {
                let left = i.checked_sub(sprite_top).and_then(|i| self.lines.get(i));
                let right = i.checked_sub(text_top).and_then(|i| text.get(i));
                format!(
                    "{}  {}",
                    left.unwrap_or(&blank),
                    right.map_or("", String::as_str)
                )
            })
            .collect()
    }
}

/// Does the 6-7 hands: the 6 is up, the 7 is down.
pub fn idle() -> Sprite {
    let rows: Vec<String> = IDLE
        .iter()
        .enumerate()
        .map(|(y, body)| {
            let six = SIX.get(y).unwrap_or(&".....");
            let seven = y
                .checked_sub(3)
                .and_then(|y| SEVEN.get(y))
                .unwrap_or(&".....");
            format!("{six}..{body}..{seven}")
        })
        .collect();
    Sprite::new(&grid(&rows))
}

pub fn lens() -> Sprite {
    Sprite::new(&grid(&LENS))
}

/// The F1 car; `frame` waves the hands, spins the wheels and blinks now and then.
pub fn car(frame: usize) -> Sprite {
    let mut g = grid(&CAR);
    let left_up = frame.is_multiple_of(2);
    if frame % 14 == 13 {
        g[3][13] = 'B';
        g[3][16] = 'B';
    }
    for (x, up) in [(10, left_up), (19, !left_up)] {
        if up {
            g[0][x] = 'H';
            g[1][x] = 'H';
            g[2][x] = 'B';
            g[3][x] = 'B';
        } else {
            g[4][x] = 'H';
        }
    }
    if !left_up {
        for x in [4, 20] {
            g[8][x] = 'T';
            g[8][x + 1] = 'h';
        }
    }
    let speed = if left_up {
        [(6, 1, 2), (8, 0, 1)]
    } else {
        [(6, 0, 1), (8, 1, 2)]
    };
    for (y, from, to) in speed {
        g[y][from..=to].fill('Z');
    }
    Sprite::new(&g)
}

fn grid(rows: &[impl AsRef<str>]) -> Vec<Vec<char>> {
    rows.iter()
        .map(|row| row.as_ref().chars().collect())
        .collect()
}

fn cell(top: Option<Color>, bottom: Option<Color>) -> String {
    let (glyph, style) = match (top, bottom) {
        (None, None) => return " ".into(),
        (Some(top), None) => ("▀", Style::new().fg_color(Some(top))),
        (None, Some(bottom)) => ("▄", Style::new().fg_color(Some(bottom))),
        (Some(top), Some(bottom)) => ("▀", Style::new().fg_color(Some(top)).bg_color(Some(bottom))),
    };
    ui::paint(style, glyph)
}

fn color(key: char) -> Option<Color> {
    let (r, g, b) = match key {
        'B' => (0xB8, 0x78, 0x38),
        'D' => (0x7E, 0x51, 0x28),
        'H' => (0xED, 0xE3, 0xD3),
        'E' => (0x2A, 0x1F, 0x1A),
        'C' => (0xD4, 0x88, 0x8A),
        'W' => (0x6B, 0x44, 0x28),
        'N' => (0xE8, 0xC0, 0x7A),
        'G' => (0xCF, 0xC4, 0xB4),
        'R' => (0xD7, 0x37, 0x2F),
        'r' => (0x9E, 0x24, 0x20),
        'S' => (0xF6, 0xEF, 0xE4),
        'T' => (0x4A, 0x40, 0x3B),
        'h' | 'Z' => (0xA0, 0x94, 0x8A),
        _ => return None,
    };
    Some(Color::Rgb(RgbColor(r, g, b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprites_are_rectangular() {
        for rows in [&LENS[..], &CAR[..], &IDLE[..]] {
            assert!(rows.iter().all(|row| row.len() == rows[0].len()));
            assert_eq!(rows.len() % 2, 0);
        }
    }

    #[test]
    fn beside_centers_the_shorter_side() {
        let lines = lens().beside(&vec!["x".to_string(); 7]);
        assert_eq!(lines.len(), 7);
        assert_eq!(lines[0], format!("{}  x", " ".repeat(10)));
    }
}
