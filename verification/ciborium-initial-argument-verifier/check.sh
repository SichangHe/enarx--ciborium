#!/bin/sh
set -eu

base='fdf3ec5c53256000f8e12f06c361ca18026a8d99'
verus_sha256='59659856b1cc28264a8d21dc11707feb435f801ff386949d60c7ab5a995e7365'
disabled_push='https://invalid.invalid/enarx/ciborium-direct-push-disabled'

if [ "$#" -ne 1 ]; then
    printf 'usage: %s /path/to/verus\n' "$0" >&2
    exit 2
fi

verus_bin=$1
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
scratch=$(mktemp -d)
trap 'rm -r -- "$scratch"' EXIT

test "$(git -C "$repo_root" show -s --format=%P HEAD)" = "$base"
test "$(git -C "$repo_root" rev-list --count "$base"..HEAD)" = '1'
test -z "$(git -C "$repo_root" status --porcelain=v1)"
git -C "$repo_root" diff --check "$base" HEAD
git -C "$repo_root" diff --quiet "$base" HEAD -- . \
    ':(exclude)verification/ciborium-initial-argument-verifier'
test "$(git -C "$repo_root" rev-parse "$base:ciborium/src/de/mod.rs")" = \
    '1952a4b896de0b780e22d095b855e1d535db82c7'
test "$(git -C "$repo_root" rev-parse "$base:ciborium-ll/src/dec.rs")" = \
    '3bd0889ceb3bed4966dc0e2d51f5423ec34a04a7'
test "$(git -C "$repo_root" rev-parse "$base:ciborium-ll/src/hdr.rs")" = \
    'dec178815447ce25991df8bd57de50d501964694'

(cd "$script_dir" && sha256sum -c reviewed-source.sha256)
test -x "$verus_bin"
test "$(sha256sum "$verus_bin" | awk '{print $1}')" = "$verus_sha256"

layout="$scratch/layout"
mkdir -p "$layout"
cp -R "$script_dir/search" "$layout/counterexample-search"
git clone --quiet --no-checkout --origin upstream \
    -c "remote.upstream.pushurl=$disabled_push" "$repo_root" "$layout/checkout"
git -C "$layout/checkout" checkout --quiet --detach "$base"

target_dir=${CIBORIUM_ARGUMENT_CARGO_TARGET_DIR:-$scratch/cargo-target}
export CARGO_TARGET_DIR="$target_dir"
search_dir="$layout/counterexample-search"

(cd "$search_dir" && cargo fmt --check)
(cd "$search_dir" && cargo run --quiet --locked --release) \
    >"$scratch/search.txt"
grep -Fx 'status=no-counterexample' "$scratch/search.txt"
grep -Fx 'direct_complete_cases=395040' "$scratch/search.txt"
grep -Fx 'facade_complete_cases=65840' "$scratch/search.txt"
grep -Fx 'reserved_cases=24' "$scratch/search.txt"
grep -Fx 'truncated_cases=90' "$scratch/search.txt"
grep -Fx 'empty_cases=1' "$scratch/search.txt"
(cd "$search_dir" && cargo test --quiet --locked)
(cd "$search_dir" && cargo clippy --quiet --locked --no-deps --all-targets \
    -- -D warnings)

mkdir "$scratch/proof"
cp "$script_dir/proof/cbor_initial.rs" "$scratch/proof/"
(cd "$scratch/proof" && "$verus_bin" --no-cheating --compile cbor_initial.rs \
    >verus.txt && ./cbor_initial)
grep -Fx 'verification results:: 10 verified, 0 errors' "$scratch/proof/verus.txt"
if grep -Eq '\b(assume|admit|external_body|assume_specification)\b' \
    "$script_dir/proof/cbor_initial.rs"; then
    exit 1
fi

printf '%s\n' 'checks=pass'
