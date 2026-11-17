# Testing

## Integration Tests

`tests/integration.rs` runs the binary with `assert_cmd`, pipes JSON through stdin, and asserts on stdout.

- Each test unsets `CLAUDE_CODE_AUTO_COMPACT_WINDOW` so the host environment does not leak in
- Tests that need it set the variable explicitly
- PLAN.md tests run in isolated temp directories

### Coverage

| Area           | Cases                                                                                                                                  |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Context        | Percentage and token formatting, rounding, clamping, color thresholds, null usage, output tokens ignored, missing fields, invalid JSON |
| Context window | `CLAUDE_CODE_AUTO_COMPACT_WINDOW` applied, capped to model window, invalid value shows `err`                                           |
| Model          | Display name, missing field shows `err`                                                                                                |
| Effort         | Level rendered as `Thinking:`, missing field shows `n/a`                                                                               |
| Plan           | Checkbox progress, no checkboxes is 100%, missing file shows `file-not-found`                                                          |
| Usage          | Labels render without cache                                                                                                            |
| Full output    | Three lines: Context, Model, 5h / PLAN.md, Thinking, 7d / Version                                                                      |

## Unit Tests

Inline `#[cfg(test)]` modules cover logic not reachable through stdin:

- `components/usage.rs`: reset time formatting, cache parsing, API fixture parsing, lock staleness
- `components/version.rs`: version parsing, comparison, rendering states
- `shared/grid.rs`: padding, truncation, unicode width

## Fixtures

`tests/fixtures/` holds sample stdin, PLAN.md, and API response data.

## Running Tests

```bash
just test
```
