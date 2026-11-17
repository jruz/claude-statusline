use lib/log.nu

const CURRENT_VERSION = '2.1.280'
const LATEST_VERSION = '2.1.285'
const FIXTURES_DIR = 'tests/fixtures'
const OUTPUT = 'screenshot.png'
const ZERO_WIDTH_SPACE = "\u{200B}"


def iso [time: datetime]: nothing -> string {
    $time | date to-timezone UTC | format date '%Y-%m-%dT%H:%M:%SZ'
}


def host-os []: nothing -> string {
    if $nu.os-info.name == macos { 'apple-darwin' } else { 'unknown-linux-musl' }
}


def binary-path []: nothing -> path {
    let name = open Cargo.toml | get package.name
    $"target/dist/($name)-(host-os)-($nu.os-info.arch)" | path expand
}


def write-fake-home [home: path]: nothing -> nothing {
    let now = date now
    let cache_dir = $home | path join .cache claude-statusline
    let bin_dir = $home | path join bin
    let claude = $bin_dir | path join claude
    mkdir $cache_dir $bin_dir
    ({fetched_at: (iso $now), latest_version: $LATEST_VERSION}
        | to json
        | save ($cache_dir | path join version.json))
    ({
        fetched_at: (iso $now)
        response: {
            five_hour: {utilization: 35.0, resets_at: (iso ($now + 2hr + 30min))}
            seven_day: {utilization: 18.0, resets_at: (iso ($now + 4day + 6hr))}
        }
    }
        | to json
        | save ($cache_dir | path join usage.json))
    $"#!/bin/sh\necho '($CURRENT_VERSION) \(Claude Code\)'\n" | save $claude
    ^chmod 755 $claude
}


def run-statusline [home: path]: nothing -> string {
    let binary = binary-path
    let bin_dir = $home | path join bin
    with-env {HOME: $home, PATH: ($env.PATH | prepend $bin_dir)} {
        cd $FIXTURES_DIR
        let stdin = open --raw demo_stdin.json
        $stdin | ^$binary
    }
}


def clean-output []: string -> string {
    (lines
        | where $it != $ZERO_WIDTH_SPACE
        | str join (char newline)
        | str replace --all (ansi default) (ansi reset)
        | str trim)
}


def freeze-png [output: path]: string -> nothing {
    ^nix run nixpkgs#charm-freeze -- --language ansi --padding 20 --output $output
}


# Render the demo statusline to screenshot.png
def main []: nothing -> nothing {
    let home = mktemp --directory --tmpdir readme-screenshot-XXXXXX
    try {
        write-fake-home $home
        let output = run-statusline $home
        $output | clean-output | freeze-png $OUTPUT
    } catch {|err|
        rm --recursive $home
        $err.raw
    }
    rm --recursive $home
    log ok $"wrote ($OUTPUT)"
}
