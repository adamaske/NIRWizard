# NIRWizard v2 — Architecture Review & Rewrite Plan

## Current State Assessment

### What You Have (and what's solid)

Your existing codebase has good bones. The domain model is clean — `SNIRF`, `Channel`, `Optode`, `Probe` are well-defined. The SNIRF HDF5 parser is thorough. The separation between `domain/`, `io/`, and `commands/` is the right instinct. The `AppState → RwLock<Session>` pattern is correct for Tauri.

### What Needs Work

**1. The "God Session" problem.** Your `Session` struct holds everything — SNIRF data, anatomy, voxel volumes, optode layout, pipeline, selected channels. This becomes unmanageable as features grow. Every command locks the entire session even if it only needs one field.

**2. Commands are doing too much.** `load_snirf` parses, stores, derives optode layout, and emits events — all in one function. Commands should be thin wrappers.

**3. Frontend state is ad-hoc.** Some state lives in Svelte `let` variables in `App.svelte`, some in stores (`sceneState.js`, `pipeline.js`), and some is fetched on-demand. There's no consistent pattern for when to cache vs. re-fetch.

**4. The preprocessing pipeline is half-built.** `processing/mod.rs` is empty, `Pipeline` and `StepKind` exist in domain but have no execution path. Since you're dropping preprocessing, this entire subtree goes away — which is the right call.

**5. Tight coupling in the 3D view.** `Viewport3D.svelte` (467 lines) manages Three.js scene graph, anatomy layers, voxels, optodes, and channel selection all in one component. This is your ImGui instincts showing — everything in one render loop. It needs decomposition.

---

## Rewrite Scope: What You're Keeping vs. Dropping

### Keep
- SNIRF parsing (`io/snirf_parser.rs`) — this is working and complex
- SNIRF export (`io/snirf_exporter.rs`)
- Core domain types: `SNIRF`, `Channel`, `Optode`, `Probe`, `Events`
- Anatomy import + mesh loading (`io/anatomy_importer.rs`, `io/mesh_importer.rs`)
- 3D scene concepts: `Transform`, `SceneObject`, `Mesh`
- Voxel volume handling

### Drop
- `processing/` module entirely
- `Pipeline`, `StepKind`, `BandpassParams`, `PruningParams`
- All pipeline-related commands and frontend components
- `PipelineEditor.svelte`, `PipelineOrder.svelte`, `ParameterEditor.svelte`, `AvailableSteps.svelte`
- `stepDefinitions.js`, `stores/pipeline.js`

### Redesign
- State architecture (Rust side)
- Command layer (thinner, service-oriented)
- Frontend state management (unified store pattern)
- 3D viewport (decomposed components)
- Layout system

---

## New Architecture

### High-Level System Flow

```mermaid
flowchart TB
    subgraph Frontend["Frontend (Svelte 5)"]
        direction TB
        UI["UI Components"]
        Stores["Svelte Stores<br/>(reactive cache)"]
        API["API Layer<br/>(invoke wrappers)"]
        
        UI -->|reads| Stores
        UI -->|user actions| API
        API -->|updates| Stores
    end
    
    subgraph Bridge["Tauri IPC Bridge"]
        Commands["Commands<br/>(thin wrappers)"]
        Events["Events<br/>(backend → frontend)"]
    end
    
    subgraph Backend["Backend (Rust)"]
        direction TB
        Services["Services<br/>(business logic)"]
        State["AppState<br/>(segmented RwLocks)"]
        Domain["Domain Types"]
        IO["I/O Layer<br/>(parsers, exporters)"]
        
        Services --> State
        Services --> IO
        Services --> Domain
        IO --> Domain
    end
    
    API -->|invoke| Commands
    Commands -->|delegates to| Services
    Events -->|emit| Stores
    Services -->|emit via AppHandle| Events

    style Frontend fill:#1a1a2e,stroke:#4a9eff,color:#e0e0e0
    style Bridge fill:#16213e,stroke:#e94560,color:#e0e0e0
    style Backend fill:#0f3460,stroke:#53bf6b,color:#e0e0e0
```

---

### Rust Backend Architecture

