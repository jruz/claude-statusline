# Version Component

Displays Claude Code version and indicates when a newer version is available.

## Output

```
v1.0.17
```

When newer version available:

```
v1.0.17 ↑
```

## Data Source

### Current Version

Run `claude --version` which outputs:

```
1.0.17 (Claude Code)
```

Parse the version number before the space.

### Latest Version

**Scope:** Shared across all instances (cached at `~/.cache/claude-statusline/version.json`)

Fetched from npm registry:

```
GET https://registry.npmjs.org/@anthropic-ai/claude-code/stable
```

Response:

```json
{
  "version": "1.0.18"
}
```

### Cache

Location: `~/.cache/claude-statusline/version.json`

```json
{
  "fetched_at": "2026-01-25T12:30:00Z",
  "current_version": "1.0.17",
  "latest_version": "1.0.18"
}
```

TTL: 1 hour (3600 seconds)

## Model

```rust
struct Version {
    current: Option<Result<String, CurrentError>>,
    latest: Option<Result<String, LatestError>>,
}

enum CurrentError {
    CommandFailed(String),
    ParseFailed,
}

enum LatestError {
    CacheParse(String),
}
```

Both fields follow the standard TEA pattern:

- `None` = loading (not yet received)
- `Some(Ok(..))` = success
- `Some(Err(..))` = error

Cache structure (separate from model):

```rust
struct CachedVersion {
    fetched_at: DateTime<Utc>,
    current_version: String,
    latest_version: String,
}
```

## States

| State | Condition | View |
| --- | --- | --- |
| Loading | `current` is `None` | `v...` |
| Current only | `current` is `Some(Ok)`, `latest` is `None` | `v1.0.17` |
| Up to date | `current` is `Some(Ok)`, `latest` is `Some(Ok)`, current >= latest | `v1.0.17` |
| Update available | `current` is `Some(Ok)`, `latest` is `Some(Ok)`, current < latest | `v1.0.17 ↑` (arrow in yellow) |
| Current error | `current` is `Some(Err)` | `v: err` (red) |
| Latest error | `current` is `Some(Ok)`, `latest` is `Some(Err)` | `v1.0.17` (no indicator, fail silent) |

## Error Handling

Current version errors show `err` in red. Latest version errors are silent (shows current version without update indicator).

Errors logged to `~/.cache/claude-statusline/error.log`:

```
2026-01-25T12:30:45Z [version] claude --version failed: command not found
2026-01-25T12:30:45Z [version] npm API request failed: connection refused
```

### Current Version Errors

| Error | Cause | Behavior |
| --- | --- | --- |
| `CommandFailed` | `claude` command not found or failed | Show `v: err` in red |
| `ParseFailed` | Output doesn't match expected format | Show `v: err` in red |

### Latest Version Errors

| Error | Cause | Behavior |
| --- | --- | --- |
| `Request` | Network failure or HTTP error | Log error, use stale cache if available |
| `Parse` | Invalid JSON response | Log error, use stale cache if available |

### Cache Errors

| Error | Cause | Behavior |
| --- | --- | --- |
| `Read` | Failed to read cache file | Treat as no cache |
| `Parse` | Invalid cache JSON | Treat as no cache |
| `Write` | Failed to write cache file | Log error, continue |
| `LockHeld` | Another process is refreshing | Skip refresh, use existing cache |

## Updates

### Main Process

Current version arrives via `VersionLoaded` message. Latest version arrives via `VersionCacheLoaded` message.

### Background Process (TEA)

The background process follows TEA with its own model, messages, and effects.

#### Background Model

```rust
struct BackgroundModel {
    response: Option<String>,
}
```

#### Background Messages

```rust
enum BackgroundMsg {
    LockAcquired(Result<(), cache::Error>),
    VersionFetched(Result<String, api::Error>),
    CacheSaved(Result<(), cache::Error>),
}
```

#### Background Effects

```rust
enum BackgroundEffect {
    AcquireLock,
    FetchLatestVersion,
    SaveCache(String),
    ReleaseLock,
    LogError(Error),
    Exit,
}
```

#### Background Update

| Message | Model Update | Effects |
| --- | --- | --- |
| (init) | Empty model | `AcquireLock` |
| `LockAcquired(Ok(()))` | — | `FetchLatestVersion` |
| `LockAcquired(Err(LockHeld))` | — | `Exit` |
| `VersionFetched(Ok(version))` | Store version | `SaveCache(version)` |
| `VersionFetched(Err(e))` | — | `LogError`, `ReleaseLock`, `Exit` |
| `CacheSaved(Ok(()))` | — | `ReleaseLock`, `Exit` |
| `CacheSaved(Err(e))` | — | `LogError`, `ReleaseLock`, `Exit` |

### Lock File

Location: `~/.cache/claude-statusline/version.lock`

Prevents concurrent background refreshes. Created with `O_CREAT | O_EXCL`, deleted on completion.

## View

```rust
fn render(version: &Version) -> String {
    match &version.current {
        None => "v...".to_string(),
        Some(Err(_)) => format!("v: {}", "err".red()),
        Some(Ok(current)) => {
            match &version.latest {
                Some(Ok(latest)) if is_newer(latest, current) => {
                    format!("v{current} {}", "↑".yellow())
                }
                _ => format!("v{current}"),
            }
        }
    }
}

fn is_newer(latest: &str, current: &str) -> bool {
    // Semantic version comparison
    // Returns true if latest > current
}
```

## Version Comparison

Uses semantic versioning comparison:

- Parse both versions as `(major, minor, patch)` tuples
- Compare major first, then minor, then patch

Examples:

| Current | Latest | Result |
| --- | --- | --- |
| `1.0.17` | `1.0.18` | Update available |
| `1.0.17` | `1.0.17` | Current |
| `1.0.18` | `1.0.17` | Current (running newer) |
| `1.0.17` | `2.0.0` | Update available |

## Dependencies

- `claude` CLI (current version)
- npm registry API (latest version)
- File system (cache)

## File Structure

```
components/version/
├── mod.rs
├── model.rs      # CachedVersion, Version
├── view.rs       # render function
├── background/
│   ├── mod.rs
│   ├── model.rs      # BackgroundModel
│   ├── msg.rs        # BackgroundMsg enum
│   ├── effect.rs     # BackgroundEffect enum
│   └── update.rs     # background update function
└── effects/
    ├── mod.rs
    ├── api.rs         # fetch_latest_version
    ├── cache.rs       # load, save, lock
    └── current.rs     # run claude --version
```
