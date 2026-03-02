# NIRWizard — Preprocessing Pipeline Implementation Plan

---

## Phase 1 — Rust Domain & Store Foundation
> Goal: data structures exist, nothing runs yet

### 1.1 Create `domain/pipeline.rs`
- [x] Add `uuid` to `Cargo.toml`
- [x] Define `BandpassParams` struct with `low_cutoff: f32`, `high_cutoff: f32`, `order: usize`
- [x] Define `PruningParams` struct with `pruning_method: PruningMethod`, `threshold: f32`
- [x] Define `PruningMethod` enum with `Sci`, `Psp`, `Snr` variants
- [x] Define `StepKind` enum with `Bandpass(BandpassParams)` and `Pruning(PruningParams)` variants
- [x] Add `#[derive(Serialize, Deserialize, Clone, Debug)]` to all param types
- [x] Define `PipelineStep` struct with fields: `id: String`, `step_kind: StepKind`, `label: String`, `enabled: bool` (cached_output deferred to Phase 5)
- [x] Implement `PipelineStep::new(kind: StepKind) -> Self` (generates UUID, derives label)
- [x] Implement `PipelineStep::invalidate(&mut self)` (stub — will clear cache in Phase 5)
- [x] Define `Pipeline` struct with `steps: Vec<PipelineStep>`, `active_step_index: Option<usize>`
- [x] Implement `Pipeline::push(&mut self, kind: StepKind)`
- [x] Implement `Pipeline::remove(&mut self, index: usize)`
- [x] Implement `Pipeline::invalidate_from(&mut self, from: usize)`
- [x] Add helper `fn label_for(kind: &StepKind) -> String` match

### 1.2 Update `state.rs`
- [x] Add `pipeline: Pipeline` field to `Session`
- [x] Update `Default` impl to include `pipeline: Pipeline::default()`

### 1.3 Wire module
- [x] Add `pub mod pipeline;` to `domain/mod.rs`
- [x] Run `cargo check` from `src-tauri/` — must compile clean

---

## Phase 2 — Rust Commands (no algorithms yet)
> Goal: frontend can talk to backend about the pipeline

### 2.1 Create `commands/pipeline.rs`
- [x] Define `PipelineStepSummary` serializable struct with `id: String`, `label: String`, `enabled: bool`, `is_active: bool`, `has_cache: bool`
- [x] Define `PipelineSummary` serializable struct with `steps: Vec<PipelineStepSummary>`, `active_index: Option<usize>`
- [x] Implement `PipelineSummary::from(pipeline: &Pipeline) -> Self`
- [x] Implement `add_pipeline_step(kind: StepKind, state) -> Result<PipelineSummary, String>`
- [x] Implement `remove_pipeline_step(index: usize, state) -> Result<PipelineSummary, String>`
- [x] Implement `move_pipeline_step(index: usize, direction: i32, state) -> Result<PipelineSummary, String>`
- [x] Implement `get_pipeline_summary(state) -> Result<PipelineSummary, String>`

### 2.2 Register in `main.rs`
- [x] Add `pub mod pipeline;` to `commands/mod.rs`
- [x] Register all four commands in the `invoke_handler![]` macro in `main.rs`
- [x] Run `cargo check` — must compile clean

---

## Phase 3 — Svelte Pipeline Editor UI
> Goal: three-column editor renders with local JS state only

### 3.1 Create `src/lib/pipeline/stepDefinitions.js`
- [ ] Create the file and export `STEP_DEFINITIONS` object
- [ ] Add `bandpass` entry with `id`, `label`, `description`, `category: "filter"`, and full `params` schema (`low_hz`, `high_hz`, `order`)
- [ ] Add `pruning` entry with `id`, `label`, `description`, `category: "quality"`, and `params` schema (`method`, `threshold`)
- [ ] Verify each param has `type`, `label`, and `default` (plus `min`/`max`/`step` for numbers, `options` for selects)

### 3.2 Create `src/lib/stores/pipeline.js`
- [ ] Create writable store with initial shape `{ steps: [], selectedId: null }`
- [ ] Implement `addStep(definitionId)` — clones defaults from definition into fresh params object, appends with `crypto.randomUUID()` as `instanceId`
- [ ] Implement `removeStep(instanceId)` — filters out step, clears `selectedId` if it was that step
- [ ] Implement `moveStep(instanceId, direction)` — swaps with neighbour, bounds-checked
- [ ] Implement `updateParam(instanceId, key, value)` — immutable update of params
- [ ] Implement `selectStep(instanceId)` — sets `selectedId`
- [ ] Implement `toggleEnabled(instanceId)` — flips `enabled` boolean
- [ ] Implement `serialize(state)` — returns JSON string with version envelope
- [ ] Implement `deserialize(json)` — reconstructs steps with fresh `instanceId` UUIDs
- [ ] Export derived `selectedStep` store (resolves `selectedId` → `{ step, def }`)

