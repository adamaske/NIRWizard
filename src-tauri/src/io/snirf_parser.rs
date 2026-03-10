use anyhow::{bail, Context, Result};
use hdf5::{File, Group};
use nalgebra::{Vector2, Vector3};
use ndarray::Array2;

use crate::domain::{SnirfError, *};

// =============================================================================
// HDF5 tree inspector (debug builds only)
// =============================================================================

#[cfg(debug_assertions)]
fn dtype_label(ds: &hdf5::Dataset) -> String {
    use hdf5::types::TypeDescriptor;
    ds.dtype()
        .ok()
        .and_then(|dt| dt.to_descriptor().ok())
        .map(|desc| match desc {
            TypeDescriptor::Float(_) => "f64".into(),
            TypeDescriptor::Integer(_) => "i32".into(),
            TypeDescriptor::Unsigned(_) => "u32".into(),
            TypeDescriptor::Boolean => "bool".into(),
            TypeDescriptor::VarLenUnicode => "str".into(),
            TypeDescriptor::VarLenAscii => "ascii".into(),
            TypeDescriptor::FixedAscii(n) => format!("ascii[{}]", n),
            TypeDescriptor::FixedUnicode(n) => format!("str[{}]", n),
            _ => format!("{:?}", desc),
        })
        .unwrap_or_else(|| "?".into())
}

#[cfg(debug_assertions)]
fn scalar_preview(ds: &hdf5::Dataset) -> String {
    let n: usize = ds.shape().iter().product();
    let is_scalar = ds.ndim() == 0 || (ds.ndim() == 1 && n == 1);
    if !is_scalar {
        return String::new();
    }
    if let Ok(s) = ds.read_scalar::<hdf5::types::VarLenUnicode>() {
        return format!(" = {:?}", s.to_string());
    }
    if let Ok(v) = ds.read_scalar::<f64>() {
        return format!(" = {}", v);
    }
    if let Ok(v) = ds.read_scalar::<i32>() {
        return format!(" = {}", v);
    }
    String::new()
}

#[cfg(debug_assertions)]
fn walk_hdf5(group: &hdf5::Group, depth: usize) {
    let indent = "  ".repeat(depth);
    let names = match group.member_names() {
        Ok(n) => n,
        Err(e) => {
            println!("{}  [err listing members: {}]", indent, e);
            return;
        }
    };
    for name in &names {
        if let Ok(ds) = group.dataset(name) {
            let shape = ds.shape();
            let ty = dtype_label(&ds);
            let preview = scalar_preview(&ds);
            println!("{}  [D] {}  {:?} <{}>{}", indent, name, shape, ty, preview);
        } else if let Ok(sub) = group.group(name) {
            println!("{}  [G] {}/", indent, name);
            walk_hdf5(&sub, depth + 1);
        } else {
            println!("{}  [?] {}", indent, name);
        }
    }
}

#[cfg(debug_assertions)]
fn print_hdf5_tree(path: &str, file: &File) {
    use std::ops::Deref;
    println!("\n╔═══ HDF5 tree: {} ═══", path);
    walk_hdf5(file.deref(), 0);
    println!("╚═══════════════════\n");
}

// =============================================================================
// String / integer helpers
// =============================================================================

/// Read a string dataset regardless of encoding or shape.
fn read_string(ds: &hdf5::Dataset) -> Result<String> {
    if let Ok(s) = ds.read_scalar::<hdf5::types::VarLenUnicode>() {
        return Ok(s.to_string());
    }
    if let Ok(s) = ds.read_scalar::<hdf5::types::VarLenAscii>() {
        return Ok(s.to_string());
    }
    if let Some(s) = ds
        .read_raw::<hdf5::types::VarLenAscii>()
        .ok()
        .and_then(|v| v.into_iter().next())
    {
        return Ok(s.to_string());
    }
    if let Some(s) = ds
        .read_raw::<hdf5::types::VarLenUnicode>()
        .ok()
        .and_then(|v| v.into_iter().next())
    {
        return Ok(s.to_string());
    }
    bail!("no readable string encoding found")
}

/// Read an i32 dataset regardless of shape.
fn read_i32(ds: &hdf5::Dataset) -> Result<i32> {
    ds.read_scalar::<i32>()
        .ok()
        .or_else(|| ds.read_raw::<i32>().ok().and_then(|v| v.into_iter().next()))
        .context("failed to read i32 value")
}

// =============================================================================
// Public entry point
// =============================================================================

/// Parse a SNIRF file from `path`.
///
/// Returns a typed [`SnirfError`] so callers can distinguish a missing `/nirs`
/// group from a lower-level parse failure.  Use `{:#}` formatting on the error
/// to print the full context chain.
pub fn parse_snirf(path: &str) -> Result<SNIRF, SnirfError> {
    parse_snirf_inner(path).map_err(SnirfError::from)
}

