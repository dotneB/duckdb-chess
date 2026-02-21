## Why

Some PGN `TimeControl` values include spaces around increment operators (for example `90 + 30`).
These values are semantically valid but currently fail or normalize inconsistently, which creates avoidable `parse_error` noise and inconsistent query results.

## What Changes

- Update the `TimeControl` normalization flow to accept optional whitespace around operator tokens used in compound controls.
- Ensure normalized output is emitted in canonical compact form (no operator-adjacent whitespace) after successful parsing.
- Extend parser and SQL-visible tests to cover space-padded operator forms and verify unchanged behavior for already-valid compact forms.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `timecontrol-normalization`: Expand accepted `TimeControl` input grammar to tolerate operator-adjacent whitespace while preserving canonical normalized output and existing failure semantics for truly invalid inputs.

## Impact

- Affected code: time-control parsing/normalization helpers and any call sites that emit normalized `TimeControl` strings.
- Affected tests: unit tests for normalization parser and SQLLogicTests that validate `read_pgn` output/`parse_error` behavior.
- Public API surface remains the same; behavior becomes more permissive for valid inputs with incidental whitespace.
