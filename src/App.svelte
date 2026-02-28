<script>
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import MenuBar from "./lib/components/menubar.svelte";
  import InfoPanel from "./lib/components/infopanel.svelte";
  import Statusbar from "./lib/components/statusbar.svelte";

  let summary = null;

  let unlisten;

  onMount(async () => {
    // Populate if a default file was loaded at startup
    summary = await invoke("get_snirf_summary");

    // Keep in sync whenever a new file is loaded
    unlisten = await listen("snirf-loaded", (event) => {
      summary = event.payload;
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<div class="app">
  <MenuBar />
  <InfoPanel {summary} />
  <Statusbar
    filename={summary?.filename ?? "No file loaded"}
    channels={summary?.channels ?? 0}
    samplingRate={summary?.sampling_rate ?? 0}
    duration={summary?.duration ?? 0}
  />
</div>

<style>
  :global(html),
  :global(body),
  :global(#app) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    font-family: "Segoe UI", system-ui, sans-serif;
  }

  .app {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: #0f0f1a;
    color: #d0d0e0;
  }
</style>
