# annotation-filtering Spec Delta

## REMOVED Requirements

### Requirement: Legacy Function Name Support
**Reason**: Extension is unpublished, no backward compatibility needed. The `filter_movetext_annotations` function name has been replaced by `chess_moves_normalize` which provides the same functionality with proper naming convention.

**Migration**: Replace all calls to `filter_movetext_annotations()` with `chess_moves_normalize()`. The function signature and behavior remain identical.

**Previous Behavior**: The extension exposed both `filter_movetext_annotations` and `chess_moves_normalize` as duplicate registrations of the same underlying functionality.

#### Scenario: Legacy function name available
- **WHEN** user calls `filter_movetext_annotations('1. e4 {comment} e5')` in SQL
- **THEN** the function was registered and returned `'1. e4 e5'`

**After removal**: This function name will no longer be registered. Users must use `chess_moves_normalize()` instead.
