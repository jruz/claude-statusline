use owo_colors::OwoColorize as _;

pub struct Bar {
    pub empty: String,
    pub filled: String,
}

pub fn render(percentage: f64, width: usize) -> Bar {
    let clamped = percentage.clamp(0.0, 100.0);
    let filled_count = ((clamped / 100.0) * width as f64).round() as usize;
    let empty_count = width.saturating_sub(filled_count);

    Bar {
        empty: "\u{2501}".repeat(empty_count).bright_black().to_string(),
        filled: "\u{2501}".repeat(filled_count),
    }
}
