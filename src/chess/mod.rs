mod filter;
mod moves;
mod reader;
mod types;
mod visitor;

use duckdb::{Connection, Result};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use filter::ChessMovesNormalizeScalar;
use libduckdb_sys as ffi;
use moves::{ChessMovesJsonScalar, ChessMovesHashScalar, ChessMovesSubsetScalar};
use reader::ReadPgnVTab;
use std::error::Error;

#[duckdb_entrypoint_c_api(ext_name = "duckdb_chess", min_duckdb_version = "v1.0.0")]
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