### 3.3 Create `AvailableSteps.svelte`
- [ ] Import `STEP_DEFINITIONS` and `pipeline` store
- [ ] Group definitions by `category` into a local object
- [ ] Render each category as a labelled section
- [ ] Render each step as a clickable row that calls `pipeline.addStep(id)`
- [ ] Style: hover highlight, `+` icon prefix

### 3.4 Create `PipelineOrder.svelte`
- [ ] Import `pipeline` and `selectedStep` stores and `STEP_DEFINITIONS`
- [ ] Render empty state message when `$pipeline.steps.length === 0`
- [ ] Render each step as a row with: index number, label, up button, down button, enable toggle, remove button
- [ ] Clicking the row calls `pipeline.selectStep(instanceId)`
- [ ] Apply `.selected` class when `$pipeline.selectedId === step.instanceId`
- [ ] Apply `.disabled` class when `!step.enabled`
- [ ] `stopPropagation` on all control buttons so they don't trigger row selection

### 3.5 Create `ParameterEditor.svelte`
- [ ] Import `selectedStep` derived store and `pipeline` store
- [ ] Render empty state message when `$selectedStep` is null
- [ ] Render step title and description when a step is selected
- [ ] Render "No configurable parameters" message when `def.params` is empty
- [ ] Iterate `Object.entries($selectedStep.def.params)` and render the correct input per type:
  - [ ] `select` → `<select>` with options
  - [ ] `number` → `<input type="number">` with min/max/step
  - [ ] `integer` → `<input type="number" step="1">` with parseInt on change
  - [ ] `boolean` → `<input type="checkbox">`
- [ ] All inputs call `pipeline.updateParam(instanceId, key, value)` on change

### 3.6 Create `PipelineEditor.svelte`
- [ ] Import all three sub-components
- [ ] Implement three-column flex layout with vertical dividers
- [ ] Add toolbar row with title, Load (stub), Save (stub), Run (stub) buttons
- [ ] Confirm the three columns render side by side

### 3.7 Mount the editor in `App.svelte`
- [ ] Import `PipelineEditor`
- [ ] Add it to the layout (new panel, modal, or tab — your choice)
- [ ] Confirm it renders and local store interactions (add/remove/select/param edit) all work visually

---

## Phase 4 — Connect Store to Backend
> Goal: UI actions persist into Rust state

### 4.1 Add `invoke` calls to `pipeline.js`
- [ ] Update `addStep` to call `invoke("add_pipeline_step", { kind: { kind: definitionId, params } })` after local update — replace store state with returned `PipelineSummary`
- [ ] Update `removeStep` to call `invoke("remove_pipeline_step", { index })` — use returned summary to sync
- [ ] Update `moveStep` to call `invoke("move_pipeline_step", { index, direction })` — use returned summary to sync
- [ ] Decide on authoritative source: let Rust `PipelineSummary` be the source of truth and rebuild frontend `steps` from it on every mutating call

### 4.2 Test the round-trip
- [ ] Add a Bandpass step — confirm it appears in Rust logs
- [ ] Remove it — confirm Rust state is empty again
- [ ] Move a step up/down — confirm Rust index order matches UI
- [ ] Param edits are frontend-only until Run — confirm that is intentional and acceptable

---

## Phase 5 — First Real Algorithm: Bandpass Filter
> Goal: Run button produces real processed data

### 5.1 Create `processing/filter.rs`
- [ ] Add function signature `pub fn apply_bandpass(data: ChannelData, params: &BandpassParams) -> Result<ChannelData, String>`
- [ ] Compute or hardcode second-order section (biquad) coefficients for a 3rd-order Butterworth at 0.01–0.5 Hz (compute offline in Python/scipy, paste as constants to start)
- [ ] Implement forward IIR pass over each channel's `hbo` and `hbr` vectors
- [ ] Implement reverse IIR pass (zero-phase filtering)
- [ ] Return new `ChannelData` with filtered values, same time vector

### 5.2 Implement `processing/mod.rs` dispatch
- [ ] Add `pub mod filter;`
- [ ] Implement `pub fn apply_step(kind: &StepKind, data: ChannelData) -> Result<ChannelData, String>`
- [ ] Add match arm for `StepKind::BandpassFilter(p)` → `filter::apply_bandpass(data, p)`

### 5.3 Implement `Pipeline::get_or_compute` in `domain/pipeline.rs`
- [ ] Find the nearest cached ancestor by scanning backwards from `target_index`
- [ ] Clone either the ancestor's `cached_output` or the original `ChannelData` as the starting point
- [ ] Loop from start to `target_index`, skipping disabled steps (pass-through), calling `apply_step` for enabled steps
- [ ] Store each result in `step.cached_output`
- [ ] Return reference to `steps[target_index].cached_output`

