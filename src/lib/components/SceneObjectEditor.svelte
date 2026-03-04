<script>
  import { createEventDispatcher } from 'svelte';
  import TransformEditor from './TransformEditor.svelte';

  export let state = null;

  const dispatch = createEventDispatcher();

  function onTransformChange(e) {
    dispatch('change', { ...state, ...e.detail });
  }

  function onOpacity(e) {
    dispatch('change', { ...state, opacity: parseFloat(e.target.value) });
  }

  function onVisible(e) {
    dispatch('change', { ...state, visible: e.target.checked });
  }
</script>

{#if state}
  <div class="scene-obj-editor">
    <TransformEditor
      transform={{ position: state.position, rotation: state.rotation, scale: state.scale }}
      on:change={onTransformChange}
    />

    <div class="prop-row">
      <span class="prop-label">Opacity</span>
      <input
        type="range" min="0" max="1" step="0.01"
        value={state.opacity}
        on:input={onOpacity}
        class="slider"
      />
      <span class="prop-value">{state.opacity.toFixed(2)}</span>
    </div>

    <div class="prop-row">
      <span class="prop-label">Visible</span>
      <input type="checkbox" checked={state.visible} on:change={onVisible} />
    </div>
  </div>
{/if}

<style>
  .scene-obj-editor {
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
    width: 28px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
