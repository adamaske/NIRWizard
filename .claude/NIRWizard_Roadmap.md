# NIRWizard Implementation Roadmap

## Reimplementing NIRSViz in Rust + Svelte + Three.js

This roadmap maps every major feature from NIRSViz (C++/OpenGL/ImGui) to its NIRWizard equivalent, organized into phases that build on each other. Each phase produces a usable application.

---

## Current NIRWizard State (What Exists)

### Rust Backend
- SNIRF parser (io/snirf_parser.rs, io/snirf_exporter.rs)
- Domain types (domain/snirf.rs, domain/pipeline.rs)
- Commands: summary, timeseries, probe data, pipeline
- AppState with RwLock Session pattern
- Processing module (stubbed)

### Svelte Frontend
- MenuBar, StatusBar, ControlPanel, InfoPanel
- DataPlotter (ECharts integration)
- ChannelSelector (2D probe map)
- Pipeline editor UI

### Not Yet Implemented
- 3D rendering (Three.js)
- Mesh loading and anatomy
- Projection system
- Coordinate system and landmarks
- MRI support
- Connectivity analysis
- HRF and GLM analysis
- Biosignal display
- Config persistence
- Project save/load

---

## Feature Mapping: NIRSViz to NIRWizard

NIRSViz SNIRFLoader + snirf-cpp maps to io/snirf_parser.rs (hdf5-metno) -- Done

NIRSViz SNIRFService maps to state.rs Session + commands -- Done

NIRSViz SNIRF struct maps to domain/snirf.rs -- Done

NIRSViz PlottingSystem (ImPlot) maps to DataPlotter.svelte (ECharts) -- Done

NIRSViz ChannelSelectorSystem (pixel buffer) maps to ChannelSelector.svelte (SVG) -- In Progress

NIRSViz AnatomyService + Anatomy class maps to Phase 3: Rust mesh loader + Three.js -- Not Started

NIRSViz ProbeSystem (OpenGL rendering) maps to Phase 3: Three.js optode rendering -- Not Started

NIRSViz ProjectionSystem (custom shader) maps to Phase 4: Three.js ShaderMaterial -- Not Started

NIRSViz CoordinateSystem + landmarks maps to Phase 4: Rust computation + Svelte UI -- Not Started

NIRSViz MRISystem + MRIImage maps to Phase 5: Niivue integration -- Not Started

NIRSViz BiosignalPlottingSystem maps to Phase 2: ECharts secondary chart -- Not Started

NIRSViz ConfigStore maps to Phase 2: Tauri filesystem -- Not Started

NIRSViz ProjectService maps to Phase 2: JSON project files -- Not Started

NIRSViz Renderer (OpenGL abstraction) replaced by Three.js (no custom abstraction needed)

NIRSViz ViewportManager (multi-viewport) replaced by Svelte layout + multiple canvases

NIRSViz EventBus (commented out) replaced by Svelte stores + Tauri events

NIRSViz SystemManager + System base class not needed (commands replace systems)

---

## Phase 1: Solid Foundation (Current to Complete)

Goal: A fully functional 2D fNIRS data viewer and preprocessor

### 1.1 Complete SNIRF Data Flow
- [x] SNIRF parsing (HDF5 to Rust struct)
- [x] Summary command (metadata, channel count, duration)
- [x] Timeseries command (channel data for plotting)
- [x] Probe layout command (2D positions for channel selector)
- [ ] Event/stimulus marker data command
- [ ] SNIRF export (write processed data back to .snirf)

NIRSViz equivalent: SNIRFLoader, SNIRFService, FileSystem

### 1.2 Channel Selector (2D Probe Map)
- [x] SVG-based 2D head map with sources/detectors
- [ ] Click-to-select channels (toggle individual)
- [ ] Select All / Clear Selection buttons
- [ ] Visual feedback: selected channels highlighted (yellow like NIRSViz)
- [ ] Pan and zoom on the 2D map
- [ ] Channel labels on hover

NIRSViz equivalent: ChannelSelectorSystem. NIRSViz used a CPU pixel buffer rendered to an OpenGL texture. The SVG approach is cleaner and natively interactive.

### 1.3 Timeseries Plotter
- [x] ECharts rendering with synthetic data
- [ ] Plot real SNIRF data from Rust backend
- [ ] HbO (red) and HbR (blue) traces per channel
- [ ] Toggle HbO/HbR/HbT visibility (legend click)
- [ ] Event markers as vertical lines with labels
- [ ] Zoom (mouse wheel), pan (drag), reset view
- [ ] Time cursor synchronized with other panels
- [ ] Multi-channel stacked view option

