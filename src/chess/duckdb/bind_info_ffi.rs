use duckdb::vtab::BindInfo;
use libduckdb_sys::{
    duckdb_bind_get_named_parameter, duckdb_bind_info, duckdb_destroy_value, duckdb_free,
    duckdb_get_varchar, duckdb_is_null_value,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum NamedParameterVarchar {
    Missing,
    Null,
    Value(String),
}

pub(crate) fn get_named_parameter_varchar(
    bind: &BindInfo,
    name: &str,
) -> Result<NamedParameterVarchar, Box<dyn std::error::Error>> {
    let name_cstr = CString::new(name)?;

    // SAFETY: The returned pointer is owned by DuckDB and valid only for this bind callback.
    // `bind_info_ptr` provides the raw C bind handle associated with `bind`.
    let mut value =
        unsafe { duckdb_bind_get_named_parameter(bind_info_ptr(bind), name_cstr.as_ptr()) };
    if value.is_null() {
        return Ok(NamedParameterVarchar::Missing);
    }

    // SAFETY: `value` is a valid `duckdb_value` handle returned by DuckDB and is destroyed
    // exactly once below via `duckdb_destroy_value`.
    let result = unsafe {
        if duckdb_is_null_value(value) {
            Ok(NamedParameterVarchar::Null)
        } else {
            let varchar = duckdb_get_varchar(value);
            if varchar.is_null() {
                Err(format!("Failed to read named parameter '{}' as VARCHAR", name).into())
            } else {
                let text = CStr::from_ptr(varchar).to_string_lossy().into_owned();
                duckdb_free(varchar as *mut c_void);
                Ok(NamedParameterVarchar::Value(text))
            }
        }
    };

    // SAFETY: `value` has not been destroyed yet and must be released once.
    unsafe {
        duckdb_destroy_value(&mut value);
    }

    result
}

fn bind_info_ptr(bind: &BindInfo) -> duckdb_bind_info {
    // SAFETY: duckdb-rs v1.4.4 stores `duckdb_bind_info` as the only field inside
    // `duckdb::vtab::BindInfo` (see duckdb/src/vtab/function.rs). The wrapper does not expose
    // a public raw accessor or null-aware typed named-parameter accessor in this version, so this
    // cast is required for `duckdb_bind_get_named_parameter` + `duckdb_is_null_value` interop.
    //
    // On duckdb-rs upgrades, re-validate this boundary by checking:
    // - `BindInfo` layout/accessors in duckdb-rs `src/vtab/function.rs`
    // - whether a stable accessor can replace this cast
    // - named-parameter behavior parity (`compression` omitted/NULL/zstd/invalid)
    // - full validation via `just full`
    unsafe { *(bind as *const BindInfo as *const duckdb_bind_info) }
}
