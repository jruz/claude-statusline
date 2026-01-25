# Coding Style

This project follows the global Rust skill. Below are project-specific clarifications.

## Functional Paradigm

All logic via pure functions. No `impl` blocks, no trait definitions, no OOP patterns.

Structs and enums are data containers only. Behavior lives in free functions that take data as parameters.

```rust
pub struct User {
    pub id: i64,
    pub name: String,
}

pub fn format_display(user: &User) -> String {
    format!("User: {}", user.name)
}
```

## Error Handling

Application never crashes. Always output something to stdout. Always exit 0.

Never provide defaults or failsafes. Always use `Option` or `Result`. All errors must be raised, never hidden.

Use `color_eyre::Result` for the main function:

```rust
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // ...
}
```

Each module defines its own `Error` enum. Always use `#[source]` to preserve error chain. Add `#[from]` only for 1:1 mappings:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse JSON")]
    ParseJson(
        #[source]
        #[from]
        serde_json::Error,
    ),

    #[error("Failed to read stdin")]
    ReadStdin(
        #[source]
        #[from]
        std::io::Error,
    ),
}
```

Main accumulates errors from independent modules, renders what succeeded, shows "err" for failures.

## Imports

Namespace imports only. Traits imported directly when needed for methods:

```rust
use std::io::Read as _;

use color_eyre;
use owo_colors;
use serde;

use crate::cache;
use crate::input;
```

Usage with namespace prefix:

```rust
fn main() -> color_eyre::Result<()> {
    let color = owo_colors::AnsiColors::Green;
}
```

## Linting

All clippy lint groups enabled at deny level. Never bypass linter.

### Alphabetical Ordering

Enum variants, struct fields, and items must be alphabetically ordered:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse JSON")]
    ParseJson(
        #[source]
        #[from]
        serde_json::Error,
    ),

    #[error("Failed to read stdin")]
    ReadStdin(
        #[source]
        #[from]
        std::io::Error,
    ),
}
```

### No Module Name Repetition

Type names must not repeat their containing module name:

```rust
// Good: claude_model::model::Model
pub struct Model { ... }

// Bad: claude_model::model::ClaudeModel
pub struct ClaudeModel { ... }
```

Access via module path provides context: `claude_model::model::Model`.
