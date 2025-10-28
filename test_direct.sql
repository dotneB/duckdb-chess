-- Load extension directly
LOAD 'target/release/duckdb_chess.dll';

-- Test read_pgn
SELECT * FROM read_pgn('test/sample.pgn');
