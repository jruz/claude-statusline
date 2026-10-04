# Shared Utilities

Utilities used across multiple components.

## Progress Bar

Renders a progress bar given a percentage and width.

### Interface

```rust
pub fn render(percentage: f64, width: usize) -> String
```

### Requirements

- Uses `█` for filled portions
- Uses `░` for empty portions
- `percentage` is 0-100
- Returns string of exact `width` characters
- Clamps percentage to 0-100 range (no panic on out-of-range)

### Examples

| Input | Output |
| --- | --- |
| `render(50.0, 10)` | `█████░░░░░` |
| `render(0.0, 5)` | `░░░░░` |
| `render(100.0, 5)` | `█████` |
| `render(42.0, 10)` | `████░░░░░░` |
| `render(-10.0, 5)` | `░░░░░` |
| `render(150.0, 5)` | `█████` |

### File

`src/components/shared/progress_bar.rs`

---

## Duration Formatting

Formats a future DateTime as human-readable duration until that time.

### Interface

```rust
pub fn format_remaining(target: chrono::DateTime<chrono::Utc>) -> String
```

### Requirements

- `< 1 hour` → `Xm` (e.g., `45m`)
- `< 24 hours` → `Xh Ym` (e.g., `2h 15m`)
- `>= 24 hours` → `Xd Yh` (e.g., `3d 12h`)
- If target is in the past → `0m`

### Examples

| Time until target | Output |
| --- | --- |
| 45 minutes | `45m` |
| 2 hours 15 minutes | `2h 15m` |
| 3 days 12 hours | `3d 12h` |
| -5 minutes (past) | `0m` |
| 0 minutes | `0m` |
| 59 minutes | `59m` |
| 1 hour 0 minutes | `1h 0m` |
| 23 hours 59 minutes | `23h 59m` |
| 24 hours 0 minutes | `1d 0h` |

### File

`src/components/shared/duration.rs`

---

## Grid Padding

Pads text to exact display width, accounting for Unicode and ANSI escape codes.

### Interface

```rust
pub fn pad_right(text: &str, width: usize) -> String
```

### Requirements

- Handles Unicode characters (█░ are 1 display width each)
- Handles ANSI color codes (zero display width)
- Pads with spaces to reach exact display width
- Truncates if text exceeds width (preserve ANSI reset)

### Dependencies

- `console` crate - use `pad_str` with `Alignment::Left` and truncation enabled

### Examples

| Input | Width | Output |
| --- | --- | --- |
| `"hello"` | 10 | `"hello     "` |
| `"█████"` | 10 | `"█████     "` |
| `"hello".red()` | 10 | `"\x1b[31mhello\x1b[0m     "` |
| `"hello world"` | 5 | `"hello"` |

### File

`src/components/shared/grid.rs`

---

## Error Logging

Appends errors to log file.

### Interface

```rust
pub fn log_error(module: &str, error: &str)
```

### Error Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cache directory not found")]
    CacheDirNotFound,

    #[error("Failed to write log entry")]
    WriteEntry(
        #[source]
        std::io::Error,
    ),
}
```

Internal functions return `Result<(), Error>`. The public `log_error` function calls the internal function and silently drops any error.

### Requirements

- Location: `~/.cache/claude-statusline/error.log`
- Format: `{timestamp} [{module}] {error}`
- Timestamp in ISO 8601 UTC format (e.g., `2026-01-25T12:30:45Z`)
- Append-only (create file if missing)
- Create parent directories if needed
- Never panic (silently fail if logging fails)

### Examples

```
2026-01-25T12:30:45Z [context] Missing context_window in stdin
2026-01-25T12:30:45Z [credentials] Failed to read credentials from keychain: No password found
2026-01-25T12:30:46Z [usage] API request failed: connection refused
```

### File

`src/core/logging.rs`

---

## File Structure

```
src/components/shared/
├── mod.rs
├── duration.rs
├── grid.rs
└── progress_bar.rs

src/core/
├── mod.rs
└── logging.rs
```
