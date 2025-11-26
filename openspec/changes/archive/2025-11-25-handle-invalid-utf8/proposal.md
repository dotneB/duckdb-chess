# Handle Invalid UTF-8 in PGN Files

## Summary
Update the PGN file reader to gracefully handle invalid UTF-8 sequences by using lossy decoding instead of strict validation. This prevents the parser from skipping entire lines or failing when encountering corrupted text or non-UTF-8 encodings (e.g., Windows-1252 artifacts) in PGN files.

## Problem
Currently, the `read_pgn` function uses strict UTF-8 validation when reading lines from PGN files. If a file contains invalid UTF-8 bytes (common in older PGNs or those created with legacy tools), the reader fails to parse that line, logging a warning and skipping potentially valuable data. Users with large datasets often encounter these errors, leading to incomplete data ingestion.

## Solution
Replace the strict `BufRead::lines()` iterator with a manual `read_until` loop that employs `String::from_utf8_lossy`. This will replace invalid byte sequences with the Unicode replacement character () rather than returning an error. This ensures that:
1.  The file reading process does not abort or skip lines due to encoding errors.
2.  The PGN parser receives a valid UTF-8 string (albeit with replacement characters) and can attempt to parse the game headers and movetext.

## Impact
- **Reliability**: Significantly improves robustness when dealing with "in the wild" PGN files.
- **Data Integrity**: Prevents silent data loss where entire games might be skipped due to a single bad byte in a header (e.g., a player's name).
- **Performance**: Negligible impact; lossy conversion is efficient.
