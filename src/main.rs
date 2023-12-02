use std::io::{self, Read, Write};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
struct Arg {
    #[clap(value_enum)]
    kind: Kind,
}

#[derive(ValueEnum, Clone)]
enum Kind {
    Normal,
    RedGreen,
    BlueYellow,
    Monochrome,
}

type Color = (f32, f32, f32);

impl Kind {
    fn get_matrix(&self) -> [Color; 3] {
        match self {
            Kind::Normal => [
                (1.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 0.0, 1.0),
            ],
            Kind::RedGreen => [
                (0.625, 0.375, 0.0),
                (0.7, 0.3, 0.0),
                (0.0, 0.3, 0.7),
            ],
            Kind::BlueYellow => [
                (0.95, 0.5, 0.0),
                (0.0, 0.433333, 0.56667),
                (0.0, 0.475, 0.525),
            ],
            Kind::Monochrome => [
                (0.299, 0.587, 0.114),
                (0.299, 0.587, 0.114),
                (0.299, 0.587, 0.114),
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

    while let Ok(_) = stdin.read_exact(&mut buf) {
        if buf[0] == 0x1b {
            let mut a = Vec::new();
            let mut k = 0;
            while let Ok(_) = stdin.read_exact(&mut buf) {
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
            if k == 'm' as u8 {
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
            stdout.write(&buf).unwrap();
        }
    }

    write!(stdout, "\x1b[0m\x1b[K").unwrap();
}

fn get_8c(color: u8, int: bool) -> Color {
    let color = color % 10;
    let base = if int { 255.0 - 192.0 } else { 0.0 };
    (
        ((color >> 0) & 1) as f32 * 192.0 + base,
        ((color >> 1) & 1) as f32 * 192.0 + base,
        ((color >> 2) & 1) as f32 * 192.0 + base,
    )
}

fn get_256c(color: u8) -> Color {
    match color {
        0..=7 => get_8c(color, false),
        8..=15 => get_8c(color & 7, true),
        16..=231 => {
            let color = color - 16;
            (
                ((color >> 0) & 7) as f32 * 8.0,
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
    new.push((color.0 * recolor[0].0 + color.1 * recolor[0].1 + color.2 * recolor[0].2) as u8);
    new.push((color.0 * recolor[1].0 + color.1 * recolor[1].1 + color.2 * recolor[1].2) as u8);
    new.push((color.0 * recolor[2].0 + color.1 * recolor[2].1 + color.2 * recolor[2].2) as u8);
}
