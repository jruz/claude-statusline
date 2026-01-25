# Usage Component

Displays API rate limit utilization with progress bars and reset times.

## Output

```
5h: ███░░ 60% (2h 15m)
7d: ██░░░ 40% (3d 12h)
```

## Data Source

**Scope:** Shared across all instances (cached at `~/.cache/claude-statusline/usage.json`)

### Primary: Anthropic Rate Limits API

```
GET {base_url}/v1/rate_limits
Headers:
  x-api-key: {api_key}
  anthropic-version: 2023-06-01
```

Base URL: `ANTHROPIC_API_URL` env var, default `https://api.anthropic.com`

Response:

```json
{
  "rate_limits": [
    { "type": "requests", "utilization": 0.6, "resets_at": "2026-01-25T15:30:00Z" },
    { "type": "tokens", "utilization": 0.4, "resets_at": "2026-01-30T00:00:00Z" }
  ]
}
```

### Credentials

**Resolution:**
1. If `CLAUDE_STATUSLINE_SKIP_KEYCHAIN=true`, use `ANTHROPIC_API_KEY` env var
2. Otherwise, read from macOS Keychain: `security find-generic-password -s "anthropic-api-key" -w`

### Cache

Location: `~/.cache/claude-statusline/usage.json`

```json
{
  "fetched_at": "2026-01-25T12:30:00Z",
  "response": {
    "five_hour": { "utilization": 0.6, "resets_at": "2026-01-25T15:30:00Z" },
    "seven_day": { "utilization": 0.4, "resets_at": "2026-01-30T00:00:00Z" }
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
    CacheRead(io::Error),
    CacheParse(serde_json::Error),
    CredentialsRead,
    ApiRequest(reqwest::Error),
    ApiParse(reqwest::Error),
}
```

Cache structure (separate from model):

```rust
struct CachedUsage {
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
|-------|-----------|------|
| Fresh | Cache exists and age < 60s | `5h: ███░░ 60% (2h 15m)` and `7d: ██░░░ 40% (3d 12h)` |
| Stale | Cache exists and age >= 60s | Same as Fresh (renders stale data, triggers background refresh) |
| Unavailable | No cache exists | `5h: err` and `7d: err` (red), triggers background refresh |
| Error | API/credentials failed | `5h: err` and `7d: err` (red), uses stale cache if available |

## Error Handling

All errors are logged to `~/.cache/claude-statusline/error.log` and displayed as `err` in red.

Log format:
```
2026-01-25T12:30:45Z [usage] API request failed: connection refused
```

### Credentials Errors

| Error | Cause | Behavior |
|-------|-------|----------|
| `KeychainFailed` | Keychain command returned non-zero | Log error, skip API fetch |
| `EmptyResponse` | Keychain returned empty string | Log error, skip API fetch |
| `InvalidUtf8` | Keychain output not valid UTF-8 | Log error, skip API fetch |
| `Execute` | Failed to run `security` command | Log error, skip API fetch |

### API Errors

| Error | Cause | Behavior |
|-------|-------|----------|
| `Request` | Network failure or HTTP error | Log error, use stale cache if available |
| `Parse` | Invalid JSON response | Log error, use stale cache if available |

### Cache Errors

| Error | Cause | Behavior |
|-------|-------|----------|
| `Read` | Failed to read cache file | Treat as no cache |
| `Parse` | Invalid cache JSON | Treat as no cache |
| `Write` | Failed to write cache file | Log error, continue |
| `LockHeld` | Another process is refreshing | Skip refresh, use existing cache |

All errors contribute to the error count displayed in the second line: `Errors(N): Last error text`

## Updates

### Main Process

Data arrives via `CacheLoaded` message from the app-level update.

### Background Process (TEA)

The background process follows TEA with its own model, messages, and effects.

#### Background Model

```rust
struct BackgroundModel {
    api_key: Option<String>,
    response: Option<Response>,
}
```

#### Background Messages

```rust
enum BackgroundMsg {
    LockAcquired(Result<(), cache::Error>),
    KeychainResult(Result<String, credentials::Error>),
    UsageFetched(Result<Response, api::Error>),
    CacheSaved(Result<(), cache::Error>),
}
```

#### Background Effects

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

#### Background Update

| Message | Model Update | Effects |
|---------|--------------|---------|
| (init) | Empty model | `AcquireLock` |
| `LockAcquired(Ok(()))` | — | `ReadKeychain` |
| `LockAcquired(Err(LockHeld))` | — | `Exit` |
| `KeychainResult(Ok(key))` | Store api_key | `FetchUsage(key)` |
| `KeychainResult(Err(e))` | — | `LogError`, `ReleaseLock`, `Exit` |
| `UsageFetched(Ok(response))` | Store response | `SaveCache(response)` |
| `UsageFetched(Err(e))` | — | `LogError`, `ReleaseLock`, `Exit` |
| `CacheSaved(Ok(()))` | — | `ReleaseLock`, `Exit` |
| `CacheSaved(Err(e))` | — | `LogError`, `ReleaseLock`, `Exit` |

### Lock File

Location: `~/.cache/claude-statusline/usage.lock`

Prevents concurrent background refreshes. Created with `O_CREAT | O_EXCL`, deleted on completion.

## View

```rust
fn render_five_hour(usage: &Usage) -> String {
    match &usage.five_hour {
        Ok(w) => format!("5h: {} {:.0}% ({})",
            progress_bar(w.utilization * 100.0, 5),
            w.utilization * 100.0,
            format_remaining(w.resets_at)),
        Err(_) => format!("5h: {}", "err".red()),
    }
}

fn render_seven_day(usage: &Usage) -> String {
    match &usage.seven_day {
        Ok(w) => format!("7d: {} {:.0}% ({})",
            progress_bar(w.utilization * 100.0, 5),
            w.utilization * 100.0,
            format_remaining(w.resets_at)),
        Err(_) => format!("7d: {}", "err".red()),
    }
}
```

View is responsible for rendering both success and error states.

Progress bar width: 5 characters.

Time remaining format:
- < 1 hour: `Xm`
- < 24 hours: `Xh Ym`
- >= 24 hours: `Xd Yh`

## Dependencies

- macOS Keychain (credentials)
- Anthropic API (rate limits)
- File system (cache)

## File Structure

```
components/usage/
├── mod.rs
├── model.rs      # CachedUsage, Response, Window, Usage
├── view.rs       # render function
├── background/
│   ├── mod.rs
│   ├── model.rs      # BackgroundModel
│   ├── msg.rs        # BackgroundMsg enum
│   ├── effect.rs     # BackgroundEffect enum
│   └── update.rs     # background update function
└── effects/
    ├── mod.rs
    ├── api.rs         # fetch_usage
    ├── cache.rs       # load, save, lock
    └── credentials.rs # read_api_key
```

Main process messages (`CacheLoaded`) and effects (`SpawnBackgroundRefresh`) are defined in `core/`.
