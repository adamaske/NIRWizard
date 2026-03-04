<script>
  import { createEventDispatcher } from 'svelte';

  export let transform = { position: [0,0,0], rotation: [0,0,0], scale: [1,1,1] };

  const dispatch = createEventDispatcher();

  function update(axis, index, value) {
    const updated = {
      position: [...transform.position],
      rotation: [...transform.rotation],
      scale: [...transform.scale],
    };
    updated[axis][index] = parseFloat(value) || 0;
    dispatch('change', updated);
  }
</script>

<div class="transform-editor">
  {#each [['position', 0.001], ['rotation', 0.1], ['scale', 0.01]] as [axis, step]}
    <div class="row">
      <span class="label">{axis[0].toUpperCase() + axis.slice(1)}</span>
      {#each ['x','y','z'] as comp, i}
        <label class="vec-field">
          <span class="axis">{comp.toUpperCase()}</span>
          <input
            type="number"
            step={step}
            value={transform[axis][i]}
            on:input={(e) => update(axis, i, e.target.value)}
          />
        </label>
      {/each}
    </div>
  {/each}
</div>

<style>
  .transform-editor {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .label {
    font-size: 9px;
    color: var(--text-muted);
    width: 44px;
    flex-shrink: 0;
    letter-spacing: 0.3px;
    text-transform: uppercase;
  }

  .vec-field {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: 1;
  }

  .axis {
    font-size: 9px;
    color: var(--text-muted);
    width: 10px;
    flex-shrink: 0;
  }

  input[type="number"] {
    width: 100%;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-size: 10px;
    padding: 2px 3px;
    border-radius: 2px;
    font-family: monospace;
    min-width: 0;
  }

  input[type="number"]:focus {
    outline: none;
    border-color: var(--accent-green);
  }
</style>
