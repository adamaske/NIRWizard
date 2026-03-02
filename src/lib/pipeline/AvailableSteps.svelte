<script>
  import { STEP_DEFINITIONS } from "./stepDefinitions.js";
  import { pipeline } from "../stores/pipeline.js";

  // Group definitions by category at module level
  const categories = {};
  for (const def of Object.values(STEP_DEFINITIONS)) {
    if (!categories[def.category]) categories[def.category] = [];
    categories[def.category].push(def);
  }

  const categoryLabels = {
    filter: "Filter",
    quality: "Quality",
  };
</script>

<div class="available-steps">
  <h3 class="panel-title">Available Steps</h3>
  <div class="step-list">
    {#each Object.entries(categories) as [cat, defs]}
      <div class="category">
        <span class="category-label">{categoryLabels[cat] || cat}</span>
        {#each defs as def}
          <button class="step-row" on:click={() => pipeline.addStep(def.id)}>
            <span class="step-add">+</span>
            <span class="step-label">{def.label}</span>
          </button>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .available-steps {
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

  .category {
    margin-bottom: 12px;
  }

  .category-label {
    display: block;
    padding: 4px 8px;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .step-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    margin: 2px 0;
    background: none;
    border: 1px solid transparent;
    border-radius: 5px;
    cursor: pointer;
    font-family: inherit;
    text-align: left;
    transition: background 0.12s, border-color 0.12s;
  }

  .step-row:hover {
    background: var(--bg-overlay);
    border-color: var(--border-default);
  }

  .step-add {
    font-size: 14px;
    font-weight: 600;
    color: var(--accent-green);
    line-height: 1;
  }

  .step-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .step-row:hover .step-label {
    color: var(--text-primary);
  }
</style>
