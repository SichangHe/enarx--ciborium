# Bounded Ciborium initial-argument verifier

This portable experiment is fixed to `enarx/ciborium` release base
`fdf3ec5c53256000f8e12f06c361ca18026a8d99` (`v0.2.2`). It changes no
production source and makes no upstream defect or full-decoder claim.

## Boundary

The experiment covers the ordinary definite initial-byte and numeric-argument
transition from `ciborium::de::from_reader` into
`ciborium_ll::Decoder::pull`: non-tag major types `0..=5`, direct arguments,
big-endian one-, two-, four-, and eight-byte arguments, successful logical
consumption, reserved additional values, and proper argument truncations.
`SCOPE.md` preserves the reviewed scope and exclusions byte-for-byte.

The production link at the fixed base is:

- `ciborium/src/de/mod.rs`, blob `1952a4b896de0b780e22d095b855e1d535db82c7`
- `ciborium-ll/src/dec.rs`, blob `3bd0889ceb3bed4966dc0e2d51f5423ec34a04a7`
- `ciborium-ll/src/hdr.rs`, blob `dec178815447ce25991df8bd57de50d501964694`

The released `ciborium` 0.2.2 archive has SHA-256
`42e69ffd6f0917f5c029256a24d0161db17cea3997d185db0d35926308770f0e`;
the released `ciborium-ll` 0.2.2 archive has SHA-256
`57663b653d948a338bfb3eeba9bb2fd5fcfaecb9e199e87e1eda4d9e8b240fd9`.

## Preserved reviewed content

The scope, proof, design note, search/host harness source, Cargo manifest,
lockfile, and harness note are byte-for-byte copies of the frozen reviewed packet.
`reviewed-source.sha256` records their identities.

The harness intentionally retains its reviewed `../checkout` layout and
checkout guard. `check.sh` creates that layout only in temporary storage by
placing the unchanged harness beside a clean detached local clone of this
commit's parent. This makes the path portable without changing reviewed Rust
or Cargo content.

## Bounded result

The reviewed finite production search found no mismatch in 395,040 direct
complete decodes, 65,840 ordinary `from_reader::<u64, _>` bridge decodes, all
24 reserved-prefix cases, all 90 proper in-scope truncations, and one empty
input. Four focused host tests passed.

The executable Verus model verified the admitted split, width selection,
big-endian argument construction, successful consumption, and
empty/reserved/truncated partition for arbitrary in-scope model inputs:
`10 verified, 0 errors` with `--no-cheating`; its compiled witness exited zero.
This is a universal result for the standalone model with finite production
binding, not a formal proof of generic I/O or serde.

## Reproduce

Use x86-64 Linux and the exact Verus release
[`0.2025.11.30.840fa61`](https://github.com/verus-lang/verus/releases/tag/release/0.2025.11.30.840fa61).
The official `verus-0.2025.11.30.840fa61-x86-linux.zip` asset has SHA-256
`c08b8959a98b15519a07f1eb813db8c34a6311606e4ff7dda7e8ae135b5cfcb2`;
its `verus` executable has SHA-256
`59659856b1cc28264a8d21dc11707feb435f801ff386949d60c7ab5a995e7365`.

Run from a clean checkout of this exact commit:

```console
./verification/ciborium-initial-argument-verifier/check.sh /path/to/verus
```

The checker validates ancestry and production blobs, stages the unchanged
reviewed harness in temporary storage, repeats the bounded search and focused
tests, runs formatting and warnings-denied Clippy, verifies the proof with
`--no-cheating`, and executes the compiled proof witness. Build, verifier, and
temporary Git state remain outside the checkout.

This commit is a local candidate awaiting distinct read-only review. It does
not authorize publication, contact, a report, an issue, or a pull request.
