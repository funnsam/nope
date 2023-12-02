#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::nursery,
    clippy::suspicious,
    clippy::style,
)]
#![allow(
    clippy::semicolon_inside_block,
    clippy::just_underscores_and_digits,
)]

use std::io::{self, Read, Write};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
struct Arg {
    #[clap(value_enum)]
    kind: Kind,
}

#[derive(ValueEnum, Clone)]
enum Kind {
    // Trichromatic
    Normal,
    // Protanopia
    RedBlind,
    // Protanomaly
    RedWeak,
    // Deuteranopia
    GreenBlind,
    //Deuteranomaly
    GreenWeak,
    // Tritanopia
    BlueBlind,
    // Tritanomaly
    BlueWeak,
    // Achromatopsia
    ColorBlind,
    // Achromatomaly
    ColorWeak,
}

type Color = (f32, f32, f32);

impl Kind {
    const fn get_matrix(&self) -> [Color; 3] {
        match self {
            Self::Normal => [
                (1.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 0.0, 1.0),
            ],
            Self::RedBlind => [
                (0.56667, 0.43333, 0.0),
                (0.55833, 0.44167, 0.0),
                (0.0, 0.24167, 0.75833),
            ],
            Self::RedWeak => [
                (0.81667, 0.18333, 0.0),
                (0.33333, 0.66667, 0.0),
                (0.0, 0.125, 0.875),
            ],
            Self::GreenBlind => [
                (0.625, 0.375, 0.0),
                (0.7, 0.3, 0.0),
                (0.0, 0.3, 0.7),
            ],
            Self::GreenWeak => [
                (0.8, 0.2, 0.0),
                (0.25833, 0.47167, 0.0),
                (0.0, 0.14167, 85.833),
            ],
            Self::BlueBlind => [
                (0.95, 0.5, 0.0),
                (0.0, 0.433333, 0.56667),
                (0.0, 0.475, 0.525),
            ],
            Self::BlueWeak => [
                (0.96667, 0.03333, 0.0),
                (0.0, 0.73333, 0.26667),
                (0.0, 0.18333, 0.81667),
            ],
            Self::ColorBlind => [
                (0.299, 0.587, 0.114),
                (0.299, 0.587, 0.114),
                (0.299, 0.587, 0.114),
            ],
            Self::ColorWeak => [
                (0.618, 0.32, 0.062),
                (0.163, 0.775, 0.062),
                (0.163, 0.32, 0.516),
            ],
        }
    }
}

fn main() {
    let args = Arg::parse();
    let mut stdin  = io::stdin();
    let mut stdout = io::stdout();
    let mut buf = [0];

    let recolor = args.kind.get_matrix();

    let mut default_colors = Vec::with_capacity(10);
    filter(true, get_8c(7, false), &recolor, &mut default_colors);
    filter(false, get_8c(0, false), &recolor, &mut default_colors);
    write!(stdout, "\x1b[{}m\x1b[K", default_colors.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(";")).unwrap();

    while stdin.read_exact(&mut buf).is_ok() {
        if buf[0] == 0x1b {
            let mut a = Vec::new();
            let mut k = 0;
            while stdin.read_exact(&mut buf).is_ok() {
                if buf[0].is_ascii_alphabetic() {
                    k = buf[0];
                    break;
                } else if buf[0] == b'\n' {
                    write!(stdout, "\x1b[{}m\x1b[K", default_colors.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(";")).unwrap();
                } else {
                    a.push(buf[0]);
                }
            }

            let a = String::from_utf8(a).unwrap();
            if k == b'm' {
                let a = a.replace('[', "");
                let mut a = a.split(';').map(|a| a.parse());
                let mut new = Vec::new();
                while let Some(Ok(esc)) = a.next() {
                    match esc {
                        30..=37 => {
                            let color = get_8c(esc, false);
                            filter(true, color, &recolor, &mut new);
                        },
                        40..=47 => {
                            let color = get_8c(esc, false);
                            filter(false, color, &recolor, &mut new);
                        },
                        90..=97 => {
                            let color = get_8c(esc, true);
                            filter(true, color, &recolor, &mut new);
                        },
                        100..=107 => {
                            let color = get_8c(esc, true);
                            filter(false, color, &recolor, &mut new);
                        },
                        38 | 48 => match a.next() {
                            Some(Ok(2)) => {
                                let r = a.next();
                                let g = a.next();
                                let b = a.next();
                                if let (Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) = (r, g, b) {
                                    filter(esc == 38, (r as f32, g as f32, b as f32), &recolor, &mut new);
                                } else {
                                    new.push(esc);
                                    new.push(2);
                                }
                            },
                            Some(Ok(5)) => {
                                if let Some(Ok(color)) = a.next() {
                                    filter(esc == 38, get_256c(color), &recolor, &mut new);
                                } else {
                                    new.push(esc);
                                    new.push(2);
                                }
                            },
                            Some(Ok(i)) => {
                                new.push(38);
                                new.push(i);
                            },
                            _ => new.push(38),
                        },
                        0 => {
                            new.push(0);
                            new.extend(default_colors.iter());
                        },
                        _ => new.push(esc),
                    }
                }
                write!(stdout, "\x1b[{}m", new.into_iter().map(|a| a.to_string()).collect::<Vec<String>>().join(";")).unwrap();
            } else {
                write!(stdout, "\x1b{}{}", a, k as char).unwrap();
            }
        } else {
            stdout.write_all(&buf).unwrap();
        }
    }

    write!(stdout, "\x1b[0m\x1b[K").unwrap();
}

fn get_8c(color: u8, int: bool) -> Color {
    let color = color % 10;
    let base = if int { 255.0 - 192.0 } else { 0.0 };
    (
        ((color & 1) as f32).mul_add(192.0, base),
        (((color >> 1) & 1) as f32).mul_add(192.0, base),
        (((color >> 2) & 1) as f32).mul_add(192.0, base),
    )
}

fn get_256c(color: u8) -> Color {
    match color {
        0..=7 => get_8c(color, false),
        8..=15 => get_8c(color & 7, true),
        16..=231 => {
            let color = color - 16;
            (
                (color & 7) as f32 * 8.0,
                ((color >> 3) & 7) as f32 * 8.0,
                ((color >> 6) & 7) as f32 * 8.0,
            )
        },
        232..=255 => {
            let c = (color - 232) as f32 * 255.0 / 24.0;
            (c, c, c)
        },
    }
}

fn filter(fg: bool, color: Color, recolor: &[Color; 3], new: &mut Vec<u8>) {
    new.push(if fg { 38 } else { 48 });
    new.push(2);
    new.push(color.2.mul_add(recolor[0].2, color.0.mul_add(recolor[0].0, color.1 * recolor[0].1)) as u8);
    new.push(color.2.mul_add(recolor[1].2, color.0.mul_add(recolor[1].0, color.1 * recolor[1].1)) as u8);
    new.push(color.2.mul_add(recolor[2].2, color.0.mul_add(recolor[2].0, color.1 * recolor[2].1)) as u8);
}
