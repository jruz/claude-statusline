# Claude Model Component

Displays the current Claude model name.

## Output

```
Model: Opus 4.5
```

## Data Source

Received via stdin JSON from Claude Code.

**Scope:** Per-instance (not cached, not shared)

```json
{
  "model": {
    "display_name": "Opus 4.5"
  }
}
```

## Model

```rust
struct Model {
    display_name: Result<String, Error>,
}

enum Error {
    MissingField,
}
```

## States

| State | Condition | View |
| --- | --- | --- |
| Available | `display_name` present in stdin | `Model: Opus 4.5` |
| Unavailable | Field missing from stdin | `Model: err` (red) |

## Error Handling

If the field is missing from stdin, display `err` in red in place of the component.

Error logged to `~/.cache/claude-statusline/error.log`:

```
2026-01-25T12:30:45Z [claude_model] Missing model in stdin
```

## Updates

Data arrives via `StdinReceived` message. The update function parses the JSON and sets `display_name` to `Ok(value)` or `Err(MissingField)`.

No caching or background refresh needed.

## View

```rust
fn render(model: &Model) -> String {
    match &model.display_name {
        Ok(name) => format!("Model: {name}"),
        Err(_) => format!("Model: {}", "err".red()),
    }
}
```

View is responsible for rendering both success and error states.

## File Structure

```
components/claude_model/
├── mod.rs
├── model.rs    # Model struct
└── view.rs     # render function
```
