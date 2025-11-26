# Refactor Monolithic lib.rs into Modules

## Summary
Refactor the monolithic `src/lib.rs` file into a set of focused, cohesive modules to improve code organization, readability, and maintainability. The refactoring will separate data structures, PGN parsing logic, DuckDB table function implementations, and utility functions.

## Motivation
The `src/lib.rs` file has grown to approximately 600 lines and currently handles multiple distinct responsibilities:
1.  **Data Definitions**: `GameRecord` and shared state structures.
2.  **Core Logic**: PGN parsing via the `Visitor` trait.
3.  **Database Integration**: DuckDB `VTab` implementations (`read_pgn`, `filter_movetext_annotations`).
4.  **Utilities**: Helper functions for text processing.

This coupling makes navigation difficult and increases the cognitive load when modifying specific features. As the project grows (e.g., adding parallel processing or new SQL functions), this structure will become increasingly unmanageable.

## Proposed Solution
Split `src/lib.rs` into the following module structure:

-   **`src/types.rs`**: Shared domain types (e.g., `GameRecord`).
-   **`src/visitor.rs`**: PGN parsing logic and the `GameVisitor` implementation.
-   **`src/reader.rs`**: The `read_pgn` table function and its binding logic.
-   **`src/filter.rs`**: The `filter_movetext_annotations` logic and table function.
-   **`src/lib.rs`**: A clean entry point responsible only for module declaration and extension registration.

## Impact
-   **Architecture**: Introduces a clear separation between the "Core Domain" (PGN parsing) and the "Adapter Layer" (DuckDB integration).
-   **Maintainability**: focused files are easier to read and test.
-   **Compatibility**: No changes to the external SQL API or behavior.
