# claude-statusline

A terminal status line for Claude Code sessions.

<img src="screenshot.png" width="600" alt="Screenshot of claude-statusline showing context, model, plan progress, and API usage">

## Install

```bash
cargo install --path .
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

- **Context**: Current context window usage percentage
- **Model**: Active Claude model name
- **Version**: Claude Code version with update indicator
- **PLAN.md**: Progress on tasks in PLAN.md
- **5h/7d**: API rate limit usage with time until reset

## How It Works

### Data Sources

| Component | Source |
|-----------|--------|
| Context | `context_window.used_percentage` from stdin JSON |
| Model | `model.display_name` from stdin JSON |
| Version | `claude --version` + npm registry for update check |
| PLAN.md | Counts `- [x]` and `- [ ]` checkboxes in `./PLAN.md` |
| 5h/7d | Anthropic API (`/api/oauth/usage`) via background fetch |

Claude Code pipes session JSON to stdin when invoking the statusline command.

### Credentials

OAuth token is read from macOS Keychain using the `security` command:

```bash
security find-generic-password -s "Claude Code-credentials" -w
```

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

[0BSD](LICENSE)
