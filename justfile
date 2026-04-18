# bin projects only
install:
    sandbox cargo install --path . --force

rust-build:
    sandbox cargo build --all-targets

rust-cargo-check:
    sandbox cargo check --all-targets

rust-check:
    just rust-cargo-check
    just rust-fmt-check
    just rust-clippy

rust-fmt-check:
    sandbox cargo fmt --all -- --check

rust-clippy *ARGS='':
    sandbox cargo clippy --all-targets {{ARGS}}

rust-clippy-fix *ARGS='':
    sandbox cargo clippy \
        --fix \
        --all-targets \
        --allow-dirty \
        {{ARGS}}

rust-fix:
    just rust-fmt-fix
    just rust-clippy-fix

rust-fmt-fix:
    sandbox cargo fmt --all

rust-outdated:
    sandbox cargo outdated

rust-test *FILTER='':
    sandbox cargo nextest run \
        --no-tests=pass \
        {{FILTER}}

rust-upgrade:
    sandbox cargo upgrade --incompatible
    sandbox cargo update

woodpecker-check:
    woodpecker-cli lint \
        --strict \
        .woodpecker/

md-check:
    markdownlint-cli2 '**/*.md'

md-fix:
    markdownlint-cli2 --fix '**/*.md'

nix-check:
    fd \
        --extension nix \
        --exec nixfmt --check
    statix check .
    deadnix --fail .

nix-fix:
    fd \
        --extension nix \
        --exec nixfmt
    statix fix .
    deadnix --edit .

toml-check:
    fd \
        --extension toml \
        --exec taplo format --check

toml-fix:
    fd \
        --extension toml \
        --exec taplo format

nu-check:
    #!/usr/bin/env nu
    (^fd --absolute-path --extension nu
        --exec nu --commands 'nu-check --debug {}')
    ^nu-lint --config nu-lint.toml

nu-fix:
    nu-lint \
        --config nu-lint.toml \
        --fix

clj-check:
    fd \
        --extension clj \
        --exec zprint -c
    fd \
        --extension clj \
        --exec clj-kondo \
        --fail-level warning \
        --lint

clj-fix:
    fd \
        --extension clj \
        --exec zprint -w

yaml-check:
    fd \
        --extension yaml \
        --extension yml \
        --hidden \
        --exec yamllint \
        --strict \
        --config-file .yamllint.yaml

sandbox-check:
    sandbox bun scripts/sandbox-check.ts

check:
    just sandbox-check
    just clj-check
    just woodpecker-check
    just md-check
    just nix-check
    just nu-check
    just rust-check
    just toml-check
    just yaml-check

fix:
    just clj-fix
    just rust-fix
    just nix-fix
    just nu-fix
    just md-fix
    just toml-fix

test:
    just rust-test

release-build OS ARCH:
    sandbox nu scripts/release-build.nu {{ OS }} {{ ARCH }}

release-bump VERSION:
    nu scripts/release-bump.nu {{ VERSION }}

release-upload TAG:
    sandbox bun scripts/release-upload.ts {{ TAG }}

release-delete TAG:
    sandbox bun scripts/release-delete.ts {{ TAG }}

worktree-add NAME:
    #!/usr/bin/env nu
    ^git worktree add -b {{ NAME }} $"../{{ NAME }}"
    cd $"../{{ NAME }}"
    ^direnv allow

worktree-remove NAME:
    #!/usr/bin/env nu
    ^git worktree remove $"../{{ NAME }}"
    ^git branch -d {{ NAME }}

demo:
    #!/usr/bin/env nu
    ^just rust-build
    (open tests/fixtures/demo_stdin.json
        | to json --raw
        | ^./target/debug/claude-statusline)
