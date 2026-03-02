<script>
  import AvailableSteps from "./AvailableSteps.svelte";
  import PipelineOrder from "./PipelineOrder.svelte";
  import ParameterEditor from "./ParameterEditor.svelte";
</script>

<div class="pipeline-editor">
  <!-- Toolbar -->
  <div class="toolbar">
    <span class="toolbar-title">Pipeline Editor</span>
    <div class="toolbar-actions">
      <button class="toolbar-btn" disabled title="Load pipeline (coming soon)"
        >Load</button
      >
      <button class="toolbar-btn" disabled title="Save pipeline (coming soon)"
        >Save</button
      >
      <button
        class="toolbar-btn toolbar-btn-primary"
        disabled
        title="Run pipeline (coming soon)">Run</button
      >
    </div>
  </div>

  <!-- Three-column layout -->
  <div class="columns">
    <div class="col col-left">
      <AvailableSteps />
    </div>
    <div class="divider-col" />
    <div class="col col-center">
      <PipelineOrder />
    </div>
    <div class="divider-col" />
    <div class="col col-right">
      <ParameterEditor />
    </div>
  </div>
</div>

<style>
  .pipeline-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
    background: var(--bg-base);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .toolbar-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.3px;
  }

  .toolbar-actions {
    display: flex;
    gap: 6px;
  }

  .toolbar-btn {
    padding: 5px 14px;
    font-size: 11px;
    font-family: inherit;
    font-weight: 500;
    background: var(--bg-raised);
    color: var(--text-muted);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    cursor: pointer;
    transition:
      background 0.12s,
      color 0.12s,
      border-color 0.12s;
  }

  .toolbar-btn:hover:not(:disabled) {
    background: var(--bg-overlay);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .toolbar-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .toolbar-btn-primary {
    color: var(--accent-green);
    border-color: var(--accent-green);
  }

  .toolbar-btn-primary:hover:not(:disabled) {
    background: var(--bg-overlay);
    color: var(--accent-green);
  }

  .columns {
    flex: 1;
    display: flex;
    flex-direction: row;
    min-height: 0;
    overflow: hidden;
  }

  .col {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .col-left {
    flex: 1 1 180px;
  }

  .col-center {
    flex: 2 1 240px;
  }

  .col-right {
    flex: 2 1 240px;
  }

  .divider-col {
    width: 4px;
    flex-shrink: 0;
    background: var(--border-default);
  }
</style>
