use predicates::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn statusline() -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("claude-statusline"));
        cmd.env_remove("CLAUDE_CODE_AUTO_COMPACT_WINDOW");
        cmd
    }

    fn context_stdin(size: u64, tokens: u64) -> String {
        format!(
            r#"{{"context_window":{{"context_window_size":{size},"current_usage":{{"input_tokens":{tokens},"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"output_tokens":5000}}}}}}"#
        )
    }

    #[test]
    fn context_displays_percentage_and_tokens() {
        let has_text = predicate::str::contains("  15% (152k)");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"context_window_size":1000000,"current_usage":{"input_tokens":2000,"cache_creation_input_tokens":50000,"cache_read_input_tokens":100000,"output_tokens":5000}}}"#)
            .assert()
            .success()
            .stdout(has_text);
    }

    #[test]
    fn context_uses_window_size_from_stdin() {
        let has_text = predicate::str::contains("  76% (152k)");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 152_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_text);
    }

    #[test]
    fn context_rounds_percentage() {
        let has_text = predicate::str::contains("  43% (85k)");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 85_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_text);
    }

    #[test]
    fn context_clamps_percentage_to_100() {
        let has_text = predicate::str::contains("100% (250k)");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 250_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_text);
    }

    #[test]
    fn context_shows_raw_tokens_below_1k() {
        let has_text = predicate::str::contains("   0% (850)");
        let mut cmd = statusline();
        let stdin = context_stdin(1_000_000, 850);
        cmd.write_stdin(stdin).assert().success().stdout(has_text);
    }

    #[test]
    fn context_shows_millions_when_rounding_reaches_1000k() {
        let has_text = predicate::str::contains("(1m)");
        let mut cmd = statusline();
        let stdin = context_stdin(2_000_000, 999_600);
        cmd.write_stdin(stdin).assert().success().stdout(has_text);
    }

    #[test]
    fn context_color_green_below_32() {
        let has_green = predicate::str::contains("\x1b[32m");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 62_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_green);
    }

    #[test]
    fn context_color_yellow_at_32() {
        let has_yellow = predicate::str::contains("\x1b[33m");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 64_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_yellow);
    }

    #[test]
    fn context_color_red_at_75() {
        let has_red = predicate::str::contains("\x1b[31m");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 150_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_red);
    }

    #[test]
    fn context_uses_compact_window_env() {
        let has_text = predicate::str::contains("  11% (44k)");
        let mut cmd = statusline();
        let stdin = context_stdin(1_000_000, 44_000);
        cmd.env("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "400000")
            .write_stdin(stdin)
            .assert()
            .success()
            .stdout(has_text);
    }

    #[test]
    fn context_caps_compact_window_to_model_window() {
        let has_text = predicate::str::contains("  50% (100k)");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 100_000);
        cmd.env("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "400000")
            .write_stdin(stdin)
            .assert()
            .success()
            .stdout(has_text);
    }

    #[test]
    fn context_invalid_compact_window_shows_err() {
        let has_err = predicate::str::contains("Context: \x1b[31merr");
        let mut cmd = statusline();
        let stdin = context_stdin(1_000_000, 44_000);
        cmd.env("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "400k")
            .write_stdin(stdin)
            .assert()
            .success()
            .stdout(has_err);
    }

    #[test]
    fn context_shows_zero_when_usage_null() {
        let has_text = predicate::str::contains("   0% (0)");
        let mut cmd = statusline();
        cmd.write_stdin(
            r#"{"context_window":{"context_window_size":200000,"current_usage":null}}"#,
        )
        .assert()
        .success()
        .stdout(has_text);
    }

    #[test]
    fn context_ignores_output_tokens() {
        let has_text = predicate::str::contains("  50% (100k)");
        let mut cmd = statusline();
        let stdin = context_stdin(200_000, 100_000);
        cmd.write_stdin(stdin).assert().success().stdout(has_text);
    }

    #[test]
    fn context_missing_window_shows_err() {
        let has_err = predicate::str::contains("Context: \x1b[31merr");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"model":{"display_name":"Any Model Name"}}"#)
            .assert()
            .success()
            .stdout(has_err);
    }

    #[test]
    fn context_missing_window_size_shows_err() {
        let has_err = predicate::str::contains("Context: \x1b[31merr");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"context_window":{"current_usage":null}}"#)
            .assert()
            .success()
            .stdout(has_err);
    }

    #[test]
    fn context_invalid_json_shows_err() {
        let has_err = predicate::str::contains("Context: \x1b[31merr");
        let mut cmd = statusline();
        cmd.write_stdin("not valid json")
            .assert()
            .success()
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
    fn usage_refresh_skips_while_lock_is_held() {
        let home = tempfile::tempdir().expect("failed to create temp home");
        let cache_dir = home.path().join(".cache").join("claude-statusline");
        std::fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        let lock_path = cache_dir.join("usage.lock");
        std::fs::write(&lock_path, "").expect("failed to write lock");

        let mut cmd = statusline();
        cmd.env("HOME", home.path())
            .arg("--refresh-usage")
            .assert()
            .success();

        assert!(lock_path.exists());
        assert!(!cache_dir.join("error.log").exists());
        assert!(!cache_dir.join("usage.json").exists());
    }

    #[test]
    fn version_refresh_skips_while_lock_is_held() {
        let home = tempfile::tempdir().expect("failed to create temp home");
        let cache_dir = home.path().join(".cache").join("claude-statusline");
        std::fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        let lock_path = cache_dir.join("version.lock");
        std::fs::write(&lock_path, "").expect("failed to write lock");

        let mut cmd = statusline();
        cmd.env("HOME", home.path())
            .arg("--refresh-version")
            .assert()
            .success();

        assert!(lock_path.exists());
        assert!(!cache_dir.join("error.log").exists());
        assert!(!cache_dir.join("version.json").exists());
    }

    #[test]
    fn error_log_rotates_when_full() {
        let home = tempfile::tempdir().expect("failed to create temp home");
        let cache_dir = home.path().join(".cache").join("claude-statusline");
        std::fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        let log_path = cache_dir.join("error.log");
        let full_log = "x".repeat(2 * 1024 * 1024);
        std::fs::write(&log_path, &full_log).expect("failed to write log");

        let mut cmd = statusline();
        cmd.env("HOME", home.path())
            .current_dir(home.path())
            .write_stdin("{}")
            .assert()
            .success();

        let rotated = std::fs::read_to_string(cache_dir.join("error.log.1"))
            .expect("rotated log should exist");
        let current = std::fs::read_to_string(&log_path).expect("log should exist");
        assert_eq!(rotated.len(), full_log.len());
        assert!(current.contains("PLAN.md not found"));
        assert!(current.len() < full_log.len());
    }

    #[test]
    fn usage_expired_cache_renders_err_without_logging() {
        let home = tempfile::tempdir().expect("failed to create temp home");
        let cache_dir = home.path().join(".cache").join("claude-statusline");
        std::fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        std::fs::write(
            cache_dir.join("usage.json"),
            r#"{
                "fetched_at": "2026-04-29T17:33:41Z",
                "response": {
                    "five_hour": { "utilization": 40.0, "resets_at": "2026-04-29T19:30:00Z" },
                    "seven_day": { "utilization": 60.0, "resets_at": "2026-05-05T18:00:00Z" }
                }
            }"#,
        )
        .expect("failed to write usage cache");
        std::fs::write(cache_dir.join("usage.lock"), "").expect("failed to write lock");

        let has_five_hour_err = predicate::str::contains("5h: \x1b[31merr");
        let has_seven_day_err = predicate::str::contains("7d: \x1b[31merr");
        let mut cmd = statusline();
        cmd.env("HOME", home.path())
            .current_dir(home.path())
            .write_stdin("{}")
            .assert()
            .success()
            .stdout(has_five_hour_err)
            .stdout(has_seven_day_err);

        let log = std::fs::read_to_string(cache_dir.join("error.log")).expect("log should exist");
        assert!(!log.contains("expired"));
    }

    #[test]
    fn effort_renders_level() {
        let has_effort = predicate::str::contains("Thinking: high");
        let mut cmd = statusline();
        cmd.write_stdin(r#"{"effort":{"level":"high"}}"#)
            .assert()
            .success()
            .stdout(has_effort);
    }

    #[test]
    fn effort_missing_field_shows_na() {
        let has_effort = predicate::str::contains("Thinking:");
        let has_na = predicate::str::contains("n/a");
        let mut cmd = statusline();
        cmd.write_stdin("{}")
            .assert()
            .success()
            .stdout(has_effort)
            .stdout(has_na);
    }

    #[test]
    fn full_output_renders_all_components_on_three_lines() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let plan_path = temp_dir.path().join("PLAN.md");
        std::fs::write(&plan_path, "- [x] Done\n- [ ] Todo\n").expect("failed to write PLAN.md");

        let json = r#"{"context_window":{"context_window_size":200000,"current_usage":{"input_tokens":2000,"cache_creation_input_tokens":50000,"cache_read_input_tokens":100000,"output_tokens":5000}},"model":{"display_name":"Opus 4.5"},"effort":{"level":"high"}}"#;

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
            output_lines.len() >= 3,
            "Expected at least 3 lines, got {}",
            output_lines.len()
        );
        let first_line = output_lines.first().expect("expected first line");
        let second_line = output_lines.get(1).expect("expected second line");
        let third_line = output_lines.get(2).expect("expected third line");
        assert!(first_line.contains("Context:"));
        assert!(first_line.contains("Model:"));
        assert!(first_line.contains("5h:"));
        assert!(second_line.contains("PLAN.md:"));
        assert!(second_line.contains("Thinking: high"));
        assert!(second_line.contains("7d:"));
        assert!(third_line.contains("Version:"));
    }
}
