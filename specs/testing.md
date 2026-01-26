# Testing

Integration tests verify parsing and rendering by piping JSON through stdin.

## Test Environment

Tests run in isolated temp directories when testing file-based features (PLAN.md).

## Integration Tests

Tests at `tests/integration.rs` using `assert_cmd` crate.

### Test Cases

#### Context Component

| Test | Stdin | Expected |
|------|-------|----------|
| Percentage display | `{"context_window":{"used_percentage":42.5}}` | `Context: ... 42%` |
| Missing field | `{}` | `Context: ... 0%` |
| Model-only JSON | `{"model":{"display_name":"X"}}` | `Context: ... 0%` |
| Invalid JSON | `not json` | `Context: err` |

#### Model Component

| Test | Stdin | Expected |
|------|-------|----------|
| Display name | `{"model":{"display_name":"Opus 4.5"}}` | `Model: Opus 4.5` |
| Missing field | `{}` | `Model: err` |

#### Plan Component

| Test | Condition | Expected |
|------|-----------|----------|
| With checkboxes | PLAN.md has `- [x]` and `- [ ]` | `PLAN.md: ... N%` |
| No checkboxes | PLAN.md exists, no checkboxes | `PLAN.md: ... 100%` |
| File missing | No PLAN.md | `PLAN.md: file-not-found` |

#### Usage Component

| Test | Condition | Expected |
|------|-----------|----------|
| No cache | Cache file missing | Shows `5h:` and `7d:` labels |

#### Full Output

| Test | Condition | Expected |
|------|-----------|----------|
| Two lines | Valid input | Line 1: Context, Model, 5h / Line 2: PLAN.md, 7d |

## Running Tests

```bash
just test
# or
cargo nextest run
```

## File Structure

```
tests/
└── integration.rs    # Rust integration tests
```
