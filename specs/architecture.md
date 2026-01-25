# Architecture

## Execution Model

One-shot process invoked by Claude Code:
- Called on every conversation message update
- At most every 300ms
- Receives JSON via stdin
- Outputs two-line status to stdout
- Must return fast (render from cache, refresh in background)

## Data Scope

| Data | Source | Scope | Cached |
|------|--------|-------|--------|
| context_window | stdin | Per-instance | No |
| model | stdin | Per-instance | No |
| plan | PLAN.md file | Per-instance | No |
| usage | API | Shared (all instances) | Yes |

Multiple instances can run simultaneously. Each has independent stdin data, but all share the same usage cache at `~/.cache/claude-statusline/usage.json`.

## Model

Each component's data is wrapped in `Option<Result<T, Error>>`. `None` means not yet loaded, `Some(Ok(..))` is success, `Some(Err(..))` is failure. The view renders all three states.

```rust
Model {
    context: Option<Result<f64, context::Error>>,
    claude: Option<Result<String, claude::Error>>,
    plan: Option<Result<Progress, plan::Error>>,
    usage: Option<Result<Usage, usage::Error>>,
}

Progress {
    completed: usize,
    total: usize,
}

Usage {
    five_hour: Window,
    seven_day: Window,
}

Window {
    utilization: f64,
    resets_at: DateTime<Utc>,
}
```

Cache structure (separate from model):

```rust
CachedUsage {
    fetched_at: DateTime
    response: Response { five_hour: Window, seven_day: Window }
}
```

## Init

Returns the initial model and startup effects.

```rust
fn init() -> (Model, Vec<Effect>) {
    let model = Model { context: None };
    let effects = vec![Effect::ReadStdin];
    (model, effects)
}
```

All component fields start as `None` until populated by messages.

## Messages

All data enters the system as messages. No direct mutations. Errors are carried inside the message payload.

```rust
Msg {
    StdinReceived(Result<String, io::Error>),
    PlanLoaded(Result<String, io::Error>),
    CacheLoaded(Result<CachedUsage, cache::Error>),
}
```

## Update

```
Msg                              → Model Update                      → Effects
─────────────────────────────────────────────────────────────────────────────────
StdinReceived(Ok(json))          → Parse and set context/model       → None
StdinReceived(Err(e))            → Set context/model to Err          → LogError
PlanLoaded(Ok(content))          → Parse checkboxes, set plan        → None
PlanLoaded(Err(e))               → Set plan to Err                   → LogError
CacheLoaded(Ok(cached))          → Set usage from cached.response    → if stale: SpawnBackgroundRefresh
CacheLoaded(Err(e))              → Set usage to Err                  → SpawnBackgroundRefresh, LogError
```

## Effects

```rust
Effect {
    LogError { module: String, error: String },
    ReadStdin,
    ReadPlan,
    LoadCache,
    SpawnBackgroundRefresh,
}
```

`execute` returns `Option<Msg>`. Effects that produce messages return `Some(Msg)`, fire-and-forget effects like `LogError` return `None`.

## Main Flow

```
1. Call init()             → (model, [ReadStdin, ReadPlan, LoadCache])
2. Execute init effects    → [StdinReceived(..), PlanLoaded(..), CacheLoaded(..)]
3. Update model from messages, collect effects
4. Render model to stdout
5. Execute update effects (SpawnBackgroundRefresh if needed, LogError if any)
6. Exit
```

## Background Refresh Flow

The background process follows TEA with its own model, messages, and effects.

### Background Model

```rust
struct BackgroundModel {
    api_key: Option<String>,
    response: Option<Response>,
}
```

### Background Messages

```rust
enum BackgroundMsg {
    LockAcquired(Result<(), cache::Error>),
    KeychainResult(Result<String, credentials::Error>),
    UsageFetched(Result<Response, api::Error>),
    CacheSaved(Result<(), cache::Error>),
}
```

### Background Effects

```rust
enum BackgroundEffect {
    AcquireLock,
    ReadKeychain,
    FetchUsage(String),
    SaveCache(Response),
    ReleaseLock,
    LogError(Error),
    Exit,
}
```

### Background Update

```
Msg                                → Model Update         → Effects
──────────────────────────────────────────────────────────────────────────────
(init)                             → Empty model          → AcquireLock
LockAcquired(Ok(()))               → —                    → ReadKeychain
LockAcquired(Err(LockHeld))        → —                    → Exit
KeychainResult(Ok(key))            → Store api_key        → FetchUsage(key)
KeychainResult(Err(e))             → —                    → LogError, ReleaseLock, Exit
UsageFetched(Ok(response))         → Store response       → SaveCache(response)
UsageFetched(Err(e))               → —                    → LogError, ReleaseLock, Exit
CacheSaved(Ok(()))                 → —                    → ReleaseLock, Exit
CacheSaved(Err(e))                 → —                    → LogError, ReleaseLock, Exit
```

