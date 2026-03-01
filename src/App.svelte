<script>
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import MenuBar from "./lib/components/menubar.svelte";
  import InfoPanel from "./lib/components/infopanel.svelte";
  import DataPlotter from "./lib/components/DataPlotter.svelte";
  import Statusbar from "./lib/components/statusbar.svelte";

  let summary = null;
  let unlisten;

  onMount(async () => {
    summary = await invoke("get_snirf_summary");
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

  <div class="workspace">
    <!--
      This is the flexible area. To change layout, just rearrange these divs.
      
      Side by side:  workspace is flex-direction: row (default)
      Stacked:       workspace is flex-direction: column
      
      Each panel wrapper gets a flex value controlling its share of space.
    -->
    <div class="panel" style="flex: 1;">
      <InfoPanel {summary} />
    </div>
    <div class="panel" style="flex: 2;">
      <DataPlotter />
    </div>
  </div>

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

  .workspace {
    flex: 1;
    display: flex;
    flex-direction: row; /* side by side */
    min-height: 0; /* critical: lets flex children shrink */
    overflow: hidden;
  }

  .panel {
    min-width: 0; /* prevents content from forcing panel wider */
    min-height: 0;
    overflow: hidden;
  }
</style>
