# Refactor filter_movetext_annotations to Scalar Function

## Problem
The `filter_movetext_annotations` function is currently implemented as a Table Function. This prevents it from being used in projection lists with column references (e.g., `SELECT filter_movetext_annotations(movetext) FROM games`), resulting in a "lateral join column parameters" error. This limits the utility of the function for cleaning existing datasets.

## Solution
Refactor `filter_movetext_annotations` to be a Scalar Function. This allows it to be used naturally in `SELECT` clauses to transform data row-by-row. The core filtering logic remains unchanged, but the interface to DuckDB changes from `VTab` to `ScalarFunction`.

## Impact
- **Breaking Change**: The function type changes from Table Function to Scalar Function. Users explicitly using `FROM filter_movetext_annotations(...)` might need to adjust their queries to `SELECT filter_movetext_annotations(...)` (though DuckDB often allows scalar functions in FROM clause as a single-value source, the primary intended usage changes).
- **Performance**: Scalar functions are generally more efficient for row-wise transformations than lateral joins to table functions.