```mermaid
flowchart LR
    subgraph Commands["commands/"]
        cmd_snirf["snirf.rs<br/>load, export, summary"]
        cmd_ts["timeseries.rs<br/>get_timeseries_data"]
        cmd_probe["probe.rs<br/>get_probe_layout<br/>set_selected_channels"]
        cmd_scene["scene.rs<br/>get/set anatomy<br/>get/set optodes"]
        cmd_anatomy["anatomy.rs<br/>load_mri"]
        cmd_voxel["voxel.rs<br/>list, info, slice"]
    end

    subgraph Services["services/"]
        svc_snirf["snirf_service.rs<br/>parse → store → derive"]
        svc_scene["scene_service.rs<br/>anatomy + optode management"]
        svc_voxel["voxel_service.rs<br/>slice extraction"]
    end

    subgraph State["state.rs"]
        s_snirf["SnirfState<br/>RwLock&lt;Option&lt;SNIRF&gt;&gt;"]
        s_scene["SceneState<br/>RwLock&lt;SceneData&gt;"]
        s_selection["SelectionState<br/>RwLock&lt;Selection&gt;"]
    end

    subgraph IO["io/"]
        io_snirf["snirf_parser.rs"]
        io_export["snirf_exporter.rs"]
        io_anatomy["anatomy_importer.rs"]
        io_mesh["mesh_importer.rs"]
    end

    subgraph Domain["domain/"]
        d_snirf["snirf.rs"]
        d_mesh["mesh.rs"]
        d_scene["scene.rs"]
        d_probe["probe.rs"]
        d_voxel["voxel.rs"]
        d_anatomy["anatomy.rs"]
    end

    cmd_snirf --> svc_snirf
    cmd_scene --> svc_scene
    cmd_voxel --> svc_voxel
    cmd_ts --> s_snirf
    cmd_probe --> s_snirf
    cmd_probe --> s_selection
    cmd_anatomy --> svc_scene

    svc_snirf --> io_snirf
    svc_snirf --> io_export
    svc_snirf --> s_snirf
    svc_snirf --> s_scene
    svc_scene --> io_anatomy
    svc_scene --> io_mesh
    svc_scene --> s_scene
    svc_voxel --> s_scene

    io_snirf --> d_snirf
    io_anatomy --> d_anatomy
    io_mesh --> d_mesh

    style Commands fill:#2d1b69,stroke:#7c5cbf,color:#e0e0e0
    style Services fill:#1b3a4b,stroke:#4ecdc4,color:#e0e0e0
    style State fill:#3d1f1f,stroke:#ff6b6b,color:#e0e0e0
    style IO fill:#1f3d1f,stroke:#6bff6b,color:#e0e0e0
    style Domain fill:#3d3d1f,stroke:#ffff6b,color:#e0e0e0
```

---

### Segmented State (replacing the God Session)

```mermaid
classDiagram
    class AppState {
        +snirf: SnirfState
        +scene: SceneState
        +selection: SelectionState
    }

    class SnirfState {
        +data: RwLock~Option~SNIRF~~
        +file_path: RwLock~Option~PathBuf~~
    }
    
    class SceneState {
        +anatomy: RwLock~Option~SubjectAnatomy~~
        +optode_layout: RwLock~Option~OptodeLayout~~
        +voxel_volumes: RwLock~HashMap~String，VoxelVolume~~
    }
    
    class SelectionState {
        +selected_channels: RwLock~Vec~usize~~
        +cursor_timepoint: RwLock~f64~
    }

    AppState *-- SnirfState
    AppState *-- SceneState
    AppState *-- SelectionState

    note for AppState "Each sub-state has its own RwLock.\nCommands only lock what they need.\nNo more single-lock bottleneck."
```

#### Why this matters:

With a single `RwLock<Session>`, a command reading channel data blocks a command writing selected channels. With segmented locks, `get_timeseries_data` locks `SnirfState` while `set_selected_channels` locks `SelectionState` — no contention.

```rust
// NEW: segmented state
pub struct AppState {
    pub snirf: SnirfState,
    pub scene: SceneState,
    pub selection: SelectionState,
}

pub struct SnirfState {
    pub data: RwLock<Option<SNIRF>>,
    pub file_path: RwLock<Option<PathBuf>>,
}

pub struct SceneState {
    pub anatomy: RwLock<Option<SubjectAnatomy>>,
    pub optode_layout: RwLock<Option<OptodeLayout>>,
    pub voxel_volumes: RwLock<HashMap<String, VoxelVolume>>,
}

pub struct SelectionState {
    pub selected_channels: RwLock<Vec<usize>>,
    pub cursor_timepoint: RwLock<f64>,
}
```

---

### Services Layer (New)

The key architectural change. Commands become thin, services hold logic:

