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
            .stdout(predicate::str::contains("Context:"))
            .stdout(predicate::str::contains("42%"));
    }

    #[test]
    fn context_color_green_at_39_percent() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":39}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b[32m"))
            .stdout(predicate::str::contains("39%"));
    }

    #[test]
    fn context_color_yellow_at_40_percent() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":40}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b[33m"))
            .stdout(predicate::str::contains("40%"));
    }

    #[test]
    fn context_color_yellow_at_59_percent() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":59}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b[33m"))
            .stdout(predicate::str::contains("59%"));
    }

    #[test]
    fn context_color_red_at_60_percent() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":60}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b[31m"))
            .stdout(predicate::str::contains("60%"));
    }

    #[test]
    fn context_color_red_at_100_percent() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":100}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b[31m"))
            .stdout(predicate::str::contains("100%"));
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
    fn context_shows_err_with_model_only_json() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"model":{"display_name":"Any Model Name"}}"#)
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

    #[test]
    fn claude_model_renders_display_name() {
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"model":{"display_name":"Opus 4.5"}}"#)
            .assert()
            .success()
            .stdout(predicate::str::contains("Model: Opus 4.5"));
    }

    #[test]
    fn claude_model_missing_field_shows_err() {
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(predicate::str::contains("Model:"))
            .stdout(predicate::str::contains("err"));
    }

    #[test]
    fn plan_renders_progress_when_file_exists() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "- [x] Done\n- [ ] Todo\n- [ ] Todo 2\n")
            .expect("failed to write PLAN.md");

        let mut cmd = statusline();
        cmd.current_dir(temp_dir.path())
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(predicate::str::contains("PLAN.md:"))
            .stdout(predicate::str::contains("33%"));
    }

    #[test]
    fn plan_shows_100_percent_when_no_checkboxes() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "# Just a heading\nNo todos here\n")
            .expect("failed to write PLAN.md");

        let mut cmd = statusline();
        cmd.current_dir(temp_dir.path())
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(predicate::str::contains("PLAN.md:"))
            .stdout(predicate::str::contains("100%"));
    }

    #[test]
    fn plan_shows_file_not_found_when_missing() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

        let mut cmd = statusline();
        cmd.current_dir(temp_dir.path())
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(predicate::str::contains("PLAN.md:"))
            .stdout(predicate::str::contains("file-not-found"));
    }

    #[test]
    fn usage_shows_err_when_no_cache() {
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(predicate::str::contains("5h:"))
            .stdout(predicate::str::contains("7d:"));
    }

    #[test]
    fn full_output_renders_all_components_on_two_lines() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "- [x] Done\n- [ ] Todo\n").expect("failed to write PLAN.md");

        let json =
            r#"{"context_window":{"used_percentage":42},"model":{"display_name":"Opus 4.5"}}"#;

        let mut cmd = statusline();
        let output = cmd
            .current_dir(temp_dir.path())
            .write_stdin(json)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8_lossy(&output);
        let output_lines: Vec<&str> = stdout.lines().collect();
        assert!(
            output_lines.len() >= 2,
            "Expected at least 2 lines, got {}",
            output_lines.len()
        );
        let first_line = output_lines.first().expect("expected first line");
        let second_line = output_lines.get(1).expect("expected second line");
        assert!(first_line.contains("Context:"));
        assert!(first_line.contains("Model:"));
        assert!(first_line.contains("5h:"));
        assert!(second_line.contains("PLAN.md:"));
        assert!(second_line.contains("7d:"));
    }
}