NIRSViz equivalent: PlottingSystem. NIRSViz used ImPlot which is immediate mode. ECharts is retained and handles zoom/pan natively.

### 1.4 Preprocessing Pipeline
- [ ] Bandpass filter (Butterworth, in Rust)
- [ ] TDDR motion correction (in Rust)
- [ ] Z-normalization (in Rust)
- [ ] Pipeline editor: drag-and-drop ordering of steps
- [ ] Pipeline execution: apply steps sequentially to SNIRF data
- [ ] Undo: revert to pre-processing state
- [ ] Processing history tracking in Session

NIRSViz equivalent: Partially in PlottingSystem::EditProcessingStream() but was incomplete. NIRWizard already has the pipeline editor UI. The Rust processing functions need implementation.

### 1.5 Info Panel and Session Management
- [x] Display SNIRF metadata
- [ ] Display per-channel statistics (mean, std, SNR)
- [ ] Config persistence (save/load app settings via Tauri filesystem)
- [ ] Project save/load (.nirw JSON project file referencing SNIRF + processing state)

NIRSViz equivalent: ConfigStore, ProjectService, SessionService

---

## Phase 2: Analysis Tools

Goal: HRF estimation, block averaging, connectivity

### 2.1 Biosignal Display
- [ ] Display auxiliary signals (respiration, GSR, PPG, heart rate)
- [ ] Separate ECharts panel or overlay on timeseries
- [ ] Synchronized time axis with main plotter

NIRSViz equivalent: BiosignalPlottingSystem

### 2.2 HRF / GLM Analysis
- [ ] Block averaging (epoch extraction around events)
- [ ] GLM-based HRF estimation
- [ ] Display HRF results in InfoPanel (HRF tab)
- [ ] Statistical output (beta weights, t-values, p-values)

New functionality not in NIRSViz.

### 2.3 Connectivity
- [ ] Channel-to-channel correlation matrix
- [ ] Heatmap visualization (ECharts heatmap or custom SVG)
- [ ] Graph metrics (degree, clustering coefficient)
- [ ] Display in InfoPanel (Connectivity tab)

New functionality not in NIRSViz.

### 2.4 Export
- [ ] Export processed SNIRF
- [ ] Export CSV (timeseries, HRF results, connectivity matrix)
- [ ] Export figures (ECharts has built-in PNG/SVG export)

---

## Phase 3: 3D Visualization

Goal: Bring back the 3D brain rendering from NIRSViz using Three.js

### 3.1 Three.js Setup
- [ ] Viewport3D.svelte component with Three.js canvas
- [ ] On-demand rendering (render only on interaction/data change)
- [ ] OrbitControls (rotate, zoom, pan)
- [ ] ResizeObserver integration
- [ ] Basic lighting (ambient + directional)

NIRSViz equivalent: Renderer::Init(), ViewportManager, OrbitCamera

### 3.2 Mesh Loading
- [ ] Rust OBJ parser (use tobj crate)
- [ ] Command: load_mesh returns flat vertex/normal/index arrays
- [ ] Three.js BufferGeometry from Rust data
- [ ] Render cortex mesh with MeshPhongMaterial
- [ ] Render head mesh with transparency

NIRSViz equivalent: Anatomy class, AnatomySystem, Assimp loader. What was approximately 80 lines of VAO/VBO code in NIRSViz is approximately 10 lines in Three.js.

### 3.3 Probe Rendering in 3D
- [ ] Render sources as red spheres, detectors as blue spheres
- [ ] Render channel lines connecting source-detector pairs
- [ ] Probe transform controls (position, rotation, spread factor)
- [ ] Optode pointing toward target (brain center)

NIRSViz equivalent: ProbeSystem. The UpdateProbeVisuals(), UpdateChannelVisuals(), CalculateProbeRotationMatrix() logic. The math is identical, just expressed in Three.js matrix operations instead of GLM.

### 3.4 Raycasting and Channel-to-Cortex Projection
- [ ] Three.js Raycaster for mesh intersection
- [ ] Project channel midpoints onto cortex surface
- [ ] Visualize projection lines
- [ ] Store intersection results per channel