### Next Invocation

Within 300ms, the next main process invocation reads fresh cache data via `CacheLoaded` message.

## View

Two-line output with optional error line:

```
Render(model) ──▶ stdout

Line 1: Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: ███░░ 60% (2h 15m)
Line 2: Plan: ████░░░░░░ 40% (12 left)               7d: ██░░░ 40% (3d 12h)
Line 3: Errors(N): {last error}                      (only if errors > 0)
```

Failed components show `err` in red.

## Grid

Two-line layout with fixed-width columns:

```
Line 1: {context}              {model}              {5h usage}
Line 2: {plan}                                      {7d usage}
Line 3: Errors(N): {last error}                     (only if errors > 0)
```

Column widths using `unicode-width` for proper display width handling:
- Column 1: 25 chars (Context, Plan)
- Column 2: 20 chars (Model, empty)
- Column 3: 30 chars (5h, 7d)

## Boundaries

### Main Process

| System | Effect | Message |
|--------|--------|---------|
| Claude Code | ReadStdin | StdinReceived |
| PLAN.md file | ReadPlan | PlanLoaded |
| Cache file | LoadCache | CacheLoaded |
| Terminal | Render | — |
| Log file | LogError | — |
| Background | SpawnBackgroundRefresh | — |

### Background Process

| System | Effect | Message |
|--------|--------|---------|
| Lock file | AcquireLock | LockAcquired |
| Lock file | ReleaseLock | — |
| macOS Keychain | ReadKeychain | KeychainResult |
| Anthropic API | FetchUsage | UsageFetched |
| Cache file | SaveCache | CacheSaved |
| Log file | LogError | — |

## Environment Variables

For testing, external dependencies can be overridden:

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_URL` | `https://api.anthropic.com` | API base URL |
| `ANTHROPIC_API_KEY` | (none) | API key value |
| `CLAUDE_STATUSLINE_CACHE_DIR` | `~/.cache/claude-statusline` | Cache directory |
| `CLAUDE_STATUSLINE_SKIP_KEYCHAIN` | `false` | Skip keychain, use `ANTHROPIC_API_KEY` |

Credential resolution:
1. If `CLAUDE_STATUSLINE_SKIP_KEYCHAIN=true`, use `ANTHROPIC_API_KEY` env var
2. Otherwise, read from macOS Keychain

## File Structure

```
src/
├── main.rs          # TEA runtime: Effect, Model, Msg, init, update, execute
├── view.rs          # Top-level render
│
├── shared/
│   ├── mod.rs
│   ├── logging.rs       # log_error
│   └── progress_bar.rs  # progress_bar
│
└── components/
    ├── mod.rs
    ├── context.rs       # Context window component
    ├── claude_model.rs  # Model name component
    ├── plan.rs          # PLAN.md progress component
    └── usage.rs         # API rate limit component
```

### Components

Self-contained units. Each component has its own error enum, parse function, and render function.

### Shared

`shared/` contains utilities used across components:
- `logging` - error logging to file
- `progress_bar` - progress bar rendering

## Cache

- Location: `~/.cache/claude-statusline/usage.json`
- TTL: 60 seconds
- Stale cache used for render while background refreshes
- Lock file prevents concurrent refresh: `~/.cache/claude-statusline/usage.lock`

## Error Handling

### Principles

- Application never crashes
- Always output something to stdout
- Always exit 0
- Modules are independent: one failure doesn't affect others

### Module Errors

Each module defines its own error enum:

```rust
mod input {
    enum Error { ReadStdin(io::Error), ParseJson(serde_json::Error) }
}

mod cache {
    enum Error { Read(io::Error), Write(io::Error), Parse(serde_json::Error) }
}

mod credentials {
    enum Error { KeychainRead(io::Error), Parse(serde_json::Error) }
}

mod usage {
    enum Error { Request(reqwest::Error), Parse(serde_json::Error) }
}
```

### Error Propagation

```
Data Source → Option<Result<T, Error>> → Model field → View renders loading, success, or error
```

Each component's model holds `Option<Result<T, Error>>`. View pattern matches to render:
- `None` → `...` (loading)
- `Some(Ok(value))` → normal display
- `Some(Err(_))` → `err` in red

Errors are collected and logged. If any errors occurred, third line shows error count and last error message.

### View with Errors

```
Context: ████░░░░░░ 42%    Model: Opus 4.5    5h: err
Plan: err                                    7d: ██░░░ 40% (3d 12h)
Errors(2): API request failed
```

- Failed component shows `err` in red
- Third line shows error count and last error message (only if errors > 0)

### Logging

- Location: `~/.cache/claude-statusline/error.log`
- Format: `{timestamp} [{module}] {error}: {source}`
- Append-only

Example:
```
2026-01-25T12:30:45Z [credentials] Failed to read credentials from keychain: No password found
```
