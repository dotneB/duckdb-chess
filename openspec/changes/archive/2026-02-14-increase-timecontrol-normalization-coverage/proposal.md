## Why

Current `TimeControl` normalization covers common shorthand, but several high-frequency OTB formats in `timecontrolfreq3.csv` still normalize to NULL. This leaves meaningful volume uncategorized and fragments time-control analysis across near-equivalent strings.

## What Changes

- Extend lenient parsing to cover additional high-confidence real-world patterns observed in the frequency corpus.
- Add compact FIDE two-stage shorthand handling for variants that encode `base/40 + rest + increment` with apostrophes, `bonus`, `additional`, or compact separators.
- Add broader single-stage minute+increment handling for compact text forms (for example `mins`, `m`, `sec`, `seconds`, `/move`, `increment`) and clock-style base notation (for example `1:30.00`).
- Allow safe, language-agnostic qualifier stripping after a recognized control core (for example `OFICIAL`) with explicit parse warnings for auditability.
- Add/expand Rust unit tests and SQLLogicTests for newly supported formats and ambiguity guardrails.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `timecontrol-normalization`: Increase normalization coverage for high-frequency non-spec `TimeControl` strings while preserving conservative inference behavior.

## Impact

- Specs: delta updates under `specs/timecontrol-normalization/spec.md` for new accepted input scenarios and inference/warning expectations.
- Parser implementation: `src/chess/timecontrol.rs`.
- Tests: `src/chess/timecontrol.rs` unit tests and `test/sql/chess_timecontrol.test`.
- User-facing behavior: fewer NULL normalized values for common OTB `TimeControl` variants.
