// Take in .nii.gz file
use crate::domain::anatomy::SubjectAnatomy;
use crate::domain::scene::SceneObject;
use crate::domain::voxel::VoxelVolume;

use nalgebra as na;
use ndarray16::Array4;
use neuroformats::{write_mgh, FsMgh, FsMghData};
use std::collections::HashMap;
use serde::Serialize;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// User interaction:

// 1. The user provides a .nii.gz
// ----> and a .snirf file
// 2. FreeSurfer segmentation runs
// 3. The resulting meshes are loaded into NIRWizard
// 4. The user manually aligns the fNIRS probe
// 5. The user presses "Calculate Sensitivity"
// 6. The sensitivity profile is calculated and visualized
// 7. Projection mode becomes available letting the user
// set a timepoint in the DataPlotter chart.
//

// Practical NIRWizard :
// Step 1: Menubar Anatomy-> Open MRI -> File Dialog ->
// step 2: Calls load_mri(path: &String)

//

//
#[derive(Serialize, Debug)]
pub struct Config {
    /// Absolute path to the input T1 NIfTI file.
    mri_path: PathBuf,

    /// Absolute path to the output root folder.
    output_folder: PathBuf,

    /// Subject ID — used as the sub-folder name inside output_folder.
    subject_id: String,

    /// Folder that contains license.txt (FreeSurfer license).
    license_folder: PathBuf,

    /// Folder containing the Python helper scripts.
    scripts_folder: PathBuf,
}

impl Config {
    fn subject_dir(&self) -> PathBuf {
        self.output_folder.join(&self.subject_id)
    }
}

// ------------------------------------------------------------------
// Tissue label values
// ------------------------------------------------------------------

#[allow(dead_code)] // background is 0 by default (Array4::zeros), never explicitly written
const LABEL_BG: u8 = 0;
const LABEL_SKULL: u8 = 1;
const LABEL_CSF: u8 = 2;
const LABEL_GM: u8 = 3;
const LABEL_WM: u8 = 4;

// FreeSurfer aseg label sets (from FreeSurfer LUT)
const WM_LABELS: &[i32] = &[2, 41, 7, 46, 77, 78, 79, 192, 251, 252, 253, 254, 255];
const GM_LABELS: &[i32] = &[
    3, 42, 8, 47, 10, 11, 12, 13, 17, 18, 26, 28, 16, 49, 50, 51, 52, 53, 54, 58, 60,
];
const CSF_LABELS: &[i32] = &[4, 5, 14, 15, 24, 31, 43, 44, 63, 72];

// ------------------------------------------------------------------
// Generic helpers
// ------------------------------------------------------------------
/// Convert a Windows path to forward-slash form for Docker volume mounts.
fn docker_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Run a subprocess with inherited stdio.  Streams output live to the terminal.
fn run(label: &str, cmd: &mut Command) -> Result<(), String> {
    println!("  [RUN] {label}");
    let status = cmd
        .status()
        .map_err(|e| format!("[{label}] failed to launch process: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "[{label}] exited with code {:?}",
            status.code().unwrap_or(-1)
        ))
    }
}

// ------------------------------------------------------------------
// MGH data extraction helpers
// ------------------------------------------------------------------

/// Extract voxel data from any MGH dtype as f32 (used for T1 intensities).
fn mgh_as_f32(mgh: &FsMgh) -> Result<Array4<f32>, String> {
    if let Some(a) = &mgh.data.mri_float {
        return Ok(a.clone());
    }
    if let Some(a) = &mgh.data.mri_uchar {
        return Ok(a.mapv(|v| v as f32));
    }
    if let Some(a) = &mgh.data.mri_int {
        return Ok(a.mapv(|v| v as f32));
    }
    if let Some(a) = &mgh.data.mri_short {
        return Ok(a.mapv(|v| v as f32));
    }
    Err("MGH volume contains no data".to_string())
}

