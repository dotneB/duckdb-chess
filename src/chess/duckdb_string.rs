use libduckdb_sys::duckdb_string_t;

/// Decode a DuckDB string value into a Rust-owned `String`.
///
/// # Safety
///
/// `s` must come from a non-NULL `duckdb_string_t` vector row provided by
/// DuckDB for the active scalar invocation. Callers must perform row null checks
/// before invoking this function.
pub unsafe fn decode_duckdb_string(s: duckdb_string_t) -> String {
    // SAFETY: Reading the inlined union field is part of DuckDB's string layout.
    let inlined_len = unsafe { s.value.inlined.length };

    if inlined_len <= 12 {
        let len = inlined_len as usize;
        if len == 0 {
            return String::new();
        }

        // SAFETY: `len <= 12` for inlined strings.
        let inlined = unsafe { &s.value.inlined.inlined };
        // SAFETY: `inlined` contains `len` initialized bytes.
        let bytes = unsafe { std::slice::from_raw_parts(inlined.as_ptr() as *const u8, len) };
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        // SAFETY: Reading pointer representation fields is valid here.
        let len = unsafe { s.value.pointer.length } as usize;
        if len == 0 {
            return String::new();
        }

        // SAFETY: DuckDB provides valid pointer storage for non-inlined strings.
        let ptr = unsafe { s.value.pointer.ptr };
        // SAFETY: `ptr` references `len` bytes for this value.
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
        String::from_utf8_lossy(bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libduckdb_sys::{
        duckdb_string_t__bindgen_ty_1, duckdb_string_t__bindgen_ty_1__bindgen_ty_1,
        duckdb_string_t__bindgen_ty_1__bindgen_ty_2,
    };
    use std::os::raw::c_char;

    fn make_inlined_duckdb_string(bytes: &[u8]) -> duckdb_string_t {
        let mut inlined = [0 as c_char; 12];
        for (dst, src) in inlined.iter_mut().zip(bytes.iter().copied()) {
            *dst = src as c_char;
        }

        duckdb_string_t {
            value: duckdb_string_t__bindgen_ty_1 {
                inlined: duckdb_string_t__bindgen_ty_1__bindgen_ty_2 {
                    length: bytes.len() as u32,
                    inlined,
                },
            },
        }
    }

    fn make_pointer_duckdb_string(bytes: &mut [u8]) -> duckdb_string_t {
        let mut prefix = [0 as c_char; 4];
        for (dst, src) in prefix.iter_mut().zip(bytes.iter().copied()) {
            *dst = src as c_char;
        }

        duckdb_string_t {
            value: duckdb_string_t__bindgen_ty_1 {
                pointer: duckdb_string_t__bindgen_ty_1__bindgen_ty_1 {
                    length: bytes.len() as u32,
                    prefix,
                    ptr: bytes.as_mut_ptr() as *mut c_char,
                },
            },
        }
    }

    #[test]
    fn test_decode_duckdb_string_inlined() {
        let input = make_inlined_duckdb_string(b"e4");
        // SAFETY: test fixture builds a valid inlined duckdb_string_t.
        let decoded = unsafe { decode_duckdb_string(input) };
        assert_eq!(decoded, "e4");
    }

    #[test]
    fn test_decode_duckdb_string_pointer() {
        let mut backing = b"abcdefghijklm".to_vec(); // 13 bytes -> pointer path
        let input = make_pointer_duckdb_string(backing.as_mut_slice());
        // SAFETY: test fixture keeps backing storage alive for decode duration.
        let decoded = unsafe { decode_duckdb_string(input) };
        assert_eq!(decoded, "abcdefghijklm");
    }

    #[test]
    fn test_decode_duckdb_string_pointer_invalid_utf8_lossy() {
        let mut backing = b"abcdefghijklm".to_vec();
        backing[1] = 0x80;
        let expected = String::from_utf8_lossy(&backing).into_owned();
        let input = make_pointer_duckdb_string(backing.as_mut_slice());
        // SAFETY: test fixture keeps backing storage alive for decode duration.
        let decoded = unsafe { decode_duckdb_string(input) };
        assert_eq!(decoded, expected);
    }
}
