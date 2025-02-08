use std::fmt::Display;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Color {
    Black, // for unknown difficulty
    Gray,
    Brown,
    Green,
    Cyan,
    Blue,
    Yellow,
    Orange,
    Red,
}

impl From<Color> for u32 {
    fn from(val: Color) -> Self {
        match val {
            Color::Black => 0x000000,
            Color::Gray => 0x808080,
            Color::Brown => 0x804000,
            Color::Green => 0x008000,
            Color::Cyan => 0x00c0c0,
            Color::Blue => 0x0000ff,
            Color::Yellow => 0xc0c000,
            Color::Orange => 0xff8000,
            Color::Red => 0xff0000,
        }
    }
}

impl From<Color> for String {
    fn from(val: Color) -> Self {
        match val {
            Color::Black => unreachable!(),
            Color::Gray => "灰",
            Color::Brown => "茶",
            Color::Green => "緑",
            Color::Cyan => "水",
            Color::Blue => "青",
            Color::Yellow => "黄",
            Color::Orange => "橙",
            Color::Red => "赤",
        }
        .to_string()
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl From<u32> for Color {
    fn from(val: u32) -> Self {
        match val {
            0..400 => Color::Gray,
            400..800 => Color::Brown,
            800..1200 => Color::Green,
            1200..1600 => Color::Cyan,
            1600..2000 => Color::Blue,
            2000..2400 => Color::Yellow,
            2400..2800 => Color::Orange,
            2800.. => Color::Red,
        }
    }
}

pub fn normalize(difficulty: i64) -> u32 {
    if difficulty >= 400 {
        difficulty as u32
    } else {
        (400.0 / (1.0 + (1.0 - difficulty as f64 / 400.0).exp())) as u32
    }
}