/// Extract voxel data from any MGH dtype as i32 (used for integer labels).
fn mgh_as_i32(mgh: &FsMgh) -> Result<Array4<i32>, String> {
    if let Some(a) = &mgh.data.mri_int {
        return Ok(a.clone());
    }
    if let Some(a) = &mgh.data.mri_short {
        return Ok(a.mapv(|v| v as i32));
    }
    if let Some(a) = &mgh.data.mri_uchar {
        return Ok(a.mapv(|v| v as i32));
    }
    if let Some(a) = &mgh.data.mri_float {
        return Ok(a.mapv(|v| v as i32));
    }
    Err("MGH volume contains no data".to_string())
}

// ------------------------------------------------------------------
// Head-mask computation
// ------------------------------------------------------------------

/// Flood-fill background from all 6-face border voxels where T1 <= threshold.
/// Returns a flat Vec<bool> (row-major x,y,z) where true = inside head.
///
/// This is equivalent to scikit-image `binary_fill_holes` after thresholding:
/// any interior dark voxel (CSF pocket, air sinus) enclosed by bright tissue
/// will be marked as "inside head" because it is not reachable from the border.
fn compute_head_mask(t1: &Array4<f32>, threshold: f32) -> Vec<bool> {
    let (nx, ny, nz, _) = t1.dim();
    let n = nx * ny * nz;
    let mut is_bg = vec![false; n];
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

    let idx = |x: usize, y: usize, z: usize| x * ny * nz + y * nz + z;

    // Seed: all boundary voxels below the intensity threshold.
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                let on_border =
                    x == 0 || x == nx - 1 || y == 0 || y == ny - 1 || z == 0 || z == nz - 1;
                if on_border && t1[[x, y, z, 0]] <= threshold {
                    let i = idx(x, y, z);
                    if !is_bg[i] {
                        is_bg[i] = true;
                        queue.push_back((x, y, z));
                    }
                }
            }
        }
    }

    // 6-connected BFS through voxels below the threshold.
    const DIRS: [(i32, i32, i32); 6] = [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];
    while let Some((x, y, z)) = queue.pop_front() {
        for (dx, dy, dz) in DIRS {
            let (x2, y2, z2) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
            if x2 < 0 || x2 >= nx as i32 || y2 < 0 || y2 >= ny as i32 || z2 < 0 || z2 >= nz as i32 {
                continue;
            }
            let (x2, y2, z2) = (x2 as usize, y2 as usize, z2 as usize);
            let i = idx(x2, y2, z2);
            if !is_bg[i] && t1[[x2, y2, z2, 0]] <= threshold {
                is_bg[i] = true;
                queue.push_back((x2, y2, z2));
            }
        }
    }

    // head_mask = NOT background
    is_bg.into_iter().map(|b| !b).collect()
}

// ------------------------------------------------------------------
// Step 1 — verify_setup
// ------------------------------------------------------------------
fn verify_setup(cfg: &Config) -> Result<(), String> {
    println!("\n=== Step 1: Verify Setup ===");

    // Docker daemon running?  (`docker info` connects to the daemon;
    // `docker --version` only reads the local binary and always succeeds.)
    run(
        "docker info (daemon check)",
        Command::new("docker").arg("info"),
    )
    .map_err(|e| format!("{e}\n  → Is Docker Desktop running? Start it and retry."))?;

    // GPU accessible inside Docker?
    run(
        "nvidia-smi in Docker",
        Command::new("docker").args([
            "run",
            "--gpus",
            "all",
            "--rm",
            "nvidia/cuda:12.0.0-base-ubuntu22.04",
            "nvidia-smi",
        ]),
    )
    .map_err(|e| format!("{e}\n  → Enable GPU support in Docker Desktop."))?;

    // FastSurfer image present locally?
    run(
        "inspect deepmi/fastsurfer:latest",
        Command::new("docker").args(["image", "inspect", "deepmi/fastsurfer:latest"]),
    )?;

    // FreeSurfer license file present?
    let license = cfg.license_folder.join("license.txt");
    if !license.is_file() {
        return Err(format!(
            "FreeSurfer license not found at: {}",
            license.display()
        ));
    }
    println!("  [OK] license.txt found");

    // Input MRI file present?
    if !cfg.mri_path.is_file() {
        return Err(format!("Input MRI not found: {}", cfg.mri_path.display()));
    }
    println!("  [OK] Input MRI found");

    println!("  All checks passed.");
    Ok(())
}

