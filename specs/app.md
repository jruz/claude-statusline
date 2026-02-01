# Claude Statusline

A status line renderer for Claude Code that displays context usage, model info, API rate limits, and plan progress.

## Mockup

```
Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: ███░░ 60% (2h 15m)
Plan: ████░░░░░░ 40% (12 left)  v1.0.17      7d: ██░░░ 40% (3d 12h)
```

When newer Claude Code version available:

```
Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: ███░░ 60% (2h 15m)
Plan: ████░░░░░░ 40% (12 left)  v1.0.17 ↑    7d: ██░░░ 40% (3d 12h)
```

## Error State

```
Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: err
Plan: err                       v1.0.17      7d: err
Errors(3): API request failed
```

Note: Version update check fails silently (shows current version without update indicator).

- Failed components display `err` in red
- Third line shows error count and last error message (only if errors > 0)
- All errors logged to `~/.cache/claude-statusline/error.log`

## Components

- [context](components/context.md) - Context window usage display
- [claude-model](components/claude-model.md) - Model name display
- [plan](components/plan.md) - PLAN.md progress display
- [usage](components/usage.md) - API rate limit display
- [version](components/version.md) - Claude Code version with update indicator
- [shared](components/shared.md) - Progress bar, duration, grid, logging utilities

## Documentation

- [architecture](architecture.md) - Execution model, TEA pattern, data flow
- [coding-style](coding-style.md) - Rust conventions
- [testing](testing.md) - Integration tests, mock server
