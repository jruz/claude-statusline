# Claude Statusline

A status line renderer for Claude Code that displays context usage, model info, API rate limits, and plan progress.

## Mockup

```
Context: ━━━━━━━━━━  15% (152k)    Model: Opus 4.6 1M    5h: ━━━━━━━━━━  72% (2h 19m)
PLAN.md: ━━━━━━━━━━  60%           Thinking: high        7d: ━━━━━━━━━━  96% (2d 5h)
Version: 2.1.280
```

When newer Claude Code version available:

```
Version: 2.1.280↑
```

## Error State

```
Context: err                       Model: err            5h: err
PLAN.md: err                       Thinking: err         7d: err
Version: err
```

Note: Version update check fails silently (shows current version without update indicator).

- Failed components display `err` in red
- All errors logged to `~/.cache/claude-statusline/error.log`

## Components

- [context](components/context.md) - Context window usage display
- [claude-model](components/claude-model.md) - Model name display
- [effort](components/effort.md) - Reasoning effort (thinking level) display
- [plan](components/plan.md) - PLAN.md progress display
- [usage](components/usage.md) - API rate limit display
- [version](components/version.md) - Claude Code version with update indicator
- [shared](components/shared.md) - Progress bar, duration, grid, logging utilities

## Documentation

- [architecture](architecture.md) - Execution model, TEA pattern, data flow
- [coding-style](coding-style.md) - Rust conventions
- [testing](testing.md) - Integration tests, mock server
