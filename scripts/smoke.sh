#!/bin/sh
# The test that matters for a copy-based registry: a fresh app, one component from every chapter, and it compiles.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
cargo build -q --manifest-path "$root/cli/Cargo.toml"
cli="$root/cli/target/debug/cargo-mok"
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
cd "$dir"
"$cli" mok new smoke --from "$root"
cd smoke
names=$(python3 - "$root/registry.json" <<'PY'
import json, sys
seen = {}
for c in json.load(open(sys.argv[1]))["components"]:
    if c["status"] == "component" and c["category"] not in seen:
        seen[c["category"]] = c["name"]
print(" ".join(seen.values()))
PY
)
# shellcheck disable=SC2086
"$cli" mok add $names --from "$root"
cargo check
echo "smoke: $(echo "$names" | wc -w) chapters compile in a fresh app"