fn parse_snirf_inner(path: &str) -> Result<SNIRF> {
    let file = File::open(path).with_context(|| format!("Failed to open '{path}'"))?;

    //#[cfg(debug_assertions)]
    //print_hdf5_tree(path, &file);

    let format_version = file
        .dataset("formatVersion")
        .context("Missing required dataset 'formatVersion'")?;
    let format_version = read_string(&format_version).context("formatVersion: read failed")?;

    let file_descriptor = FileDescriptor {
        filepath: path.to_string(),
        filename: std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
    };

    // SNIRF allows a single `/nirs` group or indexed `/nirs1`, `/nirs2`, …
    let mut nirs_entries = Vec::new();
    if let Ok(g) = file.group("nirs") {
        nirs_entries.push(parse_nirs_entry(&g).context("/nirs")?);
    } else {
        let mut i = 1usize;
        loop {
            match file.group(&format!("nirs{i}")) {
                Ok(g) => {
                    nirs_entries.push(
                        parse_nirs_entry(&g).with_context(|| format!("/nirs{i}"))?,
                    );
                    i += 1;
                }
                Err(_) => break,
            }
        }
    }

    if nirs_entries.is_empty() {
        // Propagate as the dedicated NoNirsGroup variant via the From impl.
        return Err(SnirfError::NoNirsGroup.into());
    }

    let snirf = SNIRF {
        format_version,
        file_descriptor,
        nirs_entries,
    };

    println!("Parsed SNIRF: {snirf}");

    Ok(snirf)
}

// =============================================================================
// NIRS entry
// =============================================================================

fn parse_nirs_entry(nirs: &Group) -> Result<NirsEntry> {
    let metadata = parse_metadata(nirs).context("metaDataTags")?;
    let data_blocks = parse_data_blocks(nirs).context("data blocks")?;
    let probe = parse_probe(nirs).context("probe")?;
    let events = parse_events(nirs).context("stim events")?;
    let auxiliaries = parse_auxiliaries(nirs).context("auxiliaries")?;

    Ok(NirsEntry {
        metadata,
        data_blocks,
        probe,
        events,
        auxiliaries,
    })
}

// =============================================================================
// Metadata  —  nirs/metaDataTags/*
// =============================================================================

fn parse_metadata(nirs: &Group) -> Result<Vec<MetadataTag>> {
    let group = nirs
        .group("metaDataTags")
        .context("Failed to open metaDataTags")?;

    let tags = group
        .member_names()
        .context("Failed to list metaDataTags members")?
        .into_iter()
        .filter_map(|name| {
            let ds = group.dataset(&name).ok()?;
            let value = read_string(&ds).unwrap_or_else(|_| "(non-string)".to_string());
            Some(MetadataTag { name, value })
        })
        .collect();

    Ok(tags)
}

// =============================================================================
// Probe  —  nirs/probe/*
// =============================================================================

/// Given optional 2D and 3D position arrays, return both — extrapolating the
/// missing one.  At least one must be present, otherwise an error is returned.
///
///   3D-only  →  2D = columns (x, y)
///   2D-only  →  3D = (x, y, z=0)
///   both     →  returned as-is
fn resolve_positions(
    pos2d: Option<Array2<f64>>,
    pos3d: Option<Array2<f64>>,
    label: &str,
) -> Result<(Array2<f64>, Array2<f64>)> {
    match (pos2d, pos3d) {
        (Some(d2), Some(d3)) => Ok((d2, d3)),
        (None, Some(d3)) => {
            let d2 = d3.slice(ndarray::s![.., 0..2]).to_owned();
            Ok((d2, d3))
        }
        (Some(d2), None) => {
            let n = d2.nrows();
            let mut d3 = Array2::<f64>::zeros((n, 3));
            d3.slice_mut(ndarray::s![.., 0..2]).assign(&d2);
            Ok((d2, d3))
        }
        (None, None) => bail!(
            "neither {label}Pos2D nor {label}Pos3D found"
        ),
    }
}

