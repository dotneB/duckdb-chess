## Context

`read_pgn` currently exposes `TimeControl` as a raw `VARCHAR` copied from the PGN tag. In practice, large PGN corpora contain a long tail of non-spec values (minute-based shorthand like `3+2`, punctuation variants like `75 | 30`, and free-text descriptions of classical controls). This makes it hard to reliably group/filter by time control and to compute derived metrics without per-dataset cleanup.

This change adds normalization as opt-in SQL scalar functions so we preserve the existing Lichess-compatible `read_pgn` schema and semantics.

Constraints:

- Must remain cross-platform (Windows/macOS/Linux) and avoid panics in extension code paths.
- Prefer small, pure Rust parsing logic that can be unit-tested without DuckDB.
- Preserve the raw tag value; any inferred interpretation must be clearly flagged.

## Goals / Non-Goals

**Goals:**

- Provide a lenient parser/normalizer that converts common real-world `TimeControl` tag variants into a canonical, PGN spec-shaped, seconds-based string.
- Expose SQL scalar function(s) to return:
  - normalized `TimeControl` string (or NULL on failure)
  - structured details (periods, increment, mode) plus explicit warnings and an `inferred` flag
- Keep normalization deterministic and side-effect free; no dependency on game movetext or clocks.

**Non-Goals:**

- Do not change `read_pgn` output columns or replace the `TimeControl` column value.
- Do not implement UI-style rendering (e.g. `1+0`, `1 min`) as output; only spec-shaped seconds-based normalization.
- Do not attempt full multilingual NLP for arbitrary prose; only a small set of high-frequency templates.

## Decisions

1) Public SQL API shape

- Add scalar functions prefixed `chess_`:
  - `chess_timecontrol_normalize(timecontrol) -> VARCHAR` (canonical spec-shaped string; NULL on unparseable/ambiguous input)
  - `chess_timecontrol_json(timecontrol) -> VARCHAR` (JSON string with raw/normalized/periods/warnings/inferred)
- Null handling:
  - `NULL` input yields `NULL` output (DuckDB default NULL propagation is acceptable here; do not coalesce NULL to empty string).

Rationale: keeps `read_pgn` stable while enabling downstream queries to opt into normalization. Returning JSON as `VARCHAR` matches existing patterns (e.g. `chess_moves_json`).

Alternatives considered:

- Mutate `read_pgn.TimeControl` in-place: rejected (would silently change existing behavior and break “raw tag” expectations).
- Add new columns to `read_pgn`: rejected (schema compatibility risk; higher surface area).

2) Canonical representation

- Canonical string always uses seconds and PGN delimiters:
  - unknown: `?`
  - no time limit: `-`
  - sandclock: `*<seconds>`
  - stage: `<seconds>` or `<seconds>+<inc>`
  - stage-by-moves: `<moves>/<seconds>` or `<moves>/<seconds>+<inc>`
  - multi-stage: `:` separator

Rationale: canonicalization enables grouping and comparisons without re-parsing.

3) Parsing architecture

- Implement parsing in a pure Rust module (no DuckDB types) under `src/chess/timecontrol.rs` (or similar) that exposes:
  - `parse_timecontrol(raw: &str) -> Result<ParsedTimeControl, TimeControlError>`
  - `normalize_timecontrol(raw: &str) -> Option<String>`
- `ParsedTimeControl` includes:
  - `raw: String`
  - `normalized: Option<String>`
  - `periods: Vec<Period>` where `Period { moves: Option<u32>, base_seconds: u32, increment_seconds: Option<u32> }`
  - `mode: Unknown|Unlimited|Sandclock|Normal`
  - `warnings: Vec<String>` (stable, machine-friendly codes)
  - `inferred: bool`

Rationale: keeps the core logic easy to unit test, reuse, and fuzz if needed.

4) Lenient parsing pipeline (strict core + inference)

Order matters to reduce false positives:

