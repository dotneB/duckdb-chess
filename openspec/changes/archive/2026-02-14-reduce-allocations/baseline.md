# Allocation Baseline (non-Criterion)

Date: 2026-02-14

Method:
- Used PowerShell `Measure-Command` around targeted `cargo test` invocations.
- Kept project-native workflow (no Criterion) to match DuckDB extension constraints.
- Captured one cold run (after code changes) and several warm runs.

Commands:

```powershell
powershell -NoProfile -Command '$t = Measure-Command { cargo test test_process_moves_basic --quiet }; [math]::Round($t.TotalMilliseconds, 1)'
powershell -NoProfile -Command '$t = Measure-Command { cargo test test_process_moves_with_invalid_move --quiet }; [math]::Round($t.TotalMilliseconds, 1)'
powershell -NoProfile -Command '$t = Measure-Command { cargo test test_ply_count_ignores_junk_and_stops --quiet }; [math]::Round($t.TotalMilliseconds, 1)'
powershell -NoProfile -Command '$t = Measure-Command { cargo test test_visitor_basic_parsing --quiet }; [math]::Round($t.TotalMilliseconds, 1)'
```

Observed timings:
- `test_process_moves_basic` (cold): `3811.9 ms`
- `test_process_moves_with_invalid_move` (warm): `357.5 ms`
- `test_ply_count_ignores_junk_and_stops` (warm): `362.6 ms`
- `test_visitor_basic_parsing` (warm): `333.9 ms`

Notes:
- Warm-run timings are the better comparison point for this change.
- Re-run the same commands after implementation completion to compare deltas.
