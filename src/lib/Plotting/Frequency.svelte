<script>
  import { onMount, onDestroy } from "svelte";
  import * as echarts from "echarts";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let container;
  let chart;
  let resizeObserver;
  let error = null;
  let unlisten;

  let method = "FFT";
  let window = "hann";

  async function fetchAndRender() {
    error = null;
    try {
      const spectra = await invoke("get_spectrums", { method: method.toLowerCase(), window });
      if (!spectra.length) {
        chart.clear();
        return;
      }

      const manyChannels = spectra.length > 20;
      const series = spectra.map((s) => ({
        name: `${s.channel_name} ${s.label}`,
        type: "line",
        data: s.frequencies.map((f, i) => [f, s.magnitudes[i]]),
        symbol: "none",
        lineStyle: { width: 1 },
      }));

      chart.setOption({
        backgroundColor: "transparent",
        animation: false,
        legend: {
          show: spectra.length > 1,
          type: "scroll",
          top: 4,
          right: 100,
          textStyle: { color: "#a0a0b8", fontSize: 10 },
          pageTextStyle: { color: "#a0a0b8" },
          pageIconColor: "#a0a0b8",
          pageIconInactiveColor: "#3a3a4e",
        },
        grid: { top: 38, right: 16, bottom: 36, left: 64 },
        xAxis: {
          type: "value",
          name: "Hz",
          nameLocation: "end",
          nameTextStyle: { color: "#a0a0b8", fontSize: 10 },
          axisLabel: { color: "#a0a0b8", fontSize: 10 },
          axisLine: { lineStyle: { color: "#2a2a3e" } },
          splitLine: { show: false },
        },
        yAxis: {
          type: method === "PSD" ? "log" : "value",
          axisLabel: {
            color: "#a0a0b8",
            fontSize: 10,
            ...(method === "PSD" ? { formatter: (v) => v.toExponential(0) } : {}),
          },
          splitLine: { lineStyle: { color: "#1c1c2e" } },
        },
        tooltip: manyChannels
          ? { show: false }
          : {
              trigger: "axis",
              formatter: (params) =>
                params.map((p) => `${p.seriesName}<br/>${p.data[0].toFixed(3)} Hz — ${p.data[1].toFixed(4)}`).join("<br/>"),
            },
        toolbox: {
          right: 8,
          top: 4,
          itemSize: 14,
          feature: {
            dataZoom: { yAxisIndex: 0, title: { zoom: "Box zoom", back: "Undo zoom" } },
            restore: { title: "Reset" },
          },
          iconStyle: { borderColor: "#a0a0b8" },
          emphasis: { iconStyle: { borderColor: "#ffffff" } },
        },
        dataZoom: [
          { type: "inside", xAxisIndex: 0, filterMode: "none" },
          { type: "inside", yAxisIndex: 0, filterMode: "none" },
        ],
        series,
      }, { replaceMerge: ["series"] });
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    chart = echarts.init(container, null, { renderer: "canvas" });
    resizeObserver = new ResizeObserver(() => chart.resize());
    resizeObserver.observe(container);
    fetchAndRender();

    unlisten = await listen("snirf-loaded", fetchAndRender);
    await listen("block-changed", fetchAndRender);
    await listen("channels-selected", fetchAndRender);
  });

  onDestroy(() => {
    unlisten?.();
    resizeObserver?.disconnect();
    chart?.dispose();
  });
</script>

<div class="root">
  {#if error}
    <div class="error">{error}</div>
  {/if}
  <div class="chart" bind:this={container}></div>
  <div class="toolbar">
    <span class="label">Frequency spectrum</span>
    <label>Method
      <select bind:value={method} onchange={fetchAndRender}>
        <option value="FFT">FFT</option>
        <option value="PSD">PSD</option>
      </select>
    </label>
    <label>Window
      <select bind:value={window} onchange={fetchAndRender}>
        <option value="hann">Hann</option>
        <option value="hamming">Hamming</option>
        <option value="blackman">Blackman</option>
      </select>
    </label>
    <button onclick={fetchAndRender}>Refresh</button>
  </div>
</div>

<style>
  .root {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-base);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-top: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .label {
    font-size: 11px;
    color: var(--text-muted);
    flex: 1;
  }
  label {
    font-size: 11px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 4px;
  }
  select {
    font-size: 11px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    color: var(--text-primary);
    border-radius: 3px;
    padding: 1px 4px;
    cursor: pointer;
  }
  button {
    font-size: 11px;
    padding: 2px 8px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    color: var(--text-primary);
    border-radius: 3px;
    cursor: pointer;
  }
  button:hover, select:hover { background: var(--bg-overlay); }
  .chart { flex: 1; min-height: 0; }
  .error {
    padding: 6px 8px;
    font-size: 11px;
    color: #ff6b6b;
    background: #1a0a0a;
  }
</style>