NIRSViz equivalent: ProbeSystem::ProjectChannelsToCortex(), RaycastSampler. Three.js has Raycaster built in, replacing the BVH library.

---

## Phase 4: Cortical Projection (The Crown Jewel)

Goal: Recreate the projection heatmap from NIRSViz

### 4.1 Influence Map Computation
- [ ] Port UpdateInfluenceMap() to Rust
- [ ] KD-tree for radius search (use kiddo crate, replaces nanoflann)
- [ ] Compute per-vertex influence weights per channel
- [ ] Return influence data to frontend

### 4.2 Activation Visualization
- [ ] Port UpdateActivatedVertices() to Rust
- [ ] Compute per-vertex activation = sum(influence x channel_value)
- [ ] Exponential distance falloff with configurable decay
- [ ] Send per-vertex color array to frontend

### 4.3 Custom Projection Shader
- [ ] Three.js ShaderMaterial with vertex colors
- [ ] Port Projection.vert / Projection.frag GLSL shaders
- [ ] Configurable: strength min/max, falloff, radius, cortex base color
- [ ] Time slider: scrub through recording, update projection in real-time

The GLSL is nearly identical between OpenGL and Three.js. Three.js just manages the uniforms differently.

### 4.4 Projection Settings UI
- [ ] Svelte panel for projection parameters
- [ ] Wavelength selector (HbO/HbR/HbT)
- [ ] Radius, decay, strength range sliders
- [ ] Start/Stop projection toggle

---

## Phase 5: Advanced Features

Goal: Features that go beyond NIRSViz

### 5.1 Coordinate System and Landmarks
- [ ] Port CoordinateSystemGenerator to Rust
- [ ] Sagittal and coronal path computation on mesh surface
- [ ] 10-20 system landmark placement
- [ ] Manual landmark editing UI

This is one of the most complex subsystems: mesh graph traversal (petgraph replaces Boost.Graph), geodesic paths, intersection calculations.

### 5.2 MRI Integration
- [ ] Niivue.js integration for NIfTI viewing
- [ ] Axial/coronal/sagittal slice views
- [ ] MRI-to-mesh coordinate registration

Niivue replaces all custom slice rendering from MRISystem.

### 5.3 Multi-Subject / Group Analysis
- [ ] Load multiple SNIRF files
- [ ] Grand averaging across subjects
- [ ] Group-level statistics

### 5.4 EEG Support
- [ ] EDF+ file parser in Rust
- [ ] Combined fNIRS-EEG visualization
- [ ] Synchronized timeseries display

---

## Key Architectural Differences

### What is Better in NIRWizard

Build system: CMake + vcpkg + submodules becomes Cargo + npm (trivial setup)

SNIRF parsing: snirf-cpp custom C++ becomes Rust with hdf5-metno (safer, fewer linking issues long-term)

2D channel selector: CPU pixel buffer to GL texture becomes SVG with native interaction

Plotting: ImPlot immediate mode with limited zoom becomes ECharts retained mode with rich interaction

UI layout: ImGui docking (fragile) becomes CSS flexbox (robust, resizable)

State management: Singleton services + System references becomes RwLock Session (thread safe)

3D rendering: Custom OpenGL abstraction layer becomes Three.js (less code, same visual result)

Cross-platform: Windows only (practically) becomes Windows/macOS/Linux via Tauri

### What Needs Care in the Port

Mesh graph operations: NIRSViz uses Boost.Graph. Rust has petgraph but the coordinate system generator is complex to port.

KD-tree spatial queries: NIRSViz uses nanoflann. Rust equivalent: kiddo crate.

Matrix math: NIRSViz uses GLM + Eigen. Three.js has its own math. Rust uses nalgebra for backend computation.

GLSL shaders: The projection shader ports nearly verbatim to Three.js ShaderMaterial.

Large data transfer: Mesh vertex arrays (100k+ floats) go through Tauri IPC. Binary transfer may be needed for performance.

---

## Suggested Timeline

Phase 1 (2D tool): 3-4 weeks, no dependencies

Phase 2 (Analysis): 3-4 weeks, requires Phase 1

Phase 3 (3D viz): 2-3 weeks, requires Phase 1

Phase 4 (Projection): 2-3 weeks, requires Phase 3

Phase 5 (Advanced): 4-6 weeks, requires Phase 3 and 4

Phases 2 and 3 can be developed in parallel since they are independent. Phase 4 requires Phase 3. Phase 5 is modular with each sub-feature being independent.
