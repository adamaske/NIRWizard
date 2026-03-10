<script>
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  const OPTODE_R = 12;
  const CH_LINE_W = 12;
  const CH_HIT_W = 16;

  const C_SOURCE   = "#dd3333";
  const C_DETECTOR = "#3355dd";
  const C_CHANNEL  = "#6e6e8a";
  const C_SELECTED = "#ffdd00";

  /** @type {{ sources: any[], detectors: any[], channels: any[] } | null} */
  let probeLayout = null;
  /** @type {Set<number>} */
  let selectedIds = new Set();

  let tx = 0, ty = 0, scale = 1;
  let isPanning = false;
  let panStart = { x: 0, y: 0 };
  let panOrigin = { tx: 0, ty: 0 };

  let svgEl;
  let unlisten;
  let resizeObserver;

  onMount(async () => {
    const layout = await invoke("get_probe_layout");
    if (layout) applyLayout(layout);
    unlisten = await listen("snirf-loaded", async () => {
      const layout = await invoke("get_probe_layout");
      if (layout) applyLayout(layout);
    });
    resizeObserver = new ResizeObserver(() => { fitView(); });
    if (svgEl) resizeObserver.observe(svgEl);
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (resizeObserver) resizeObserver.disconnect();
  });

  function applyLayout(layout) {
    probeLayout = layout;
    selectedIds = new Set(layout.channels.map((ch) => ch.id));
    fitView();
    notifyRust();
  }

  function fitView() {
    if (!probeLayout || !svgEl) return;
    const all = [...probeLayout.sources, ...probeLayout.detectors];
    if (!all.length) return;
    const xs = all.map((o) => o.x), ys = all.map((o) => o.y);
    const minX = Math.min(...xs), maxX = Math.max(...xs);
    const minY = Math.min(...ys), maxY = Math.max(...ys);
    const dataW = maxX - minX || 1, dataH = maxY - minY || 1;
    const PAD = 0.18;
    const w = svgEl.clientWidth, h = svgEl.clientHeight;
    const s = Math.min((w * (1 - 2 * PAD)) / dataW, (h * (1 - 2 * PAD)) / dataH);
    scale = s;
    tx = w / 2 - ((minX + maxX) / 2) * s;
    ty = h / 2 - ((minY + maxY) / 2) * s;
  }

  function selectChannel(id, e) {
    e.stopPropagation();
    if (e.ctrlKey || e.metaKey) {
      if (selectedIds.has(id)) selectedIds.delete(id);
      else selectedIds.add(id);
      selectedIds = new Set(selectedIds);
    } else {
      if (selectedIds.size === 1 && selectedIds.has(id)) return;
      selectedIds = new Set([id]);
    }
    notifyRust();
  }

  function selectAll() {
    if (!probeLayout) return;
    selectedIds = new Set(probeLayout.channels.map((ch) => ch.id));
    notifyRust();
  }

  function clearAll() { selectedIds = new Set(); notifyRust(); }

  async function notifyRust() {
    await invoke("set_selected_channels", { channelIds: [...selectedIds] });
  }

  function onWheel(e) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
    const rect = svgEl.getBoundingClientRect();
    const mx = e.clientX - rect.left, my = e.clientY - rect.top;
    scale *= factor;
    tx = mx - (mx - tx) * factor;
    ty = my - (my - ty) * factor;
  }

  function onBgMouseDown(e) {
    if (e.button === 0 || e.button === 1) {
      isPanning = true;
      panStart = { x: e.clientX, y: e.clientY };
      panOrigin = { tx, ty };
      e.preventDefault();
    }
  }

  function onSvgMouseMove(e) {
    if (!isPanning) return;
    tx = panOrigin.tx + (e.clientX - panStart.x);
    ty = panOrigin.ty + (e.clientY - panStart.y);
  }

  function endPan() { isPanning = false; }

  $: transformStr = `translate(${tx.toFixed(2)},${ty.toFixed(2)}) scale(${scale.toFixed(4)})`;
  $: sources   = probeLayout?.sources ?? [];
  $: detectors = probeLayout?.detectors ?? [];
  $: channels  = probeLayout?.channels ?? [];

  function srcOf(ch) { return sources[ch.source_idx]; }
  function detOf(ch) { return detectors[ch.detector_idx]; }
  function validCh(ch) { return srcOf(ch) && detOf(ch); }