fn parse_probe(nirs: &Group) -> Result<Probe> {
    let probe = nirs.group("probe").context("probe group missing")?;

    // Wavelengths (required)
    let wavelengths: Vec<f64> = probe
        .dataset("wavelengths")
        .context("probe/wavelengths missing")?
        .read_raw()
        .context("probe/wavelengths: read failed")?;

    let wavelength_emission: Option<Vec<f64>> = probe
        .dataset("wavelengthEmission")
        .and_then(|ds| ds.read_raw())
        .ok();

    // SNIRF spec: either *Pos2D or *Pos3D is sufficient.
    // resolve_positions extrapolates whichever is absent.
    let (s2d, s3d) = resolve_positions(
        probe.dataset("sourcePos2D").and_then(|ds| ds.read_2d()).ok(),
        probe.dataset("sourcePos3D").and_then(|ds| ds.read_2d()).ok(),
        "source",
    )
    .context("probe source positions")?;

    let (d2d, d3d) = resolve_positions(
        probe
            .dataset("detectorPos2D")
            .and_then(|ds| ds.read_2d())
            .ok(),
        probe
            .dataset("detectorPos3D")
            .and_then(|ds| ds.read_2d())
            .ok(),
        "detector",
    )
    .context("probe detector positions")?;

    let row2 = |arr: &Array2<f64>, i: usize| Vector2::new(arr[[i, 0]], arr[[i, 1]]);
    let row3 = |arr: &Array2<f64>, i: usize| Vector3::new(arr[[i, 0]], arr[[i, 1]], arr[[i, 2]]);

    let sources: Vec<Optode> = (0..s3d.nrows())
        .map(|i| Optode {
            id: i,
            name: format!("S{}", i + 1),
            pos_2d: row2(&s2d, i),
            pos_3d: row3(&s3d, i),
        })
        .collect();

    let detectors: Vec<Optode> = (0..d3d.nrows())
        .map(|i| Optode {
            id: i,
            name: format!("D{}", i + 1),
            pos_2d: row2(&d2d, i),
            pos_3d: row3(&d3d, i),
        })
        .collect();

    let coordinate_system = probe
        .dataset("coordinateSystem")
        .ok()
        .and_then(|ds| read_string(&ds).ok());

    let coordinate_system_description = probe
        .dataset("coordinateSystemDescription")
        .ok()
        .and_then(|ds| read_string(&ds).ok());

    let use_local_index = probe
        .dataset("useLocalIndex")
        .ok()
        .and_then(|ds| read_i32(&ds).ok());

    let landmarks = parse_landmarks(&probe).unwrap_or(None);

    Ok(Probe {
        wavelengths,
        wavelength_emission,
        sources,
        detectors,
        landmarks,
        coordinate_system,
        coordinate_system_description,
        use_local_index,
    })
}

fn parse_landmarks(probe: &Group) -> Result<Option<Vec<Landmark>>> {
    let labels_ds = match probe.dataset("landmarkLabels") {
        Ok(ds) => ds,
        Err(_) => return Ok(None),
    };

    let labels: Vec<String> = labels_ds
        .read_raw::<hdf5::types::VarLenUnicode>()
        .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        .or_else(|_| {
            labels_ds
                .read_raw::<hdf5::types::VarLenAscii>()
                .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        })
        .context("probe/landmarkLabels: read failed")?;

    let pos2d: Option<Array2<f64>> = probe
        .dataset("landmarkPos2D")
        .and_then(|ds| ds.read_2d())
        .ok();

    let pos3d: Option<Array2<f64>> = probe
        .dataset("landmarkPos3D")
        .and_then(|ds| ds.read_2d())
        .ok();

    let landmarks = labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| Landmark {
            label,
            pos_2d: pos2d
                .as_ref()
                .and_then(|arr| (i < arr.nrows()).then(|| [arr[[i, 0]], arr[[i, 1]]])),
            pos_3d: pos3d.as_ref().and_then(|arr| {
                (i < arr.nrows()).then(|| [arr[[i, 0]], arr[[i, 1]], arr[[i, 2]]])
            }),
        })
        .collect();

    Ok(Some(landmarks))
}

// =============================================================================
// Data blocks  —  nirs/data{j}/*
// =============================================================================

fn parse_data_blocks(nirs: &Group) -> Result<Vec<DataBlock>> {
    (1..)
        .map_while(|j| nirs.group(&format!("data{j}")).ok().map(|g| (j, g)))
        .map(|(j, g)| parse_data_block(&g, j).with_context(|| format!("data{j}")))
        .collect()
}

fn parse_data_block(data: &Group, block_idx: usize) -> Result<DataBlock> {
    let time: Vec<f64> = data
        .dataset("time")
        .context("time dataset missing")?
        .read_raw()
        .context("time: read failed")?;

    let ts: Array2<f64> = data
        .dataset("dataTimeSeries")
        .context("dataTimeSeries dataset missing")?
        .read_2d()
        .context("dataTimeSeries: read failed")?;

    let n_cols = ts.ncols();
    let measurements = (0..n_cols)
        .map(|col| {
            parse_measurement(data, block_idx, col, &ts)
                .with_context(|| format!("measurementList{}", col + 1))
        })
        .collect::<Result<Vec<Measurement>>>()?;

    Ok(DataBlock { time, measurements })
}

