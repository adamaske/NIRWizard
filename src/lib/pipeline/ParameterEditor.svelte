<script>
  import { pipeline, selectedStep } from "../stores/pipeline.js";

  function handleChange(instanceId, key, paramDef, e) {
    let value;
    if (paramDef.type === "number") {
      value = parseFloat(e.target.value);
    } else if (paramDef.type === "integer") {
      value = parseInt(e.target.value, 10);
    } else if (paramDef.type === "boolean") {
      value = e.target.checked;
    } else {
      value = e.target.value;
    }
    pipeline.updateParam(instanceId, key, value);
  }
</script>

<div class="parameter-editor">
  <h3 class="panel-title">Parameters</h3>

  {#if !$selectedStep}
    <div class="empty">
      <p class="empty-text">Select a step to configure</p>
    </div>
  {:else}
    <div class="param-scroll">
      <div class="step-header">
        <h4 class="step-name">{$selectedStep.step.label}</h4>
        <p class="step-desc">{$selectedStep.def.description}</p>
      </div>

      <div class="param-list">
        {#each Object.entries($selectedStep.def.params) as [key, paramDef]}
          <div class="param-row">
            <label class="param-label" for="param-{key}">{paramDef.label}</label>

            {#if paramDef.type === "select"}
              <select
                id="param-{key}"
                class="param-input param-select"
                value={$selectedStep.step.params[key]}
                on:change={(e) => handleChange($selectedStep.step.instanceId, key, paramDef, e)}
              >
                {#each paramDef.options as opt}
                  <option value={opt.value}>{opt.label}</option>
                {/each}
              </select>
            {:else if paramDef.type === "number"}
              <input
                id="param-{key}"
                class="param-input"
                type="number"
                value={$selectedStep.step.params[key]}
                min={paramDef.min}
                max={paramDef.max}
                step={paramDef.step}
                on:change={(e) => handleChange($selectedStep.step.instanceId, key, paramDef, e)}
              />
            {:else if paramDef.type === "integer"}
              <input
                id="param-{key}"
                class="param-input"
                type="number"
                value={$selectedStep.step.params[key]}
                min={paramDef.min}
                max={paramDef.max}
                step="1"
                on:change={(e) => handleChange($selectedStep.step.instanceId, key, paramDef, e)}
              />
            {:else if paramDef.type === "boolean"}
              <input
                id="param-{key}"
                class="param-input param-checkbox"
                type="checkbox"
                checked={$selectedStep.step.params[key]}
                on:change={(e) => handleChange($selectedStep.step.instanceId, key, paramDef, e)}
              />
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .parameter-editor {
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
    align-items: center;
    justify-content: center;
  }

  .empty-text {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
  }

  .param-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
  }

  .param-scroll::-webkit-scrollbar {
    width: 4px;
  }
  .param-scroll::-webkit-scrollbar-thumb {
    background: var(--border-default);
    border-radius: 2px;
  }

  .step-header {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .step-name {
    margin: 0 0 6px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .step-desc {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .param-list {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .param-row {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .param-label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .param-input {
    padding: 7px 10px;
    font-size: 13px;
    font-family: inherit;
    background: var(--bg-raised);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    border-radius: 5px;
    outline: none;
    transition: border-color 0.15s;
  }

  .param-input:focus {
    border-color: var(--accent-green);
  }

  .param-select {
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%239090a0' stroke-width='1.5' fill='none'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    padding-right: 28px;
  }

  .param-select option {
    background: var(--bg-raised);
    color: var(--text-primary);
  }

  .param-checkbox {
    width: 16px;
    height: 16px;
    padding: 0;
    accent-color: var(--accent-green);
  }

  /* Number input spinner styling */
  .param-input[type="number"]::-webkit-inner-spin-button,
  .param-input[type="number"]::-webkit-outer-spin-button {
    opacity: 0.5;
  }
</style>
