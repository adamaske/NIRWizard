<script>
    import { onMount, onDestroy } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";

    // Layout
    import MenuBar from "./lib/Bars/menubar.svelte";
    import Statusbar from "./lib/Bars/statusbar.svelte";

    // NIRS
    import Info from "./lib/NIRS/Info.svelte";
    
    // Plotting
    import TimeSeries from "./lib/Plotting/TimeSeries.svelte";
    import Spectrogram from "./lib/Plotting/Spectrogram.svelte";
    import Frequency from "./lib/Plotting/Frequency.svelte";
    import HRF_CM_Panel from "./lib/Plotting/HRF_CM_Panel.svelte";

    // Channel Selection
    import ChannelSelector2D from "./lib/ChannelSelection/ChannelSelector2D.svelte";

    // Anatomy 3D
    import AnatomyViewport from "./lib/Anatomy/Viewport.svelte";
    import AnatomyInspector from "./lib/Anatomy/SceneInspector.svelte";

    // HDF5
    import HDF5TreeWalker from "./lib/HDF5/HDF5TreeWalker.svelte";

    // ── Tab state ─────────────────────────────────────────────────────────────
    let activeTopLeft = "info"; // "info" | "hdf5"
    let activeTimeFreq = "spectrogram"; // "spectrogram" | "frequency"

    // ── App state ─────────────────────────────────────────────────────────────
    let summary = null;
    let unlisten;

    // ── DOM refs ──────────────────────────────────────────────────────────────
    let workspaceEl;
    let topRowEl;
    let rightPlotEl;
    let bottomRowEl;

    // ── Panel sizes (px); null = flex defaults until first layout ─────────────
    let topHeight = null; // top row height
    let topLeftWidth = null; // left panel in top row
    let innerTopHeight = null; // TimeSeries height inside right column
    let botChWidth = null; // ChannelSelector2D width
    let botHRFWidth = null; // HRF_CM_Panel width

    // ── Drag state ────────────────────────────────────────────────────────────
    // One variable tracks which divider is active; null = not dragging
    let dragging = null; // 'row' | 'topCol' | 'innerRow' | 'botCol1' | 'botCol2'
    let dragStartX = 0;
    let dragStartY = 0;
    let dragStartSize = 0;

    const LAYOUT_KEY = "nirwizard_layout";

    function saveLayout() {
        localStorage.setItem(
            LAYOUT_KEY,
            JSON.stringify({
                topHeight,
                topLeftWidth,
                innerTopHeight,
                botChWidth,
                botHRFWidth,
            }),
        );
    }

    onMount(async () => {
        summary = await invoke("get_snirf_summary");
        unlisten = await listen("snirf-loaded", (event) => {
            summary = event.payload;
        });

        const w = workspaceEl.clientWidth;
        const h = workspaceEl.clientHeight;

        const saved = JSON.parse(localStorage.getItem(LAYOUT_KEY) ?? "null");
        topHeight = saved?.topHeight ?? Math.round(h * 0.6);
        topLeftWidth = saved?.topLeftWidth ?? Math.round(w * 0.22);
        innerTopHeight = saved?.innerTopHeight ?? Math.round(topHeight * 0.65);
        botChWidth = saved?.botChWidth ?? Math.round(w * 0.22);
        botHRFWidth = saved?.botHRFWidth ?? Math.round(w * 0.3);
    });

    onDestroy(() => {
        if (unlisten) unlisten();
    });

    // ── Drag helpers ──────────────────────────────────────────────────────────

    function startDrag(type, e, currentSize) {
        e.preventDefault();
        dragging = type;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
        dragStartSize = currentSize;
        document.body.style.userSelect = "none";
        document.body.style.cursor =
            type === "row" || type === "innerRow" ? "ns-resize" : "ew-resize";
    }

    function onMouseMove(e) {
        if (!dragging) return;
        const dy = e.clientY - dragStartY;
        const dx = e.clientX - dragStartX;

        if (dragging === "row") {
            const maxH = workspaceEl.clientHeight - 80;
            topHeight = Math.max(80, Math.min(maxH, dragStartSize + dy));
        } else if (dragging === "topCol") {
            const maxW = topRowEl.clientWidth - 120;
            topLeftWidth = Math.max(120, Math.min(maxW, dragStartSize + dx));
        } else if (dragging === "innerRow") {
            const maxH = rightPlotEl.clientHeight - 60;
            innerTopHeight = Math.max(60, Math.min(maxH, dragStartSize + dy));
        } else if (dragging === "botCol1") {
            const maxW = bottomRowEl.clientWidth - 240;
            botChWidth = Math.max(100, Math.min(maxW, dragStartSize + dx));
        } else if (dragging === "botCol2") {
            const maxW = bottomRowEl.clientWidth - botChWidth - 120;
            botHRFWidth = Math.max(100, Math.min(maxW, dragStartSize + dx));
        }
    }

    function onMouseUp() {
        if (dragging) {
            dragging = null;
            document.body.style.cursor = "";
            document.body.style.userSelect = "";
            saveLayout();
        }
    }
</script>

<!-- Track mouse globally so drags work even when cursor leaves the divider -->
<svelte:window on:mousemove={onMouseMove} on:mouseup={onMouseUp} />

