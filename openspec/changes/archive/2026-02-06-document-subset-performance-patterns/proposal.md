## Why

Users working with large game datasets need clear guidance on when to use `chess_moves_subset` versus faster string-prefix checks over pre-normalized movetext. Without explicit documentation, it is easy to choose a slower pattern or use incorrect semantics such as `contains`.

## What Changes

- Add README guidance for subset checks at scale, including decision criteria by data quality.
- Document recommended SQL patterns for raw/noisy movetext (`chess_moves_subset`) versus normalized/materialized canonical movetext (`starts_with`).
- Add explicit warning that `contains` is not equivalent to subset prefix semantics.
- Include examples covering result-marker caveats and normalization assumptions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `move-analysis`: add documentation-level requirements describing correct performance/semantics usage patterns for subset checks.

## Impact

- Affected docs: `README.md` usage/performance guidance for subset workflows.
- Affected specs: `openspec/specs/move-analysis/spec.md` documentation guidance scenarios.
- User-visible effect: clearer guidance and fewer incorrect subset queries on large datasets.
