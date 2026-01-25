# Context Component

Displays context window usage as a progress bar with percentage.

## Output

```
Context: ████░░░░░░ 42%
```

## Data Source

Received via stdin JSON from Claude Code.

**Scope:** Per-instance (not cached, not shared)

```json
{
  "context_window": {
    "used_percentage": 42.5
  }
}
```

## Model

```rust
struct ContextWindow {
    used_percentage: Result<f64, Error>,
}

enum Error {
    MissingField,
}
```

## States

| State | Condition | View |
|-------|-----------|------|
| Available | `used_percentage` present in stdin | `Context: ████░░░░░░ 42%` |
| Unavailable | Field missing from stdin | `Context: err` (red) |

## Error Handling

If the field is missing from stdin, display `err` in red in place of the component.

Error logged to `~/.cache/claude-statusline/error.log`:
```
2026-01-25T12:30:45Z [context] Missing context_window in stdin
```

## Updates

Data arrives via `StdinReceived` message. The update function parses the JSON and sets `used_percentage` to `Ok(value)` or `Err(MissingField)`.

No caching or background refresh needed.

## View

```rust
fn render(context: &ContextWindow) -> String {
    match &context.used_percentage {
        Ok(pct) => format!("Context: {} {:.0}%", progress_bar(*pct, 10), pct),
        Err(_) => format!("Context: {}", "err".red()),
    }
}
```

View is responsible for rendering both success and error states.

Progress bar width: 10 characters.

## File Structure

```
components/context/
├── mod.rs
├── model.rs    # ContextWindow struct
└── view.rs     # render function
```
