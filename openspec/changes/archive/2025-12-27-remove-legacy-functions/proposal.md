# Change: Remove Legacy Function Names

## Why
The extension has two legacy function names (`filter_movetext_annotations` and `moves_json`) that were kept for backward compatibility when introducing the `chess_` prefix naming convention. Since this extension is not yet published, there are no external users depending on these legacy names, making this the ideal time to remove them before any stable release.

## What Changes
- Remove registration of `filter_movetext_annotations` function (replaced by `chess_moves_normalize`)
- Remove registration of `moves_json` function (replaced by `chess_moves_json`)
- Update test files to use only the new `chess_` prefixed names
- Remove backward compatibility test sections

**BREAKING**: This removes the legacy function names. Since the extension is unpublished, this has no external impact.

## Impact
- Affected specs: `annotation-filtering`, `move-analysis`
- Affected code:
  - `src/chess/mod.rs:19-21` - Remove legacy function registrations
  - `test/sql/filter_movetext_annotations.test` - Remove backward compatibility test
  - `test/sql/filter_movetext_column.test` - Update function calls
  - `test/sql/moves_json.test` - Remove backward compatibility test
- Code simplification: Removes maintenance burden of supporting duplicate function names
- Testing: All existing functionality remains available through `chess_` prefixed functions
