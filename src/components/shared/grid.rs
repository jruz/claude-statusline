use console;

pub fn pad_right(text: &str, width: usize) -> String {
    console::pad_str(text, width, console::Alignment::Left, Some("")).into_owned()
}
