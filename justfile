mod rust '.just/rust.justfile'
mod just '.just/just.justfile'
mod oxfmt '.just/oxfmt.justfile'
mod git '.just/git.justfile'
mod tools '.just/tools.justfile'

log-dir := home_directory() / '.cache' / 'claude-statusline'

check:
    just just::fmt-check
    just oxfmt::fmt-check
    just rust::check

fix:
    just just::fmt-fix
    just oxfmt::fmt-fix
    just rust::fix

setup:
    direnv allow
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
    (open tests/fixtures/demo_stdin.json
        | to json --raw
        | ^just rust::run)

logs-tail:
    tail -f {{ log-dir }}/error.log

logs-clear:
    #!/usr/bin/env nu
    "" | save --force {{ log-dir }}/error.log
    rm --force {{ log-dir }}/error.log.1

readme-screenshot:
    just rust::build-host
    nu scripts/readme-screenshot.nu

[positional-arguments]
release +ARGS:
    nu scripts/release.nu "$@"
