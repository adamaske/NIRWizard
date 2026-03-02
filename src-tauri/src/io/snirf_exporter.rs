use crate::domain::*;
use hdf5::File;
use ndarray::Array2;

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
    let file = File::create(path)
        .map_err(|e| format!("Failed to create file '{}': {}", path, e))?;

    let nirs = file
        .create_group("nirs")
        .map_err(|e| format!("Failed to create /nirs group: {}", e))?;

    write_metadata(&nirs, &snirf.metadata)?;
    write_probe(&nirs, &snirf.probe, &snirf.wavelengths)?;
    write_channels(&nirs, &snirf.channels)?;
    write_events(&nirs, &snirf.events)?;
    write_biosignals(&nirs, &snirf.biosignals)?;

    println!("Exported SNIRF to: {}", path);
    Ok(())
}

// =============================================================================
// Section writers — each mirrors the corresponding parse_* function
// =============================================================================

/// `/nirs/metaDataTags/*` — one VarLenUnicode scalar dataset per tag.
fn write_metadata(nirs: &hdf5::Group, metadata: &Metadata) -> Result<(), String> {
    let group = nirs
        .create_group("metaDataTags")
        .map_err(|e| format!("Failed to create metaDataTags group: {}", e))?;

    for tag in &metadata.tags {
        write_string(&group, &tag.name, &tag.value)?;
    }

    Ok(())
}

/// `/nirs/probe/*` — source/detector positions + wavelengths.
///
/// Probe and wavelengths share the same HDF5 group, so they are written together
/// (mirrors the fact that both `parse_probe` and `parse_wavelengths` open `/nirs/probe`).
fn write_probe(
    nirs: &hdf5::Group,
    probe: &Probe,
    wavelengths: &Wavelengths,
) -> Result<(), String> {
    let group = nirs
        .create_group("probe")
        .map_err(|e| format!("Failed to create probe group: {}", e))?;

    // ── Wavelengths ───────────────────────────────────────────────────────────
    // Written highest-first [hbo_wl, hbr_wl] so that after the parser's
    // `sort_unstable_by(desc)` step, index 0 = HbO and index 1 = HbR.
    let wl = [wavelengths.hbo_wl as f64, wavelengths.hbr_wl as f64];
    write_f64_1d(&group, "wavelengths", &wl)?;

    // ── Source positions ──────────────────────────────────────────────────────
    let n_sources = probe.sources.len();
    let mut s2d = Array2::<f64>::zeros((n_sources, 2));
    let mut s3d = Array2::<f64>::zeros((n_sources, 3));

    for (i, src) in probe.sources.iter().enumerate() {
        s2d[[i, 0]] = src.pos_2d.x;
        s2d[[i, 1]] = src.pos_2d.y;
        s3d[[i, 0]] = src.pos_3d.x;
        s3d[[i, 1]] = src.pos_3d.y;
        s3d[[i, 2]] = src.pos_3d.z;
    }

    write_f64_2d(&group, "sourcePos2D", &s2d)?;
    write_f64_2d(&group, "sourcePos3D", &s3d)?;

    // ── Detector positions ────────────────────────────────────────────────────
    let n_detectors = probe.detectors.len();
    let mut d2d = Array2::<f64>::zeros((n_detectors, 2));
    let mut d3d = Array2::<f64>::zeros((n_detectors, 3));

    for (i, det) in probe.detectors.iter().enumerate() {
        d2d[[i, 0]] = det.pos_2d.x;
        d2d[[i, 1]] = det.pos_2d.y;
        d3d[[i, 0]] = det.pos_3d.x;
        d3d[[i, 1]] = det.pos_3d.y;
        d3d[[i, 2]] = det.pos_3d.z;
    }

    write_f64_2d(&group, "detectorPos2D", &d2d)?;
    write_f64_2d(&group, "detectorPos3D", &d3d)?;

    Ok(())
}