</script>

<div class="channel-selector">
  <div class="toolbar">
    <button class="tb-btn" on:click={selectAll} disabled={!probeLayout}>Select All</button>
    <button class="tb-btn" on:click={clearAll} disabled={!probeLayout}>Clear</button>
    <button class="tb-btn" on:click={fitView} disabled={!probeLayout}>Fit View</button>
    {#if probeLayout}
      <span class="ch-count">{selectedIds.size} / {channels.length} ch</span>
    {/if}
    <span class="hint">Scroll: zoom · Drag: pan · Ctrl+click: multi-select</span>
  </div>

  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <svg
    bind:this={svgEl}
    class="canvas"
    width="100%"
    height="100%"
    on:wheel={onWheel}
    on:mousemove={onSvgMouseMove}
    on:mouseup={endPan}
    on:mouseleave={endPan}
  >
    {#if !probeLayout}
      <text x="50%" y="50%" text-anchor="middle" dominant-baseline="middle" fill="#3a3a5a" font-size="14" font-family="'Segoe UI', system-ui, sans-serif">No SNIRF file loaded</text>
    {:else}
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <rect x="0" y="0" width="100%" height="100%" fill="transparent" on:mousedown={onBgMouseDown} />
      <g transform={transformStr}>
        {#each channels as ch}
          {#if validCh(ch)}
            <line x1={srcOf(ch).x} y1={srcOf(ch).y} x2={detOf(ch).x} y2={detOf(ch).y}
              stroke={selectedIds.has(ch.id) ? C_SELECTED : C_CHANNEL}
              stroke-width={CH_LINE_W / scale} stroke-linecap="round" />
          {/if}
        {/each}

        {#each channels as ch}
          {#if validCh(ch)}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <line x1={srcOf(ch).x} y1={srcOf(ch).y} x2={detOf(ch).x} y2={detOf(ch).y}
              stroke="rgba(255,255,255,0.01)" stroke-width={CH_HIT_W / scale} stroke-linecap="round"
              style="cursor: pointer"
              on:click={(e) => selectChannel(ch.id, e)}
              on:mousedown|stopPropagation>
              <title>{ch.name}</title>
            </line>
          {/if}
        {/each}

        {#each detectors as d}
          <circle cx={d.x} cy={d.y} r={OPTODE_R / scale} fill={C_DETECTOR} stroke="#0a0a18" stroke-width={1.5 / scale} />
        {/each}
        {#each sources as s}
          <circle cx={s.x} cy={s.y} r={OPTODE_R / scale} fill={C_SOURCE} stroke="#0a0a18" stroke-width={1.5 / scale} />
        {/each}

        {#each sources as s}
          <text x={s.x} y={s.y} text-anchor="middle" dominant-baseline="central" fill="#ffffff"
            font-size={OPTODE_R * 0.75 / scale} font-family="'Segoe UI', system-ui, sans-serif"
            font-weight="700" pointer-events="none">{s.name}</text>
        {/each}
        {#each detectors as d}
          <text x={d.x} y={d.y} text-anchor="middle" dominant-baseline="central" fill="#ffffff"
            font-size={OPTODE_R * 0.75 / scale} font-family="'Segoe UI', system-ui, sans-serif"
            font-weight="700" pointer-events="none">{d.name}</text>
        {/each}
      </g>
    {/if}
  </svg>
</div>

<style>
  .channel-selector {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-base);
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .tb-btn {
    padding: 4px 11px;
    font-size: 11px;
    font-family: inherit;
    background: var(--bg-raised);
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s, border-color 0.12s;
    user-select: none;
  }

  .tb-btn:hover:not(:disabled) { background: var(--bg-overlay); color: var(--text-primary); border-color: var(--border-strong); }
  .tb-btn:disabled { opacity: 0.35; cursor: default; }

  .ch-count { font-size: 11px; color: var(--text-muted); font-variant-numeric: tabular-nums; margin-left: 2px; }

  .hint { margin-left: auto; font-size: 10px; color: var(--text-faint); user-select: none; white-space: nowrap; }

  .canvas { flex: 1; min-width: 0; min-height: 0; display: block; cursor: grab; }
  .canvas:active { cursor: grabbing; }
</style>
