-- Test the chess extension
.echo on

-- Try to load the extension
INSTALL 'target/release/duckdb_chess.dll' FROM local_file;
LOAD duckdb_chess;

-- Test read_pgn function
SELECT * FROM read_pgn('test/pgn_files/sample.pgn');

-- Count games
SELECT COUNT(*) as num_games FROM read_pgn('test/pgn_files/sample.pgn');

-- Show columns
SELECT
    white_id,
    black_id,
    result,
    opening_name,
    LENGTH(movetext) as movetext_len
FROM read_pgn('test/pgn_files/sample.pgn');