- Step A: pre-normalize
  - trim; strip surrounding quotes
  - remove spaces around operators (`15 + 10` -> `15+10`)
  - map common separators to spec (`|` -> `+`, `_` -> `+`)
  - strip trailing apostrophes in otherwise-valid tokens (`...+30'` -> `...+30`)
  - record warnings for each transform

- Step B: strict token parse (spec-shaped grammar)
  - parse `?`, `-`, `*<seconds>`, and numeric stages separated by `:`
  - parse stage as `seconds` or `moves/seconds`, each optionally with `+inc`

- Step C: high-confidence unit inference
  - interpret minute shorthand only when confidence is high and emit `inferred=true`:
    - `N+I` with `I <= 60` and `N < 60` => minutes+seconds (`3+2` -> `180+2`)
    - `N+30` where `N in {75, 90}` => minutes+seconds (`90+30` -> `5400+30`)
    - bare `N` where `N < 60` => minutes (`3` -> `180`)
  - apostrophe units:
    - `N'` => minutes; `N''` => seconds (e.g. `10'+5''` -> `600+5`)
  - `G/` or `G:` prefix shorthands:
    - strip `G`/`Game` prefix plus separators (`/`, `:`, `;`) and treat the leading numeric base as minutes
    - support common increment spellings (`+N`, `+Ninc`, `+N seconds/move`, `+N seconds per move`, `+N seconds added per move`)
    - emit warning `interpreted_g_prefix_as_minutes`

- Step D: free-text templates (limited scope)
  - apply a small library of patterns focused on the dominant observed prose:
    - “X minutes for Y moves + Z minutes for rest + W seconds per move”
    - “game in X minutes + W seconds per move”
  - on match, generate a canonical multi-stage string and populate `periods`
  - if no template matches, return failure (normalized NULL) with a concise error

Rationale: strict parsing handles already-correct values cheaply; inference and templates are opt-in only when needed.

Alternatives considered:

- Always interpret small integers as seconds: rejected (produces nonsense like 3-second games from minute shorthand).
- Aggressive inference of `moves/2` as “hours”: deferred (too risky without more observed evidence; keep behind explicit, later rules if needed).

5) DuckDB integration

- Add new VScalar implementations similar to existing ones in `src/chess/moves.rs`.
- Register in `src/chess/mod.rs` with public function names (no macro wrapper needed if we want NULL-in => NULL-out).

For `chess_timecontrol_json`, emit a compact JSON string (no pretty printing) with keys:

- `raw`
- `normalized` (string or null)
- `mode`
- `periods` (array)
- `warnings` (array)
- `inferred` (boolean)

## Risks / Trade-offs

- [False positive inference] interpreting a seconds-based value as minutes (or vice versa) -> Mitigation: apply inference only under narrow, explicit guardrails and always set `inferred=true` + warning codes.
- [Template brittleness] free-text parsing may miss variants or match incorrectly -> Mitigation: keep templates narrow, add fixtures from `timecontrolfreq.csv`, and return NULL rather than guessing broadly.
- [Dependency footprint] adding a regex crate (if used) increases compile size/time -> Mitigation: prefer simple parsing where possible; if regex is introduced, keep patterns few and compile them once.
- [User expectations] consumers may assume normalized output is always available -> Mitigation: return NULL on failure and provide JSON warnings so callers can audit coverage.

## Migration Plan

- No migration required: existing tables/queries using `read_pgn` remain unchanged.
- Adoption is opt-in by calling `chess_timecontrol_normalize(TimeControl)` or `chess_timecontrol_json(TimeControl)` in queries.
- Rollback is simply removing the new scalar registrations; no data migrations.

## Open Questions

- Final function names: keep `chess_timecontrol_json` vs `chess_timecontrol_parse` (spec will lock this down).
- Error surface: should `chess_timecontrol_json` include a dedicated `error` field vs encoding failures as `normalized=null` plus warnings?
- Free-text scope: which non-English tokens (e.g. “seg”, “sek”) are worth including in v1 templates based on frequency?