fn parse_measurement(
    data: &Group,
    block_idx: usize,
    col: usize,
    ts: &Array2<f64>,
) -> Result<Measurement> {
    let ml_name = format!("measurementList{}", col + 1);
    let ml = data
        .group(&ml_name)
        .with_context(|| format!("data{block_idx}/{ml_name}: group missing"))?;

    let read_required_i32 = |field: &str| -> Result<i32> {
        let ds = ml
            .dataset(field)
            .with_context(|| format!("{field} dataset missing"))?;
        read_i32(&ds).with_context(|| format!("{field}: read failed"))
    };

    let source_index = read_required_i32("sourceIndex")? as usize;
    let detector_index = read_required_i32("detectorIndex")? as usize;
    let wavelength_index = read_required_i32("wavelengthIndex")? as usize;
    let data_type = read_required_i32("dataType")?;

    let data_type_label = ml
        .dataset("dataTypeLabel")
        .ok()
        .and_then(|ds| read_string(&ds).ok())
        .unwrap_or_default();

    let data_type_index = ml
        .dataset("dataTypeIndex")
        .ok()
        .and_then(|ds| read_i32(&ds).ok())
        .unwrap_or(0);

    let data_unit = ml
        .dataset("dataUnit")
        .ok()
        .and_then(|ds| read_string(&ds).ok());

    let wavelength_actual = ml
        .dataset("wavelengthActual")
        .ok()
        .and_then(|ds| ds.read_scalar::<f64>().ok());

    let source_power = ml
        .dataset("sourcePower")
        .ok()
        .and_then(|ds| ds.read_scalar::<f64>().ok());

    let detector_gain = ml
        .dataset("detectorGain")
        .ok()
        .and_then(|ds| ds.read_scalar::<f64>().ok());

    let module_index = ml
        .dataset("moduleIndex")
        .ok()
        .and_then(|ds| ds.read_scalar::<f64>().ok());

    Ok(Measurement {
        source_index,
        detector_index,
        wavelength_index,
        data_type,
        data_type_label,
        data_type_index,
        data_unit,
        data: ts.column(col).to_vec(),
        wavelength_actual,
        source_power,
        detector_gain,
        module_index,
    })
}

// =============================================================================
// Events  —  nirs/stim{i}/*
// =============================================================================

fn parse_events(nirs: &Group) -> Result<Vec<Event>> {
    (1..)
        .map_while(|i| nirs.group(&format!("stim{i}")).ok().map(|g| (i, g)))
        .map(|(i, stim)| {
            parse_event(&stim).with_context(|| format!("stim{i}"))
        })
        .collect()
}

fn parse_event(stim: &Group) -> Result<Event> {
    let name_ds = stim.dataset("name").context("name dataset missing")?;
    let name = read_string(&name_ds).context("name: read failed")?;

    let data: Array2<f64> = stim
        .dataset("data")
        .context("data dataset missing")?
        .read_2d()
        .context("data: read failed")?;

    let mut markers: Vec<EventMarker> = data
        .rows()
        .into_iter()
        .filter(|row| row.len() >= 3)
        .map(|row| EventMarker {
            onset: row[0],
            duration: row[1],
            value: row[2],
        })
        .collect();

    markers.sort_unstable_by(|a, b| a.onset.partial_cmp(&b.onset).unwrap());

    Ok(Event { name, markers })
}

// =============================================================================
// Auxiliaries  —  nirs/aux{i}/*
// =============================================================================

fn parse_auxiliaries(nirs: &Group) -> Result<Vec<AuxiliaryData>> {
    (1..)
        .map_while(|i| nirs.group(&format!("aux{i}")).ok().map(|g| (i, g)))
        .map(|(i, aux)| {
            parse_auxiliary(&aux).with_context(|| format!("aux{i}"))
        })
        .collect()
}

fn parse_auxiliary(aux: &Group) -> Result<AuxiliaryData> {
    let name_ds = aux.dataset("name").context("name dataset missing")?;
    let name = read_string(&name_ds).context("name: read failed")?;

    let unit_ds = aux.dataset("dataUnit").context("dataUnit dataset missing")?;
    let unit = read_string(&unit_ds).context("dataUnit: read failed")?;

    let data: Vec<f64> = aux
        .dataset("dataTimeSeries")
        .context("dataTimeSeries dataset missing")?
        .read_raw()
        .context("dataTimeSeries: read failed")?;

    let time: Vec<f64> = aux
        .dataset("time")
        .context("time dataset missing")?
        .read_raw()
        .context("time: read failed")?;

    let time_offset = aux
        .dataset("timeOffset")
        .ok()
        .and_then(|ds| ds.read_scalar::<f64>().ok());

    Ok(AuxiliaryData {
        name,
        unit,
        data,
        time,
        time_offset,
    })
}
