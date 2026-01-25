# Testing

Integration tests verify the full application flow using a mock API server.

## Test Environment

```
ANTHROPIC_API_URL=http://localhost:3000
ANTHROPIC_API_KEY=test-api-key
CLAUDE_STATUSLINE_SKIP_KEYCHAIN=true
CLAUDE_STATUSLINE_CACHE_DIR=/tmp/claude-statusline-test
```

## Mock Server

Bun server at `tests/mock-server.ts` that simulates Anthropic API.

### Endpoints

**GET /v1/rate_limits**

Success response:
```json
{
  "rate_limits": [
    { "type": "requests", "utilization": 0.6, "resets_at": "2026-01-25T15:30:00Z" },
    { "type": "tokens", "utilization": 0.4, "resets_at": "2026-01-30T00:00:00Z" }
  ]
}
```

### Scenarios

Control mock behavior via query params or headers:

| Scenario | Trigger | Response |
|----------|---------|----------|
| Success | Default | 200 with rate limits |
| Server error | `?scenario=error` | 500 Internal Server Error |
| Timeout | `?scenario=timeout` | No response for 10s |
| Malformed JSON | `?scenario=malformed` | 200 with invalid JSON |
| Empty rate limits | `?scenario=empty` | 200 with empty array |

### File

`tests/mock-server.ts`

```typescript
const server = Bun.serve({
  port: 3000,
  fetch(req) {
    const url = new URL(req.url);
    const scenario = url.searchParams.get("scenario");

    if (url.pathname === "/v1/rate_limits") {
      if (scenario === "error") {
        return new Response("Internal Server Error", { status: 500 });
      }
      if (scenario === "timeout") {
        return new Promise(() => {}); // never resolves
      }
      if (scenario === "malformed") {
        return new Response("not json", { status: 200 });
      }
      if (scenario === "empty") {
        return Response.json({ rate_limits: [] });
      }
      return Response.json({
        rate_limits: [
          { type: "requests", utilization: 0.6, resets_at: "2026-01-25T15:30:00Z" },
          { type: "tokens", utilization: 0.4, resets_at: "2026-01-30T00:00:00Z" },
        ],
      });
    }
    return new Response("Not Found", { status: 404 });
  },
});
```

---

## Integration Tests

Tests at `tests/integration.rs` using `assert_cmd` crate.

### Test Cases

#### Happy Path

| Test | Stdin | Expected Output |
|------|-------|-----------------|
| Full status line | `{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42}}` | `Context: ████░░ 42% \| Model: Opus \| Plan: ... \| 5h: ██░ 60% \| 7d: █░ 40%` |
| Context only | `{"context_window":{"used_percentage":100}}` | `Context: ██████ 100% \| Model: err \| Plan: ... \| 5h: ... \| 7d: ...` |
| Model only | `{"model":{"display_name":"Sonnet"}}` | `Context: err \| Model: Sonnet \| Plan: ... \| 5h: ... \| 7d: ...` |

#### Error States

| Test | Condition | Expected Output |
|------|-----------|-----------------|
| Empty stdin | `{}` | `Context: err \| Model: err \| Plan: ... \| 5h: ... \| 7d: ... \| Errors(N)` |
| Invalid JSON | `not json` | `Context: err \| Model: err \| Plan: err \| 5h: err \| 7d: err \| Errors(N)` |
| API error | `?scenario=error` | `... \| 5h: err \| 7d: err \| Errors(N)` |
| API timeout | `?scenario=timeout` | Uses cached data or shows err |

#### Cache Behavior

| Test | Condition | Expected Behavior |
|------|-----------|-------------------|
| Fresh cache | Cache < 60s old | No background refresh spawned |
| Stale cache | Cache >= 60s old | Background refresh spawned |
| No cache | Cache missing | Background refresh spawned |
| Lock held | Lock file exists | Skip refresh, use stale cache |

---

## Running Tests

```bash
# Start mock server (in background)
bun tests/mock-server.ts &

# Run integration tests
cargo nextest run

# Stop mock server
pkill -f "bun tests/mock-server.ts"
```

Or via justfile:

```bash
just test
```

---

## File Structure

```
tests/
├── mock-server.ts    # Bun mock API server
└── integration.rs    # Rust integration tests
```
