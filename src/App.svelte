<script>
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import MenuBar from "./lib/components/menubar.svelte";
  import Statusbar from "./lib/components/statusbar.svelte";
  import ControlPanel from "./lib/components/controlpanel.svelte";

  let filename = "No file loaded";
  let channels = 0;
  let samplingRate = 0;
  let duration = 0;

  function applySnirfSummary(summary) {
    if (!summary) return;
    filename = summary.filename;
    channels = summary.channels;
    samplingRate = summary.sampling_rate;
    duration = summary.duration;
  }

  let unlisten;

  onMount(async () => {
    // Populate statusbar if a default file was loaded at startup
    const summary = await invoke("get_snirf_summary");
    applySnirfSummary(summary);

    // Keep statusbar in sync whenever a new file is loaded
    unlisten = await listen("snirf-loaded", (event) => {
      applySnirfSummary(event.payload);
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<div class="app">
  <MenuBar />
  <ControlPanel />
  <Statusbar {filename} {channels} {samplingRate} {duration} />
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
