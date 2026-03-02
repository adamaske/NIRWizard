<script>
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import MenuBar from "./lib/components/menubar.svelte";
  import InfoPanel from "./lib/components/infopanel.svelte";
  import ChannelSelector from "./lib/components/ChannelSelector.svelte";
  import DataPlotter from "./lib/components/DataPlotter.svelte";
  import PipelineEditor from "./lib/pipeline/PipelineEditor.svelte";
  import Statusbar from "./lib/components/statusbar.svelte";

  let activeTab = "chart";

  let summary = null;
  let unlisten;

  // DOM refs for measuring
  let workspaceEl;
  let bottomRowEl;

  // Panel sizes in pixels; null = use flex defaults until first layout
  let topHeight = null;
  let leftWidth = null;

  // Drag state
  let draggingRow = false; // top / bottom divider
  let draggingCol = false; // left / right divider
  let dragStartY = 0,
    dragStartTop = 0;
  let dragStartX = 0,
    dragStartLeft = 0;

  onMount(async () => {
    summary = await invoke("get_snirf_summary");
    unlisten = await listen("snirf-loaded", (event) => {
      summary = event.payload;
    });

    // Seed pixel sizes from actual DOM dimensions so layout is stable on load
    topHeight = Math.round(workspaceEl.clientHeight * 0.6);
    leftWidth = Math.round(bottomRowEl.clientWidth * 0.6);
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  // ── Row divider (top ↕ bottom) ────────────────────────────────────────────
  function startRowDrag(e) {
    e.preventDefault();
    draggingRow = true;
    dragStartY = e.clientY;
    dragStartTop = topHeight;
    document.body.style.cursor = "ns-resize";
    document.body.style.userSelect = "none";
  }

  // ── Column divider (left ↔ right) ─────────────────────────────────────────
  function startColDrag(e) {
    e.preventDefault();
    draggingCol = true;
    dragStartX = e.clientX;
    dragStartLeft = leftWidth;
    document.body.style.cursor = "ew-resize";
    document.body.style.userSelect = "none";
  }

  // ── Global mouse tracking ─────────────────────────────────────────────────
  function onMouseMove(e) {
    if (draggingRow) {
      const maxH = workspaceEl.clientHeight - 80;
      topHeight = Math.max(
        80,
        Math.min(maxH, dragStartTop + (e.clientY - dragStartY)),
      );
    }
    if (draggingCol) {
      const maxW = bottomRowEl.clientWidth - 120;
      leftWidth = Math.max(
        120,
        Math.min(maxW, dragStartLeft + (e.clientX - dragStartX)),
      );
    }
  }

  function onMouseUp() {
    if (draggingRow || draggingCol) {
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      draggingRow = false;
      draggingCol = false;
    }
  }
</script>

<!-- Track mouse globally so drags work even when cursor leaves the divider -->
<svelte:window on:mousemove={onMouseMove} on:mouseup={onMouseUp} />

<div class="app">
  <MenuBar />

  <div class="workspace" bind:this={workspaceEl}>
    <!-- ── Top panel: tabbed (Chart / Pipeline) ── -->
    <div
      class="panel"
      style={topHeight !== null ? `height:${topHeight}px; flex:none` : "flex:2"}
    >
      <div class="tab-bar">
        <button
          class="tab-btn"
          class:active={activeTab === "chart"}
          on:click={() => (activeTab = "chart")}
        >Chart</button>
        <button
          class="tab-btn"
          class:active={activeTab === "pipeline"}
          on:click={() => (activeTab = "pipeline")}
        >Pipeline</button>
      </div>
      {#if activeTab === "chart"}
        <DataPlotter />
      {:else}
        <PipelineEditor />
      {/if}
    </div>

    <!-- ── Row divider ── -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="divider divider-row"
      class:dragging={draggingRow}
      on:mousedown={startRowDrag}
    />

    <!-- ── Bottom strip ── -->
    <div class="bottom-row" bind:this={bottomRowEl}>
      <!-- Left: ChannelSelector -->
      <div
        class="panel"
        style={leftWidth !== null
          ? `width:${leftWidth}px; flex:none`
          : "flex:1.5"}
      >
        <ChannelSelector />
      </div>

      <!-- ── Column divider ── -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div
        class="divider divider-col"
        class:dragging={draggingCol}
        on:mousedown={startColDrag}
      />

      <!-- Right: InfoPanel -->
      <div class="panel" style="flex:1">
        <InfoPanel {summary} />
      </div>
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
    background: var(--bg-base);
    color: var(--text-primary);
  }

  /* ── Main workspace: stacks plotter on top, bottom strip below ── */
  .workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Bottom strip: channel selector left, info panel right ── */
  .bottom-row {
    flex: 1; /* fills remaining height after DataPlotter */
    display: flex;
    flex-direction: row;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Generic panel wrapper ── */
  .panel {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Dividers ── */
  .divider {
    flex-shrink: 0;
    background: var(--border-subtle);
    transition: background 0.15s;
    position: relative;
  }

  /* Widen the actual hit-target without changing visual thickness */
  .divider::after {
    content: "";
    position: absolute;
    inset: 0;
  }

  .divider-row {
    height: 4px;
    cursor: ns-resize;
  }

  .divider-row::after {
    top: -4px;
    bottom: -4px;
  }

  .divider-col {
    width: 4px;
    cursor: ew-resize;
  }

  .divider-col::after {
    left: -4px;
    right: -4px;
  }

  .divider:hover,
  .divider.dragging {
    background: var(--accent-green);
  }

  /* ── Tab bar ── */
  .tab-bar {
    display: flex;
    gap: 0;
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

  .tab-btn:hover {
    color: var(--text-secondary);
  }

  .tab-btn.active {
    color: var(--accent-green);
    border-bottom-color: var(--accent-green);
  }
</style>
