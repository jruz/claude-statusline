#!/usr/bin/env nu

def upload [file: path, tag: string]: nothing -> nothing {
    let headers = [Authorization $"token ($env.CODEBERG_TOKEN)"]
    let repo = $env.CI_REPO
    let url = $"https://codeberg.org/api/v1/repos/($repo)/releases/tags/($tag)/assets?name=($file)"
    try {
        (open --raw $file
            | http post
                --headers $headers
                --content-type application/octet-stream
                $url)
    }
}

def main [tag: string] {
    for file in (glob claude-statusline-*) {
        try {
            print $"Uploading ($file)..."
            upload $file $tag
            print $"(ansi green)OK: Uploaded ($file)(ansi reset)"
        } catch {|e|
            error make {msg: $"Failed to upload ($file) - ($e.msg)"}
        }
    }
    print $"(ansi green)OK: Release upload complete(ansi reset)"
}
