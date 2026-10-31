#!/bin/sh
# Pre-review gate: build rules plus house rules.
set -eu
cargo fmt --check
cargo clippy --all-targets --features test-support -- -D warnings
cargo test --lib --features test-support --quiet
rustup target list --installed | grep -qx wasm32-unknown-unknown || {
  echo "the web lint needs wasm32: rustup target add wasm32-unknown-unknown" >&2
  exit 1
}
# gpui_web's wasm_thread asks for an unstable feature, as in scripts/web.sh.
RUSTC_BOOTSTRAP=1 MOK_GALLERY_ASSETS=https://example.invalid/ \
  cargo clippy --example gallery --target wasm32-unknown-unknown -- -D warnings
python3 scripts/stories.py --check
# The CLI beside the library: its own lints and tests, then the registry it feeds and the captures every entry needs.
cli="--manifest-path cli/Cargo.toml"
cargo fmt $cli --check
cargo clippy $cli --all-targets --target-dir target/cli -- -D warnings
cargo test $cli --target-dir target/cli --quiet
cargo run $cli --target-dir target/cli --quiet -- registry . --check
node site/scripts/captures.mjs
fail=0
files=$(git ls-files -co --exclude-standard '*.rs')
multi=$(awk 'FNR==1{prev=0} /^[[:space:]]*\/\// {if(prev) print FILENAME":"FNR; prev=1; next} {prev=0}' $files)
[ -z "$multi" ] || { echo "multi-line comments:"; echo "$multi"; fail=1; }
for f in $files AGENTS.md README.md TASKS.md tasks/*.md; do
  n=$(wc -l < "$f")
  [ "$n" -le 500 ] || { echo "$f has $n lines"; fail=1; }
done
components=$(find registry -name '*.rs' ! -path 'registry/theme/*' ! -path 'registry/motion/*' ! -path '*/tests/*' ! -name 'tests.rs')
raw=$(awk '/#\[cfg\(test\)\]/{nextfile} /(^|[^.[:alnum:]_])px\(/{print FILENAME":"FNR": "$0}' $components)
[ -z "$raw" ] || { echo "raw px in components:"; echo "$raw"; fail=1; }
deferred=$(grep -rn 'deferred(' registry --include='*.rs' | grep -v '^registry/primitives/layer.rs' || true)
[ -z "$deferred" ] || { echo "gpui deferred outside primitives::raise:"; echo "$deferred"; fail=1; }
instant=$(grep -rnE '(^|[^_[:alnum:]])time::(Instant|\{[^}]*Instant)' src examples || true)
[ -z "$instant" ] || { echo "std Instant panics in the browser, use web_time::Instant:"; echo "$instant"; fail=1; }
for f in examples/docs/*.rs; do
  [ -e "$f" ] || continue
  n=$(basename "$f" .rs)
  grep -qx "path = \"$f\"" Cargo.toml && grep -qx "name = \"docs_$n\"" Cargo.toml || { echo "$f needs [[example]] docs_$n in Cargo.toml"; fail=1; }
done
exit $fail
