use num_traits::ToPrimitive as _;
use owo_colors::OwoColorize as _;

pub struct Bar {
    pub empty: String,
    pub filled: String,
}

pub fn render(percentage: f64, width: u8) -> Bar {
    let clamped = percentage.clamp(0.0, 100.0).round();
    let pct = match clamped.to_u8() {
        Some(val) => u16::from(val),
        None => 0_u16,
    };
    let w = u16::from(width);
    let raw_filled = (pct * w + 50_u16).div_euclid(100);
    let filled_count = usize::from(raw_filled);
    let empty_count = usize::from(width).saturating_sub(filled_count);

    Bar {
        empty: "\u{2501}".repeat(empty_count).bright_black().to_string(),
        filled: "\u{2501}".repeat(filled_count),
    }
}
