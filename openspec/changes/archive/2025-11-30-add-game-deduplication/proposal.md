# Add Game Deduplication Functions

## Summary
Add scalar functions to DuckDB to support identifying duplicate chess games based on move sequences. This includes functions for hashing move sequences, normalizing them, and checking if one game is a subset of another (subsumption).

## Motivation
Large PGN databases often contain duplicate games or fragments of games. To clean these datasets, users need tools to identify exact duplicates (same moves) and subsumed duplicates (one game is a prefix of another). While full fuzzy matching (for player names etc.) can be complex, structural deduplication based on moves is deterministic and high-value.

Ref: https://lumbrasgigabase.com/en/how-duplicate-games-are-found-en/

## Relationship to existing functions
This proposal unifies and renames existing functions to follow a consistent `chess_` prefix and consolidating cleaning logic.

1.  **Renaming**: `moves_json` -> `chess_moves_json`.
2.  **Unification**: `filter_movetext_annotations` -> `chess_moves_normalize`.
    *   The new `chess_moves_normalize` will be stricter than the old `filter...`. It will remove comments `{...}`, recursive variations `(...)`, and numeric annotation glyphs (NAGs like `$1`).
    *   This provides a single canonical "main line" function for deduplication and cleaning.

## Proposed Changes
1.  **Rename**: `moves_json` to `chess_moves_json`.
2.  **Move Normalization**: Implement `chess_moves_normalize(movetext)` to strip comments/NAGs/RAVs and standardize spacing. Replace `filter_movetext_annotations` with this function.
3.  **Move Hashing**: Add `chess_moves_hash(movetext)` to compute a hash of the normalized move sequence.
4.  **Subsumption Check**: Add `chess_moves_subset(short_movetext, long_movetext)` to check if `short_movetext` is a prefix of `long_movetext`.

## Impact
-   **Breaking Change**: `moves_json` renamed to `chess_moves_json`.
-   **Breaking Change**: `filter_movetext_annotations` removed/replaced by `chess_moves_normalize`.
-   Enables SQL-based deduplication logic:
    ```sql
    -- Exact duplicates
    SELECT chess_moves_hash(movetext), count(*) 
    FROM games 
    GROUP BY 1 
    HAVING count(*) > 1;
    ```
-   Consistent `chess_` prefix for all scalar functions.
