<!--
  DataPlotter.svelte

  Multi-channel ECharts time-series plot with stacked and unstacked modes.
  Listens to `snirf-loaded` (fetch + cache all data) and `channels-selected`
  (update which channels are rendered).
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

  // Cached full timeseries payload (fetched once per file load)
  let allData = null;
  // Currently selected channel IDs
  let selectedIds = [];
  // Display mode
  let stacked = true;

  const GRID_HEIGHT = 120; // px per channel in unstacked mode
  const DOWNSAMPLE_TARGET = 2000; // max points per series sent to ECharts

  // Mirror CSS vars — ECharts config needs raw hex strings
  const HBO_COLOR = "#ff2255"; // --color-hbo
  const HBR_COLOR = "#2266ff"; // --color-hbr
  const CHART_BG = "#0a0a10";  // --chart-bg
  const CHART_AXIS = "#333340"; // --chart-axis
  const CHART_GRID = "#161620"; // --chart-grid

  /**
   * Largest-Triangle-Three-Buckets downsampling.
   * Reduces an array of [x,y] pairs to `target` points while preserving shape.
   */
  function lttb(data, target) {
    const len = data.length;
    if (target >= len || target < 3) return data;

    const out = [data[0]]; // always keep first
    const bucketSize = (len - 2) / (target - 2);

    let prevIndex = 0;
    for (let i = 1; i < target - 1; i++) {
      const avgStart = Math.floor((i + 0) * bucketSize) + 1;
      const avgEnd = Math.min(Math.floor((i + 1) * bucketSize) + 1, len);

      // Average of next bucket (for triangle area)
      let avgX = 0, avgY = 0;
      for (let j = avgStart; j < avgEnd; j++) {
        avgX += data[j][0];
        avgY += data[j][1];
      }
      avgX /= (avgEnd - avgStart);
      avgY /= (avgEnd - avgStart);

      // Current bucket range
      const rangeStart = Math.floor((i - 1) * bucketSize) + 1;
      const rangeEnd = Math.min(Math.floor(i * bucketSize) + 1, len);

      const px = data[prevIndex][0];
      const py = data[prevIndex][1];

      let maxArea = -1;
      let maxIdx = rangeStart;
      for (let j = rangeStart; j < rangeEnd; j++) {
        const area = Math.abs(
          (px - avgX) * (data[j][1] - py) -
          (px - data[j][0]) * (avgY - py)
        );
        if (area > maxArea) {
          maxArea = area;
          maxIdx = j;
        }
      }

      out.push(data[maxIdx]);
      prevIndex = maxIdx;
    }

    out.push(data[len - 1]); // always keep last
    return out;
  }

  /** Downsample a y-array against the shared time array. Returns [x,y] pairs. */
  function downsample(time, values) {
    if (time.length <= DOWNSAMPLE_TARGET) {
      // Small enough — return pairs directly, no copy of string labels
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

  /** Debounce resize to avoid rapid-fire re-renders during panel drag */
  let resizeTimer;
  function debouncedResize() {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => { if (chart) chart.resize(); }, 60);
  }

  /** Filter cached data to selected IDs, build ECharts option, setOption */
  function updateChart() {
    if (!chart || !allData) return;

    const channels = selectedIds.length > 0
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
        },
        true,
      );
      if (wrapper) wrapper.style.overflowY = "hidden";
      container.style.height = "100%";
      chart.resize();
      return;
    }

    const time = allData.time; // numeric — no .toFixed() copies

    if (stacked) {
      buildStacked(channels, time);
    } else {
      buildUnstacked(channels, time);
    }
  }

  /** Shared series options for performance */
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

    channels.forEach((ch) => {
      const hboName = `${ch.name} HbO`;
      const hbrName = `${ch.name} HbR`;
      legendData.push(hboName, hbrName);

      series.push({
        ...PERF_SERIES,
        name: hboName,
        data: downsample(time, ch.hbo),
        lineStyle: { color: HBO_COLOR, width: 1.5 },
        itemStyle: { color: HBO_COLOR },
      });
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
          : {
              trigger: "axis",
              axisPointer: { type: "cross" },
            },
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
            name: "ΔC (M)",
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
        dataZoom: [
          { type: "inside", xAxisIndex: 0 },
        ],
        series,
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

      grids.push({
        left: 80,
        right: 30,
        top: gridTop,
        height: gridHeight,
      });

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

      series.push({
        ...PERF_SERIES,
        name: `${ch.name} HbO`,
        data: downsample(time, ch.hbo),
        xAxisIndex: i,
        yAxisIndex: i,
        lineStyle: { color: HBO_COLOR, width: 1.5 },
        itemStyle: { color: HBO_COLOR },
      });
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
        tooltip: manyChannels ? { show: false } : {
          trigger: "axis",
          axisPointer: { type: "cross" },
        },
        legend: { show: false },
        grid: grids,
        xAxis: xAxes,
        yAxis: yAxes,
        dataZoom: [
          { type: "inside", xAxisIndex: xAxisIndices },
        ],
        series,
      },
      true,
    );

    chart.resize();
  }

  async function fetchAndCacheData() {
    const payload = await invoke("get_timeseries_data");
    if (payload) {
      allData = payload;
      // Default: all channels selected
      selectedIds = payload.channels.map((ch) => ch.id);
      updateChart();
    }
  }

  function onToggle(mode) {
    stacked = mode;
    updateChart();
  }

  onMount(async () => {
    chart = echarts.init(container, "dark");

    // Fetch data if a file is already loaded
    await fetchAndCacheData();

    // Refresh chart whenever a new file is loaded
    unlistenSnirf = await listen("snirf-loaded", async () => {
      await fetchAndCacheData();
    });

    // Update selection when ChannelSelector changes
    unlistenChannels = await listen("channels-selected", (event) => {
      selectedIds = event.payload.channel_ids;
      updateChart();
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
  </div>
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
</style>
