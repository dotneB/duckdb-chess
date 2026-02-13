mod duckdb_string;
mod filter;
mod moves;
mod reader;
mod timecontrol;
mod types;
mod visitor;

use duckdb::{Connection, Result};
use duckdb_ext_macros::duckdb_extension;
use filter::ChessMovesNormalizeScalar;
use moves::{
    ChessFenEpdScalar, ChessMovesHashScalar, ChessMovesJsonScalar, ChessMovesSubsetScalar,
    ChessPlyCountScalar,
};
use reader::ReadPgnVTab;
use std::error::Error;
use timecontrol::{
    ChessTimecontrolCategoryScalar, ChessTimecontrolJsonScalar, ChessTimecontrolNormalizeScalar,
};

#[duckdb_extension(name = "chess", api_version = "v1.0.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    // Table functions
    con.register_table_function::<ReadPgnVTab>("read_pgn")?;

    // Scalar functions
    // Register internal implementations, then expose stable public names via SQL macros.
    // This avoids DuckDB's default NULL-in-NULL-out behavior for scalar functions.
    con.register_scalar_function::<ChessMovesJsonScalar>("chess_moves_json_impl")?;
    con.register_scalar_function::<ChessMovesNormalizeScalar>("chess_moves_normalize")?;
    con.register_scalar_function::<ChessMovesHashScalar>("chess_moves_hash")?;
    con.register_scalar_function::<ChessMovesSubsetScalar>("chess_moves_subset")?;
    con.register_scalar_function::<ChessFenEpdScalar>("chess_fen_epd")?;
    con.register_scalar_function::<ChessPlyCountScalar>("chess_ply_count_impl")?;
    con.register_scalar_function::<ChessTimecontrolNormalizeScalar>("chess_timecontrol_normalize")?;
    con.register_scalar_function::<ChessTimecontrolJsonScalar>("chess_timecontrol_json")?;
    con.register_scalar_function::<ChessTimecontrolCategoryScalar>("chess_timecontrol_category")?;

    con.execute_batch(
        "CREATE OR REPLACE MACRO chess_moves_json(movetext, max_ply := NULL) AS
           chess_moves_json_impl(coalesce(movetext, ''), coalesce(max_ply, 9223372036854775807));
         CREATE OR REPLACE MACRO chess_ply_count(movetext) AS
           chess_ply_count_impl(coalesce(movetext, ''));",
    )?;

    Ok(())
}
