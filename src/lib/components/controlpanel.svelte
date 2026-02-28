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
    background: #0f0f1a;
  }

  .load-btn {
    padding: 12px 32px;
    font-size: 15px;
    font-family: inherit;
    background: #1e1e3a;
    color: #a0a0cc;
    border: 1px solid #3a3a5e;
    border-radius: 6px;
    cursor: pointer;
    letter-spacing: 0.4px;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }

  .load-btn:hover {
    background: #2a2a4e;
    color: #e0e0f8;
    border-color: #6060aa;
  }

  .load-btn:active {
    background: #333360;
  }
</style>
