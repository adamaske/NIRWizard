<script>
  import { pipeline } from "../stores/pipeline.js";
</script>

<div class="pipeline-order">
  <h3 class="panel-title">Pipeline Order</h3>

  {#if $pipeline.steps.length === 0}
    <div class="empty">
      <p class="empty-text">No steps added</p>
      <p class="empty-hint">Click a step from the left panel to add it</p>
    </div>
  {:else}
    <div class="step-list">
      {#each $pipeline.steps as step, i (step.instanceId)}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          class="step-row"
          class:selected={$pipeline.selectedId === step.instanceId}
          class:disabled={!step.enabled}
          on:click={() => pipeline.selectStep(step.instanceId)}
        >
          <span class="step-index">{i + 1}.</span>
          <span class="step-label">{step.label}</span>
          <div class="step-controls">
            <button
              class="ctrl-btn"
              title="Move up"
              disabled={i === 0}
              on:click|stopPropagation={() => pipeline.moveStep(step.instanceId, -1)}
            >&#9650;</button>
            <button
              class="ctrl-btn"
              title="Move down"
              disabled={i === $pipeline.steps.length - 1}
              on:click|stopPropagation={() => pipeline.moveStep(step.instanceId, 1)}
            >&#9660;</button>
            <button
              class="ctrl-btn toggle-btn"
              class:toggled-off={!step.enabled}
              title={step.enabled ? "Disable" : "Enable"}
              on:click|stopPropagation={() => pipeline.toggleEnabled(step.instanceId)}
            >{step.enabled ? "ON" : "OFF"}</button>
            <button
              class="ctrl-btn remove-btn"
              title="Remove"
              on:click|stopPropagation={() => pipeline.removeStep(step.instanceId)}
            >&#10005;</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .pipeline-order {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-title {
    margin: 0;
    padding: 10px 14px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1px;
    text-transform: uppercase;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-subtle);
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .empty-text {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
  }

  .empty-hint {
    margin: 0;
    font-size: 11px;
    color: var(--text-faint);
  }

  .step-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .step-list::-webkit-scrollbar {
    width: 4px;
  }
  .step-list::-webkit-scrollbar-thumb {
    background: var(--border-default);
    border-radius: 2px;
  }

  .step-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    margin: 2px 0;
    border: 1px solid transparent;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s, opacity 0.15s;
  }

  .step-row:hover {
    background: var(--bg-overlay);
  }

  .step-row.selected {
    background: var(--bg-overlay);
    border-color: var(--accent-green);
  }

  .step-row.disabled {
    opacity: 0.45;
  }

  .step-index {
    font-size: 11px;
    color: var(--text-faint);
    min-width: 18px;
    font-variant-numeric: tabular-nums;
  }

  .step-label {
    flex: 1;
    font-size: 12px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .step-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .ctrl-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    font-size: 9px;
    font-family: inherit;
    background: var(--bg-raised);
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s, border-color 0.12s;
  }

  .ctrl-btn:hover:not(:disabled) {
    background: var(--bg-overlay);
    color: var(--text-primary);
    border-color: var(--border-default);
  }

  .ctrl-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .toggle-btn {
    width: auto;
    padding: 0 5px;
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--accent-green);
  }

  .toggle-btn.toggled-off {
    color: var(--text-faint);
  }

  .remove-btn:hover:not(:disabled) {
    color: var(--accent-pink);
    border-color: var(--accent-pink);
  }
</style>
