mod filter;
mod moves;
mod reader;
mod types;
mod visitor;

use duckdb::{Connection, Result};
use duckdb_ext_macros::duckdb_extension;
use filter::ChessMovesNormalizeScalar;
use moves::{ChessMovesHashScalar, ChessMovesJsonScalar, ChessMovesSubsetScalar};
use reader::ReadPgnVTab;
use std::error::Error;

#[duckdb_extension(name = "duckdb_chess", api_version = "v1.0.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    // Table functions
    con.register_table_function::<ReadPgnVTab>("read_pgn")?;

    // Scalar functions
    con.register_scalar_function::<ChessMovesJsonScalar>("chess_moves_json")?;
    con.register_scalar_function::<ChessMovesNormalizeScalar>("chess_moves_normalize")?;
    con.register_scalar_function::<ChessMovesHashScalar>("chess_moves_hash")?;
    con.register_scalar_function::<ChessMovesSubsetScalar>("chess_moves_subset")?;

    Ok(())
}
