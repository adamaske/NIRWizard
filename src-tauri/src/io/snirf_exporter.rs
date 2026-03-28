use crate::domain::*;
use hdf5::File;
use ndarray16::Array2;

// =============================================================================
// Low-level write helpers — symmetric counterparts to the parser's read helpers
// =============================================================================

/// Write a VarLenUnicode scalar dataset (mirrors `read_string`).
/// The parser tries VarLenUnicode first, so this encoding round-trips perfectly.
fn write_string(group: &hdf5::Group, name: &str, value: &str) -> Result<(), String> {
    use hdf5::types::VarLenUnicode;
    use std::str::FromStr;
    // FromStr for VarLenUnicode is infallible; map_err is a compile-time no-op
    let s = VarLenUnicode::from_str(value)
        .map_err(|_| format!("write_string '{}': invalid value '{}'", name, value))?;
    let ds = group
        .new_dataset::<VarLenUnicode>()
        .create(name)
        .map_err(|e| format!("write_string: failed to create '{}': {}", name, e))?;
    ds.write_scalar(&s)
        .map_err(|e| format!("write_string: failed to write '{}': {}", name, e))
}

/// Write an i32 scalar dataset (mirrors `read_i32`).
fn write_i32(group: &hdf5::Group, name: &str, value: i32) -> Result<(), String> {
    let ds = group
        .new_dataset::<i32>()
        .create(name)
        .map_err(|e| format!("write_i32: failed to create '{}': {}", name, e))?;
    ds.write_scalar(&value)
        .map_err(|e| format!("write_i32: failed to write '{}': {}", name, e))
}

/// Write a contiguous 1-D f64 dataset from a slice.
fn write_f64_1d(group: &hdf5::Group, name: &str, data: &[f64]) -> Result<(), String> {
    let ds = group
        .new_dataset::<f64>()
        .shape([data.len()])
        .create(name)
        .map_err(|e| format!("write_f64_1d: failed to create '{}': {}", name, e))?;
    ds.write_raw(data)
        .map_err(|e| format!("write_f64_1d: failed to write '{}': {}", name, e))
}

/// Write a 2-D f64 dataset from a row-major Array2 (mirrors `read_2d`).
fn write_f64_2d(group: &hdf5::Group, name: &str, array: &Array2<f64>) -> Result<(), String> {
    let (rows, cols) = array.dim();
    let ds = group
        .new_dataset::<f64>()
        .shape([rows, cols])
        .create(name)
        .map_err(|e| format!("write_f64_2d: failed to create '{}': {}", name, e))?;
    ds.write(array)
        .map_err(|e| format!("write_f64_2d: failed to write '{}': {}", name, e))
}

// =============================================================================
// Public exporter entry
// =============================================================================

pub fn export_snirf(snirf: &SNIRF, path: &str) -> Result<(), String> {
    // TODO : Rewrite exporter based on sNIRF specification
    //
    // For each nirs entry
    //  write metadata, probe, events, auxiliaries,
    //  for each data block
    //      write measurementList, time, dataTimeSeries, etc
    //
    Ok(())
}
