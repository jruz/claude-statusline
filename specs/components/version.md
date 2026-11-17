# Version Component

Displays Claude Code version and indicates when a newer version is available.

## Output

```
Version: 1.0.17
```

When newer version available:

```
Version: 1.0.17↑
```

## Data Source

### Current Version

Run `claude --version` which outputs:

```
1.0.17 (Claude Code)
```

Parse the first whitespace-separated token.

### Latest Version

**Scope:** Shared across all instances (cached at `~/.cache/claude-statusline/version.json`)

Fetched from npm registry for the selected channel:

```
GET https://registry.npmjs.org/@anthropic-ai/claude-code/{channel}
```

Channel comes from `--channel <latest|next|stable>`, `stable` when omitted. Unknown channel = error.

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

enum Channel {
    Latest,
    Next,
    Stable,
}
```

Both fields follow the standard TEA pattern:

- `None` = loading (not yet received)
- `Some(Ok(..))` = success
- `Some(Err(..))` = error

Cache structure (separate from model):

```rust
struct Cached {
    fetched_at: DateTime<Utc>,
    latest_version: String,
}
```

## States

| State            | Condition                                                          | View                                          |
| ---------------- | ------------------------------------------------------------------ | --------------------------------------------- |
| Loading          | `version` or `current` is `None`                                   | `Version: ...`                                |
| Current only     | `current` is `Some(Ok)`, `latest` is `None`                        | `Version: 1.0.17`                             |
| Up to date       | `current` is `Some(Ok)`, `latest` is `Some(Ok)`, current >= latest | `Version: 1.0.17`                             |
| Update available | `current` is `Some(Ok)`, `latest` is `Some(Ok)`, current < latest  | `Version: 1.0.17↑` (arrow in yellow)          |
| Current error    | `current` is `Some(Err)`                                           | `Version: err` (red)                          |
| Latest error     | `current` is `Some(Ok)`, `latest` is `Some(Err)`                   | `Version: 1.0.17` (no indicator, fail silent) |

## Error Handling

Current version errors show `err` in red. Latest version errors are silent (shows current version without update indicator).

All errors are logged to `~/.cache/claude-statusline/error.log` with module `version`:

```
2026-01-25T12:30:45+01:00 [version] command failed: No such file or directory
2026-01-25T12:30:45+01:00 [version] api: request failed: connection refused
```

### Main Process

| Error           | Cause                                | Behavior                                                        |
| --------------- | ------------------------------------ | --------------------------------------------------------------- |
| `CommandFailed` | `claude` command not found or failed | Render `err`, log                                               |
| `ParseFailed`   | Empty `claude --version` output      | Render `err`, log                                               |
| `CacheParse`    | Invalid cache JSON                   | No indicator, log, triggers background refresh                  |
| Cache read      | Cache file missing or unreadable     | No indicator, log unless not found, triggers background refresh |
| Version compare | Version not `major.minor.patch`      | No indicator, log                                               |

### Background Refresh

Errors are strings, logged with a prefix before the process exits.

| Prefix                 | Cause                                                                        | Lock      |
| ---------------------- | ---------------------------------------------------------------------------- | --------- |
| `lock acquire failed`  | Lock held by another refresh, or cannot be created                           | Untouched |
| `api`                  | Request failed, non-success status, invalid JSON, or missing `version` field | Released  |
| `cache path not found` | Home directory not found                                                     | Released  |
| `cache write`          | Cache directory creation, serialization, or file write failed                | Released  |

## Updates

### Main Process

1. `ReadVersion` runs `claude --version` and sends `VersionLoaded`
2. `ReadVersionCache` reads the cache file and sends `VersionCacheLoaded`
3. `handle_version_cache_loaded` emits `SpawnVersionRefresh` when the cache is stale, invalid, or missing
4. `SpawnVersionRefresh` runs the current executable with `--refresh-version --channel <channel>` on a thread, stderr to the error log

### Background Refresh

`run_version_refresh` runs sequentially:

1. Remove the lock if stale
2. Acquire the lock, exit if held
3. Fetch the latest version from npm
4. Write `Cached { fetched_at: now, latest_version }` to the cache file
5. Release the lock

### Lock File

Location: `~/.cache/claude-statusline/version.lock`

Prevents concurrent background refreshes. Created with `O_CREAT | O_EXCL`, deleted on completion. Treated as stale after 300s.

## Version Comparison

`is_newer(latest, current)` returns `Option<bool>`:

- Parse both versions as `(major, minor, patch)` tuples
- Compare tuples
- `None` when either version does not parse

Examples:

| Current  | Latest   | Result                  |
| -------- | -------- | ----------------------- |
| `1.0.17` | `1.0.18` | Update available        |
| `1.0.17` | `1.0.17` | Current                 |
| `1.0.18` | `1.0.17` | Current (running newer) |
| `1.0.17` | `2.0.0`  | Update available        |

## Dependencies

- `claude` CLI (current version)
- npm registry API (latest version)
- File system (cache)

## File Structure

```
components/version.rs  # Errors, Channel, Version, Cached, cache and lock paths, parse, render
main.rs                # Background refresh: lock, npm fetch, cache write
```

Main process messages (`VersionLoaded`, `VersionCacheLoaded`) and effects (`ReadVersion`, `ReadVersionCache`, `SpawnVersionRefresh`) are defined in `effect.rs`.
