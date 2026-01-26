# claude-statusline

A terminal status line for Claude Code sessions.

## Install

```bash
cargo install --path .
```

## Usage

Pipe Claude Code session JSON to stdin:

```bash
echo '{"context_window":{"used_percentage":42},"model":{"display_name":"Opus 4.5"}}' | claude-statusline
```

Output:
```
Context: ━━━━━━━━━━  42% Model: Opus 4.5     5h: ━━━━━━━━━━  28% (1h 19m)
PLAN.md: ━━━━━━━━━━   0%                     7d: ━━━━━━━━━━  54% (3d 21h)
```

## Components

- **Context**: Current context window usage percentage
- **Model**: Active Claude model name
- **PLAN.md**: Progress on tasks in PLAN.md (checkboxes)
- **5h/7d**: API rate limit usage with time until reset

## License

0BSD
