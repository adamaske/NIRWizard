# SNIRF Parsing — Real-World Compatibility Notes

During initial testing against real SNIRF files we found several divergences between what
the parser assumed and what the files actually contained. This document records each
discrepancy and the corresponding code change.

---

## 1. String encoding: `VarLenUnicode` vs `VarLenAscii`

### Expected
The parser called `read_scalar::<hdf5::types::VarLenUnicode>()` for every string-typed
dataset (stim names, metadata tag values, aux channel names and units).

### Actual
Files exported by MNE-NIRS and similar toolchains store all string datasets as
**variable-length ASCII** (`VarLenAscii`), not variable-length UTF-8. HDF5 treats these
as distinct types and will not silently coerce one to the other, so every string read
returned an error.

Affected paths:
- `/nirs/stim*/name`
- `/nirs/metaDataTags/*` (all tag values)
- `/nirs/aux*/name`
- `/nirs/aux*/dataUnit`

### Fix
A `read_string` helper was added to `snirf_parser.rs` that tries `VarLenUnicode` first
and falls back to `VarLenAscii`:

```rust
fn read_string(ds: &hdf5::Dataset) -> Result<String, hdf5::Error> {
    ds.read_scalar::<hdf5::types::VarLenUnicode>()
        .map(|s| s.to_string())
        .or_else(|_| {
            ds.read_scalar::<hdf5::types::VarLenAscii>()
                .map(|s| s.to_string())
        })
}
```

All string reads in `parse_events`, `parse_biosignals`, and `parse_metadata` were updated
to use this helper. Previously the metadata parser silently fell back to the literal string
`"(non-string)"` for every tag; it now reads ASCII values correctly.

---

## 2. Wavelength storage type: float vs integer

### Expected
`parse_wavelenghts` read the wavelengths dataset as `Vec<usize>`:

```rust
let mut wl_array: Vec<usize> = wl_ds.read_raw()?;
```

### Actual
The SNIRF spec defines wavelengths as floating-point values (e.g. `760.0`, `850.0`).
The HDF5 dataset type is `<f64>`, and the library refuses to read it as `usize`, returning
a type-mismatch error that aborted the entire parse.

Confirmed via the inspector: `/nirs/probe/wavelengths  [2] <f64>`

### Fix
Read as `Vec<f64>` and round to `usize` when storing:

```rust
let mut wl_array: Vec<f64> = wl_ds.read_raw()?;
wl_array.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
Ok(Wavelengths {
    hbo_wl: wl_array[0].round() as usize,
    hbr_wl: wl_array[1].round() as usize,
})
```

---

## 3. Missing 2D probe positions

### Expected
`parse_probe` required both `detectorPos2D` / `sourcePos2D` **and** `detectorPos3D` /
`sourcePos3D` to be present, returning an error if any were missing.

### Actual
Files processed by MNE or exported from hardware that only tracks 3D optode positions
omit the 2D datasets entirely.

Confirmed via the inspector: `/nirs/probe/` contained only `detectorPos3D` and
`sourcePos3D`; no 2D equivalents were present.

### Fix
3D positions are now required (they are needed for spatial analysis). 2D positions are
treated as optional and silently default to zero vectors when absent:

```rust
let d2d_array: Array2<f64> = probe
    .dataset("detectorPos2D")
    .and_then(|ds| ds.read_2d())
    .unwrap_or_else(|_| Array2::zeros((n_detectors, 2)));
```

---

## 4. Silent frontend errors

### Expected
`invoke("load_snirf", { path })` in `menubar.svelte` would surface any Rust error to the
user.

### Actual
The call had no `try/catch`. When the Rust command returned `Err(...)`, the rejected
promise went unhandled and the UI simply did not react — no feedback at all.

### Fix
Wrapped the invoke call in `try/catch` with an `alert` fallback:

```js
try {
    await invoke("load_snirf", { path });
} catch (err) {
    console.error("Failed to load SNIRF:", err);
    alert(`Failed to load file:\n\n${err}`);
}
```

---

## Diagnostic tooling added

To make future file compatibility issues immediately visible, two tools were added:

### HDF5 tree inspector binary
`src-tauri/src/bin/inspect_snirf.rs` — a standalone binary that walks the full HDF5
tree of any file and prints every group, dataset shape, and type to stdout, with scalar
value previews. Run with:

```bash
cargo run --bin inspect_snirf -- path/to/file.snirf
```

### Auto-print on `tauri dev`
`parse_snirf` calls `print_hdf5_tree` in debug builds (`#[cfg(debug_assertions)]`),
so the full file structure appears in the terminal automatically whenever a file is
loaded during development. This is stripped from release builds.

---

## Summary table

| # | Dataset / location | Expected type | Actual type | Resolution |
|---|---|---|---|---|
| 1 | `stim*/name`, metadata tags, aux name/unit | `VarLenUnicode` | `VarLenAscii` | `read_string` helper with ASCII fallback |
| 2 | `probe/wavelengths` | `usize` / integer | `f64` | Read as `f64`, round to `usize` |
| 3 | `probe/detectorPos2D`, `sourcePos2D` | Required | Absent | Optional with zero-vector fallback |
| 4 | Frontend `invoke` error handling | Visible error | Silent failure | `try/catch` + `alert` |
