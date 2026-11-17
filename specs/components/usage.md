# Usage Component

Displays remaining API rate limit with progress bars and reset times.

## Output

```
5h: ━━━━━━━━━━  60% (2h 15m)
7d: ━━━━━━━━━━  40% (3d 12h)
```

## Calculation

- Remaining = `100 - utilization`, rounded to nearest integer, clamped to 0-100
- Bar and percentage show remaining (filled = left), not usage
- Bar color by remaining:
  - red: < 20%
  - yellow: 20-40%
  - green: > 40%

## Data Source

**Scope:** Shared across all instances (cached at `~/.cache/claude-statusline/usage.json`)

### Primary: Anthropic OAuth Usage API

```
GET https://api.anthropic.com/api/oauth/usage
Headers:
  Authorization: Bearer {access_token}
  anthropic-beta: oauth-2025-04-20
```

Response:

```json
{
  "five_hour": { "utilization": 14.0, "resets_at": "2026-01-25T15:30:00Z" },
  "seven_day": { "utilization": 53.0, "resets_at": "2026-01-30T00:00:00Z" }
}
```

Note: `utilization` is a percentage (0-100), not a fraction.

`resets_at` is `null` when a window has no reset time yet, such as right after it resets.

### Credentials

Read the OAuth access token from Claude Code's credentials:

- macOS: Keychain `security find-generic-password -s "Claude Code-credentials" -w`
- Linux: file `~/.claude/.credentials.json`

Both return JSON with structure:

```json
{
  "claudeAiOauth": {
    "accessToken": "sk-ant-oat01-..."
  }
}
```

### Cache

Location: `~/.cache/claude-statusline/usage.json`

```json
{
  "fetched_at": "2026-01-25T12:30:00Z",
  "response": {
    "five_hour": { "utilization": 40.0, "resets_at": "2026-01-25T15:30:00Z" },
    "seven_day": { "utilization": 60.0, "resets_at": "2026-01-30T00:00:00Z" }
  }
}
```

## Model

```rust
struct Usage {
    five_hour: Result<Window, Error>,
    seven_day: Result<Window, Error>,
}

struct Window {
    utilization: f64,
    resets_at: Option<DateTime<Utc>>,
}

enum Error {
    CacheParse(String),
    CacheRead(String),
    Expired(DateTime<Utc>),
    InvalidUtilization(f64),
}
```

Cache structure (separate from model):

```rust
struct Cached {
    fetched_at: DateTime<Utc>,
    response: Response,
}

struct Response {
    five_hour: Window,
    seven_day: Window,
}
```

## States

| State       | Condition                   | View                                                              |
| ----------- | --------------------------- | ----------------------------------------------------------------- |
| Fresh       | Cache exists and age < 60s  | `5h: ━━━━━━━━━━  60% (2h 15m)` and `7d: ━━━━━━━━━━  40% (3d 12h)` |
| Stale       | Cache exists and age >= 60s | Same as Fresh (renders stale data, triggers background refresh)   |
| Unavailable | Cache missing or unreadable | `5h: err` and `7d: err` (red), triggers background refresh        |
| Invalid     | Cache JSON does not parse   | `5h: err` and `7d: err` (red), triggers background refresh        |
| Expired     | Window `resets_at` has past | `err` (red) for that window, not logged                           |
| Not started | Window `resets_at` is null  | `5h: ━━━━━━━━━━ 100% (5h 0m)` and `7d: ━━━━━━━━━━ 100% (7d 0h)`   |

A failed background refresh leaves the cache untouched, so the last successful response keeps rendering.

## Error Handling

All errors are logged to `~/.cache/claude-statusline/error.log` with module `usage`:

```
2026-01-25T12:30:45+01:00 [usage] api: request failed: connection refused
```

### Main Process

| Error                | Cause                            | Behavior                           |
| -------------------- | -------------------------------- | ---------------------------------- |
| `CacheRead`          | Cache file missing or unreadable | Render `err`, log unless not found |
| `CacheParse`         | Invalid cache JSON               | Render `err`, log                  |
| `Expired`            | Window reset time has passed     | Render `err` for that window       |
| `InvalidUtilization` | Utilization not a number         | Render `err` for that window, log  |

### Background Refresh

Errors are strings, logged with a prefix before the process exits. A failed refresh keeps the lock, so the next attempt waits until it goes stale.

| Prefix                 | Cause                                                                      | Lock      |
| ---------------------- | -------------------------------------------------------------------------- | --------- |
| `lock acquire failed`  | Lock cannot be created                                                     | Untouched |
| `credentials`          | Credentials read failed, or `accessToken` missing                          | Kept      |
| `api`                  | Request failed, non-success status, or invalid JSON response (body logged) | Kept      |
| `cache path not found` | Home directory not found                                                   | Kept      |
| `cache write`          | Cache directory creation, serialization, or file write failed              | Kept      |

## Updates

### Main Process

1. `ReadUsageCache` reads the cache file and sends `CacheLoaded`
2. `handle_cache_loaded` parses the cache and emits `SpawnBackgroundRefresh` when stale or missing
3. `SpawnBackgroundRefresh` runs the current executable with `--refresh-usage` on a thread, stderr to the error log

### Background Refresh

`run_background_refresh` runs sequentially:

1. Exit if the lock is held
2. Remove the lock if stale
3. Acquire the lock
4. Read the access token
5. Fetch usage from the API
6. Write `Cached { fetched_at: now, response }` to the cache file
7. Release the lock

### Lock File

Location: `~/.cache/claude-statusline/usage.lock`

Prevents concurrent background refreshes and backs off after a failed one. Created with `O_CREAT | O_EXCL`, deleted on success. Treated as stale after 300s.

## View

`render_window(label, window)` renders one line for both `5h` and `7d`, success and error states.

Progress bar width: 10 characters.

Time remaining format:

- < 1 hour: `Xm`
- < 24 hours: `Xh Ym`
- > = 24 hours: `Xd Yh`
- No reset time: the full window length, `5h 0m` or `7d 0h`

## Dependencies

- Claude Code credentials file or macOS Keychain (credentials)
- Anthropic API (rate limits)
- File system (cache)

## File Structure

```
components/usage.rs    # Error enum, Cached, Response, Window, Usage, cache and lock paths, render
main.rs                # Background refresh: lock, credentials, API fetch, cache write
```

Main process messages (`CacheLoaded`) and effects (`SpawnBackgroundRefresh`) are defined in `effect.rs`.
