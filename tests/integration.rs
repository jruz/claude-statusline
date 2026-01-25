use predicates::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn statusline() -> assert_cmd::Command {
        assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("claude-statusline"))
    }

    #[test]
    fn context_renders_percentage_and_progress_bar() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":42.5}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\u{2588}\u{2588}\u{2588}\u{2588}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591} 42%"));
    }

    #[test]
    fn context_missing_field_shows_err() {
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(predicate::str::contains("Context:"))
            .stdout(predicate::str::contains("err"));
    }

    #[test]
    fn context_invalid_json_shows_err() {
        let mut cmd = statusline();
        cmd.write_stdin("not valid json")
            .assert()
            .success()
            .stdout(predicate::str::contains("Context:"))
            .stdout(predicate::str::contains("err"));
    }
}
