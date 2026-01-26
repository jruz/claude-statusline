# Claude Code Project Notes

## Architecture

TEA (The Elm Architecture): Model → Update → View

- `main.rs`: Entry point, background refresh worker
- `model.rs`: State + update function
- `view.rs`: Render output
- `effect.rs`: Side effects (stdin, file I/O, spawning)

## Components

- `context.rs`: Parse `context_window.used_percentage` from stdin JSON
- `claude_model.rs`: Parse `model.display_name` from stdin JSON
- `plan.rs`: Count `- [x]` / `- [ ]` checkboxes in PLAN.md
- `usage.rs`: Read cached API usage from `~/.cache/claude-statusline/usage.json`

## Usage API

Background refresh fetches from `https://api.anthropic.com/api/oauth/usage` using OAuth token from macOS keychain (`Claude Code-credentials`).

## Commands

- `just check`: Lint + test
- `just demo`: Run with sample data
