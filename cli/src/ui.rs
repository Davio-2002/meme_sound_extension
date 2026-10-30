use std::io::{self, IsTerminal};
use std::path::Path;

use anstream::{AutoStream, ColorChoice, eprintln, println};
use anstyle::{Color, RgbColor, Style};
use clap::builder::Styles;

pub const AMBER: Style = rgb(0xE8, 0xA1, 0x5A);
pub const SAGE: Style = rgb(0x9C, 0xB3, 0x80);
pub const ROSE: Style = rgb(0xD4, 0x88, 0x8A);
pub const ASH: Style = rgb(0xA0, 0x94, 0x8A);

pub const HELP: Styles = Styles::styled()
    .header(AMBER.bold())
    .usage(AMBER.bold())
    .literal(SAGE)
    .placeholder(ASH)
    .valid(SAGE)
    .invalid(ROSE)
    .error(ROSE.bold());

const fn rgb(r: u8, g: u8, b: u8) -> Style {
    Style::new().fg_color(Some(Color::Rgb(RgbColor(r, g, b))))
}

pub fn paint(style: Style, text: &str) -> String {
    format!("{style}{text}{style:#}")
}

pub fn line(style: Style, glyph: &str, text: &str) {
    println!("{}", lead(style, glyph, text));
}

pub fn lead(style: Style, glyph: &str, text: &str) -> String {
    format!("{}  {text}", paint(style, glyph))
}

/// Parts in backticks show in amber.
pub fn hint(text: &str) {
    println!("   {}", hint_text(text));
}

pub fn error(err: &anyhow::Error) {
    let message = err.to_string();
    let mut lines = message.lines();
    eprintln!("{}  {}", paint(ROSE, "✗"), lines.next().unwrap_or_default());
    for text in lines
        .map(str::to_string)
        .chain(err.chain().skip(1).map(|e| e.to_string()))
    {
        eprintln!("   {}", hint_text(&text));
    }
}

fn hint_text(text: &str) -> String {
    text.split('`')
        .enumerate()
        .map(|(i, part)| paint(if i % 2 == 0 { ASH } else { AMBER }, part))
        .collect()
}

pub fn path(path: &Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".into(),
        Some(rest) => Path::new("~").join(rest).display().to_string(),
        None => path.display().to_string(),
    }
}

pub type Row<'a> = (&'a str, Style, String);

pub fn status_box(rows: &[Row], footer: &str) -> Vec<String> {
    const LABEL: usize = 9;
    let inner = rows
        .iter()
        .map(|(_, _, value)| 2 + LABEL + width(value) + 2)
        .chain([width(footer) + 5, 10])
        .max()
        .unwrap_or_default();
    let border = |text: &str| paint(ASH, text);
    let top = format!(
        "{}{}{}",
        border("╭─ "),
        paint(AMBER.bold(), "faaah"),
        border(&format!(" {}╮", "─".repeat(inner - 8)))
    );
    let middle = rows.iter().map(|(label, style, value)| {
        let pad = " ".repeat(inner - 2 - LABEL - width(value));
        format!(
            "{}  {}{}{pad}{}",
            border("│"),
            paint(ASH, &format!("{label:<LABEL$}")),
            paint(*style, value),
            border("│")
        )
    });
    let bottom = border(&format!(
        "╰─ {footer} {}╯",
        "─".repeat(inner - 3 - width(footer))
    ));
    std::iter::once(top).chain(middle).chain([bottom]).collect()
}

/// True when stdout is a terminal that shows colors, so the mascot can be drawn.
pub fn fancy() -> bool {
    let stdout = io::stdout();
    stdout.is_terminal() && AutoStream::choice(&stdout) != ColorChoice::Never
}

fn width(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_becomes_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(path(&home), "~");
        assert_eq!(
            path(&home.join(".zshrc")),
            format!("~{}.zshrc", std::path::MAIN_SEPARATOR)
        );
        assert_eq!(path(Path::new("/elsewhere")), "/elsewhere");
    }

    #[test]
    fn backticks_mark_commands() {
        let text = hint_text("run `faaah on`");
        assert!(text.contains(&paint(AMBER, "faaah on")));
        assert!(!text.contains('`'));
    }
}
