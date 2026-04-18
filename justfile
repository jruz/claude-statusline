# bin projects only
install:
    cargo install --path . --force

rust-build:
    cargo build --all-targets

rust-cargo-check:
    cargo check --all-targets

rust-check:
    just rust-cargo-check
    just rust-fmt-check
    just rust-clippy
    just rust-test

rust-fmt-check:
    cargo fmt --all -- --check

rust-clippy *ARGS='':
    cargo clippy --all-targets {{ARGS}}

rust-clippy-fix *ARGS='':
    cargo clippy \
        --fix \
        --all-targets \
        --allow-dirty \
        {{ARGS}}

rust-fix:
    just rust-fmt-fix
    just rust-clippy-fix

rust-fmt-fix:
    cargo fmt --all

rust-outdated:
    cargo outdated

rust-test *FILTER='':
    cargo nextest run \
        --no-tests=pass \
        {{FILTER}}

rust-upgrade:
    cargo upgrade --incompatible
    cargo update

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

check:
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
    docker run --rm \
        --volume {{ justfile_directory() }}:/work \
        --workdir /work \
        --env REVISION={{ env("REVISION") }} \
        ghcr.io/rust-cross/cargo-zigbuild:0.21.4 \
        scripts/release-build.sh {{ OS }} {{ ARCH }}

release-upload TAG:
    bun scripts/release-upload.ts {{ TAG }}

release-delete TAG:
    bun scripts/release-delete.ts {{ TAG }}

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
    let json = ({
        context_window: {
            used_percentage: 42
            total_input_tokens: 100000
            total_output_tokens: 52000
        }
        model: { display_name: "Opus 4.6 (with 1M context)" }
    } | to json --raw)
    $json | ^cargo run --quiet
