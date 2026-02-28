<!--

-->

<script>
    // 'export let' declares a prop i.e data passed in from the parent
    // The parent uses: <StatusBar filename="test.snirf" channels={24} />
    // Default values is used when no params are passed;

    export let filename = "No file loaded";
    export let channels = 0;
    export let samplingRate = 0;
    export let duration = 0;

    // $: creates a reactive declaration, i.e. it updates when changed
    $: formattedDuration = duration > 0 ? `${duration.toFixed(1)}s` : "--";
    $: formattedRate =
        samplingRate > 0 ? `${samplingRate.toFixed(1)} Hz` : "--";
</script>

<footer class="statusbar">
    <div class="status-section">
        <span class="status-icon">📄</span>
        <span>{filename}</span>
    </div>

    <div class="status-divider"></div>

    <div class="status-section">
        <span class="status-label">Channels:</span>
        <span>{channels || "--"}</span>
    </div>

    <div class="status-divider"></div>

    <div class="status-section">
        <span class="status-label">Rate:</span>
        <span>{formattedRate}</span>
    </div>

    <div class="status-divider"></div>

    <div class="status-section">
        <span class="status-label">Duration:</span>
        <span>{formattedDuration}</span>
    </div>

    <!-- Spacer pushes the right side to the edge -->
    <div class="spacer"></div>

    <div class="status-section">
        <span class="ready-dot"></span>
        <span>Ready</span>
    </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    height: 24px;
    background: #1a1a2e;
    border-top: 1px solid #2a2a3e;
    padding: 0 12px;
    font-size: 12px;
    color: #8888a8;
    flex-shrink: 0;
    gap: 8px;
  }

  .status-section {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-label {
    color: #666688;
  }

  .status-icon {
    font-size: 11px;
  }

  .status-divider {
    width: 1px;
    height: 12px;
    background: #2a2a3e;
  }

  .spacer {
    flex: 1;
  }

  .ready-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #44cc66;
  }
</style>