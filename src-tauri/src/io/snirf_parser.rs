use crate::domain::*;
use nalgebra::{Vector2, Vector3};
use hdf5::File;
use ndarray::Array2;

// =============================================================================
// HDF5 tree inspector (debug builds only — prints to terminal on `tauri dev`)
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

/// For scalar (single-element) datasets, tries to read and return a preview
/// string like ` = "PPG"` or ` = 830`. Returns empty string for arrays.
#[cfg(debug_assertions)]
fn scalar_preview(ds: &hdf5::Dataset) -> String {
    let n: usize = ds.shape().iter().product();
    // shape() returns [] for 0-d scalars (product = 1) and [k] for 1-d
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
// String helper — SNIRF files use either VarLenUnicode or VarLenAscii
// =============================================================================

/// Read a string dataset regardless of encoding (Unicode/ASCII) or shape (0-d scalar or [1] array).
fn read_string(ds: &hdf5::Dataset) -> Result<String, String> {
    // 0-d VarLenUnicode
    if let Ok(s) = ds.read_scalar::<hdf5::types::VarLenUnicode>() {
        return Ok(s.to_string());
    }
    // 0-d VarLenAscii
    if let Ok(s) = ds.read_scalar::<hdf5::types::VarLenAscii>() {
        return Ok(s.to_string());
    }
    // [1] VarLenAscii (SATORI style)
    if let Some(s) = ds
        .read_raw::<hdf5::types::VarLenAscii>()
        .ok()
        .and_then(|v| v.into_iter().next())
    {
        return Ok(s.to_string());
    }
    // [1] VarLenUnicode
    if let Some(s) = ds
        .read_raw::<hdf5::types::VarLenUnicode>()
        .ok()
        .and_then(|v| v.into_iter().next())
    {
        return Ok(s.to_string());
    }
    Err("no readable string encoding found".to_string())
}

/// Read an i32 dataset regardless of shape (0-d scalar or [1] array).
fn read_i32(ds: &hdf5::Dataset) -> Result<i32, String> {
    ds.read_scalar::<i32>()
        .ok()
        .or_else(|| ds.read_raw::<i32>().ok().and_then(|v| v.into_iter().next()))
        .ok_or_else(|| "failed to read i32 value".to_string())
}

// =============================================================================
// Public parser entry
// =============================================================================

pub fn parse_snirf(path: &str) -> Result<SNIRF, String> {
    let _file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    //#[cfg(debug_assertions)]
    //print_hdf5_tree(path, &_file);

    let _fd = FileDescriptor {
        path: path.to_string(),
        name: std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
    };

    let _metadata: Metadata = parse_metadata(&_file)?;
    let _auxilires: BiosignalData = parse_biosignals(&_file)?;
    let _wavelengths: Wavelengths = parse_wavelenghts(&_file)?;
    let _probe = parse_probe(&_file)?;
    let _channels = parse_measurement_list(&_file)?;
    let _events = parse_events(&_file)?;

    let snirf: SNIRF = SNIRF {
        fd: _fd,
        metadata: _metadata,
        wavelengths: _wavelengths,
        channels: _channels,
        probe: _probe,
        events: _events,
        biosignals: _auxilires,
    };

    println!("Parsed SNIRF: {}", snirf);

    Ok(snirf)
}

pub fn parse_timeseries_data(file: &File) -> Result<TimeSeriesData, String> {
    let time_ds = file
        .dataset("nirs/data1/time")
        .map_err(|e| format!("Failed to read time: {}", e))?;

    let time: Vec<f64> = time_ds
        .read_raw()
        .map_err(|e| format!("Failed to parse time: {}", e))?;

    let data_ds = file
        .dataset("/nirs/data1/dataTimeSeries")
        .map_err(|e| format!("Failed to read data: {}", e))?;

    let array: Array2<f64> = data_ds
        .read_2d()
        .map_err(|e| format!("Failed to parse data: {}", e))?;

    // HDF5 layout is [timepoints, channels]; collect each column as a channel
    let n_channels = array.ncols();
    let data: Vec<Vec<f64>> = (0..n_channels)
        .map(|ch| array.column(ch).to_vec())
        .collect();

    Ok(TimeSeriesData {
        time,
        data, // Placeholder, actual data parsing logic needed
    })
}

pub fn parse_biosignals(file: &File) -> Result<BiosignalData, String> {
    let nirs = file
        .group("/nirs")
        .map_err(|e| format!("Failed to open /nirs group: {}", e))?;

    let mut auxilaries = Vec::new();
    let mut i = 1usize;

    loop {
        let aux = match nirs.group(&format!("aux{}", i)) {
            Ok(g) => g,
            Err(_) => break,
        };

        let name = aux
            .dataset("name")
            .map_err(|e| format!("aux{}: failed to read name: {}", i, e))
            .and_then(|ds| {
                read_string(&ds).map_err(|e| format!("aux{}: failed to parse name: {}", i, e))
            })?;

        let unit = aux
            .dataset("dataUnit")
            .map_err(|e| format!("aux{}: failed to read dataUnit: {}", i, e))
            .and_then(|ds| {
                read_string(&ds).map_err(|e| format!("aux{}: failed to parse dataUnit: {}", i, e))
            })?;

        let data: Vec<f64> = aux
            .dataset("dataTimeSeries")
            .map_err(|e| format!("aux{}: failed to read dataTimeSeries: {}", i, e))?
            .read_raw()
            .map_err(|e| format!("aux{}: failed to parse dataTimeSeries: {}", i, e))?;

        let time: Vec<f64> = aux
            .dataset("time")
            .map_err(|e| format!("aux{}: failed to read time: {}", i, e))?
            .read_raw()
            .map_err(|e| format!("aux{}: failed to parse time: {}", i, e))?;

        auxilaries.push(AuxiliaryData {
            name,
            unit,
            data,
            time,
        });
        i += 1;
    }

    Ok(BiosignalData { auxilaries })
}

pub fn parse_probe(file: &File) -> Result<Probe, String> {
    // We have
    let probe = file
        .group("/nirs/probe")
        .map_err(|e| format!("Failed to read probe: {}", e))?;

    let d3d_array: Array2<f64> = probe
        .dataset("detectorPos3D")
        .map_err(|e| format!("Failed to read 3D detector positions: {}", e))?
        .read_2d()
        .map_err(|e| format!("Failed to parse 3D detector positions: {}", e))?;

    let s3d_array: Array2<f64> = probe
        .dataset("sourcePos3D")
        .map_err(|e| format!("Failed to read 3D source positions: {}", e))?
        .read_2d()
        .map_err(|e| format!("Failed to parse 3D source positions: {}", e))?;

    let n_detectors = d3d_array.nrows();
    let n_sources = s3d_array.nrows();

    // 2D positions are optional — fall back to x,y from 3D positions if absent
    let d2d_array: Array2<f64> = probe
        .dataset("detectorPos2D")
        .and_then(|ds| ds.read_2d())
        .unwrap_or_else(|_| d3d_array.slice(ndarray::s![.., 0..2]).to_owned());

    let s2d_array: Array2<f64> = probe
        .dataset("sourcePos2D")
        .and_then(|ds| ds.read_2d())
        .unwrap_or_else(|_| s3d_array.slice(ndarray::s![.., 0..2]).to_owned());

    let row_to_vec2 = |arr: &Array2<f64>, i: usize| Vector2::new(arr[[i, 0]], arr[[i, 1]]);
    let row_to_vec3 =
        |arr: &Array2<f64>, i: usize| Vector3::new(arr[[i, 0]], arr[[i, 1]], arr[[i, 2]]);

    let _sources: Vec<Optode> = (0..n_sources)
        .map(|i| Optode {
            name: format!("S{}", i + 1),
            id: i,
            pos_2d: row_to_vec2(&s2d_array, i),
            pos_3d: row_to_vec3(&s3d_array, i),
        })
        .collect();

    let _detectors: Vec<Optode> = (0..n_detectors)
        .map(|i| Optode {
            name: format!("D{}", i + 1),
            id: i,
            pos_2d: row_to_vec2(&d2d_array, i),
            pos_3d: row_to_vec3(&d3d_array, i),
        })
        .collect();

    Ok(Probe {
        sources: _sources,     // Placeholder
        detectors: _detectors, // Placeholder
    })
}

pub fn parse_wavelenghts(file: &File) -> Result<Wavelengths, String> {
    let probe = file
        .group("/nirs/probe")
        .map_err(|e| format!("Failed to read probe: {}", e))?;

    let wl_ds = probe
        .dataset("wavelengths")
        .map_err(|e| format!("Failed to read wavelengths: {}", e))?;

    let mut wl_array: Vec<f64> = wl_ds
        .read_raw()
        .map_err(|e| format!("Failed to parse wavelengths: {}", e))?;

    if wl_array.len() < 2 {
        return Err(format!(
            "Expected at least 2 wavelengths, got {}",
            wl_array.len()
        ));
    }

    // Sort descending: highest wavelength = HbO, lowest = HbR
    wl_array.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

    Ok(Wavelengths {
        hbo_wl: wl_array[0].round() as usize,
        hbr_wl: wl_array[1].round() as usize,
    })
}

pub fn parse_measurement_list(file: &File) -> Result<ChannelData, String> {
    let ts = parse_timeseries_data(file)?;
    let half = ts.data.len() / 2;
    let mut ts_data = ts.data;

    let data1 = file
        .group("/nirs/data1")
        .map_err(|e| format!("Failed to read data1 group: {}", e))?;

    // The first half of dataTimeSeries columns are HbR (lower wavelength),
    // the second half are HbO (higher wavelength).
    // We read measurementList1..half to get source/detector IDs.
    let channels = (0..half)
        .map(|i| {
            let ml = data1
                .group(&format!("measurementList{}", i + 1))
                .map_err(|e| format!("measurementList{}: failed to open: {}", i + 1, e))?;

            let source_id = ml
                .dataset("sourceIndex")
                .map_err(|e| format!("measurementList{}: sourceIndex: {}", i + 1, e))
                .and_then(|ds| {
                    read_i32(&ds)
                        .map_err(|e| format!("measurementList{}: sourceIndex: {}", i + 1, e))
                })? as usize;

            let detector_id = ml
                .dataset("detectorIndex")
                .map_err(|e| format!("measurementList{}: detectorIndex: {}", i + 1, e))
                .and_then(|ds| {
                    read_i32(&ds)
                        .map_err(|e| format!("measurementList{}: detectorIndex: {}", i + 1, e))
                })? as usize;

            Ok(Channel {
                id: i,
                name: format!("S{}-D{}", source_id, detector_id),
                source_id,
                detector_id,
                hbr: std::mem::take(&mut ts_data[i]),
                hbo: std::mem::take(&mut ts_data[i + half]),
            })
        })
        .collect::<Result<Vec<Channel>, String>>()?;

    Ok(ChannelData {
        time: ts.time,
        channels,
    })
}

pub fn parse_events(file: &File) -> Result<Events, String> {
    let nirs = file
        .group("/nirs")
        .map_err(|e| format!("Failed to read nirs group: {}", e))?;

    let mut events = Vec::new();
    let mut i = 1usize;

    loop {
        let stim = match nirs.group(&format!("stim{}", i)) {
            Ok(g) => g,
            Err(_) => break,
        };

        let name = stim
            .dataset("name")
            .map_err(|e| format!("stim{}: failed to read name: {}", i, e))
            .and_then(|ds| {
                read_string(&ds).map_err(|e| format!("stim{}: failed to parse name: {}", i, e))
            })?;

        let data: Array2<f64> = stim
            .dataset("data")
            .map_err(|e| format!("stim{}: failed to read data: {}", i, e))?
            .read_2d()
            .map_err(|e| format!("stim{}: failed to parse data: {}", i, e))?;

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

        events.push(Event { name, markers });
        i += 1;
    }

    Ok(Events { events })
}

pub fn parse_metadata(file: &File) -> Result<Metadata, String> {
    let group = file
        .group("/nirs/metaDataTags")
        .map_err(|e| format!("Failed to read metadata group: {}", e))?;

    let tags = group
        .member_names()
        .map_err(|e| format!("Failed to list metadata members: {}", e))?
        .into_iter()
        .filter_map(|name| {
            // .ok()? skips this member if it isn't a dataset (e.g. a sub-group)
            let ds = group.dataset(&name).ok()?;
            let value = read_string(&ds).unwrap_or_else(|_| "(non-string)".to_string());
            Some(MetadataTag { name, value })
        })
        .collect();

    Ok(Metadata { tags })
}
