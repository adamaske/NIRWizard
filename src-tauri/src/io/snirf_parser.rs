use crate::domain::{SNIRF, FileDescriptor, TimeSeriesData, BiosignalData};
use hdf5::File;

pub fn parse_snirf(path: &str) -> Result<SNIRF, String> {
    // Placeholder for actual SNIRF parsing logic
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let fd = FileDescriptor {
        path: path.to_string(),
        name: path.split('/').last().unwrap_or("unknown").to_string(),
    };



    let ts_data : TimeSeriesData = parse_timeseries_data(&file)?;

    let auxiliares : BiosignalData = parse_biosignals(&file)?; 

    let snirf: SNIRF = SNIRF {
        fd,
        data: ts_data,
        biosignal: auxiliares,
    };

    println!("Parsed SNIRF: {}", snirf);

    Ok(snirf)
}

pub fn parse_timeseries_data(file: &File) -> Result<TimeSeriesData, String> {
    let time_ds =  file
        .dataset("nirs/data1/time")
        .map_err(|e| format!("Failed to read time: {}", e))?;

    let time: Vec<f64> = time_ds
        .read_raw()        
        .map_err(|e| format!("Failed to parse time: {}", e))?;

    let data_ds = file
        .dataset("/nirs/data1/dataTimeSeries")
        .map_err(|e| format!("Failed to read data: {}", e))?;

    let shape = data_ds.shape();
    let raw: Vec<f64> = data_ds
        .read_raw()
        .map_err(|e| format!("Failed to parse data: {}", e))?;

    let n_timepoints = shape[0];
    let n_channels = shape[1];
    // TODO : Optimize with ranges
    let mut data = vec![vec![0.0; n_timepoints]; n_channels];
    for t in 0..n_timepoints{
        for ch in 0..n_channels{
            data[ch][t] = raw[t * n_channels + ch];
        }
    }   

    Ok(TimeSeriesData{
        time,
        data, // Placeholder, actual data parsing logic needed
    })
}

pub fn parse_biosignals(file: &File) -> Result<BiosignalData, String> {
    // Placeholder for actual biosignal parsing logic
    Ok(BiosignalData{
        time: vec![], // Placeholder
        auxilaries: vec![], // Placeholder
    })
}