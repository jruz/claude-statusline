# Claude Statusline

A status line renderer for Claude Code that displays context usage, model info, API rate limits, and plan progress.

## Mockup

```
Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: ███░░ 60% (2h 15m)
Plan: ████░░░░░░ 40% (12 left)               7d: ██░░░ 40% (3d 12h)
```

## Error State

```
Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: err
Plan: err                                    7d: err
Errors(3): API request failed
```

- Failed components display `err` in red
- Third line shows error count and last error message (only if errors > 0)
- All errors logged to `~/.cache/claude-statusline/error.log`

## Components

- [context](components/context.md) - Context window usage display
- [claude-model](components/claude-model.md) - Model name display
- [plan](components/plan.md) - PLAN.md progress display
- [usage](components/usage.md) - API rate limit display
- [shared](components/shared.md) - Progress bar, duration, grid, logging utilities

## Documentation

- [architecture](architecture.md) - Execution model, TEA pattern, data flow
- [coding-style](coding-style.md) - Rust conventions
- [testing](testing.md) - Integration tests, mock server
