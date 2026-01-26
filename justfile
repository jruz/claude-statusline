# bin projects only
install:
    cargo install --path . --force

rust-build:
    cargo build --all-targets

rust-cargo-check:
    cargo check --all-targets

rust-check:
    just rust-cargo-check
    just rust-lint
    just rust-test

rust-check-allow:
    #!/usr/bin/env nu
    let matches = (rg '#\[allow\(' src --type rust | complete)
    if $matches.exit_code == 0 {
        print $matches.stdout
        error make {msg: "ERROR: #[allow(...)] is banned - refactor code to satisfy the linter"}
    }

rust-clippy *ARGS='':
    cargo clippy --all-targets {{ARGS}}

rust-clippy-fix *ARGS='':
    cargo clippy --fix --all-targets --allow-dirty {{ARGS}}

rust-fix:
    just rust-fmt-fix
    just rust-clippy-fix

rust-fmt *FILES='':
    cargo fmt --all -- --check {{FILES}}

rust-fmt-fix:
    cargo fmt --all

rust-lint:
    just rust-check-allow
    just rust-fmt
    just rust-clippy

rust-outdated:
    cargo outdated

rust-test *FILTER='':
    cargo nextest run --no-tests=pass {{FILTER}}

rust-upgrade:
    cargo upgrade --incompatible
    cargo update

check:
    just fix
    just rust-check

fix:
    just rust-fix

test:
    just rust-test

demo:
    echo '{"context_window":{"used_percentage":42},"model":{"display_name":"Opus 4.5"}}' | cargo run --quiet
