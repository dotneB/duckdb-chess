extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;

mod filter;
mod reader;
mod types;
mod visitor;

use duckdb::{Connection, Result};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use filter::FilterMovetextScalar;
use libduckdb_sys as ffi;
use reader::ReadPgnVTab;
use std::error::Error;

const EXTENSION_NAME: &str = "read_pgn";

#[duckdb_entrypoint_c_api(ext_name = "duckdb_chess", min_duckdb_version = "v1.0.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<ReadPgnVTab>(EXTENSION_NAME)?;
    con.register_scalar_function::<FilterMovetextScalar>("filter_movetext_annotations")?;
    Ok(())
}
