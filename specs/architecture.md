# Architecture

## Execution Model

One-shot process invoked by Claude Code:

- Called on every conversation message update
- At most every 300ms
- Receives JSON via stdin
- Outputs three-line status to stdout
- Must return fast (render from cache, refresh in background)

## Data Scope

| Data           | Source        | Scope                  | Cached        |
| -------------- | ------------- | ---------------------- | ------------- |
| context_window | stdin         | Per-instance           | No            |
| model          | stdin         | Per-instance           | No            |
| effort         | stdin         | Per-instance           | No            |
| plan           | PLAN.md file  | Per-instance           | No            |
| usage          | API           | Shared (all instances) | Yes (60s TTL) |
| version        | CLI + npm API | Shared (all instances) | Yes (1h TTL)  |

Multiple instances can run simultaneously. Each has independent stdin data, but all share the same caches at `~/.cache/claude-statusline/`.

## Model

Each component's data is wrapped in `Option<Result<T, Error>>`. `None` means not yet loaded, `Some(Ok(..))` is success, `Some(Err(..))` is failure. The view renders all three states.

```rust
Model {
    channel: version::Channel,
    claude: Option<Result<String, claude_model::Error>>,
    compact_window: Result<Option<NonZeroU64>, env::Error>,
    context: Option<Result<context::Context, context::Error>>,
    effort: Option<Result<Option<String>, effort::Error>>,
    plan: Option<Result<plan::Progress, plan::Error>>,
    usage: Option<Result<usage::Usage, usage::Error>>,
    version: Option<version::Version>,
}
```

Component types are defined in each component spec.

## Init

`init(channel, compact_window)` returns the initial model and startup effects.

- `channel` comes from `--channel`, `stable` when omitted
- `compact_window` comes from `CLAUDE_CODE_AUTO_COMPACT_WINDOW`
- All component fields start as `None`

Startup effects:

```rust
[ReadStdin, ReadFile { path: "PLAN.md" }, ReadUsageCache, ReadVersion, ReadVersionCache]
```

## Messages

All data enters the system as messages. No direct mutations. Errors are carried inside the message payload.

```rust
Msg {
    CacheLoaded(Result<String, io::Error>),
    PlanLoaded(Result<String, io::Error>),
    StdinReceived(Result<String, io::Error>),
    VersionCacheLoaded(Result<String, io::Error>),
    VersionLoaded(Result<String, io::Error>),
}
```

## Update

```
Msg                              → Model Update                        → Effects
───────────────────────────────────────────────────────────────────────────────────────
StdinReceived(result)            → Parse context, claude, effort       → LogError per failed parse
PlanLoaded(result)               → Parse checkboxes, set plan          → LogError if failed
CacheLoaded(Ok(content))         → Parse usage cache                   → if stale or invalid: SpawnBackgroundRefresh
CacheLoaded(Err(e))              → Set usage to Err                    → SpawnBackgroundRefresh, LogError unless not found
VersionLoaded(result)            → Set version.current                 → LogError if failed
VersionCacheLoaded(Ok(content))  → Set version.latest                  → if stale or invalid: SpawnVersionRefresh
VersionCacheLoaded(Err(e))       → Leave version.latest None           → SpawnVersionRefresh, LogError unless not found
```

## Effects

```rust
Effect {
    LogError { module: String, error: String },
    ReadFile { path: PathBuf },
    ReadStdin,
    ReadUsageCache,
    ReadVersion,
    ReadVersionCache,
    SpawnBackgroundRefresh,
    SpawnVersionRefresh { channel: version::Channel },
}
```

`execute` returns `Option<Msg>`. Effects that produce messages return `Some(Msg)`, fire-and-forget effects like `LogError` and spawns return `None`.

## Main Flow

```
1. init(channel, compact_window)  → (model, startup effects)
2. Pop and execute effects until the stack is empty
3. Feed each returned Msg to update, push its effects onto the stack
4. Render model to stdout
5. Exit
```

Spawn effects run during step 2, before rendering.

## Background Refresh Flow

The same binary runs in refresh mode:

| Argument                                | Function                 | Spec                             |
| --------------------------------------- | ------------------------ | -------------------------------- |
| `--refresh-usage`                       | `run_background_refresh` | [usage](components/usage.md)     |
| `--refresh-version --channel <channel>` | `run_version_refresh`    | [version](components/version.md) |

Both run sequentially: acquire lock, fetch, write cache, release lock. Errors are logged and release the lock.

### Next Invocation

The next main process invocation reads the fresh cache via `CacheLoaded` and `VersionCacheLoaded`.

## View

Three-line output followed by a zero-width space line:

```
Line 1: {context}              {model}              {5h usage}
Line 2: {plan}                 {effort}             {7d usage}
Line 3: {version}
```

Column widths using `unicode-width` for proper display width handling:

- Column 1: 35 chars (Context, Plan)
- Column 2: 22 chars (Model, Thinking)
- Column 3: unpadded (5h, 7d)

## Boundaries

| System               | Effect                 | Message            |
| -------------------- | ---------------------- | ------------------ |
| Claude Code          | ReadStdin              | StdinReceived      |
| PLAN.md file         | ReadFile               | PlanLoaded         |
| Claude CLI           | ReadVersion            | VersionLoaded      |
| Cache file (usage)   | ReadUsageCache         | CacheLoaded        |
| Cache file (version) | ReadVersionCache       | VersionCacheLoaded |
| Log file             | LogError               | —                  |
| Background (usage)   | SpawnBackgroundRefresh | —                  |
| Background (version) | SpawnVersionRefresh    | —                  |

## Environment Variables

| Variable                          | Description                                                   |
| --------------------------------- | ------------------------------------------------------------- |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | Caps the context window, see [context](components/context.md) |

## File Structure

```
src/
├── main.rs          # Entry point, argument dispatch, background refresh
├── model.rs         # Model, init, update
├── effect.rs        # Effect, Msg, execute
├── view.rs          # Top-level render
├── env.rs           # Environment variable parsing
├── version.rs       # --version output
│
├── shared/
│   ├── mod.rs
│   ├── grid.rs          # Column padding
│   ├── logging.rs       # log_error
│   └── progress_bar.rs  # Progress bar rendering
│
└── components/
    ├── mod.rs
    ├── claude_model.rs  # Model name component
    ├── context.rs       # Context window component
    ├── effort.rs        # Thinking level component
    ├── plan.rs          # PLAN.md progress component
    ├── usage.rs         # API rate limit component
    └── version.rs       # Claude Code version component
```

Components are self-contained: each has its own error enum, parse function, and render function.

## Cache

| Cache   | Location                                  | TTL | Lock           |
| ------- | ----------------------------------------- | --- | -------------- |
| Usage   | `~/.cache/claude-statusline/usage.json`   | 60s | `usage.lock`   |
| Version | `~/.cache/claude-statusline/version.json` | 1h  | `version.lock` |

Stale cache is rendered while the background refreshes. Locks are treated as stale after 300s.

## Error Handling

- Modules are independent: one failure doesn't affect others
- Failed components render instead of crashing
- Invalid `--channel` exits with an error

Rendering by state:

- `None` → `...`
- `Some(Ok(value))` → normal display
- `Some(Err(_))` → `err` in red
- Version update check fails silently (shows current version only)

### Logging

- Location: `~/.cache/claude-statusline/error.log`
- Format: `{local timestamp} [{module}] {error}`
- Append-only

Example:

```
2026-01-25T12:30:45+01:00 [usage] credentials: keychain failed: No password found
```
