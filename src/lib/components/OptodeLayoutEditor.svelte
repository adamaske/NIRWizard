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
        type="range" min="0.001" max="0.05" step="0.001"
        value={state.settings.optode_radius}
        on:input={onRadius}
        class="slider"
      />
      <span class="prop-value">{state.settings.optode_radius.toFixed(3)}</span>
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

  .prop-value {
    font-size: 10px;
    color: var(--text-primary);
    font-family: monospace;
    width: 32px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
