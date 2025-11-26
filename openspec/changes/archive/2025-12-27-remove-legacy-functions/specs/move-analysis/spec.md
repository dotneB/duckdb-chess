# move-analysis Spec Delta

## REMOVED Requirements

### Requirement: Legacy Function Name Support for JSON Export
**Reason**: Extension is unpublished, no backward compatibility needed. The `moves_json` function name has been replaced by `chess_moves_json` which provides identical functionality with proper naming convention.

**Migration**: Replace all calls to `moves_json()` with `chess_moves_json()`. The function signature and behavior remain identical.

**Previous Behavior**: The extension exposed both `moves_json` and `chess_moves_json` as duplicate registrations of the `ChessMovesJsonScalar` implementation.

#### Scenario: Legacy moves_json function available
- **WHEN** user calls `moves_json('1. e4 e5')` in SQL
- **THEN** the function was registered and returned the same JSON array as `chess_moves_json('1. e4 e5')`

#### Scenario: Backward compatibility verification
- **WHEN** test queries compared `moves_json('1. e4 e5') = chess_moves_json('1. e4 e5')`
- **THEN** both functions returned identical results

**After removal**: The `moves_json` function name will no longer be registered. Users must use `chess_moves_json()` instead.
