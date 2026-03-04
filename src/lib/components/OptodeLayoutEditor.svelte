<script>
  import { createEventDispatcher } from 'svelte';
  import TransformEditor from './TransformEditor.svelte';

  export let state = null;

  const dispatch = createEventDispatcher();

  function onTransformChange(e) {
    dispatch('change', { ...state, transform: e.detail });
  }

  function onSpread(e) {
    const spread_factor = parseFloat(e.target.value);
    dispatch('change', { ...state, settings: { ...state.settings, spread_factor } });
  }

  function onRadius(e) {
    const optode_radius = parseFloat(e.target.value);
    dispatch('change', { ...state, settings: { ...state.settings, optode_radius } });
  }

  function onVisible(e) {
    dispatch('change', { ...state, visible: e.target.checked });
  }
</script>

{#if state}
  <div class="optode-editor">
    <TransformEditor transform={state.transform} on:change={onTransformChange} />

    <div class="prop-row">
      <span class="prop-label">Spread</span>
      <input
        type="range" min="0.1" max="5" step="0.01"
        value={state.settings.spread_factor}
        on:input={onSpread}
        class="slider"
      />
      <span class="prop-value">{state.settings.spread_factor.toFixed(2)}</span>
    </div>

    <div class="prop-row">
      <span class="prop-label">Radius</span>
      <input
        type="number" step="0.1"
        value={state.settings.optode_radius}
        on:change={onRadius}
        class="num-input"
      />
    </div>

    <div class="prop-row">
      <span class="prop-label">Visible</span>
      <input type="checkbox" checked={state.visible} on:change={onVisible} />
    </div>
  </div>
{/if}

<style>
  .optode-editor {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .prop-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .prop-label {
    font-size: 9px;
    color: var(--text-muted);
    width: 44px;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .slider {
    flex: 1;
    accent-color: var(--accent-green);
    min-width: 0;
  }

  .num-input {
    flex: 1;
    min-width: 0;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-family: monospace;
    font-size: 10px;
    padding: 2px 4px;
    border-radius: 3px;
  }

  .prop-value {
    font-size: 10px;
    color: var(--text-primary);
    font-family: monospace;
    width: 32px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