// ------------------------------------------------------------------
// Step 2 — run_fastsurfer
// ------------------------------------------------------------------

fn run_fastsurfer(cfg: &Config) -> Result<(), String> {
    println!("\n=== Step 2: Run FastSurfer ===");

    // Idempotent — skip if aseg.mgz already exists.
    if cfg.subject_dir().join("mri").join("aseg.mgz").is_file() {
        println!("  aseg.mgz already exists — skipping FastSurfer.");
        return Ok(());
    }

    let input_folder = cfg.mri_path.parent().ok_or("mri_path has no parent")?;
    let filename = cfg
        .mri_path
        .file_name()
        .ok_or("mri_path has no filename")?
        .to_string_lossy()
        .to_string();

    fs::create_dir_all(&cfg.output_folder)
        .map_err(|e| format!("Cannot create output folder: {e}"))?;

    run(
        "docker run deepmi/fastsurfer",
        Command::new("docker").args([
            "run",
            "--gpus",
            "all",
            "-v",
            &format!("{}:/data", docker_path(input_folder)),
            "-v",
            &format!("{}:/output", docker_path(&cfg.output_folder)),
            "-v",
            &format!("{}:/fs_license", docker_path(&cfg.license_folder)),
            "--rm",
            "--user",
            "root",
            "deepmi/fastsurfer:latest",
            "--fs_license",
            "/fs_license/license.txt",
            "--t1",
            &format!("/data/{filename}"),
            "--sid",
            &cfg.subject_id,
            "--sd",
            "/output",
            "--3T",
            "--threads",
            "max",
            "--allow_root",
        ]),
    )?;

    println!("  FastSurfer complete.");
    Ok(())
}

// ------------------------------------------------------------------
// Step 3 — Build Labeled Voxel Representation
// ------------------------------------------------------------------
// ------------------------------------------------------------------
// Step 4 — Build .Obj Meshes for Visualization.
// ------------------------------------------------------------------

//
// Reads aseg.mgz, T1.mgz, brainmask.mgz directly — no Python or Docker needed.
// Writes head_labels.mgz with the same grid geometry as the input volumes.
fn run_voxelization(cfg: &Config) -> Result<(), String> {
    println!("\n=== Step 3: Build Labeled Voxel Volume (native Rust) ===");

    let out_path = cfg.subject_dir().join("mri").join("head_labels.mgz");
    if out_path.is_file() {
        println!("  head_labels.mgz already exists — skipping.");
        return Ok(());
    }

    let aseg_path = cfg.subject_dir().join("mri").join("aseg.mgz");
    let t1_path = cfg.subject_dir().join("mri").join("T1.mgz");
    let bm_path = cfg.subject_dir().join("mri").join("brainmask.mgz");

    println!("  Loading aseg.mgz ...");
    let aseg_mgh =
        FsMgh::from_file(&aseg_path).map_err(|e| format!("Cannot read aseg.mgz: {e}"))?;

    println!("  Loading T1.mgz ...");
    let t1_mgh = FsMgh::from_file(&t1_path).map_err(|e| format!("Cannot read T1.mgz: {e}"))?;

    println!("  Loading brainmask.mgz ...");
    let bm_mgh =
        FsMgh::from_file(&bm_path).map_err(|e| format!("Cannot read brainmask.mgz: {e}"))?;

    let aseg_data = mgh_as_i32(&aseg_mgh)?;
    let t1_data = mgh_as_f32(&t1_mgh)?;
    let bm_data = mgh_as_i32(&bm_mgh)?;

    let (nx, ny, nz, _) = aseg_data.dim();
    println!("  Volume: {nx} × {ny} × {nz}");

    // Head mask: flood-fill background from border, head = everything else.
    println!("  Computing head mask (threshold = 15) ...");
    let head_mask = compute_head_mask(&t1_data, 15.0);

    // Build output label volume.
    println!("  Mapping tissue labels ...");
    let mut labels = Array4::<u8>::zeros((nx, ny, nz, 1));

    let flat_idx = |x: usize, y: usize, z: usize| x * ny * nz + y * nz + z;

    // 1. Skull/scalp: inside head mask but outside brain.
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                if head_mask[flat_idx(x, y, z)] && bm_data[[x, y, z, 0]] == 0 {
                    labels[[x, y, z, 0]] = LABEL_SKULL;
                }
            }
        }
    }

    // 2-4. CSF → GM → WM from aseg (each pass has higher priority).
    for ((x, y, z, _), &lbl) in aseg_data.indexed_iter() {
        if CSF_LABELS.contains(&lbl) {
            labels[[x, y, z, 0]] = LABEL_CSF;
        }
        if GM_LABELS.contains(&lbl) {
            labels[[x, y, z, 0]] = LABEL_GM;
        }
        if WM_LABELS.contains(&lbl) {
            labels[[x, y, z, 0]] = LABEL_WM;
        }
    }

    // Print voxel counts.
    let mut counts = [0usize; 5];
    for &v in labels.iter() {
        counts[v as usize] += 1;
    }
    let total = labels.len() as f64;
    for (i, name) in ["background", "skull", "CSF", "grey matter", "white matter"]
        .iter()
        .enumerate()
    {
        println!(
            "  {:15}: {:>10} voxels  ({:.1}%)",
            name,
            counts[i],
            100.0 * counts[i] as f64 / total
        );
    }

    // Write output MGZ.  Reuse aseg header for geometry (same voxel grid),
    // but set dtype = 0 (MRI_UCHAR) to match our u8 data.
    let mut out_header = aseg_mgh.header;
    out_header.dtype = 0; // MRI_UCHAR
    out_header.dim4len = 1;

    let out_mgh = FsMgh {
        header: out_header,
        data: FsMghData {
            mri_uchar: Some(labels),
            mri_float: None,
            mri_int: None,
            mri_short: None,
        },
    };

    println!("  Writing: {}", out_path.display());
    write_mgh(&out_path, &out_mgh).map_err(|e| format!("Cannot write head_labels.mgz: {e}"))?;

    println!("  [OK] head_labels.mgz written.");
    Ok(())
}

