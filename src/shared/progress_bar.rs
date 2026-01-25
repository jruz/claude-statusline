pub fn render(percentage: f64, width: usize) -> String {
    let clamped = percentage.clamp(0.0, 100.0);
    let filled_count = ((clamped / 100.0) * width as f64).round() as usize;
    let empty_count = width.saturating_sub(filled_count);

    let filled = "\u{2588}".repeat(filled_count);
    let empty = "\u{2591}".repeat(empty_count);

    format!("{filled}{empty}")
}
