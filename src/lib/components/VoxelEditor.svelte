<script>
  import { createEventDispatcher } from 'svelte';
  import { labelToCss } from '../utils/colormap.js';

  /** @type {import('../utils/colormap.js')} */
  export let name   = '';   // volume name
  export let info   = null; // VoxelVolumeInfo from backend
  export let state  = null; // { axis, sliceIndex, visibleLabels: Set, visible }

  const dispatch = createEventDispatcher();

  $: maxLabel = info ? Math.max(...info.labels_present, 1) : 1;
  $: dimForAxis = info
    ? (state.axis === 'x' ? info.dims[0] : state.axis === 'y' ? info.dims[1] : info.dims[2])
    : 1;

  function emit(patch) {
    dispatch('change', { ...state, ...patch });
  }

  function setAxis(axis) {
    const maxIdx = info
      ? (axis === 'x' ? info.dims[0] : axis === 'y' ? info.dims[1] : info.dims[2]) - 1
      : 0;
    emit({ axis, sliceIndex: Math.min(state.sliceIndex, maxIdx) });
  }

  function onSlice(e) {
    emit({ sliceIndex: parseInt(e.target.value) });
  }

  function toggleLabel(label) {
    const next = new Set(state.visibleLabels);
    if (next.has(label)) next.delete(label); else next.add(label);
    emit({ visibleLabels: next });
  }

  function onVisible(e) {
    emit({ visible: e.target.checked });
  }
</script>

{#if info && state}
  <div class="voxel-editor">

    <!-- Visibility + name -->
    <div class="prop-row">
      <input type="checkbox" checked={state.visible} on:change={onVisible} />
      <span class="vol-name">{name}</span>
    </div>

    <!-- Axis selector -->
    <div class="prop-row">
      <span class="prop-label">Axis</span>
      <div class="btn-group">
        {#each ['x','y','z'] as ax}
          <button
            class="axis-btn"
            class:active={state.axis === ax}
            on:click={() => setAxis(ax)}
          >{ax.toUpperCase()}</button>
        {/each}
      </div>
    </div>

    <!-- Slice slider -->
    <div class="prop-row">
      <span class="prop-label">Slice</span>
      <input
        type="range"
        min="0"
        max={dimForAxis - 1}
        step="1"
        value={state.sliceIndex}
        on:input={onSlice}
        class="slider"
      />
      <span class="prop-value">{state.sliceIndex}</span>
    </div>

    <!-- Label visibility -->
    <div class="label-list">
      {#each info.labels_present as label}
        <label class="label-row">
          <input
            type="checkbox"
            checked={state.visibleLabels.has(label)}
            on:change={() => toggleLabel(label)}
          />
          <span
            class="swatch"
            style="background:{labelToCss(label, maxLabel)}"
          ></span>
          <span class="label-name">
            {info.label_names[label] ?? `Label ${label}`}
          </span>
        </label>
      {/each}
    </div>

  </div>
{/if}

<style>
  .voxel-editor {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .prop-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .vol-name {
    font-size: 10px;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .prop-label {
    font-size: 9px;
    color: var(--text-muted);
    width: 30px;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .btn-group {
    display: flex;
    gap: 2px;
  }

  .axis-btn {
    padding: 2px 7px;
    font-size: 9px;
    font-family: monospace;
    font-weight: 600;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    border-radius: 3px;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .axis-btn.active {
    background: var(--accent-green);
    color: #000;
    border-color: var(--accent-green);
  }

  .slider {
    flex: 1;
    min-width: 0;
    accent-color: var(--accent-green);
  }

  .prop-value {
    font-size: 10px;
    color: var(--text-primary);
    font-family: monospace;
    width: 28px;
    text-align: right;
    flex-shrink: 0;
  }

  .label-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-top: 2px;
  }

  .label-row {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }

  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    border: 1px solid rgba(255,255,255,0.15);
    flex-shrink: 0;
  }

  .label-name {
    font-size: 9px;
    color: var(--text-secondary);
  }
</style>
