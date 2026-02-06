## 1. Spec and Documentation Content

- [x] 1.1 Add a README section that explains when to use `chess_moves_subset` versus prefix filtering on canonical movetext
- [x] 1.2 Add SQL examples for raw/noisy movetext workflows that use `chess_moves_subset`
- [x] 1.3 Add SQL examples for normalized/materialized workflows that use `chess_moves_normalize` + `starts_with`
- [x] 1.4 Add explicit warning and example showing why `contains` is not equivalent to subset-prefix semantics
- [x] 1.5 Add result-marker and normalization caveats to prevent semantic misuse of string-only filters

## 2. Consistency and Verification

- [x] 2.1 Ensure README examples are consistent with current function signatures and behavior (`chess_moves_subset`, `chess_moves_normalize`)
- [x] 2.2 Confirm docs do not imply behavior changes to existing SQL functions
- [x] 2.3 Run `make check` and fix any formatting/lint issues introduced by docs updates
