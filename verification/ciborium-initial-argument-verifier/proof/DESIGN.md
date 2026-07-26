# Executable Verus harness

- modeled production transition
  - split one initial byte into `major = prefix >> 5` and `additional = prefix & 31`
  - select argument widths `0`, `1`, `2`, `4`, and `8`
  - construct the argument in big-endian order
  - return the exact successful logical consumption `1 + width`
  - distinguish empty input, reserved additional values, and truncation
- universal inputs
  - arbitrary prefix in the admitted `0..=30` additional-value domain
  - arbitrary eight argument bytes
  - arbitrary available-argument-byte count
- production binding limit
  - the model mirrors the pinned `pull_title` and `TryFrom<Title>` arithmetic
  - the separate host harness checks the compiled production functions
  - the proof does not verify the generic reader, serde, indefinite forms, tags, floats, or full values
- executable gate
  - run the staged verifier with `--no-cheating --compile`
  - run the produced native executable
  - reject `assume`, `admit`, `external_body`, and `assume_specification`
