# Claude Code Project Notes

## Architecture

TEA (The Elm Architecture): Model → Update → View

- `main.rs`: Entry point, background refresh worker
- `model.rs`: State + update function
- `view.rs`: Render output
- `effect.rs`: Side effects (stdin, file I/O, spawning)

## Components

- `components/context.rs`: Sum `context_window.current_usage` input/cache tokens from stdin JSON (matches Claude Code's `/context`)
- `components/claude_model.rs`: Parse `model.display_name` from stdin JSON
- `components/plan.rs`: Count `- [x]` / `- [ ]` checkboxes in PLAN.md
- `components/usage.rs`: Read cached API usage from `~/.cache/claude-statusline/usage.json`
- `components/version.rs`: Run `claude --version`, check npm registry for updates

## Usage API

Background refresh fetches from `https://api.anthropic.com/api/oauth/usage` using OAuth token from macOS keychain (`Claude Code-credentials`).

## Version API

Current version from `claude --version`. Latest version fetched from `https://registry.npmjs.org/@anthropic-ai/claude-code/stable` (1h cache).

## Commands

- `just check`: Lint + test
- `just demo`: Run with sample data (builds debug binary, uses real cache/version)
- `--debug`: Log raw stdin payload to `~/.cache/claude-statusline/error.log`
- `just build-linux`: Release build for Linux x86_64
- `just build-macos`: Release build for macOS x86_64 + ARM64
- `just release-upload <tag>`: Upload binaries to Codeberg release
- `just worktree-add <name>`: Create a new worktree with branch `<name>` at `../<name>`
- `just worktree-remove <name>`: Remove worktree and delete branch `<name>`

## Codeberg Settings

CLI: `codeberg-settings`. Config: `codeberg-settings.toml`. Manages repo description, topics, labels, merge config, releases.

- `codeberg-settings plan`: Preview changes
- `codeberg-settings apply`: Apply changes

## Woodpecker Settings

CLI: `woodpecker-settings`. Config: `woodpecker-settings.yaml`. Manages CI secrets, permissions, timeouts.

- `woodpecker-settings plan`: Preview changes
- `woodpecker-settings apply`: Apply changes

## Release Pipeline

1. Bump version in `Cargo.toml`, push to main
2. `.woodpecker/check.yml` runs `just check`, then `.woodpecker/tag.yml` AUTO_TAG extracts version and pushes `v<VERSION>` tag
3. `.woodpecker/release.yml` triggers on tag: builds Linux and macOS binaries
4. `scripts/release-upload.ts` uploads binaries to Codeberg release
