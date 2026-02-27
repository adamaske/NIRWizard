use hdf5::File;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SnirfData{
    pub time: Vec<f64>, // Time vector
    pub data: Vec<Vec<f64>>, // Channels x timepoints
    pub source_pos: Vec<[f64; 3]>,
    pub detector_pos: Vec<[f64; 3]>,
}

pub fn parse_snirf(path: &str) -> Result<SnirfData, String>{
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

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

    // Read Positions
    let source_pos = read_positions(&file, "/nirs/data1/sourcePos3D")
        .or_else(|_| read_positions(&file, "/nirs/data1/sourcePos2D"))
        .unwrap_or_default();
    let detector_pos = read_positions(&file, "/nirs/data1/detectorPos3D")
        .or_else(|_| read_positions(&file, "/nirs/data1/detectorPos2D"))
        .unwrap_or_default();

    Ok(SnirfData{
        time,
        data,
        source_pos,
        detector_pos
    })
}

fn read_positions(file: &File, path: &str) -> Result<Vec<[f64; 3]>, String>{
    let ds = file
        .dataset(path)
        .map_err(|e| format!("Failed to read positions: {}", e))?;

    let shape = ds.shape();
    let raw: Vec<f64> = ds
        .read_raw()
        .map_err(|e| format!("Failed to parse positions: {}", e))?;

    let n_points = shape[0];
    let n_dims = shape[1];

    // TODO : Optimize with ranges
    let mut positions = Vec::with_capacity(3);
    for i in 0..n_points{
        positions.push([
            raw[i * n_dims],
            raw[i* n_dims + 1],
            if n_dims > 2 { raw[i * n_dims + 2] } else { 0.0 }
        ]);
    }

    Ok(positions)
}