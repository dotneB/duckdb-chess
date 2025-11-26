## REMOVED Requirements

### Requirement: Movetext Annotation Removal
**Reason**: Superseded by `chess_moves_normalize` which provides stricter cleaning (removing variations and NAGs as well) and follows the new naming convention.
**Migration**: Use `chess_moves_normalize(movetext)` instead of `filter_movetext_annotations(movetext)`.
