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

/*
Copyright (c) 2023 funnsam

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

Subject to the terms and conditions of this license, each copyright holder and contributor hereby grants to those receiving rights under this license a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable (except for failure to satisfy the conditions of this license) patent license to make, have made, use, offer to sell, sell, import, and otherwise transfer this software, where such license applies only to those patent claims, already acquired or hereafter acquired, licensable by such copyright holder or contributor that are necessarily infringed by:

(a) their Contribution(s) (the licensed copyrights of copyright holders and non-copyrightable additions of contributors, in source or binary form) alone; or

(b) combination of their Contribution(s) with the work of authorship to which such Contribution(s) was added by such copyright holder or contributor, if, at the time the Contribution is added, such addition causes such combination to be necessarily infringed. The patent license shall not apply to any other combinations which include the Contribution.

Except as expressly stated above, no rights or licenses from any copyright holder or contributor is granted under this license, whether expressly, by implication, estoppel or otherwise.

DISCLAIMER

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use std::io::{self, Read, Write};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
struct Arg {
    #[clap(value_enum)]
    kind: Kind,
}

#[derive(ValueEnum, Clone)]
enum Kind {
    /// Trichromatic
    Normal,
    /// Protanopia
    RedBlind,
    /// Protanomaly
    RedWeak,
    /// Deuteranopia
    GreenBlind,
    /// Deuteranomaly
    GreenWeak,
    /// Tritanopia
    BlueBlind,
    /// Tritanomaly
    BlueWeak,
    /// Achromatopsia
    ColorBlind,
    /// Achromatomaly
    ColorWeak,
}

type Color = (f32, f32, f32);

impl Kind {
    // http://web.archive.org/web/20081014161121/http://www.colorjack.com/labs/colormatrix/
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

    let mut default_fg = Vec::with_capacity(5);
    let mut default_bg = Vec::with_capacity(5);
    filter(true, FG_COLOR, &recolor, &mut default_fg);
    filter(false, BG_COLOR, &recolor, &mut default_bg);

    let default_colors = [default_fg.clone(), default_bg.clone()].concat();
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
                        39 => new.extend(default_fg.iter()),
                        49 => new.extend(default_bg.iter()),
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
                    if new.len() != 0 {
                        write!(stdout, "\x1b[{}m", new.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(";")).unwrap();
                        new.clear();
                    }
                }
            } else {
                write!(stdout, "\x1b{}{}", a, k as char).unwrap();
            }
        } else {
            stdout.write_all(&buf).unwrap();
        }
    }

    write!(stdout, "\x1b[0m\x1b[K").unwrap();
}

macro_rules! rgb {
    ($rgb: expr) => {{
        let a = $rgb;
        (
            ((a >> 16) % 256) as f32,
            ((a >> 8) % 256) as f32,
            (a % 256) as f32,
        )
    }};
}

const FG_COLOR: Color = rgb!(0xcad3f5);
const BG_COLOR: Color = rgb!(0x24273a);

const COLOR_16: [Color; 16] = [
    rgb!(0x494d64),
    rgb!(0xed8796),
    rgb!(0xa6da95),
    rgb!(0xeed49f),
    rgb!(0x8aadf4),
    rgb!(0xf5bde6),
    rgb!(0x8bd5ca),
    rgb!(0xa5adcb),
    rgb!(0x5b6078),
    rgb!(0xed8796),
    rgb!(0xa6da95),
    rgb!(0xeed49f),
    rgb!(0x8aadf4),
    rgb!(0xf5bde6),
    rgb!(0x8bd5ca),
    rgb!(0xb8c0e0),
];

fn get_8c(color: u8, int: bool) -> Color {
    let color = color % 10;
    COLOR_16[color as usize + int as usize * 8]
}

fn get_256c(color: u8) -> Color {
    match color {
        0..=7 => get_8c(color, false),
        8..=15 => get_8c(color & 7, true),
        16..=231 => {
            let color = color - 16;
            (
                ((color / 36) % 6) as f32 * 42.667,
                ((color /  6) % 6) as f32 * 42.667,
                ((color /  1) % 6) as f32 * 42.667,
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
