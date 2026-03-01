<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  async function loadSnirf() {
    const path = await open({
      multiple: false,
      filters: [{ name: "SNIRF", extensions: ["snirf"] }],
    });

    if (path) {
      await invoke("load_snirf", { path });
    }
  }
</script>

<div class="control-panel">
  <button class="load-btn" on:click={loadSnirf}>Load sNIRF</button>
</div>

<style>
  .control-panel {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-base);
  }

  .load-btn {
    padding: 12px 32px;
    font-size: 15px;
    font-family: inherit;
    background: var(--bg-raised);
    color: var(--text-secondary);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    cursor: pointer;
    letter-spacing: 0.4px;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }

  .load-btn:hover {
    background: var(--bg-overlay);
    color: var(--text-primary);
    border-color: var(--accent-green);
  }

  .load-btn:active {
    background: var(--bg-overlay);
  }
</style>
