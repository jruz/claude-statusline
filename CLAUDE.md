# Claude Code Project Notes

## Architecture

TEA (The Elm Architecture): Model → Update → View

- `main.rs`: Entry point, background refresh worker
- `model.rs`: State + update function
- `view.rs`: Render output
- `effect.rs`: Side effects (stdin, file I/O, spawning)

## Components

- `components/context.rs`: Sum `context_window.current_usage` input/cache tokens, percentage of `context_window.context_window_size` from stdin JSON
- `components/claude_model.rs`: Parse `model.display_name` from stdin JSON
- `components/effort.rs`: Parse `effort.level` from stdin JSON (rendered as `Thinking:`)
- `components/plan.rs`: Count `- [x]` / `- [ ]` checkboxes in PLAN.md
- `components/usage.rs`: Read cached API usage from `~/.cache/claude-statusline/usage.json`, rendered as remaining percentage
- `components/version.rs`: Run `claude --version`, check npm registry for updates

## Usage API

Background refresh fetches from `https://api.anthropic.com/api/oauth/usage` using OAuth token from macOS keychain (`Claude Code-credentials`).

## Version API

Current version from `claude --version`. Latest version fetched from `https://registry.npmjs.org/@anthropic-ai/claude-code/stable` (1h cache).

## Commands

- `just check`: Run lint and format checks
- `just test`: Run Rust tests
- `just demo`: Run with sample data (builds debug binary, uses real cache/version)
- `--debug`: Log raw stdin payload to `~/.cache/claude-statusline/error.log`
- `just release-build <target>`: Build one target triple from `scripts/release-build.nu` using persistent Podman caches
- `just release-build-all`: Build all release targets
- `just release-upload <tag>`: Upload binaries to Codeberg release
- `just release-delete <tag>`: Delete local tag, remote tag and Codeberg release
- `just worktree-add <name>`: Create a new worktree with branch `<name>` at `../<name>`
- `just worktree-remove <name>`: Remove worktree and delete branch `<name>`
- `just readme-screenshot`: Regenerate `screenshot.png` for the README

## UI Changes

When a change affects rendered output, run `just readme-screenshot`.

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