### 5.4 Add `run_pipeline` command to `commands/pipeline.rs`
- [ ] Accept `steps: Vec<StepSpec>` where `StepSpec = { kind: String, params: serde_json::Value }`
- [ ] Rebuild `session.pipeline` from the incoming specs via `StepKind::from_spec`
- [ ] Call `get_or_compute` on the last step index
- [ ] Return `TimeseriesPayload` of the result
- [ ] Emit `"pipeline-complete"` event via `app.emit`
- [ ] Register `run_pipeline` in `main.rs`

### 5.5 Wire Run button in `PipelineEditor.svelte`
- [ ] Replace stub with `invoke("run_pipeline", { steps: ... })` call
- [ ] Serialize current `$pipeline.steps` into the `StepSpec` format Rust expects
- [ ] On success, emit or store the returned timeseries so `DataPlotter` refreshes
- [ ] Add basic loading state (disable Run button while running)

### 5.6 Validate output
- [ ] Load the FRESH example SNIRF from `examples/`
- [ ] Add a Bandpass step, hit Run
- [ ] Confirm the DataPlotter shows visibly smoother signal
- [ ] Confirm the original raw data is untouched (toggle to step 0)

---

## Phase 6 — Save / Load Pipeline Files
> Goal: `.nirpipeline` files round-trip correctly

### 6.1 Finalise serialization format
- [ ] Confirm `serialize()` in the store wraps steps in `{ version: "1.0", created: <ISO date>, steps: [...] }`
- [ ] Confirm `deserialize()` validates the `version` field and throws a readable error on mismatch

### 6.2 Implement Save in `PipelineEditor.svelte`
- [ ] Import `save` from `@tauri-apps/plugin-dialog` and `writeTextFile` from `@tauri-apps/plugin-fs`
- [ ] Wire Save button: open save dialog filtered to `.nirpipeline`, write `pipeline.serialize($pipeline)`
- [ ] Confirm the written file is valid JSON you can open in a text editor

### 6.3 Implement Load in `PipelineEditor.svelte`
- [ ] Import `open` from dialog plugin and `readTextFile` from fs plugin
- [ ] Wire Load button: open file dialog, read file, call `pipeline.deserialize(json)`
- [ ] After deserialize, call `invoke("set_pipeline", { steps })` to sync Rust state

### 6.4 Add `set_pipeline` Rust command
- [ ] Accept `steps: Vec<StepSpec>`, rebuild `session.pipeline` from scratch
- [ ] Invalidate all caches on rebuild
- [ ] Return `PipelineSummary`
- [ ] Register in `main.rs`

### 6.5 End-to-end test
- [ ] Build a pipeline, save it, clear the pipeline, load it back
- [ ] Confirm loaded pipeline matches the saved one exactly
- [ ] Run the loaded pipeline and confirm output matches pre-save output

---

## Phase 7 — Second Algorithm: TDDR Motion Correction
> Goal: exercise the full pipeline with a second real step

### 7.1 Extend the Rust domain
- [ ] Add `MotionCorrection(MotionParams)` variant to `StepKind`
- [ ] Define `MotionParams { method: MotionMethod }` and `MotionMethod { Tddr }` enum
- [ ] Add match arm in `apply_step` dispatch

### 7.2 Extend the JS definitions
- [ ] Add `tddr` entry to `STEP_DEFINITIONS` with empty `params` object
- [ ] Confirm `ParameterEditor` renders the "No configurable parameters" message for it

### 7.3 Create `processing/motion.rs`
- [ ] Implement `pub fn apply_tddr(data: ChannelData) -> Result<ChannelData, String>`
- [ ] Step 1: compute temporal derivative of signal
- [ ] Step 2: estimate the distribution of the derivative (mean, std)
- [ ] Step 3: identify outlier timepoints exceeding threshold (typically 4.685× MAD)
- [ ] Step 4: interpolate over identified outlier spans
- [ ] Apply independently to each channel's `hbo` and `hbr`
- [ ] Return corrected `ChannelData`

### 7.4 Wire and test
- [ ] Add TDDR match arm in `apply_step`
- [ ] Add `tddr` to `STEP_DEFINITIONS` JS
- [ ] Build a two-step pipeline: TDDR → Bandpass
- [ ] Run on the example SNIRF and visually confirm motion spikes are reduced
- [ ] Save the pipeline as `default.nirpipeline` for future use

---

## Milestone Summary

| Milestone | Completed when… |
|---|---|
| Phase 1 done | `cargo check` passes with pipeline domain types |
| Phase 2 done | All four pipeline commands registered and reachable |
| Phase 3 done | Three-column UI renders and local state works |
| Phase 4 done | Add/remove/move steps persist in Rust |
| Phase 5 done | Run button filters real data, DataPlotter updates |
| Phase 6 done | `.nirpipeline` files save and load correctly |
| Phase 7 done | Two-step TDDR → Bandpass pipeline runs on real data |
