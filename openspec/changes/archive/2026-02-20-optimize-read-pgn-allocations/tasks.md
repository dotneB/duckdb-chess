## 1. Tests & Baseline

- [x] 1.1 Add unit test covering unknown header tags are ignored by output mapping
- [x] 1.2 Add unit test covering duplicate known header tags use the first value (no override)
- [x] 1.3 Add unit test covering movetext finalization matches `trim()` semantics

## 2. Header Tag Allocation Fast Path

- [x] 2.1 Refactor `HeaderFields` to match known keys and decode/allocate tag values only when the destination field is unset
- [x] 2.2 Update `GameVisitor::tag` to skip decoding for unknown tags and duplicates (delegate to the new helper)
- [x] 2.3 Verify `Result` precedence (outcome marker vs `Result` tag fallback) remains unchanged

## 3. Movetext Finalization

- [x] 3.1 Update `build_game_record()` to avoid unconditional `trim().to_string()` cloning when movetext is already trimmed
- [x] 3.2 Validate movetext behavior parity for both success and error-finalized games

## 4. Verification

- [x] 4.1 Run `just test` (unit + SQLLogicTest) and confirm all tests pass
- [x] 4.2 Run `just check` (fmt + clippy) and fix any failures
- [x] 4.3 Run `just dev`
- [x] 4.4 Run `just full`