```rust
// commands/snirf.rs — THIN
#[tauri::command]
pub fn load_snirf(
    path: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<SnirfSummary, String> {
    services::snirf::load_and_store(&path, &state, &app)
}

// services/snirf.rs — LOGIC LIVES HERE
pub fn load_and_store(
    path: &str,
    state: &AppState,
    app: &AppHandle,
) -> Result<SnirfSummary, String> {
    let snirf = io::snirf_parser::parse_snirf(path)?;
    let summary = SnirfSummary::from(&snirf);
    let optode_layout = OptodeLayout::from_snirf(&snirf);

    // Lock only what we need, separately
    {
        let mut data = state.snirf.data.write().map_err(|e| e.to_string())?;
        *data = Some(snirf);
    }
    {
        let mut layout = state.scene.optode_layout.write().map_err(|e| e.to_string())?;
        *layout = Some(optode_layout);
    }

    app.emit("snirf-loaded", &summary).map_err(|e| e.to_string())?;
    Ok(summary)
}
```

This separation means:
- Services are testable without Tauri (pass `&AppState` directly)
- Commands are trivially auditable (just delegation)
- Logic can be shared between commands without pub-use gymnastics

---

### Data Flow: Loading a SNIRF File

```mermaid
sequenceDiagram
    actor User
    participant Menu as MenuBar.svelte
    participant API as api/snirf.ts
    participant Cmd as commands/snirf.rs
    participant Svc as services/snirf.rs
    participant Parser as io/snirf_parser.rs
    participant State as AppState
    participant Event as Tauri Events
    participant Stores as Svelte Stores

    User->>Menu: Click "Open SNIRF"
    Menu->>Menu: Tauri file dialog
    Menu->>API: loadSnirf(path)
    API->>Cmd: invoke("load_snirf", {path})
    Cmd->>Svc: load_and_store(path, state, app)
    Svc->>Parser: parse_snirf(path)
    Parser-->>Svc: SNIRF struct
    Svc->>Svc: derive OptodeLayout
    Svc->>State: write snirf data
    Svc->>State: write optode layout
    Svc->>Event: emit("snirf-loaded", summary)
    Svc-->>Cmd: Ok(SnirfSummary)
    Cmd-->>API: SnirfSummary
    API->>Stores: snirfStore.set(summary)
    Event-->>Stores: listener updates stores
    Stores-->>Menu: reactive UI update
    
    Note over Stores: All components reading<br/>snirfStore react automatically
```

---

### Frontend Architecture

```mermaid
flowchart TB
    subgraph Stores["stores/ (single source of truth)"]
        snirfStore["snirfStore<br/>summary, loading state"]
        selectionStore["selectionStore<br/>selected channels, cursor"]
        sceneStore["sceneStore<br/>anatomy state, optode state<br/>voxel volume state"]
    end

    subgraph API["api/ (invoke wrappers)"]
        apiSnirf["snirf.ts<br/>loadSnirf(), exportSnirf()"]
        apiTimeseries["timeseries.ts<br/>getTimeseriesData()"]
        apiProbe["probe.ts<br/>getProbeLayout()"]
        apiScene["scene.ts<br/>getAnatomyGeometry()<br/>loadMri()"]
        apiVoxel["voxel.ts<br/>listVolumes(), getSlice()"]
    end

    subgraph Layout["Layout Components"]
        App["App.svelte<br/>(shell + panels)"]
        MenuBar["MenuBar.svelte"]
        StatusBar["StatusBar.svelte"]
    end

    subgraph Panels["Feature Panels"]
        DataPlotter["DataPlotter.svelte<br/>(ECharts timeseries)"]
        ChannelSelector["ChannelSelector.svelte<br/>(SVG probe map)"]
        InfoPanel["InfoPanel.svelte<br/>(metadata cards)"]
    end

    subgraph Scene3D["3D Viewport (decomposed)"]
        Viewport["Viewport3D.svelte<br/>(Three.js canvas + camera)"]
        AnatomyRenderer["AnatomyRenderer.svelte<br/>(skull/csf/gm/wm meshes)"]
        OptodeRenderer["OptodeRenderer.svelte<br/>(sources/detectors/channels)"]
        VoxelRenderer["VoxelRenderer.svelte<br/>(instanced mesh slices)"]
        SceneInspector["SceneInspector.svelte<br/>(transform/opacity controls)"]
    end

    Stores --> Layout
    Stores --> Panels
    Stores --> Scene3D
    API --> Stores

    App --> MenuBar
    App --> StatusBar
    App --> DataPlotter
    App --> ChannelSelector
    App --> InfoPanel
    App --> Viewport

    Viewport --> AnatomyRenderer
    Viewport --> OptodeRenderer
    Viewport --> VoxelRenderer

    style Stores fill:#1a2744,stroke:#4a9eff,color:#e0e0e0
    style API fill:#1a3a2a,stroke:#4ecdc4,color:#e0e0e0
    style Layout fill:#2a1a2a,stroke:#c080c0,color:#e0e0e0
    style Panels fill:#2a2a1a,stroke:#c0c080,color:#e0e0e0
    style Scene3D fill:#1a2a2a,stroke:#80c0c0,color:#e0e0e0
```

