#!/usr/bin/env nu

def validate-token [headers: list]: nothing -> string {
    let user = try { http get "https://codeberg.org/api/v1/user" --headers $headers } catch { null }
    if $user == null or ($user | describe) == "string" {
        error make {
    msg: "Token validation failed: API returned null or invalid response. Check CODEBERG_TOKEN."
}
    }
    $user.login
}

def get-or-create-release []: record -> record<id: int, created: bool> {
    let ctx = $in
    let release_url = $"($ctx.api_base)/releases/tags/($ctx.tag)"
    let release = try { http get $release_url --headers $ctx.headers } catch { null }
    if $release == null {
        let body = {tag_name: $ctx.tag, name: $ctx.tag}
        let created = try {
            http post $"($ctx.api_base)/releases" $body --headers $ctx.headers --content-type application/json
        } catch {|e|
            error make {msg: $"Could not create release - ($e.msg)"}
        }
        {id: $created.id, created: true}
    } else {
        {id: $release.id, created: false}
    }
}

def get-existing-assets []: record -> list<string> {
    let ctx = $in
    let release_url = $"($ctx.api_base)/releases/tags/($ctx.tag)"
    try {
        let rel = http get $release_url --headers $ctx.headers
        $rel.assets | get name? | default []
    } catch { [] }
}

def upload-single-asset []: record -> record<asset: string, status: string> {
    let ctx = $in
    if not ($ctx.asset | path exists) {
        {asset: $ctx.asset, status: "missing"}
    } else if $ctx.asset in $ctx.existing {
        {asset: $ctx.asset, status: "exists"}
    } else {
        let upload_url = $"($ctx.api_base)/releases/($ctx.release_id)/assets?name=($ctx.asset)"
        try {
            open $ctx.asset | http post $upload_url --headers $ctx.headers --content-type application/octet-stream
            {asset: $ctx.asset, status: "uploaded"}
        } catch {|e|
            error make {msg: $"Failed to upload ($ctx.asset) - ($e.msg)"}
        }
    }
}

def upload-all-assets []: record -> list<record<asset: string, status: string>> {
    let ctx = $in
    $ctx.assets | each {|asset| $ctx | insert asset $asset | upload-single-asset }
}

def print-upload-results [results: list]: nothing -> nothing {
    for r in $results {
        match $r.status {
            "uploaded" => { print $"  OK: ($r.asset) uploaded" }
            "exists" => { print $"  SKIP: ($r.asset) - already on release" }
            "missing" => { print $"  SKIP: ($r.asset) - file not found locally" }
        }
    }
}

def print-summary [results: list, total: int]: nothing -> nothing {
    let uploaded = $results | where status == "uploaded" | length
    let skipped = $results | where status == "exists" | length
    let missing = $results | where status == "missing" | length
    print ""
    print "=== Summary ==="
    print $"Uploaded: ($uploaded)
Skipped \(exists\): ($skipped)
Skipped \(missing\): ($missing)"
    if $uploaded > 0 {
        print "Release updated successfully"
    } else if $skipped == $total {
        print "Nothing to do - all assets already uploaded"
    } else {
        print "No assets were uploaded"
    }
}

def main [tag: string]: nothing -> nothing {
    let token = $env.CODEBERG_TOKEN? | default ""
    if $token == "" {
        print --stderr "ERROR: CODEBERG_TOKEN not set"
        exit 1
    }
    let repo = $env.CI_REPO? | default "jruz/claude-statusline"
    mut ctx = {
        api_base: $"https://codeberg.org/api/v1/repos/($repo)"
        tag: $tag
        headers: [Authorization $"token ($token)"]
        assets: [
    "claude-statusline-linux-x86_64"
    "claude-statusline-macos-x86_64"
    "claude-statusline-macos-arm64"
    "claude-statusline-windows-x86_64.exe"
]
    }
    print $"=== Release Upload: ($tag) ==="
    print ""
    print "[1/4] Validating token..."
    let username = validate-token $ctx.headers
    print $"OK: Authenticated as ($username)"
    print ""
    print "[2/4] Checking release..."
    let release = $ctx | get-or-create-release
    if $release.created { print $"OK: Created release ($tag)" } else { print $"OK: Release ($tag) exists" }
    $ctx = $ctx | insert release_id $release.id
    print ""
    print "[3/4] Checking existing assets..."
    let existing = $ctx | get-existing-assets
    print $"Found ($existing | length) existing assets"
    $ctx = $ctx | insert existing $existing
    print ""
    print "[4/4] Uploading assets..."
    let results = $ctx | upload-all-assets
    print-upload-results $results
    print-summary $results ($ctx.assets | length)
}
