<script>
  import { invoke } from "@tauri-apps/api/core"; // Default? To call invoke via Rust?
  import { open } from "@tauri-apps/plugin-dialog";

  let snirfData = null;
  let error = null;
  let filePath = "";
  let loading = false;

  async function OpenFile(){
      const selected = await open({
        filters: [{name: "SNIRF", extensions: ["snirf"] }],
      });

      if (!selected) return;

      try {
        snirfData = await invoke("load_snirf", { path: selected });
      } catch (e){
        error = e;
        snirfData = null;
      } finally {
        loading = false;
      }
  }
</script>

<main>
  <h1>NIRWizard</h1>

  <button on:click={OpenFile}>Open SNIRF File</button>

  {#if loading}
    <p>Loading</p>
  {/if}

  {#if error}
    <p class="error">Error: {error}</p>
  {/if}

  {#if filePath}
    <p class="path">{filePath}</p>
  {/if}

  {#if snirfData}
    <div class="info">
      <h2>File Loaded</h2>
      <p>Channels: {snirfData.data.data.length}</p>
      <p>Timepoints: {snirfData.data.time.length}</p>
      <p>Sources: {snirfData.biosignal.auxilaries.length}</p>
      <p>Detectors: {snirfData.biosignal.auxilaries.length}</p>
      <p>Duration: {(snirfData.data.time[snirfData.data.time.length - 1] - snirfData.data.time[0]).toFixed(2)}s</p>
    </div>
  {/if}
</main>

<style>
  main{
    font-family: system-ui, sans-serif;
    max-width: 600px;
    margin: 2rem auto;
    padding: 1rem;
  }
  button{
    padding: 0.5rem 1rem;
    font-size: 1rem;
    cursor: pointer;
  }
  .error{
    color: #d32f2f;
  }
  .path{
    color: #666;
    font-size: 0.85rem;
    word-break: break-all;
  }
  .info{
    margin-top: 1rem;
    padding: 1rem;
    background: #1aff00;
    border-radius: 4px;
  }
</style>