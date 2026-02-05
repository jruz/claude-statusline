# Claude Code Project Notes

## Architecture

TEA (The Elm Architecture): Model → Update → View

- `main.rs`: Entry point, background refresh worker
- `model.rs`: State + update function
- `view.rs`: Render output
- `effect.rs`: Side effects (stdin, file I/O, spawning)

## Components

- `components/context.rs`: Parse `context_window.used_percentage` from stdin JSON
- `components/claude_model.rs`: Parse `model.display_name` from stdin JSON
- `components/plan.rs`: Count `- [x]` / `- [ ]` checkboxes in PLAN.md
- `components/usage.rs`: Read cached API usage from `~/.cache/claude-statusline/usage.json`
- `components/version.rs`: Run `claude --version`, check npm registry for updates

## Usage API

Background refresh fetches from `https://api.anthropic.com/api/oauth/usage` using OAuth token from macOS keychain (`Claude Code-credentials`).

## Version API

Current version from `claude --version`. Latest version fetched from `https://registry.npmjs.org/@anthropic-ai/claude-code/latest` (1h cache).

## Commands

- `just check`: Lint + test
- `just demo`: Run with sample data