<div class="app">
    <MenuBar />

    <div class="workspace" bind:this={workspaceEl}>
        <!-- ════════════════════════════════════════════════════════════════
             TOP ROW  ·  Info/HDF5 (left)  |  TimeSeries + Spectrogram (right)
             ════════════════════════════════════════════════════════════════ -->
        <div
            class="top-row"
            bind:this={topRowEl}
            style={topHeight !== null
                ? `height:${topHeight}px; flex:none`
                : "flex:3"}
        >
            <!-- Left: Info / HDF5 Walker -->
            <div
                class="panel"
                style={topLeftWidth !== null
                    ? `width:${topLeftWidth}px; flex:none`
                    : "flex:1"}
            >
                <div class="tab-bar">
                    <button
                        class="tab-btn"
                        class:active={activeTopLeft === "info"}
                        on:click={() => (activeTopLeft = "info")}>Info</button
                    >
                    <button
                        class="tab-btn"
                        class:active={activeTopLeft === "hdf5"}
                        on:click={() => (activeTopLeft = "hdf5")}>HDF5</button
                    >
                </div>
                {#if activeTopLeft === "info"}
                    <Info {summary} />
                {:else}
                    <HDF5TreeWalker />
                {/if}
            </div>

            <!-- ── Top-row column divider ── -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="divider divider-col"
                class:dragging={dragging === "topCol"}
                on:mousedown={(e) => startDrag("topCol", e, topLeftWidth)}
            />

            <!-- Right column: TimeSeries (top) + Spectrogram/Frequency (bottom) -->
            <div class="panel right-plots" bind:this={rightPlotEl}>
                <!-- TimeSeries -->
                <div
                    class="panel"
                    style={innerTopHeight !== null
                        ? `height:${innerTopHeight}px; flex:none`
                        : "flex:2"}
                >
                    <TimeSeries />
                </div>

                <!-- ── Inner row divider ── -->
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <div
                    class="divider divider-row"
                    class:dragging={dragging === "innerRow"}
                    on:mousedown={(e) =>
                        startDrag("innerRow", e, innerTopHeight)}
                />

                <!-- Spectrogram / Frequency tabs -->
                <div class="panel" style="flex:1">
                    <div class="tab-bar">
                        <button
                            class="tab-btn"
                            class:active={activeTimeFreq === "spectrogram"}
                            on:click={() => (activeTimeFreq = "spectrogram")}
                            >Spectrogram</button
                        >
                        <button
                            class="tab-btn"
                            class:active={activeTimeFreq === "frequency"}
                            on:click={() => (activeTimeFreq = "frequency")}
                            >Frequency</button
                        >
                    </div>
                    {#if activeTimeFreq === "spectrogram"}
                        <Spectrogram />
                    {:else}
                        <Frequency />
                    {/if}
                </div>
            </div>
        </div>

        <!-- ── Main row divider ── -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
            class="divider divider-row"
            class:dragging={dragging === "row"}
            on:mousedown={(e) => startDrag("row", e, topHeight)}
        />

        <!-- ════════════════════════════════════════════════════════════════
             BOTTOM ROW  ·  ChSel2D  |  HRF/CM  |  Anatomy 3D
             ════════════════════════════════════════════════════════════════ -->
        <div class="bottom-row" bind:this={bottomRowEl}>
            <!-- ChannelSelector 2D -->
            <div
                class="panel"
                style={botChWidth !== null
                    ? `width:${botChWidth}px; flex:none`
                    : "flex:1"}
            >
                <ChannelSelector2D />
            </div>

            <!-- ── Bottom col divider 1 ── -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="divider divider-col"
                class:dragging={dragging === "botCol1"}
                on:mousedown={(e) => startDrag("botCol1", e, botChWidth)}
            />

            <!-- HRF / Connectivity panel -->
            <div
                class="panel"
                style={botHRFWidth !== null
                    ? `width:${botHRFWidth}px; flex:none`
                    : "flex:1"}
            >
                <HRF_CM_Panel />
            </div>

            <!-- ── Bottom col divider 2 ── -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
                class="divider divider-col"
                class:dragging={dragging === "botCol2"}
                on:mousedown={(e) => startDrag("botCol2", e, botHRFWidth)}
            />

            <!-- Anatomy 3D View -->
            <div class="panel" style="flex:1">
                <div class="scene-view">
                    <AnatomyViewport />
                    <AnatomyInspector />
                </div>
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

    /* ── Main workspace ── */
    .workspace {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-height: 0;
        overflow: hidden;
    }

    /* ── Top row: left info panel + right plotting column ── */
    .top-row {
        display: flex;
        flex-direction: row;
        min-height: 0;
        overflow: hidden;
    }

    /* ── Right plotting column: TimeSeries stacked over Spectrogram/Freq ── */
    .right-plots {
        flex: 1;
    }

    /* ── Bottom row: ChSel2D + HRF/CM + Anatomy ── */
    .bottom-row {
        flex: 1;
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

    /* ── Anatomy scene view: viewport + inspector side by side ── */
    .scene-view {
        flex: 1;
        display: flex;
        flex-direction: row;
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
        transition:
            color 0.12s,
            border-color 0.12s;
    }

    .tab-btn:hover {
        color: var(--text-secondary);
    }

    .tab-btn.active {
        color: var(--accent-green);
        border-bottom-color: var(--accent-green);
    }
</style>
