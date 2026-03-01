<!--
  DataPlotter.svelte

  Basic ECharts integration. The pattern:
  1. bind:this to get a DOM element reference
  2. onMount to initialize ECharts on that element
  3. onDestroy to clean up
  4. ResizeObserver to handle panel resizing
-->
<script>
  import { onMount, onDestroy } from "svelte";
  import * as echarts from "echarts";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let container;
  let chart;
  let resizeObserver;
  let unlistenSnirf;

  function setChartData(time, hbo, hbr) {
    chart.setOption({
      xAxis: { data: time.map((t) => t.toFixed(1)) },
      series: [
        { name: "HbO", data: hbo },
        { name: "HbR", data: hbr },
      ],
    });
  }

  onMount(async () => {
    chart = echarts.init(container, "dark");

    // Set static chart options once
    chart.setOption({
      backgroundColor: "#0d0d18",
      tooltip: {
        trigger: "axis",
        axisPointer: { type: "cross" },
      },
      legend: {
        data: ["HbO", "HbR"],
        textStyle: { color: "#999" },
        top: 8,
      },
      grid: {
        left: 70,
        right: 30,
        top: 50,
        bottom: 70,
      },
      xAxis: {
        type: "category",
        data: [],
        name: "Time (s)",
        nameLocation: "middle",
        nameGap: 30,
        axisLabel: {
          interval: 49,
          color: "#888",
        },
        axisLine: { lineStyle: { color: "#333" } },
      },
      yAxis: {
        type: "value",
        name: "ΔC (M)",
        nameLocation: "middle",
        nameGap: 50,
        axisLabel: {
          color: "#888",
          formatter: (val) => val.toExponential(1),
        },
        axisLine: { lineStyle: { color: "#333" } },
        splitLine: { lineStyle: { color: "#1a1a2e" } },
      },
      dataZoom: [
        {
          type: "inside",
          xAxisIndex: 0,
        },
        {
          type: "slider",
          xAxisIndex: 0,
          bottom: 10,
          height: 20,
          borderColor: "#333",
          backgroundColor: "#111",
          fillerColor: "rgba(100, 100, 200, 0.15)",
          textStyle: { color: "#888" },
        },
      ],
      series: [
        {
          name: "HbO",
          type: "line",
          data: [],
          symbol: "none",
          lineStyle: { color: "#ff2255", width: 1.5 },
          itemStyle: { color: "#ff2255" },
        },
        {
          name: "HbR",
          type: "line",
          data: [],
          symbol: "none",
          lineStyle: { color: "#2266ff", width: 1.5 },
          itemStyle: { color: "#2266ff" },
        },
      ],
    });

    // Fetch real data if a file is already loaded
    const payload = await invoke("get_timeseries_data");
    if (payload) {
      const ch = payload.channels[0];
      setChartData(payload.time, ch.hbo, ch.hbr);
    }

    // Refresh chart whenever a new file is loaded
    unlistenSnirf = await listen("snirf-loaded", async () => {
      const payload = await invoke("get_timeseries_data");
      if (payload) {
        const ch = payload.channels[0];
        setChartData(payload.time, ch.hbo, ch.hbr);
      }
    });

    resizeObserver = new ResizeObserver(() => chart.resize());
    resizeObserver.observe(container);
  });

  onDestroy(() => {
    if (unlistenSnirf) unlistenSnirf();
    if (resizeObserver) resizeObserver.disconnect();
    if (chart) chart.dispose();
  });
</script>

<div class="plotter" bind:this={container}></div>

<style>
  .plotter {
    width: 100%;
    height: 100%;
    min-height: 200px;
  }
</style>
