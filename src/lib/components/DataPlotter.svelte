<!--
  DataPlotter.svelte

  Multi-channel ECharts time-series plot with stacked and unstacked modes.
  Listens to snirf-loaded (fetch + cache all data) and channels-selected
  (update which channels are rendered).

  // TODO : Update timepoint selector with a dragable line
  Features:
  - LTTB downsampling for large datasets
  - Event markers and shaded regions
  - Stacked / unstacked view modes
-->
<script>
  import { onMount, onDestroy } from "svelte";
  import * as echarts from "echarts";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let container;
  let wrapper;
  let chart;
  let resizeObserver;
  let unlistenSnirf;
  let unlistenChannels;

  let allData = null;
  let selectedIds = [];
  let stacked = true;

  let eventTypes = [];
  let showEventsPanel = false;

  // Time cursor state
  let cursorTime = 0;
  let cursorInputValue = "0";
  let maxTime = 0;

  const GRID_HEIGHT = 120;
  const DOWNSAMPLE_TARGET = 2000;

  const HBO_COLOR = "#ff2255";
  const HBR_COLOR = "#2266ff";
  const CHART_BG = "#0a0a10";
  const CHART_AXIS = "#333340";
  const CHART_GRID = "#161620";
  const CURSOR_COLOR = "#ff4444";
  const CURSOR_WIDTH = 2;

  const EVENT_COLORS = [
    "#33cc66",
    "#ffaa22",
    "#22ccdd",
    "#dd44aa",
    "#aadd22",
    "#7766ff",
    "#ff6644",
    "#44ddaa",
    "#cc88ff",
    "#dddd33",
    "#ff88aa",
    "#6699ff",
  ];

  function syncEventTypes(events) {
    const prevMap = new Map(eventTypes.map((et) => [et.name, et.visible]));
    eventTypes = events.map((ev, i) => ({
      name: ev.name,
      color: EVENT_COLORS[i % EVENT_COLORS.length],
      visible: prevMap.has(ev.name) ? prevMap.get(ev.name) : true,
    }));
  }

  function toggleEventType(name) {
    eventTypes = eventTypes.map((et) =>
      et.name === name ? { ...et, visible: !et.visible } : et,
    );
    updateChart();
  }

  function buildMarkLines(xAxisIndex) {
    if (!allData?.events) return [];
    const visibleNames = new Set(
      eventTypes.filter((et) => et.visible).map((et) => et.name),
    );
    const colorMap = new Map(eventTypes.map((et) => [et.name, et.color]));
    const lines = [];
    for (const ev of allData.events) {
      if (!visibleNames.has(ev.name)) continue;
      const color = colorMap.get(ev.name);
      for (const m of ev.markers) {
        lines.push({
          xAxis: m.onset,
          lineStyle: { color, width: 1.5, type: "solid" },
          label: { show: false },
        });
      }
    }
    return lines;
  }

  function buildMarkAreas(xAxisIndex) {
    if (!allData?.events) return [];
    const visibleNames = new Set(
      eventTypes.filter((et) => et.visible).map((et) => et.name),
    );
    const colorMap = new Map(eventTypes.map((et) => [et.name, et.color]));
    const areas = [];
    for (const ev of allData.events) {
      if (!visibleNames.has(ev.name)) continue;
      const color = colorMap.get(ev.name);
      for (const m of ev.markers) {
        if (m.duration > 0) {
          areas.push([
            { xAxis: m.onset, itemStyle: { color: color + "18" } },
            { xAxis: m.onset + m.duration },
          ]);
        }
      }
    }
    return areas;
  }

  // -- Time cursor helpers --

  function findClosestTimeIndex(t) {
    if (!allData?.time || allData.time.length === 0) return 0;
    const time = allData.time;
    let lo = 0,
      hi = time.length - 1;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (time[mid] < t) lo = mid + 1;
      else hi = mid;
    }
    if (lo > 0 && Math.abs(time[lo - 1] - t) < Math.abs(time[lo] - t)) {
      return lo - 1;
    }
    return lo;
  }

  function clampAndSnap(t) {
    if (!allData?.time || allData.time.length === 0)
      return { time: 0, index: 0 };
    const timeArr = allData.time;
    const clamped = Math.max(
      timeArr[0],
      Math.min(timeArr[timeArr.length - 1], t),
    );
    const idx = findClosestTimeIndex(clamped);
    return { time: timeArr[idx], index: idx };
  }

  function setCursorTime(newTime, notifyRust = true) {
    const { time: snapped, index: idx } = clampAndSnap(newTime);
    cursorTime = snapped;
    cursorInputValue = snapped.toFixed(3);
    syncCursorLinePosition();
    if (notifyRust) {
      invoke("set_cursor_timepoint", { time: snapped, index: idx });
    }
  }

  function syncCursorLinePosition() {
    if (!chart || !allData) return;
    const px = chart.convertToPixel({ xAxisIndex: 0 }, [cursorTime, 0]);
    if (!px) return;
    const gridRect = getGridPixelRect();
    chart.setOption({
      graphic: [
        {
          id: "timeCursorLine",
          shape: { x1: 0, y1: gridRect.top, x2: 0, y2: gridRect.bottom },
          x: px[0],
        },
        {
          id: "timeCursorHandle",
          x: px[0],
          y: gridRect.top - 10,
        },
        {
          id: "timeCursorLabel",
          x: px[0],
          y: gridRect.top - 22,
          style: { text: cursorTime.toFixed(2) + "s" },
        },
      ],
    });
  }

  function getGridPixelRect() {
    if (!chart) return { top: 50, bottom: 500 };
    const height = container?.clientHeight ?? 600;
    return { top: 50, bottom: height - 40 };
  }

  function handleCursorDrag(params) {
    if (!chart || !allData) return;
    const dataPoint = chart.convertFromPixel({ xAxisIndex: 0 }, [
      params.offsetX,
      0,
    ]);
    if (!dataPoint) return;
    setCursorTime(dataPoint[0]);
  }

  function buildCursorGraphic() {
    if (!chart || !allData) return [];
    const px = chart.convertToPixel({ xAxisIndex: 0 }, [cursorTime, 0]);
    const x = px ? px[0] : 0;
    const gridRect = getGridPixelRect();
    return [
      {
        id: "timeCursorLine",
        type: "line",
        draggable: "horizontal",
        x: x,
        y: 0,
        shape: { x1: 0, y1: gridRect.top, x2: 0, y2: gridRect.bottom },
        style: {
          stroke: CURSOR_COLOR,
          lineWidth: CURSOR_WIDTH,
          lineDash: [6, 4],
        },
        z: 200,
        cursor: "ew-resize",
        ondrag: handleCursorDrag,
      },
      {
        id: "timeCursorHandle",
        type: "polygon",
        draggable: "horizontal",
        x: x,
        y: gridRect.top - 10,
        shape: {
          points: [
            [-6, 0],
            [6, 0],
            [0, 10],
          ],
        },
        style: { fill: CURSOR_COLOR },
        z: 201,
        cursor: "ew-resize",
        ondrag: handleCursorDrag,
      },
      {
        id: "timeCursorLabel",
        type: "text",
        draggable: "horizontal",
        x: x,
        y: gridRect.top - 22,
        style: {
          text: cursorTime.toFixed(2) + "s",
          fill: CURSOR_COLOR,
          fontSize: 10,
          fontFamily: "monospace",
          textAlign: "center",
        },
        z: 202,
        cursor: "ew-resize",
        ondrag: handleCursorDrag,
      },
    ];
  }

  function submitCursorTime() {
    if (!allData) return;
    const val = parseFloat(cursorInputValue);
    if (isNaN(val)) return;
    setCursorTime(val);
  }

  function onCursorKeydown(e) {
    if (e.key === "Enter") {
      submitCursorTime();
    }
  }

  // -- LTTB downsampling --

  function lttb(data, target) {
    const len = data.length;
    if (target >= len || target < 3) return data;
    const out = [data[0]];
    const bucketSize = (len - 2) / (target - 2);
    let prevIndex = 0;
    for (let i = 1; i < target - 1; i++) {
      const avgStart = Math.floor((i + 0) * bucketSize) + 1;
      const avgEnd = Math.min(Math.floor((i + 1) * bucketSize) + 1, len);
      let avgX = 0,
        avgY = 0;
      for (let j = avgStart; j < avgEnd; j++) {
        avgX += data[j][0];
        avgY += data[j][1];
      }
      avgX /= avgEnd - avgStart;
      avgY /= avgEnd - avgStart;
      const rangeStart = Math.floor((i - 1) * bucketSize) + 1;
      const rangeEnd = Math.min(Math.floor(i * bucketSize) + 1, len);
      const px = data[prevIndex][0];
      const py = data[prevIndex][1];
      let maxArea = -1;
      let maxIdx = rangeStart;
      for (let j = rangeStart; j < rangeEnd; j++) {
        const area = Math.abs(
          (px - avgX) * (data[j][1] - py) - (px - data[j][0]) * (avgY - py),
        );
        if (area > maxArea) {
          maxArea = area;
          maxIdx = j;
        }
      }
      out.push(data[maxIdx]);
      prevIndex = maxIdx;
    }
    out.push(data[len - 1]);
    return out;
  }

  function downsample(time, values) {
    if (time.length <= DOWNSAMPLE_TARGET) {
      const pairs = new Array(time.length);
      for (let i = 0; i < time.length; i++) {
        pairs[i] = [time[i], values[i]];
      }
      return pairs;
    }
    const paired = new Array(time.length);
    for (let i = 0; i < time.length; i++) {
      paired[i] = [time[i], values[i]];
    }
    return lttb(paired, DOWNSAMPLE_TARGET);
  }

  // -- Resize --

  let resizeTimer;
  function debouncedResize() {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      if (chart) {
        chart.resize();
        syncCursorLinePosition();
      }
    }, 60);
  }

  // -- Chart building --

  function updateChart() {
    if (!chart || !allData) return;

    const channels =
      selectedIds.length > 0
        ? allData.channels.filter((ch) => selectedIds.includes(ch.id))
        : [];

    if (channels.length === 0) {
      chart.setOption(
        {
          backgroundColor: CHART_BG,
          animation: false,
          title: {
            text: "No channels selected",
            left: "center",
            top: "center",
            textStyle: { color: "#555", fontSize: 14 },
          },
          xAxis: [],
          yAxis: [],
          series: [],
          grid: [],
          dataZoom: [],
          legend: { show: false },
          tooltip: { show: false },
          graphic: [],
        },
        true,
      );
      if (wrapper) wrapper.style.overflowY = "hidden";
      container.style.height = "100%";
      chart.resize();
      return;
    }

    const time = allData.time;
    if (stacked) {
      buildStacked(channels, time);
    } else {
      buildUnstacked(channels, time);
    }
  }

  const PERF_SERIES = {
    type: "line",
    symbol: "none",
    large: true,
    largeThreshold: 3000,
    progressive: 500,
    animation: false,
  };

  function buildStacked(channels, time) {
    if (wrapper) wrapper.style.overflowY = "hidden";
    container.style.height = "100%";

    const series = [];
    const legendData = [];
    const manyChannels = channels.length > 20;
    const markLines = buildMarkLines(0);
    const markAreas = buildMarkAreas(0);

    channels.forEach((ch, idx) => {
      const hboName = `${ch.name} HbO`;
      const hbrName = `${ch.name} HbR`;
      legendData.push(hboName, hbrName);

      const hboSeries = {
        ...PERF_SERIES,
        name: hboName,
        data: downsample(time, ch.hbo),
        lineStyle: { color: HBO_COLOR, width: 1.5 },
        itemStyle: { color: HBO_COLOR },
      };
      if (idx === 0) {
        hboSeries.markLine = { symbol: "none", silent: true, data: markLines };
        if (markAreas.length > 0) {
          hboSeries.markArea = { silent: true, data: markAreas };
        }
      }
      series.push(hboSeries);

      series.push({
        ...PERF_SERIES,
        name: hbrName,
        data: downsample(time, ch.hbr),
        lineStyle: { color: HBR_COLOR, width: 1.5, type: "dotted" },
        itemStyle: { color: HBR_COLOR },
      });
    });

    chart.setOption(
      {
        backgroundColor: "#0d0d18",
        animation: false,
        title: { show: false },
        tooltip: manyChannels
          ? { show: false }
          : { trigger: "axis", axisPointer: { type: "cross" } },
        legend: {
          show: true,
          type: "scroll",
          data: legendData,
          textStyle: { color: "#9090a0" },
          top: 8,
        },
        grid: [{ left: 70, right: 30, top: 50, bottom: 40 }],
        xAxis: [
          {
            type: "value",
            gridIndex: 0,
            name: "Time (s)",
            nameLocation: "middle",
            nameGap: 30,
            axisLabel: { color: "#9090a0" },
            axisLine: { lineStyle: { color: CHART_AXIS } },
          },
        ],
        yAxis: [
          {
            type: "value",
            gridIndex: 0,
            name: "\u0394C (M)",
            nameLocation: "middle",
            nameGap: 50,
            axisLabel: {
              color: "#9090a0",
              formatter: (val) => val.toExponential(1),
            },
            axisLine: { lineStyle: { color: CHART_AXIS } },
            splitLine: { lineStyle: { color: CHART_GRID } },
          },
        ],
        dataZoom: [{ type: "inside", xAxisIndex: 0 }],
        series,
        graphic: buildCursorGraphic(),
      },
      true,
    );
    chart.resize();
  }

  function buildUnstacked(channels, time) {
    const n = channels.length;
    const needsScroll = n > 8;
    const totalHeight = needsScroll ? n * GRID_HEIGHT : undefined;

    if (wrapper) wrapper.style.overflowY = needsScroll ? "auto" : "hidden";
    container.style.height = totalHeight ? `${totalHeight}px` : "100%";

    const grids = [];
    const xAxes = [];
    const yAxes = [];
    const series = [];
    const xAxisIndices = [];
    const manyChannels = n > 20;

    channels.forEach((ch, i) => {
      const gridHeight = needsScroll
        ? GRID_HEIGHT - 38
        : `${(100 - 10) / n - 2}%`;
      const gridTop = needsScroll
        ? 10 + i * GRID_HEIGHT
        : `${5 + (i * (100 - 10)) / n}%`;

      grids.push({ left: 80, right: 30, top: gridTop, height: gridHeight });

      const isLast = i === n - 1;
      xAxes.push({
        type: "value",
        gridIndex: i,
        axisLabel: { show: isLast, color: "#9090a0" },
        axisTick: { show: isLast },
        axisLine: { lineStyle: { color: CHART_AXIS } },
        ...(isLast
          ? { name: "Time (s)", nameLocation: "middle", nameGap: 30 }
          : {}),
      });
      xAxisIndices.push(i);

      yAxes.push({
        type: "value",
        gridIndex: i,
        name: ch.name,
        nameLocation: "middle",
        nameGap: 55,
        nameTextStyle: { color: "#9090a0", fontSize: 11 },
        axisLabel: {
          color: "#9090a0",
          formatter: (val) => val.toExponential(1),
        },
        axisLine: { lineStyle: { color: CHART_AXIS } },
        splitLine: { lineStyle: { color: CHART_GRID } },
      });

      const markLines = buildMarkLines(i);
      const markAreas = buildMarkAreas(i);

      const hboSeries = {
        ...PERF_SERIES,
        name: `${ch.name} HbO`,
        data: downsample(time, ch.hbo),
        xAxisIndex: i,
        yAxisIndex: i,
        lineStyle: { color: HBO_COLOR, width: 1.5 },
        itemStyle: { color: HBO_COLOR },
        markLine: { symbol: "none", silent: true, data: markLines },
      };
      if (markAreas.length > 0) {
        hboSeries.markArea = { silent: true, data: markAreas };
      }
      series.push(hboSeries);

      series.push({
        ...PERF_SERIES,
        name: `${ch.name} HbR`,
        data: downsample(time, ch.hbr),
        xAxisIndex: i,
        yAxisIndex: i,
        lineStyle: { color: HBR_COLOR, width: 1.5, type: "dotted" },
        itemStyle: { color: HBR_COLOR },
      });
    });

    chart.setOption(
      {
        backgroundColor: "#0d0d18",
        animation: false,
        title: { show: false },
        tooltip: manyChannels
          ? { show: false }
          : { trigger: "axis", axisPointer: { type: "cross" } },
        legend: { show: false },
        grid: grids,
        xAxis: xAxes,
        yAxis: yAxes,
        dataZoom: [{ type: "inside", xAxisIndex: xAxisIndices }],
        series,
        graphic: buildCursorGraphic(),
      },
      true,
    );
    chart.resize();
  }

  async function fetchAndCacheData() {
    const payload = await invoke("get_timeseries_data");
    if (payload) {
      allData = payload;
      syncEventTypes(payload.events || []);
      selectedIds = payload.channels.map((ch) => ch.id);
      if (payload.time.length > 0) {
        maxTime = payload.time[payload.time.length - 1];
        cursorTime = 0;
        cursorInputValue = "0";
      }
      updateChart();
    }
  }

  function onToggle(mode) {
    stacked = mode;
    updateChart();
  }

  onMount(async () => {
    chart = echarts.init(container, "dark");

    await fetchAndCacheData();

    unlistenSnirf = await listen("snirf-loaded", async () => {
      await fetchAndCacheData();
    });

    unlistenChannels = await listen("channels-selected", (event) => {
      selectedIds = event.payload.channel_ids;
      updateChart();
    });

    // Re-sync cursor line after zoom/pan since pixel coordinates shift
    chart.on("dataZoom", () => {
      syncCursorLinePosition();
    });

    resizeObserver = new ResizeObserver(debouncedResize);
    resizeObserver.observe(wrapper);
  });

  onDestroy(() => {
    if (unlistenSnirf) unlistenSnirf();
    if (unlistenChannels) unlistenChannels();
    if (resizeObserver) resizeObserver.disconnect();
    if (chart) chart.dispose();
  });