---

### The API Layer Pattern

Instead of calling `invoke` directly from components (which scatters IPC knowledge everywhere), wrap every command in a typed function:

```typescript
// src/lib/api/snirf.ts
import { invoke } from '@tauri-apps/api/core';
import { snirfStore } from '../stores/snirf';

export interface SnirfSummary {
  filename: string;
  channels: number;
  sources: number;
  detectors: number;
  timepoints: number;
  sampling_rate: number;
  duration: number;
  hbo_wavelength: number;
  hbr_wavelength: number;
  events: { name: string; count: number }[];
  aux_count: number;
}

export async function loadSnirf(path: string): Promise<SnirfSummary> {
  const summary = await invoke<SnirfSummary>('load_snirf', { path });
  snirfStore.set(summary);
  return summary;
}

export async function exportSnirf(path: string): Promise<void> {
  await invoke('export_snirf', { path });
}

export async function getSnirfSummary(): Promise<SnirfSummary | null> {
  return invoke<SnirfSummary | null>('get_snirf_summary');
}
```

Components never import `invoke` directly. They import from `api/`:

```svelte
<script>
  // BEFORE (scattered invoke calls)
  import { invoke } from '@tauri-apps/api/core';
  const data = await invoke('get_timeseries_data', { channelIds });
  
  // AFTER (typed, centralized)
  import { getTimeseriesData } from '$lib/api/timeseries';
  const data = await getTimeseriesData(channelIds);
</script>
```

---

### 3D Viewport Decomposition

Your current `Viewport3D.svelte` (467 lines) does too much. Break it apart:

```mermaid
flowchart TB
    Viewport["Viewport3D.svelte<br/>───────────────────<br/>• Owns: canvas, renderer, camera, controls<br/>• Creates: THREE.Scene<br/>• Provides: scene context to children<br/>• Handles: resize, animation loop"]
    
    Viewport --> AR["AnatomyRenderer<br/>───────────────────<br/>• Receives: scene from parent<br/>• Owns: skull/csf/gm/wm meshes<br/>• Reacts to: anatomyLayerStates store<br/>• Handles: opacity, visibility, transform"]
    
    Viewport --> OR["OptodeRenderer<br/>───────────────────<br/>• Receives: scene from parent<br/>• Owns: optode spheres, channel lines<br/>• Reacts to: optodeState, selectionStore<br/>• Handles: spread, radius, highlights"]
    
    Viewport --> VR["VoxelRenderer<br/>───────────────────<br/>• Receives: scene from parent<br/>• Owns: instanced mesh slices<br/>• Reacts to: voxelVolumeStates store<br/>• Handles: axis, slice index, labels"]

    style Viewport fill:#1a2744,stroke:#4a9eff,color:#e0e0e0
    style AR fill:#2d1b1b,stroke:#ff6b6b,color:#e0e0e0
    style OR fill:#1b2d1b,stroke:#6bff6b,color:#e0e0e0
    style VR fill:#1b1b2d,stroke:#6b6bff,color:#e0e0e0
```

Use Svelte's context API to share the Three.js scene:

```svelte
<!-- Viewport3D.svelte -->
<script>
  import { setContext } from 'svelte';
  const scene = new THREE.Scene();
  setContext('three-scene', scene);
</script>

<canvas bind:this={canvas}></canvas>
<AnatomyRenderer />
<OptodeRenderer />
<VoxelRenderer />
```

```svelte
<!-- AnatomyRenderer.svelte -->
<script>
  import { getContext } from 'svelte';
  const scene = getContext('three-scene');
  // Only manages anatomy meshes — nothing else
</script>
```

---

### New Project Structure