/// `/nirs/data1/*` — time vector, dataTimeSeries matrix, and measurementLists.
///
/// Column layout of dataTimeSeries (mirrors parser's `half` logic):
///   cols [0 .. N)       → HbR (lower wavelength)
///   cols [N .. 2N)      → HbO (higher wavelength)
///
/// measurementList{1..N}   → HbR channels, wavelengthIndex = 2
/// measurementList{N+1..2N} → HbO channels, wavelengthIndex = 1
///
/// `source_id` / `detector_id` are stored 1-based in the domain (per SNIRF spec)
/// and are written as-is — no index adjustment needed.
fn write_channels(nirs: &hdf5::Group, channels: &ChannelData) -> Result<(), String> {
    let group = nirs
        .create_group("data1")
        .map_err(|e| format!("Failed to create data1 group: {}", e))?;

    let n_channels = channels.channels.len();
    let n_timepoints = channels.time.len();

    // ── Time ─────────────────────────────────────────────────────────────────
    write_f64_1d(&group, "time", &channels.time)?;

    // ── dataTimeSeries [T × 2N] ───────────────────────────────────────────────
    let mut matrix = Array2::<f64>::zeros((n_timepoints, 2 * n_channels));

    for (i, ch) in channels.channels.iter().enumerate() {
        for t in 0..n_timepoints {
            matrix[[t, i]] = ch.hbr[t];              // cols 0..N  → HbR
            matrix[[t, i + n_channels]] = ch.hbo[t]; // cols N..2N → HbO
        }
    }

    write_f64_2d(&group, "dataTimeSeries", &matrix)?;

    // ── measurementLists ──────────────────────────────────────────────────────
    for (i, ch) in channels.channels.iter().enumerate() {
        // HbR entry: list index i+1, wavelengthIndex=2 (lower wl = second after desc sort)
        let hbr_idx = i + 1;
        let ml_hbr = group
            .create_group(&format!("measurementList{}", hbr_idx))
            .map_err(|e| format!("measurementList{}: failed to create: {}", hbr_idx, e))?;
        write_i32(&ml_hbr, "sourceIndex", ch.source_id as i32)?;
        write_i32(&ml_hbr, "detectorIndex", ch.detector_id as i32)?;
        write_i32(&ml_hbr, "wavelengthIndex", 2)?;
        write_i32(&ml_hbr, "dataType", 99)?; // 99 = processed (HbO/HbR concentrations)

        // HbO entry: list index i+1+N, wavelengthIndex=1 (higher wl = first after desc sort)
        let hbo_idx = i + n_channels + 1;
        let ml_hbo = group
            .create_group(&format!("measurementList{}", hbo_idx))
            .map_err(|e| format!("measurementList{}: failed to create: {}", hbo_idx, e))?;
        write_i32(&ml_hbo, "sourceIndex", ch.source_id as i32)?;
        write_i32(&ml_hbo, "detectorIndex", ch.detector_id as i32)?;
        write_i32(&ml_hbo, "wavelengthIndex", 1)?;
        write_i32(&ml_hbo, "dataType", 99)?;
    }

    Ok(())
}

/// `/nirs/stim{i}/*` — stimulus / event markers.
///
/// Each event is written as a `name` string + a `data` matrix with shape [K × 3]:
/// column 0 = onset (s), column 1 = duration (s), column 2 = value.
fn write_events(nirs: &hdf5::Group, events: &Events) -> Result<(), String> {
    for (i, event) in events.events.iter().enumerate() {
        let stim_name = format!("stim{}", i + 1);
        let stim = nirs
            .create_group(&stim_name)
            .map_err(|e| format!("{}: failed to create group: {}", stim_name, e))?;

        write_string(&stim, "name", &event.name)?;

        // Build [K × 3] matrix — onset | duration | value
        let n_markers = event.markers.len();
        let mut data = Array2::<f64>::zeros((n_markers, 3));

        for (k, marker) in event.markers.iter().enumerate() {
            data[[k, 0]] = marker.onset;
            data[[k, 1]] = marker.duration;
            data[[k, 2]] = marker.value;
        }

        write_f64_2d(&stim, "data", &data)?;
    }

    Ok(())
}

/// `/nirs/aux{i}/*` — auxiliary / biosignal channels.
fn write_biosignals(nirs: &hdf5::Group, biosignals: &BiosignalData) -> Result<(), String> {
    for (i, aux) in biosignals.auxilaries.iter().enumerate() {
        let aux_name = format!("aux{}", i + 1);
        let group = nirs
            .create_group(&aux_name)
            .map_err(|e| format!("{}: failed to create group: {}", aux_name, e))?;

        write_string(&group, "name", &aux.name)?;
        write_string(&group, "dataUnit", &aux.unit)?;
        write_f64_1d(&group, "dataTimeSeries", &aux.data)?;
        write_f64_1d(&group, "time", &aux.time)?;
    }

    Ok(())
}
