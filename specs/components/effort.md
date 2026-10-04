# Effort Component

Displays the current reasoning effort (thinking level).

## Output

```
Thinking: high
```

## Data Source

Received via stdin JSON from Claude Code. Reflects live `/effort` changes.

**Scope:** Per-instance (not cached, not shared)

```json
{
  "effort": {
    "level": "high"
  }
}
```

Values: `low`, `medium`, `high`, `xhigh`, `max`. The `effort` field is absent when the model does not support the effort parameter.

## States

| State | Condition | View |
| --- | --- | --- |
| Available | `effort.level` present in stdin | `Thinking: high` |
| Unsupported | `effort` missing from stdin | `Thinking: n/a` (dimmed) |
| Error | stdin unreadable or invalid JSON | `Thinking: err` (red) |

## Updates

Data arrives via `StdinReceived` message. No caching or background refresh needed.