```
NIRWizard/
├── src/                              # Svelte frontend
│   ├── App.svelte                    # Shell layout only
│   ├── main.ts                       # Mount point (switch to TS)
│   ├── app.css                       # Global theme vars
│   └── lib/
│       ├── api/                      # ← NEW: typed invoke wrappers
│       │   ├── snirf.ts
│       │   ├── timeseries.ts
│       │   ├── probe.ts
│       │   ├── scene.ts
│       │   └── voxel.ts
│       ├── stores/                   # Unified reactive state
│       │   ├── snirf.ts
│       │   ├── selection.ts
│       │   └── scene.ts
│       ├── components/               # UI components
│       │   ├── layout/
│       │   │   ├── MenuBar.svelte
│       │   │   ├── StatusBar.svelte
│       │   │   └── PanelLayout.svelte
│       │   ├── data/
│       │   │   ├── DataPlotter.svelte
│       │   │   ├── ChannelSelector.svelte
│       │   │   └── InfoPanel.svelte
│       │   └── scene/
│       │       ├── Viewport3D.svelte
│       │       ├── AnatomyRenderer.svelte
│       │       ├── OptodeRenderer.svelte
│       │       ├── VoxelRenderer.svelte
│       │       └── SceneInspector.svelte
│       └── utils/
│           └── colormap.ts
│
├── src-tauri/
│   └── src/
│       ├── main.rs                   # Entry point (thin)
│       ├── state.rs                  # Segmented AppState
│       ├── commands/                 # Thin command layer
│       │   ├── mod.rs
│       │   ├── snirf.rs
│       │   ├── timeseries.rs
│       │   ├── probe.rs
│       │   ├── scene.rs
│       │   ├── anatomy.rs
│       │   └── voxel.rs
│       ├── services/                 # ← NEW: business logic
│       │   ├── mod.rs
│       │   ├── snirf_service.rs
│       │   ├── scene_service.rs
│       │   └── voxel_service.rs
│       ├── domain/                   # Pure data types
│       │   ├── mod.rs
│       │   ├── snirf.rs
│       │   ├── mesh.rs
│       │   ├── scene.rs
│       │   ├── probe.rs
│       │   ├── anatomy.rs
│       │   └── voxel.rs
│       └── io/                       # File I/O
│           ├── mod.rs
│           ├── snirf_parser.rs
│           ├── snirf_exporter.rs
│           ├── anatomy_importer.rs
│           └── mesh_importer.rs
```

---

### Event System (backend → frontend)

```mermaid
flowchart LR
    subgraph Rust Events
        E1["snirf-loaded<br/>SnirfSummary"]
        E2["channels-selected<br/>Vec&lt;usize&gt;"]
        E3["anatomy-loaded<br/>layer names"]
        E4["voxel-loaded<br/>VoxelVolumeInfo"]
        E5["cursor-moved<br/>f64 timepoint"]
    end

    subgraph Svelte Listeners
        L1["snirfStore<br/>updates summary"]
        L2["selectionStore<br/>updates channels"]
        L3["sceneStore<br/>updates anatomy"]
        L4["sceneStore<br/>updates voxels"]
        L5["selectionStore<br/>updates cursor"]
    end

    E1 --> L1
    E2 --> L2
    E3 --> L3
    E4 --> L4
    E5 --> L5

    style Rust Events fill:#0f3460,stroke:#53bf6b,color:#e0e0e0
    style Svelte Listeners fill:#1a1a2e,stroke:#4a9eff,color:#e0e0e0
```

Set up listeners once at app startup, writing to stores. Components subscribe to stores — they never listen to raw events.

```typescript
// src/lib/api/events.ts — initialized once in App.svelte onMount
import { listen } from '@tauri-apps/api/event';
import { snirfStore } from '../stores/snirf';
import { selectionStore } from '../stores/selection';
import { sceneStore } from '../stores/scene';

export async function initEventListeners(): Promise<() => void> {
  const unlisteners = await Promise.all([
    listen('snirf-loaded', (e) => snirfStore.set(e.payload)),
    listen('channels-selected', (e) => selectionStore.setChannels(e.payload)),
    listen('anatomy-loaded', (e) => sceneStore.setAnatomy(e.payload)),
    listen('voxel-loaded', (e) => sceneStore.addVoxelVolume(e.payload)),
  ]);
  
  return () => unlisteners.forEach(fn => fn());
}
```

---

## Migration Strategy

### Phase 1: Foundation (do first)
1. Switch frontend to TypeScript
2. Create `api/` layer — wrap all existing `invoke` calls
3. Create unified stores
4. Restructure Rust: add `services/`, move logic out of commands
5. Implement segmented `AppState`

### Phase 2: Cleanup
1. Delete all `processing/` and `pipeline/` code (both sides)
2. Delete pipeline Svelte components and stores
3. Remove `StepKind`, `PipelineStep`, `Pipeline` from domain

### Phase 3: Decompose
1. Break `Viewport3D.svelte` into sub-renderers
2. Break `DataPlotter.svelte` into smaller chart-focused components
3. Extract panel layout logic from `App.svelte` into `PanelLayout.svelte`

### Phase 4: Polish
1. Proper error handling (Rust error types, not `.to_string()` everywhere)
2. Loading states in stores
3. TypeScript interfaces matching Rust serde output
