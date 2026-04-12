use console::Alignment;

pub fn pad_right(text: &str, width: usize) -> String {
    let truncation = Some("...");
    console::pad_str(text, width, Alignment::Left, truncation).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_short_text() {
        let result = pad_right("hello", 10);
        assert_eq!(result.len(), 10);
        assert!(result.starts_with("hello"));
    }

    #[test]
    fn truncates_long_text() {
        let result = pad_right("hello world", 5);
        assert_eq!(console::measure_text_width(&result), 5);
    }

    #[test]
    fn handles_unicode() {
        let result = pad_right("\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}", 10);
        assert_eq!(console::measure_text_width(&result), 10);
    }
}
