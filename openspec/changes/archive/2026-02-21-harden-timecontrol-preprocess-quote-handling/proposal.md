## Why

Some real-world PGN `TimeControl` tags contain unmatched, nested, or mixed quote characters that currently survive preprocessing and cause otherwise valid controls to fail normalization. Hardening quote handling now improves parse success and consistency for downstream category/JSON functions without changing intended canonical outputs.

## What Changes

- Tighten `TimeControl` preprocessing so quote normalization/removal is deterministic for mixed single/double/apostrophe variants and does not corrupt numeric/operator tokens.
- Ensure malformed or noisy quote usage degrades safely (parse returns NULL) instead of producing partially mangled intermediates.
- Add regression coverage for quote-heavy inputs across normalize/category/JSON paths to guarantee stable behavior.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `timecontrol-normalization`: Refine preprocessing requirements so quote handling accepts common noisy quoting forms while preserving structural tokens and existing normalization semantics.

## Impact

- Affected code: TimeControl preprocessing/parsing logic in Rust and associated scalar wrappers.
- Affected tests: SQLLogicTest and Rust tests for `chess_timecontrol_normalize`, `chess_timecontrol_category`, and `chess_timecontrol_json`.
- Public API: No new functions; behavior is a hardening update to existing normalization capability.
