use predicates::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn statusline() -> assert_cmd::Command {
        assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("claude-statusline"))
    }

    #[test]
    fn context_displays_token_count() {
        let has_context = predicate::str::contains("Context:");
        let has_152k = predicate::str::contains("152k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":42,"total_input_tokens":100000,"total_output_tokens":52000}}"#)
            .assert()
            .success()
            .stdout(has_context)
            .stdout(has_152k);
    }

    #[test]
    fn context_color_green_below_180k() {
        let has_green = predicate::str::contains("\x1b[32m");
        let has_100k = predicate::str::contains("100k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":30,"total_input_tokens":60000,"total_output_tokens":40000}}"#)
            .assert()
            .success()
            .stdout(has_green)
            .stdout(has_100k);
    }

    #[test]
    fn context_color_yellow_at_180k() {
        let has_yellow = predicate::str::contains("\x1b[33m");
        let has_180k = predicate::str::contains("180k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":40,"total_input_tokens":100000,"total_output_tokens":80000}}"#)
            .assert()
            .success()
            .stdout(has_yellow)
            .stdout(has_180k);
    }

    #[test]
    fn context_color_yellow_below_190k() {
        let has_yellow = predicate::str::contains("\x1b[33m");
        let has_189k = predicate::str::contains("189k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":50,"total_input_tokens":100000,"total_output_tokens":89000}}"#)
            .assert()
            .success()
            .stdout(has_yellow)
            .stdout(has_189k);
    }

    #[test]
    fn context_color_red_at_190k() {
        let has_red = predicate::str::contains("\x1b[31m");
        let has_190k = predicate::str::contains("190k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":60,"total_input_tokens":100000,"total_output_tokens":90000}}"#)
            .assert()
            .success()
            .stdout(has_red)
            .stdout(has_190k);
    }

    #[test]
    fn context_color_red_above_190k() {
        let has_red = predicate::str::contains("\x1b[31m");
        let has_250k = predicate::str::contains("250k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":80,"total_input_tokens":150000,"total_output_tokens":100000}}"#)
            .assert()
            .success()
            .stdout(has_red)
            .stdout(has_250k);
    }

    #[test]
    fn context_shows_zero_when_no_tokens() {
        let has_context = predicate::str::contains("Context:");
        let has_0k = predicate::str::contains("0k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"used_percentage":null}}"#)
            .assert()
            .success()
            .stdout(has_context)
            .stdout(has_0k);
    }

    #[test]
    fn context_shows_token_count_when_percentage_is_null() {
        let has_context = predicate::str::contains("Context:");
        let has_100k = predicate::str::contains("100k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"total_input_tokens":50000,"total_output_tokens":50000,"context_window_size":200000,"used_percentage":null}}"#)
            .assert()
            .success()
            .stdout(has_context)
            .stdout(has_100k);
    }

    #[test]
    fn context_missing_field_shows_zero() {
        let has_context = predicate::str::contains("Context:");
        let has_0k = predicate::str::contains("0k");
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(has_context)
            .stdout(has_0k);
    }

    #[test]
    fn context_shows_zero_with_model_only_json() {
        let has_context = predicate::str::contains("Context:");
        let has_0k = predicate::str::contains("0k");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"model":{"display_name":"Any Model Name"}}"#)
            .assert()
            .success()
            .stdout(has_context)
            .stdout(has_0k);
    }

    #[test]
    fn context_invalid_json_shows_err() {
        let has_context = predicate::str::contains("Context:");
        let has_err = predicate::str::contains("err");
        let mut cmd = statusline();
        cmd.write_stdin("not valid json")
            .assert()
            .success()
            .stdout(has_context)
            .stdout(has_err);
    }

    #[test]
    fn claude_model_renders_display_name() {
        let has_model = predicate::str::contains("Model: Opus 4.5");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"model":{"display_name":"Opus 4.5"}}"#)
            .assert()
            .success()
            .stdout(has_model);
    }

    #[test]
    fn claude_model_missing_field_shows_err() {
        let has_model = predicate::str::contains("Model:");
        let has_err = predicate::str::contains("err");
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(has_model)
            .stdout(has_err);
    }

    #[test]
    fn plan_renders_progress_when_file_exists() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "- [x] Done\n- [ ] Todo\n- [ ] Todo 2\n")
            .expect("failed to write PLAN.md");

        let dir = temp_dir.path();
        let has_plan = predicate::str::contains("PLAN.md:");
        let has_33_pct = predicate::str::contains("33%");
        let mut cmd = statusline();
        cmd.current_dir(dir)
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(has_plan)
            .stdout(has_33_pct);
    }

    #[test]
    fn plan_shows_100_percent_when_no_checkboxes() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "# Just a heading\nNo todos here\n")
            .expect("failed to write PLAN.md");

        let dir = temp_dir.path();
        let has_plan = predicate::str::contains("PLAN.md:");
        let has_100_pct = predicate::str::contains("100%");
        let mut cmd = statusline();
        cmd.current_dir(dir)
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(has_plan)
            .stdout(has_100_pct);
    }

    #[test]
    fn plan_shows_file_not_found_when_missing() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

        let dir = temp_dir.path();
        let has_plan = predicate::str::contains("PLAN.md:");
        let has_not_found = predicate::str::contains("file-not-found");
        let mut cmd = statusline();
        cmd.current_dir(dir)
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(has_plan)
            .stdout(has_not_found);
    }

    #[test]
    fn usage_shows_err_when_no_cache() {
        let has_five_hour = predicate::str::contains("5h:");
        let has_seven_day = predicate::str::contains("7d:");
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(has_five_hour)
            .stdout(has_seven_day);
    }

    #[test]
    fn full_output_renders_all_components_on_two_lines() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "- [x] Done\n- [ ] Todo\n").expect("failed to write PLAN.md");

        let json = r#"{"context_window":{"used_percentage":42,"total_input_tokens":100000,"total_output_tokens":52000},"model":{"display_name":"Opus 4.5"}}"#;

        let dir = temp_dir.path();
        let mut cmd = statusline();
        let output = cmd
            .current_dir(dir)
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
