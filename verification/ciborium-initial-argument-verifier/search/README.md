# Pre-edit counterexample search

- production binding
  - path dependencies on the clean detached 0.2.2 checkout
  - direct public `ciborium_ll::Decoder::pull`
  - ordinary public `ciborium::de::from_reader::<u64, _>` bridge for positive integers
- finite domain
  - ordinary non-tag major types `0..=5`
  - all direct arguments `0..=23`
  - all one-byte and two-byte argument values
  - boundary-pattern arguments for four-byte and eight-byte widths
  - every proper truncation of each selected argument width
  - reserved additional values `28..=30` for all eight major prefixes
- oracle
  - RFC 8949 initial-byte split
  - widths `0`, `1`, `2`, `4`, and `8` for additional values `0..=27`
  - big-endian unsigned argument construction
  - exactly `1 + width` logical bytes consumed on success
  - syntax error at offset zero for reserved additional values
  - I/O failure and no completed argument advance for truncation
- exclusions
  - additional value `31`, indefinite forms, tags, and major type `7` semantics
  - float, simple-value, full-value, recursive, serde visitor, and downstream behavior
  - preferred or canonical serialization policy

The executable checks the checkout identity, cleanliness, detached state, and disabled push URL before running. It exits unsuccessfully on the first mismatch and otherwise prints exact case totals.
