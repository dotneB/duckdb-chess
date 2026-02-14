## Context

`chess_timecontrol_normalize` currently handles strict PGN forms plus several inferred shorthand families (`N+I` minute inference, apostrophes, `G/` prefixes, and selected free-text templates). Review of `timecontrolfreq3.csv` shows a remaining high-frequency OTB tail with consistent semantics but unsupported notation, especially compact FIDE two-stage strings and abbreviated minute/second phrasing. The change should increase coverage without weakening the parser's conservative stance on ambiguous values.

## Goals / Non-Goals

**Goals:**

- Normalize additional high-confidence, high-frequency `TimeControl` variants to canonical seconds-based output.
- Add targeted parsing for compact FIDE two-stage forms that imply `40/<base>+<inc>:<rest>+<inc>`.
- Support compact minute/second abbreviations and clock-style bases used in single-stage controls.
- Preserve auditability with explicit warning codes and `inferred=true` for all non-strict interpretations.
- Keep strict parsing behavior unchanged for already canonical values.

**Non-Goals:**

- No broad NLP or multilingual grammar engine for arbitrary prose.
- No changes to `read_pgn` schema or raw `TimeControl` storage.
- No aggressive reinterpretation of unclear values lacking strong structural anchors.

## Decisions

1) Extend inference with narrowly scoped pattern families

- Add explicit helpers for:
  - apostrophe increment with per-move suffix (for example `3' + 2''/mv from move 1`)
  - compact minute/second text forms (for example `90m+30s`, `Standard: 90mins + 30sec increment`)
  - clock-style base notation (for example `1:30.00 + 30 seconds increment from move 1`)
  - compact FIDE two-stage shorthand (for example `90'/40+30'/G+30''`, `90'/40m + 30'/end & 30/m`)

Rationale: these families recur with meaningful frequency and map to deterministic canonical outputs.

Alternatives considered:

- Generic free-text extraction for all numeric phrases: rejected due to elevated false-positive risk.

2) Introduce conservative normalization aliases and qualifier handling

- Treat trailing alphabetic qualifier tails as ignorable only after a valid control core is recognized; do not maintain a language-specific keyword list.
- Restrict qualifier stripping to suffixes that contain no digits and no time-control operators so structural content is never discarded.
- Normalize symbolic connectors (`&`) only inside recognized staged templates.

Rationale: supports common suffix/punctuation noise without turning the parser into a permissive text sanitizer.

Alternatives considered:

- Global stopword stripping before parse: rejected because it can hide malformed values and create accidental matches.

3) Define explicit policy for ambiguous staged shorthand

- Parse `90 + 30 + 30s per move` as compact two-stage FIDE shorthand: `40/5400+30:1800+30`.
- Parse `90+30/30+30` as two-stage shorthand with unknown move cutoff: `5400+30:1800+30`.
- Emit dedicated warnings for inferred compact staged formats and missing move qualifier cases.

Rationale: this preserves usable normalization while exposing uncertainty in structured output.

Alternatives considered:

- Forcing all staged shorthands to include `/40`: rejected because it would miss established corpus variants.
- Inventing `/40` when omitted: rejected to avoid asserting unavailable structure.

4) Keep parser pipeline ordering stable

- Continue strict parse before inference.
- Keep new families in targeted inference/template phase after existing strict/spec guards.
- Maintain NULL on unmatched/low-confidence input.

Rationale: avoids regressions for canonical values and keeps failure behavior predictable.

## Risks / Trade-offs

- [Over-matching compact numeric forms] -> Mitigation: require unit markers or staged structural anchors; add negative tests.
- [Interpretation drift for shorthand with missing move qualifiers] -> Mitigation: use explicit warning codes and avoid inventing `/40` in outputs where not specified.
- [Regex complexity/performance] -> Mitigation: keep pattern set small and focused on observed high-frequency variants.
- [Behavioral surprise from qualifier stripping] -> Mitigation: only strip bounded alphabetic suffixes after successful core parse and record warnings.

## Migration Plan

- No data/schema migration required.
- Implement parser updates in `src/chess/timecontrol.rs` with unit tests first.
- Extend `test/sql/chess_timecontrol.test` for user-visible SQL behavior.
- Validate with `just dev` and optionally spot-check normalization coverage on the frequency corpus.

## Open Questions

- Should ambiguous staged shorthand like `90+30/30+30` also include an inferred move cutoff in normalized output, or remain move-unspecified with warnings? (Current decision: remain move-unspecified.)
