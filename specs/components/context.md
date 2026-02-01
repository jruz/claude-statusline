# Context Component

Displays context window usage as a progress bar with percentage.

## Output

```
Context: ████░░░░░░ 42%
```

## Progress Bar Colors

| Range | Color |
|-------|-------|
| 0-39% | Green (default) |
| 40-59% | Orange |
| 60%+ | Red |

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
Option<Result<f64, Error>>

enum Error {
    MissingField,
    ParseJson,
    ReadStdin,
}
```

## States

| State | Condition | View |
|-------|-----------|------|
| Loading | Effect not yet executed (`None`) | `Context: ...` |
| Available | `used_percentage` present in stdin (`Some(Ok(...))`) | `Context: ████░░░░░░ 42%` |
| Unavailable | Read, parse, or field error (`Some(Err(...))`) | `Context: err` (red) |

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

Renders progress bar with percentage. Progress bar width: 10 characters.

Bar color based on thresholds. Percentage text colored only for yellow and red ranges (default color for green).

## File Structure

```
components/context.rs    # Error enum, parse, render
```
