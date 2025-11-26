use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
};
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

/// Spec: annotation-filtering - Movetext Annotation Removal
/// Removes curly brace annotations from chess movetext while preserving move structure
/// Spec: annotation-filtering - Nested Annotation Handling (tracks brace depth)
/// Spec: annotation-filtering - Whitespace Normalization (collapses spaces and trims)
pub fn filter_movetext_annotations(movetext: &str) -> String {
    let mut result = String::new();
    let mut in_annotation = false;
    let mut brace_depth = 0;
    let mut prev_was_space = false;

    for ch in movetext.chars() {
        match ch {
            '{' => {
                in_annotation = true;
                brace_depth += 1;
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    in_annotation = false;
                    // Mark that we should skip next space if any
                    prev_was_space = true;
                }
            }
            ' ' if !in_annotation => {
                if !prev_was_space && !result.is_empty() {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            _ if !in_annotation => {
                prev_was_space = false;
                result.push(ch);
            }
            _ => {}
        }
    }

    result.trim().to_string()
}

#[repr(C)]
pub struct FilterMovetextBindData {
    movetext: String,
}

#[repr(C)]
pub struct FilterMovetextInitData {
    done: AtomicBool,
}

pub struct FilterMovetextVTab;

impl VTab for FilterMovetextVTab {
    type InitData = FilterMovetextInitData;
    type BindData = FilterMovetextBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        let movetext = bind.get_parameter(0).to_string();

        bind.add_result_column(
            "filtered_movetext",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );

        Ok(FilterMovetextBindData { movetext })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(FilterMovetextInitData {
            done: AtomicBool::new(false),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();

        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
            return Ok(());
        }

        let filtered = filter_movetext_annotations(&bind_data.movetext);
        let result_vec = output.flat_vector(0);
        result_vec.insert(0, CString::new(filtered)?);
        output.set_len(1);

        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)])
    }
}
