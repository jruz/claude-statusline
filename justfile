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
    just rust-fmt
    just rust-clippy

rust-outdated:
    cargo outdated

rust-test *FILTER='':
    cargo nextest run --no-tests=pass {{FILTER}}

rust-upgrade:
    cargo upgrade --incompatible
    cargo update

ci-lint:
    woodpecker-cli lint --strict .woodpecker/

md-lint:
    markdownlint-cli2 '**/*.md'

md-lint-fix:
    markdownlint-cli2 --fix '**/*.md'

nix-lint:
    statix check .
    deadnix --fail .

nix-lint-fix:
    statix fix .
    deadnix --edit .

nix-fmt:
    fd --extension nix --exec nixfmt --check

nix-fmt-fix:
    fd --extension nix --exec nixfmt

toml-fmt:
    taplo format --check

toml-fmt-fix:
    taplo format

yaml-lint:
    fd --extension yaml --extension yml --hidden --exec yamllint --config-file .yamllint.yaml

lint:
    just ci-lint
    just md-lint
    just nix-lint
    just rust-lint
    just toml-fmt
    just yaml-lint

fmt:
    just nix-fmt
    just rust-fmt
    just toml-fmt

check:
    just fix
    just lint
    just fmt
    just rust-test

fix:
    just rust-fix
    just nix-lint-fix
    just nix-fmt-fix
    just md-lint-fix
    just toml-fmt-fix

test:
    just rust-test

demo:
    echo '{"context_window":{"used_percentage":42},"model":{"display_name":"Opus 4.5"}}' | cargo run --quiet
