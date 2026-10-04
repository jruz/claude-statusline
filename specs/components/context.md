# Context Component

Displays context window usage as a progress bar with percentage and tokens used.

## Output

```
Context: ━━━━━━━━━━  15% (152k)
```

## Data Source

Received via stdin JSON from Claude Code.

**Scope:** Per-instance (not cached, not shared)

```json
{
  "context_window": {
    "context_window_size": 1000000,
    "current_usage": {
      "input_tokens": 2000,
      "cache_creation_input_tokens": 50000,
      "cache_read_input_tokens": 100000
    }
  }
}
```

## Calculation

- Tokens = `input_tokens + cache_creation_input_tokens + cache_read_input_tokens` (output tokens excluded, matches Claude Code's `/context`)
- `current_usage: null` (session start, before first message) = 0 tokens
- Percentage used = `tokens * 100 / context_window_size`, rounded to nearest integer, clamped to 100
- Tokens rounded to nearest unit (`k` or `m`), no decimals
- Bar and percentage show usage (filled = used), not remaining

## Model

```rust
Option<Result<Context, Error>>

struct Context {
    percentage: u8,
    tokens: u64,
}

enum Error {
    MissingContextWindow,
    MissingWindowSize,
    ParseJson,
    ReadStdin,
}
```

## States

| State | Condition | View |
| --- | --- | --- |
| Loading | Effect not yet executed (`None`) | `Context: ...` |
| Available | `context_window_size` present (`Some(Ok(...))`) | `Context: ━━━━━━━━━━  15% (152k)` |
| Unavailable | Read, parse, or missing field (`Some(Err(...))`) | `Context: err` (red) |

## Error Handling

If `context_window` or `context_window_size` is missing from stdin, display `err` in red in place of the component.

Error logged to `~/.cache/claude-statusline/error.log`:

```
2026-01-25T12:30:45Z [context] Missing context_window_size in stdin
```

## Updates

Data arrives via `StdinReceived` message. No caching or background refresh needed.

## View

`Context: {bar} {percentage} ({tokens})`

- Bar width: 10 characters
- Percentage right-aligned to 3 characters (e.g. `4%`, `76%`, `100%`)
- Tokens formatting:

| Tokens | Format |
| --- | --- |
| < 1,000 | `850` |
| < 1,000,000 | `152k` |
| ≥ 1,000,000 | `1m` |

## Bar Colors

Bar color based on percentage. Text uses default color.

| Range | Color |
| --- | --- |
| 0-49% | Green |
| 50-69% | Orange |
| 70%+ | Red |

## File Structure

```
components/context.rs    # Error enum, parse, render
```
