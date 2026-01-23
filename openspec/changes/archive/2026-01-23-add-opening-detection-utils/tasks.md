# Implementation Tasks

## 1. Spec and Validation
- [x] 1.1 Run `openspec validate add-opening-detection-utils --strict`
- [x] 1.2 Resolve any validation errors

## 2. Implement `chess_fen_epd(fen)`
- [x] 2.1 Add scalar function `chess_fen_epd(fen)` returning EPD as first 4 FEN fields
- [x] 2.2 Define behavior for NULL/empty input (return `NULL`)
- [x] 2.3 Add unit tests for common cases:
  - [x] Standard start position
  - [x] Positions with en-passant `-` and with a square (e.g., `e3`)
  - [x] Invalid FEN input (return `NULL`)

## 3. Implement `chess_ply_count(movetext)`
- [x] 3.1 Add scalar function `chess_ply_count(movetext)` returning the number of valid plies in movetext
- [x] 3.2 Define behavior for NULL/empty input (return 0; match `chess_moves_json` convention)
- [x] 3.3 Add unit tests for counting behavior:
  - [x] Ignores move numbers, comments, NAGs, and result markers
  - [x] Stops at first illegal/unparseable SAN token

## 4. Extend `chess_moves_json` to include `epd` and add `max_ply` overload
- [x] 4.1 Update JSON objects to include "epd" alongside `ply`, `move`, and `fen`
- [x] 4.2 Add overload `chess_moves_json(movetext, max_ply)` that returns at most `max_ply` move objects
- [x] 4.3 Ensure EPD is computed from the generated FEN (not from input)
- [x] 4.4 Add/adjust unit tests to assert `epd` is present and matches the FEN-derived value
- [x] 4.5 Define and test edge cases for `max_ply`:
  - [x] `max_ply <= 0` returns `'[]'`
  - [x] `max_ply IS NULL` behaves like `chess_moves_json(movetext)`

## 5. Pipeline Example (non-code)
- [x] 5.1 Add a short example SQL snippet showing how to:
  - [x] Determine maximum opening ply from the reference dataset
  - [x] Use `chess_moves_json(movetext, max_opening_ply)` and join by `epd`
  - [x] Choose the highest `ply` match

## 6. Verification
- [x] 6.1 Run `make dev`
- [x] 6.2 Run `make test-release`