pub fn load_subject_anatomy(mri_path: &str) -> Result<SubjectAnatomy, String> {
    let mri_path = PathBuf::from(mri_path);

    // mri_path is at data_root/input/<file>.nii.gz
    // so data_root is two levels up (parent of the input folder)
    let data_root = mri_path
        .parent() // .../input
        .and_then(|p| p.parent()) // .../NIRWizard_DATA
        .ok_or("mri_path must be inside an 'input' subfolder (e.g. data_root/input/file.nii.gz)")?
        .to_path_buf();

    // Derive subject_id from BIDS filename:
    //   sub-116_ses-BL_T1w.nii.gz  →  sub-116
    // Take only the first _-delimited BIDS entity (the sub- part).
    let subject_id = {
        let fname = mri_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("subject");
        let stem = fname
            .strip_suffix(".nii.gz")
            .or_else(|| fname.strip_suffix(".nii"))
            .unwrap_or(fname);
        stem.split('_').next().unwrap_or(stem).to_string()
    };

    let cfg = Config {
        subject_id,
        mri_path: mri_path.clone(),
        output_folder: data_root.join("output"),
        license_folder: data_root.join("license"),
        scripts_folder: data_root.join("scripts"),
    };

    println!("[anatomy] subject_id  : {}", cfg.subject_id);
    println!("[anatomy] subject_dir  : {}", cfg.subject_dir().display());
    println!("[anatomy] mri_path     : {}", cfg.mri_path.display());
    println!("[anatomy] output_folder: {}", cfg.output_folder.display());

    verify_setup(&cfg)?;
    run_fastsurfer(&cfg)?;
    // run_voxelization and meshify are future steps — skipped for now

    // We have 5 files we are expecting in the subject dir / surf /
    // "csf.obj", "grey_matter.obj", "white_matter.obj", "skull.obj", "scalp.obj"
    // Load resulting OBJ files (each optional if not yet produced)
    let surf_dir = cfg.subject_dir().join("surf");
    let load_opt = |name: &str| -> Option<SceneObject> {
        let path = surf_dir.join(format!("{name}.obj"));
        if !path.is_file() {
            return None;
        }
        let path_str = path.to_string_lossy().to_string();
        match crate::io::mesh_importer::load_mesh(&path_str) {
            Ok(mesh) => Some(SceneObject::new(name, mesh)),
            Err(e) => {
                eprintln!("[anatomy] failed to load {name}.obj: {e}");
                None
            }
        }
    };

    let labels_mgz = cfg.subject_dir().join("mri").join("head_labels.mgz");

    Ok(SubjectAnatomy {
        skull: load_opt("skull"),
        csf: load_opt("csf"),
        grey_matter: load_opt("grey_matter"),
        white_matter: load_opt("white_matter"),
        labels_mgz_path: if labels_mgz.is_file() { Some(labels_mgz) } else { None },
    })
}

