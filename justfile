mod rust '.just/rust.justfile'
mod release '.just/release.justfile'
mod just '.just/just.justfile'
mod oxfmt '.just/oxfmt.justfile'
mod git '.just/git.justfile'
mod tools '.just/tools.justfile'

musl-target := arch() + '-unknown-linux-musl'

check:
    just just::fmt-check
    just oxfmt::fmt-check
    just rust::check

fix:
    just just::fmt-fix
    just oxfmt::fmt-fix
    just rust::fix

setup:
    just git::hooks-setup

install:
    just rust::install

test:
    just rust::test

run *ARGS='':
    just rust::run {{ ARGS }}

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
    ^sandbox cargo build --target {{ musl-target }}
    (open tests/fixtures/demo_stdin.json
        | to json --raw
        | ^./target/docker/{{ musl-target }}/debug/claude-statusline)

readme-screenshot:
    sandbox cargo build --target {{ musl-target }}
    nu scripts/readme-screenshot.nu
