# Implementation Tasks

## 1. Data Structure Updates
- [x] 1.1 Add `parse_error: Option<String>` field to `GameRecord` struct
- [x] 1.2 Update `GameRecord::default()` to initialize parse_error as None
- [x] 1.3 Add helper method to set parse error on GameRecord

## 2. Schema Extension
- [x] 2.1 Add 17th column `parse_error` (VARCHAR, nullable) in bind phase
- [x] 2.2 Update column count validation if any exists
- [x] 2.3 Verify schema definition matches new requirements

## 3. Error Handling Modifications
- [x] 3.1 Modify first game parsing error handler (line ~272) to capture game with error
- [x] 3.2 Modify last game parsing error handler (line ~307) to capture game with error
- [x] 3.3 Create partial GameRecord with available data when parsing fails at any stage
- [x] 3.4 Handle header parsing errors (capture what was parsed before error)
- [x] 3.5 Handle movetext parsing errors (capture headers, set movetext empty)
- [x] 3.6 Consider handling line reading errors (line ~251) with context
- [x] 3.7 Store error message in parse_error field instead of only logging
- [x] 3.8 Continue to log warnings to stderr for backward compatibility
- [x] 3.9 Include error stage context in message (e.g., "Header parsing error", "Movetext parsing error")

## 4. Output Phase Updates
- [x] 4.1 Add parse_error vector to output in func() method
- [x] 4.2 Insert parse_error values (NULL or error string) for each game
- [x] 4.3 Handle parse_error as nullable column with set_null() for successful games

## 5. Testing
- [x] 5.1 Update existing tests to handle 17-column schema
- [x] 5.2 Create `test/sql/read_pgn_parse_errors.test` to test error column
- [x] 5.3 Test games with movetext parsing errors are returned with error messages
- [x] 5.4 Test games with header parsing errors are returned with error messages
- [x] 5.5 Test successful games have NULL parse_error
- [x] 5.6 Test filtering with `WHERE parse_error IS NOT NULL`
- [x] 5.7 Test filtering with `WHERE parse_error IS NULL`
- [x] 5.8 Create test PGN files with intentional header parsing errors
- [x] 5.9 Create test PGN files with intentional movetext parsing errors
- [x] 5.10 Test that error messages indicate the stage of parsing failure

## 6. Documentation
- [x] 6.1 Update code comments to reference new spec requirements
- [x] 6.2 Document parse_error column behavior
- [x] 6.3 Add examples of querying parse errors in tests

## 7. Validation
- [x] 7.1 Run `make test_debug` and ensure all tests pass
- [x] 7.2 Run `make test_release` and ensure all tests pass
- [x] 7.3 Manually test with various error scenarios
- [x] 7.4 Verify backward compatibility for queries not using parse_error
- [x] 7.5 Validate specs with `openspec validate add-parse-error-column --strict`