// ------------------------------------------------------------------
// Voxel volume loader
// ------------------------------------------------------------------

/// Build the vox2ras (4×4) from an MGH header using the FreeSurfer convention.
fn mgh_vox2ras(header: &neuroformats::FsMghHeader, nx: usize, ny: usize, nz: usize) -> na::Matrix4<f64> {
    // mdc_raw: [xras(3), yras(3), zras(3)] — each triple is a direction cosine vector
    let r = &header.mdc_raw;
    let xras = na::Vector3::new(r[0] as f64, r[1] as f64, r[2] as f64);
    let yras = na::Vector3::new(r[3] as f64, r[4] as f64, r[5] as f64);
    let zras = na::Vector3::new(r[6] as f64, r[7] as f64, r[8] as f64);

    // Mdc has xras, yras, zras as columns
    let mdc = na::Matrix3::from_columns(&[xras, yras, zras]);

    let dx = header.delta[0] as f64;
    let dy = header.delta[1] as f64;
    let dz = header.delta[2] as f64;
    let d   = na::Matrix3::from_diagonal(&na::Vector3::new(dx, dy, dz));
    let rd  = mdc * d;

    let cras = na::Vector3::new(
        header.p_xyz_c[0] as f64,
        header.p_xyz_c[1] as f64,
        header.p_xyz_c[2] as f64,
    );
    let half = na::Vector3::new(nx as f64 / 2.0, ny as f64 / 2.0, nz as f64 / 2.0);
    let p0   = cras - rd * half;

    na::Matrix4::new(
        rd[(0,0)], rd[(0,1)], rd[(0,2)], p0[0],
        rd[(1,0)], rd[(1,1)], rd[(1,2)], p0[1],
        rd[(2,0)], rd[(2,1)], rd[(2,2)], p0[2],
        0.0,       0.0,       0.0,       1.0,
    )
}

/// Load head_labels.mgz into a VoxelVolume.
pub fn load_head_labels_volume(path: &PathBuf) -> Result<VoxelVolume, String> {
    println!("[voxel] Loading head_labels.mgz: {}", path.display());

    let mgh = FsMgh::from_file(path)
        .map_err(|e| format!("Cannot read head_labels.mgz: {e}"))?;

    let arr = mgh.data.mri_uchar.as_ref()
        .ok_or("head_labels.mgz is not MRI_UCHAR dtype")?;

    let (nx, ny, nz, _) = arr.dim();

    // Flatten: labels[x + y*nx + z*nx*ny]
    let mut labels = Vec::with_capacity(nx * ny * nz);
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                labels.push(arr[[i, j, k, 0]]);
            }
        }
    }

    let vox2ras = if mgh.header.is_ras_good == 1 {
        mgh_vox2ras(&mgh.header, nx, ny, nz)
    } else {
        na::Matrix4::identity()
    };

    let mut label_names = HashMap::new();
    label_names.insert(1u8, "Skull".to_string());
    label_names.insert(2u8, "CSF".to_string());
    label_names.insert(3u8, "Grey Matter".to_string());
    label_names.insert(4u8, "White Matter".to_string());

    println!("[voxel] Loaded {}×{}×{} label volume", nx, ny, nz);

    Ok(VoxelVolume {
        name: "head_labels".to_string(),
        dims: [nx, ny, nz],
        vox2ras,
        labels,
        label_names,
    })
}