</script>

<div class="plotter-root">
  <div class="toolbar">
    <button
      class="toggle-btn"
      class:active={stacked}
      on:click={() => onToggle(true)}
    >
      Stacked
    </button>
    <button
      class="toggle-btn"
      class:active={!stacked}
      on:click={() => onToggle(false)}
    >
      Unstacked
    </button>
    <span class="channel-count">
      {selectedIds.length} channel{selectedIds.length !== 1 ? "s" : ""} plotted
    </span>

    {#if eventTypes.length > 0}
      <div class="toolbar-sep"></div>
      <button
        class="toggle-btn"
        class:active={showEventsPanel}
        on:click={() => (showEventsPanel = !showEventsPanel)}
      >
        Events ({eventTypes.filter((e) => e.visible)
          .length}/{eventTypes.length})
      </button>
    {/if}

    {#if allData}
      <div class="toolbar-sep"></div>
      <span class="cursor-label">t (s):</span>
      <input
        class="cursor-input"
        type="text"
        bind:value={cursorInputValue}
        on:keydown={onCursorKeydown}
        on:blur={submitCursorTime}
        placeholder="0.000"
      />
      <span class="cursor-range">/ {maxTime.toFixed(1)}</span>
    {/if}
  </div>

  {#if showEventsPanel && eventTypes.length > 0}
    <div class="events-panel">
      {#each eventTypes as et}
        <label class="event-toggle">
          <input
            type="checkbox"
            checked={et.visible}
            on:change={() => toggleEventType(et.name)}
          />
          <span class="event-swatch" style="background:{et.color}"></span>
          <span class="event-label">{et.name}</span>
          <span class="event-count"
            >{allData?.events?.find((e) => e.name === et.name)?.markers
              .length ?? 0}</span
          >
        </label>
      {/each}
    </div>
  {/if}
  <div class="chart-wrapper" bind:this={wrapper}>
    <div class="chart-container" bind:this={container}></div>
  </div>
</div>

<style>
  .plotter-root {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .toggle-btn {
    padding: 3px 10px;
    font-size: 12px;
    border: 1px solid var(--border-default);
    border-radius: 4px;
    background: var(--bg-raised);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s;
  }

  .toggle-btn:hover {
    background: var(--bg-overlay);
    color: var(--text-secondary);
  }

  .toggle-btn.active {
    background: var(--bg-overlay);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .channel-count {
    margin-left: 12px;
    font-size: 11px;
    color: var(--text-muted);
  }

  .chart-wrapper {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .chart-container {
    width: 100%;
    height: 100%;
  }

  .toolbar-sep {
    width: 1px;
    height: 18px;
    background: var(--border-default);
    margin: 0 6px;
  }

  .events-panel {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    padding: 4px 8px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .event-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .event-toggle input[type="checkbox"] {
    width: 12px;
    height: 12px;
    margin: 0;
    cursor: pointer;
    accent-color: var(--accent-green);
  }

  .event-swatch {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }

  .event-label {
    color: var(--text-secondary);
  }

  .event-count {
    color: var(--text-muted);
    font-size: 10px;
  }

  .cursor-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .cursor-input {
    width: 72px;
    padding: 2px 6px;
    font-size: 12px;
    font-family: inherit;
    font-variant-numeric: tabular-nums;
    background: var(--bg-base);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    border-radius: 4px;
    outline: none;
  }

  .cursor-input:focus {
    border-color: var(--accent-green);
  }

  .cursor-range {
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
