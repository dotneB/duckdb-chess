//! Shared DuckDB scalar invoke helpers.
//!
//! These helpers centralize common scalar boilerplate:
//! - flat vector access
//! - per-row NULL checks
//! - `duckdb_string_t` decoding
//! - output insertion / validity handling
//!
//! # Safety
//! These helpers MUST only be called from within a DuckDB scalar `invoke()` while the
//! underlying vectors are valid.
//!
//! Callers MUST ensure the input/output column logical types match the helper being used
//! (e.g., `VARCHAR` inputs for `duckdb_string_t`, `BIGINT` outputs for `i64`, etc.).

use std::error::Error;
use std::ffi::CString;

use duckdb::{
    Result,
    core::{DataChunkHandle, FlatVector, Inserter, LogicalTypeId},
    vtab::arrow::WritableVector,
};
use libduckdb_sys::duckdb_string_t;

use super::string::decode_duckdb_string;

#[derive(Debug, Clone, Copy)]
pub enum VarcharNullBehavior {
    /// Output NULL when the input row is NULL.
    Null,
    /// Output a static string when the input row is NULL.
    Static(&'static str),
}

#[derive(Debug, Clone)]
pub enum VarcharOutput {
    Null,
    Value(String),
}

fn ensure_type(
    vec: &FlatVector,
    expected: LogicalTypeId,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let actual = vec.logical_type().id();
    if actual != expected {
        return Err(format!(
            "scalar helper type mismatch: {label} expected {expected:?}, got {actual:?}"
        )
        .into());
    }
    Ok(())
}

/// Invoke a unary `VARCHAR -> VARCHAR` scalar.
pub fn invoke_unary_varchar_to_varchar<F>(
    input: &DataChunkHandle,
    output: &mut dyn WritableVector,
    null_behavior: VarcharNullBehavior,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&str) -> Result<VarcharOutput, Box<dyn Error>>,
{
    let len = input.len();
    let input_vec = input.flat_vector(0);
    ensure_type(&input_vec, LogicalTypeId::Varchar, "input[0]")?;
    let input_slice = input_vec.as_slice::<duckdb_string_t>();
    let mut output_vec = output.flat_vector();
    ensure_type(&output_vec, LogicalTypeId::Varchar, "output")?;

    for (i, s) in input_slice.iter().take(len).enumerate() {
        if input_vec.row_is_null(i as u64) {
            match null_behavior {
                VarcharNullBehavior::Null => output_vec.set_null(i),
                VarcharNullBehavior::Static(v) => output_vec.insert(i, CString::new(v)?),
            }
            continue;
        }

        // SAFETY: Row nullability is checked above.
        let val = unsafe { decode_duckdb_string(s) };
        match f(val.as_ref())? {
            VarcharOutput::Null => output_vec.set_null(i),
            VarcharOutput::Value(v) => output_vec.insert(i, CString::new(v)?),
        }
    }

    Ok(())
}

/// Invoke a unary `VARCHAR -> BIGINT` scalar with a default output value for NULL inputs.
pub fn invoke_unary_varchar_to_i64_default<F>(
    input: &DataChunkHandle,
    output: &mut dyn WritableVector,
    default_on_null: i64,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&str) -> i64,
{
    let len = input.len();
    let input_vec = input.flat_vector(0);
    ensure_type(&input_vec, LogicalTypeId::Varchar, "input[0]")?;
    let input_slice = input_vec.as_slice::<duckdb_string_t>();
    let mut output_vec = output.flat_vector();
    ensure_type(&output_vec, LogicalTypeId::Bigint, "output")?;
    let output_slice = output_vec.as_mut_slice::<i64>();

    for (i, s) in input_slice.iter().take(len).enumerate() {
        if input_vec.row_is_null(i as u64) {
            output_slice[i] = default_on_null;
            continue;
        }

        // SAFETY: Row nullability is checked above.
        let val = unsafe { decode_duckdb_string(s) };
        output_slice[i] = f(val.as_ref());
    }

    Ok(())
}

/// Invoke a unary `VARCHAR -> UBIGINT` scalar.
///
/// This helper outputs NULL when the input row is NULL or when `f` returns `None`.
///
pub fn invoke_unary_varchar_to_u64_nullable<F>(
    input: &DataChunkHandle,
    output: &mut dyn WritableVector,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&str) -> Option<u64>,
{
    let len = input.len();
    let input_vec = input.flat_vector(0);
    ensure_type(&input_vec, LogicalTypeId::Varchar, "input[0]")?;
    let input_slice = input_vec.as_slice::<duckdb_string_t>();
    let mut output_vec = output.flat_vector();
    ensure_type(&output_vec, LogicalTypeId::UBigint, "output")?;

    for (i, s) in input_slice.iter().take(len).enumerate() {
        if input_vec.row_is_null(i as u64) {
            output_vec.set_null(i);
            continue;
        }

        // SAFETY: Row nullability is checked above.
        let val = unsafe { decode_duckdb_string(s) };
        match f(val.as_ref()) {
            Some(v) => output_vec.as_mut_slice::<u64>()[i] = v,
            None => output_vec.set_null(i),
        }
    }

    Ok(())
}

/// Invoke a binary `VARCHAR, VARCHAR -> BOOLEAN` scalar that outputs NULL when either input is
/// NULL.
pub fn invoke_binary_varchar_varchar_to_bool_nullable<F>(
    input: &DataChunkHandle,
    output: &mut dyn WritableVector,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&str, &str) -> bool,
{
    let len = input.len();
    let input_vec_0 = input.flat_vector(0);
    let input_vec_1 = input.flat_vector(1);
    ensure_type(&input_vec_0, LogicalTypeId::Varchar, "input[0]")?;
    ensure_type(&input_vec_1, LogicalTypeId::Varchar, "input[1]")?;
    let input_slice_0 = input_vec_0.as_slice::<duckdb_string_t>();
    let input_slice_1 = input_vec_1.as_slice::<duckdb_string_t>();
    let mut output_vec = output.flat_vector();
    ensure_type(&output_vec, LogicalTypeId::Boolean, "output")?;

    for (i, (left_s, right_s)) in input_slice_0
        .iter()
        .take(len)
        .zip(input_slice_1.iter().take(len))
        .enumerate()
    {
        if input_vec_0.row_is_null(i as u64) || input_vec_1.row_is_null(i as u64) {
            output_vec.set_null(i);
            continue;
        }

        // SAFETY: Both input rows are checked non-NULL above.
        let left = unsafe { decode_duckdb_string(left_s) };
        // SAFETY: Both input rows are checked non-NULL above.
        let right = unsafe { decode_duckdb_string(right_s) };
        output_vec.as_mut_slice::<bool>()[i] = f(left.as_ref(), right.as_ref());
    }

    Ok(())
}

/// Invoke a `VARCHAR -> VARCHAR` scalar that optionally reads a per-row `BIGINT` argument from
/// column 1 if present.
pub fn invoke_unary_varchar_optional_i64_to_varchar<F>(
    input: &DataChunkHandle,
    output: &mut dyn WritableVector,
    null_behavior: VarcharNullBehavior,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&str, Option<i64>) -> Result<VarcharOutput, Box<dyn Error>>,
{
    let len = input.len();
    let input_vec = input.flat_vector(0);
    ensure_type(&input_vec, LogicalTypeId::Varchar, "input[0]")?;
    let input_slice = input_vec.as_slice::<duckdb_string_t>();
    let max_arg_vec = if input.num_columns() > 1 {
        Some(input.flat_vector(1))
    } else {
        None
    };
    if let Some(vec) = &max_arg_vec {
        ensure_type(vec, LogicalTypeId::Bigint, "input[1]")?;
    }
    let max_arg_slice = max_arg_vec.as_ref().map(|v| v.as_slice::<i64>());

    let mut output_vec = output.flat_vector();
    ensure_type(&output_vec, LogicalTypeId::Varchar, "output")?;

    for (i, s) in input_slice.iter().take(len).enumerate() {
        if input_vec.row_is_null(i as u64) {
            match null_behavior {
                VarcharNullBehavior::Null => output_vec.set_null(i),
                VarcharNullBehavior::Static(v) => output_vec.insert(i, CString::new(v)?),
            }
            continue;
        }

        // SAFETY: Row nullability is checked above.
        let val = unsafe { decode_duckdb_string(s) };
        let arg = match (&max_arg_vec, &max_arg_slice) {
            (Some(vec), Some(slice)) => {
                if vec.row_is_null(i as u64) {
                    None
                } else {
                    Some(slice[i])
                }
            }
            _ => None,
        };

        match f(val.as_ref(), arg)? {
            VarcharOutput::Null => output_vec.set_null(i),
            VarcharOutput::Value(v) => output_vec.insert(i, CString::new(v)?),
        }
    }

    Ok(())
}
