use crate::domain::*;
use hdf5::File;
use ndarray::Array2;

pub fn parse_snirf(path: &str) -> Result<SNIRF, String> {
    // Placeholder for actual SNIRF parsing logic
    let _file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

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
    // The biosignals may or may not be present, we need to check
    let max_aux = 99;

    // Each auxiliary is stored aux1, aux2, and so on

    // Placeholder for actual biosignal parsing logic
    Ok(BiosignalData {
        time: vec![],       // Placeholder
        auxilaries: vec![], // Placeholder
    })
}

pub fn parse_probe(file: &File) -> Result<Probe, String> {
    // We have
    let probe = file
        .group("/nirs/probe")
        .map_err(|e| format!("Failed to read probe: {}", e))?;

    let d2d_array: Array2<f64> = probe
        .dataset("detectorPos2D")
        .map_err(|e| format!("Failed to read 2D positions: {}", e))?
        .read_2d()
        .map_err(|e| format!("Failed to parse 2D detector positions: {}", e))?;

    let d3d_array: Array2<f64> = probe
        .dataset("detectorPos3D")
        .map_err(|e| format!("Failed to read 3D positions: {}", e))?
        .read_2d()
        .map_err(|e| format!("Failed to parse 3D detector positions: {}", e))?;

    let s2d_array: Array2<f64> = probe
        .dataset("sourcePos2D")
        .map_err(|e| format!("Failed to read source 2D positions: {}", e))?
        .read_2d()
        .map_err(|e| format!("Failed to parse 2D source positions: {}", e))?;

    let s3d_array: Array2<f64> = probe
        .dataset("sourcePos3D")
        .map_err(|e| format!("Failed to read source 3D positions: {}", e))?
        .read_2d()
        .map_err(|e| format!("Failed to parse 3D source positions: {}", e))?;

    let n_detectors = d2d_array.nrows();
    let n_sources = s2d_array.nrows();

    let row_to_vec2 = |arr: &Array2<f64>, i: usize| Vec2 {
        x: arr[[i, 0]],
        y: arr[[i, 1]],
    };

    let row_to_vec3 = |arr: &Array2<f64>, i: usize| Vec3 {
        x: arr[[i, 0]],
        y: arr[[i, 1]],
        z: arr[[i, 2]],
    };

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

    let mut wl_array: Vec<usize> = wl_ds
        .read_raw()
        .map_err(|e| format!("Failed to parse wavelengths: {}", e))?;

    wl_array.sort_unstable_by(|a, b| b.cmp(a));

    Ok(Wavelengths {
        hbo_wl: wl_array[0],
        hbr_wl: wl_array[1],
    })
}

pub fn parse_measurement_list(file: &File) -> Result<ChannelData, String> {
    // Cache the full timeseries; data is distributed into channels below
    let ts = parse_timeseries_data(file)?;
    let half = ts.data.len() / 2;
    let mut ts_data = ts.data;
    let mut take_col = |idx: usize| std::mem::take(&mut ts_data[idx]);

    let data1 = file
        .group("/nirs/data1")
        .map_err(|e| format!("Failed to read data1 group: {}", e))?;

    let channels = (0..half)
        .map(|i| {
            let ml = data1
                .group(&format!("measurementList{}", i + 1))
                .map_err(|e| format!("measurementList{}: failed to open: {}", i + 1, e))?;

            let source_id = ml
                .dataset("sourceIndex")
                .map_err(|e| format!("measurementList{}: failed to read sourceIndex: {}", i + 1, e))?
                .read_scalar::<i32>()
                .map_err(|e| format!("measurementList{}: failed to parse sourceIndex: {}", i + 1, e))? as usize;

            let detector_id = ml
                .dataset("detectorIndex")
                .map_err(|e| format!("measurementList{}: failed to read detectorIndex: {}", i + 1, e))?
                .read_scalar::<i32>()
                .map_err(|e| format!("measurementList{}: failed to parse detectorIndex: {}", i + 1, e))? as usize;

            let wl_idx = ml
                .dataset("wavelengthIndex")
                .map_err(|e| format!("measurementList{}: failed to read wavelengthIndex: {}", i + 1, e))?
                .read_scalar::<i32>()
                .map_err(|e| format!("measurementList{}: failed to parse wavelengthIndex: {}", i + 1, e))? as usize;

            // wavelengthIndex 1 = wavelengths[0] = HbO (highest, sorted descending)
            let (hbo_col, hbr_col) = if wl_idx == 1 { (i, i + half) } else { (i + half, i) };

            Ok(Channel {
                id: i,
                name: format!("S{}-D{}", source_id, detector_id),
                source_id,
                detector_id,
                hbo: take_col(hbo_col),
                hbr: take_col(hbr_col),
            })
        })
        .collect::<Result<Vec<Channel>, String>>()?;

    Ok(ChannelData { time: ts.time, channels })
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
            .map_err(|e| format!("stim{}: failed to read name: {}", i, e))?
            .read_scalar::<hdf5::types::VarLenUnicode>()
            .map(|s| s.to_string())
            .map_err(|e| format!("stim{}: failed to parse name: {}", i, e))?;

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
            let value = ds
                .read_scalar::<hdf5::types::VarLenUnicode>()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "(non-string)".to_string());
            Some(MetadataTag { name, value })
        })
        .collect();

    Ok(Metadata { tags })
}
