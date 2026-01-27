# Plan Component

Displays progress of PLAN.md todos as a progress bar.

## Output

```
Plan: ████░░░░░░ 40%
```

## Data Source

Reads `PLAN.md` from current working directory.

**Scope:** Per-instance (not cached, not shared)

Parses markdown checkboxes:

- `- [ ]` = incomplete
- `- [x]` or `- [X]` = complete

## Model

```rust
struct Plan {
    progress: Result<Progress, Error>,
}

struct Progress {
    completed: usize,
    total: usize,
}

enum Error {
    FileNotFound,
    ReadError(io::Error),
}
```

## States

| State | Condition | View |
|-------|-----------|------|
| Available | PLAN.md exists and has todos | `Plan: ████░░░░░░ 40%` |
| No todos | PLAN.md exists but no checkboxes | `Plan: ██████████ 100%` |
| File missing | PLAN.md not found | `Plan: err` (red) |
| Read error | Failed to read file | `Plan: err` (red) |

## Error Handling

If PLAN.md is missing or unreadable, display `err` in red.

Error logged to `~/.cache/claude-statusline/error.log`:

```
2026-01-25T12:30:45Z [plan] PLAN.md not found in current directory
```

## Updates

Data arrives via `PlanLoaded` message. The update function parses the markdown content for checkboxes and sets `progress` to `Ok(Progress)` or `Err(error)`.

No caching or background refresh needed.

## View

```rust
fn render(plan: &Plan) -> String {
    match &plan.progress {
        Ok(p) => {
            let pct = if p.total == 0 { 100.0 } else { (p.completed as f64 / p.total as f64) * 100.0 };
            format!("Plan: {} {:.0}%", progress_bar(pct, 10), pct)
        }
        Err(_) => format!("Plan: {}", "err".red()),
    }
}
```

View is responsible for rendering both success and error states.

Progress bar width: 10 characters.

## File Structure

```
components/plan/
├── mod.rs
├── model.rs    # Plan, Progress, Error
└── view.rs     # render function
```
