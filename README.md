# claude-statusline

A terminal status line for Claude Code sessions.

<img src="screenshot.png" width="600" alt="Screenshot of claude-statusline showing context, model, plan progress, and API usage">

## Install from releases

Download the binary for your platform from [the latest release](https://github.com/jruz/claude-statusline/releases/latest):

| Platform            | Download                                                                                                                    |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Linux x86_64        | [Download](https://github.com/jruz/claude-statusline/releases/latest/download/claude-statusline-unknown-linux-musl-x86_64)  |
| Linux ARM64         | [Download](https://github.com/jruz/claude-statusline/releases/latest/download/claude-statusline-unknown-linux-musl-aarch64) |
| macOS Intel         | [Download](https://github.com/jruz/claude-statusline/releases/latest/download/claude-statusline-apple-darwin-x86_64)        |
| macOS Apple Silicon | [Download](https://github.com/jruz/claude-statusline/releases/latest/download/claude-statusline-apple-darwin-aarch64)       |

For example, on Linux x86_64:

```sh
curl --fail --location --output claude-statusline \
  https://github.com/jruz/claude-statusline/releases/latest/download/claude-statusline-unknown-linux-musl-x86_64
chmod +x claude-statusline
./claude-statusline --version
```

Move the binary into a directory on your `PATH` to run it from anywhere.

## Install with Nix

Requires Nix with flakes enabled.

```sh
nix profile add github:jruz/claude-statusline
```

Installs a published binary without compiling Rust. Supports Linux and macOS on x86_64 and aarch64.

Verify the installation:

```sh
claude-statusline --version
```

Upgrade:

```sh
nix profile upgrade claude-statusline
```

Run without installing:

```sh
nix run github:jruz/claude-statusline -- --help
```

Build from source instead:

```sh
nix build github:jruz/claude-statusline#source
```

## Configuration

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.cargo/bin/claude-statusline"
  }
}
```

## Components

- **Context**: Context window usage percentage and tokens used
- **Model**: Active Claude model name
- **Version**: Claude Code version with update indicator
- **PLAN.md**: Progress on tasks in PLAN.md
- **5h/7d**: API rate limit remaining with time until reset

## How It Works

### Data Sources

| Component | Source                                                                                                                                                      |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Context   | `context_window.current_usage` (input + cache tokens) and `context_window.context_window_size` from stdin JSON, capped by `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| Model     | `model.display_name` from stdin JSON                                                                                                                        |
| Version   | `claude --version` + npm registry for update check                                                                                                          |
| PLAN.md   | Counts `- [x]` and `- [ ]` checkboxes in `./PLAN.md`                                                                                                        |
| 5h/7d     | Anthropic API (`/api/oauth/usage`) via background fetch                                                                                                     |

Claude Code pipes session JSON to stdin when invoking the statusline command.

### Credentials

OAuth token is read from `~/.claude/.credentials.json` (Linux), with macOS Keychain as fallback.

### Cache

Data is fetched in the background and cached at:

```
~/.cache/claude-statusline/usage.json    # API usage, 60s TTL
~/.cache/claude-statusline/version.json  # Latest version, 1h TTL
```

Lock files prevent concurrent fetches:

```
~/.cache/claude-statusline/usage.lock
~/.cache/claude-statusline/version.lock
```

### Logs

Errors are appended to:

```
~/.cache/claude-statusline/error.log
```

## License

Licensed under the EUPL.
