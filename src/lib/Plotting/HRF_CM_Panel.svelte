<script>
  import HRF from "./HRF.svelte";
  import Connectivity from "./Connectivity.svelte";

  let activeTab = "hrf";
</script>

<div class="panel-root">
  <div class="tab-bar">
    <button class="tab-btn" class:active={activeTab === "hrf"} on:click={() => (activeTab = "hrf")}>HRF</button>
    <button class="tab-btn" class:active={activeTab === "connectivity"} on:click={() => (activeTab = "connectivity")}>Connectivity</button>
  </div>
  {#if activeTab === "hrf"}
    <HRF />
  {:else}
    <Connectivity />
  {/if}
</div>

<style>
  .panel-root {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tab-bar {
    display: flex;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .tab-btn {
    padding: 6px 18px;
    font-size: 11px;
    font-family: inherit;
    font-weight: 500;
    letter-spacing: 0.3px;
    background: none;
    color: var(--text-muted);
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 0.12s, border-color 0.12s;
  }

  .tab-btn:hover { color: var(--text-secondary); }
  .tab-btn.active { color: var(--accent-green); border-bottom-color: var(--accent-green); }
</style>
