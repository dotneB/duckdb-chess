## 1. Add Dependency

- [x] 1.1 Add `smallvec` crate to `Cargo.toml` dependencies

## 2. Update filter.rs

- [x] 2.1 Add `use smallvec::SmallVec` import
- [x] 2.2 Add `MoveList` type alias: `type MoveList = SmallVec<[String; 128]>` (capacity based on 17.6M game dataset)
- [x] 2.3 Update `ParsedMovetext::sans` field type from `Vec<String>` to `MoveList`
- [x] 2.4 Update `NormalizeVisitor::sans` field type from `Vec<String>` to `MoveList`

## 3. Update moves.rs

- [x] 3.1 Add `use smallvec::SmallVec` import
- [x] 3.2 Update visitors that collect moves to use `MoveList` type alias

## 4. Verification

- [x] 4.1 Run `just dev` to verify lint, build, and tests pass
- [x] 4.2 Run `just full` to verify release build and tests pass