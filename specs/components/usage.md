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

### Credentials

Read the OAuth access token from Claude Code's credentials, first match wins:

1. File: `~/.claude/.credentials.json`
2. macOS Keychain: `security find-generic-password -s "Claude Code-credentials" -w`

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
    resets_at: DateTime<Utc>,
}

enum Error {
    CacheParse(String),
    CacheRead(String),
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

| State | Condition | View |
| --- | --- | --- |
| Fresh | Cache exists and age < 60s | `5h: ━━━━━━━━━━  60% (2h 15m)` and `7d: ━━━━━━━━━━  40% (3d 12h)` |
| Stale | Cache exists and age >= 60s | Same as Fresh (renders stale data, triggers background refresh) |
| Unavailable | Cache missing or unreadable | `5h: err` and `7d: err` (red), triggers background refresh |
| Invalid | Cache JSON does not parse | `5h: err` and `7d: err` (red), triggers background refresh |

A failed background refresh leaves the cache untouched, so the last successful response keeps rendering.

## Error Handling

All errors are logged to `~/.cache/claude-statusline/error.log` with module `usage`:

```
2026-01-25T12:30:45Z [usage] api: request failed: connection refused
```

### Main Process

| Error | Cause | Behavior |
| --- | --- | --- |
| `CacheRead` | Cache file missing or unreadable | Render `err`, log unless not found |
| `CacheParse` | Invalid cache JSON | Render `err`, log |
| `InvalidUtilization` | Utilization not a number | Render `err` for that window, log |

### Background Refresh

Errors are strings, logged with a prefix before the process exits.

| Prefix | Cause | Lock |
| --- | --- | --- |
| `lock acquire failed` | Lock held by another refresh, or cannot be created | Untouched |
| `credentials` | Credentials file and keychain both failed, or `accessToken` missing | Released |
| `api` | Request failed, non-success status, or invalid JSON response | Released |
| `cache path not found` | Home directory not found | Released |

## Updates

### Main Process

1. `ReadUsageCache` reads the cache file and sends `CacheLoaded`
2. `handle_cache_loaded` parses the cache and emits `SpawnBackgroundRefresh` when stale or missing
3. `SpawnBackgroundRefresh` runs the current executable with `--refresh-usage` on a thread, stderr to the error log

### Background Refresh

`run_background_refresh` runs sequentially:

1. Remove the lock if stale
2. Acquire the lock, exit if held
3. Read the access token
4. Fetch usage from the API
5. Write `Cached { fetched_at: now, response }` to the cache file
6. Release the lock

### Lock File

Location: `~/.cache/claude-statusline/usage.lock`

Prevents concurrent background refreshes. Created with `O_CREAT | O_EXCL`, deleted on completion. Treated as stale after 300s.

## View

`render_window(label, window)` renders one line for both `5h` and `7d`, success and error states.

Progress bar width: 10 characters.

Time remaining format:

- < 1 hour: `Xm`
- < 24 hours: `Xh Ym`
- >= 24 hours: `Xd Yh`

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
