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

- `just setup`: Configure git hooks
- `just check`: Run lint and format checks
- `just test`: Run Rust tests
- `just tools::check`: Lint Nix, Nushell, Woodpecker and Renovate config
- `just demo`: Run with sample data (builds musl debug binary, uses real cache/version)
- `--debug`: Log raw stdin payload to `~/.cache/claude-statusline/error.log`
- `just release build <target>`: Cross-compile one target triple, or `all`, with persistent Podman caches
- `just worktree-add <name>`: Create a new worktree with branch `<name>` at `../<name>`
- `just worktree-remove <name>`: Remove worktree and delete branch `<name>`
- `just readme-screenshot`: Regenerate `screenshot.png` for the README

## UI Changes

When a change affects rendered output, run `just readme-screenshot`.

## Template Sync

Shared tooling follows `jruz/template-rs`. `template-sync.json` records the synced revision.

- `template-sync check`: Show pending template changes
- `template-sync pull`: Apply template changes

## GitHub Settings

CLI: `github-settings`. Config: `github-settings.json`. Manages repo description, topics, labels, merge config, branch protection.

- `github-settings plan`: Preview changes
- `github-settings push`: Apply changes

## Woodpecker Settings

CLI: `woodpecker-settings`. Config: `woodpecker-settings.yaml`. Manages CI secrets, permissions, timeouts.

- `woodpecker-settings plan`: Preview changes
- `woodpecker-settings apply`: Apply changes

## Release

- `just release bump <version>` updates Cargo manifests and commits the version bump.
- Push the bump to `main`; CI handles tagging and publishing.
- `just release delete <tag>` deletes a published release and its tag.

## Release Pipeline

1. `.woodpecker/check.yml` runs `just check` and `just test`
2. `.woodpecker/release.yml` runs `just release publish` when `release.json` is behind the `Cargo.toml` version
3. `publish` builds Linux and macOS binaries into a draft release, commits `release.json` from the draft assets, then publishes tag `v<VERSION>` on that commit
