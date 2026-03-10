<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  /** @type {import('../../types').SnirfSummary | null} */
  export let summary = null;

  async function loadSnirf() {
    const path = await open({
      multiple: false,
      filters: [{ name: "SNIRF", extensions: ["snirf"] }],
    });
    if (path) {
      await invoke("load_snirf", { path });
    }
  }

  $: dur =
    summary && summary.duration > 0
      ? `${summary.duration.toFixed(2)} s`
      : "--";
  $: rate =
    summary && summary.sampling_rate > 0
      ? `${summary.sampling_rate.toFixed(2)} Hz`
      : "--";
</script>

<div class="infopanel">
  {#if !summary}
    <!-- ── Empty state ── -->
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="12" y1="12" x2="12" y2="18"/>
          <line x1="9" y1="15" x2="15" y2="15"/>
        </svg>
      </div>
      <p class="empty-title">No file loaded</p>
      <p class="empty-hint">Open a SNIRF file to inspect its contents</p>
      <button class="load-btn" on:click={loadSnirf}>Load sNIRF</button>
    </div>
  {:else}
    <!-- ── Loaded state ── -->
    <div class="panel-scroll">
      <!-- File -->
      <section class="card">
        <h2 class="card-title">File</h2>
        <div class="row single">
          <span class="value filename">{summary.filename}</span>
        </div>
      </section>

      <!-- Signal -->
      <section class="card">
        <h2 class="card-title">Signal</h2>
        <div class="grid-2col">
          <div class="kv">
            <span class="key">Channels</span>
            <span class="value">{summary.channels}</span>
          </div>
          <div class="kv">
            <span class="key">Sampling Rate</span>
            <span class="value">{rate}</span>
          </div>
          <div class="kv">
            <span class="key">Sources</span>
            <span class="value">{summary.sources}</span>
          </div>
          <div class="kv">
            <span class="key">Duration</span>
            <span class="value">{dur}</span>
          </div>
          <div class="kv">
            <span class="key">Detectors</span>
            <span class="value">{summary.detectors}</span>
          </div>
          <div class="kv">
            <span class="key">Timepoints</span>
            <span class="value">{summary.timepoints.toLocaleString()}</span>
          </div>
        </div>
      </section>

      <!-- Wavelengths -->
      <section class="card">
        <h2 class="card-title">Wavelengths</h2>
        <div class="grid-2col">
          <div class="kv">
            <span class="key">HbO</span>
            <span class="value wavelength">{summary.hbo_wavelength} nm</span>
          </div>
          <div class="kv">
            <span class="key">HbR</span>
            <span class="value wavelength">{summary.hbr_wavelength} nm</span>
          </div>
        </div>
      </section>

      <!-- Events -->
      <section class="card">
        <h2 class="card-title">
          Events
          <span class="card-badge">{summary.events.length}</span>
        </h2>
        {#if summary.events.length === 0}
          <p class="empty-section">No events recorded</p>
        {:else}
          <div class="event-list">
            {#each summary.events as ev}
              <div class="event-row">
                <span class="event-name">{ev.name}</span>
                <span class="event-count">{ev.marker_count} marker{ev.marker_count !== 1 ? "s" : ""}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Auxiliary -->
      <section class="card">
        <h2 class="card-title">Auxiliary</h2>
        <div class="row single">
          {#if summary.aux_count === 0}
            <span class="empty-section">No auxiliary signals</span>
          {:else}
            <span class="value">{summary.aux_count} signal{summary.aux_count !== 1 ? "s" : ""}</span>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .infopanel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-base);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--text-muted);
  }

  .empty-icon { color: var(--text-faint); margin-bottom: 4px; }
  .empty-title { margin: 0; font-size: 15px; color: var(--text-secondary); font-weight: 500; }
  .empty-hint { margin: 0; font-size: 12px; color: var(--text-muted); }

  .load-btn {
    margin-top: 12px;
    padding: 10px 28px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-raised);
    color: var(--text-secondary);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    cursor: pointer;
    letter-spacing: 0.3px;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }

  .load-btn:hover {
    background: var(--bg-overlay);
    color: var(--text-primary);
    border-color: var(--accent-green);
  }

  .panel-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .panel-scroll::-webkit-scrollbar { width: 6px; }
  .panel-scroll::-webkit-scrollbar-track { background: transparent; }
  .panel-scroll::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 3px; }

  .card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 14px 18px;
  }

  .card-title {
    margin: 0 0 12px 0;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1px;
    text-transform: uppercase;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .card-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: 9px;
    font-size: 10px;
    color: var(--text-muted);
    letter-spacing: 0;
  }

  .grid-2col { display: grid; grid-template-columns: 1fr 1fr; gap: 10px 24px; }

  .kv { display: flex; flex-direction: column; gap: 2px; }

  .key { font-size: 10px; color: var(--text-faint); text-transform: uppercase; letter-spacing: 0.6px; }

  .value { font-size: 14px; color: var(--text-primary); font-variant-numeric: tabular-nums; }

  .filename { font-size: 13px; color: var(--text-secondary); word-break: break-all; }

  .wavelength { font-size: 14px; color: var(--text-primary); }

  .row.single { display: flex; align-items: center; }

  .event-list { display: flex; flex-direction: column; gap: 6px; }

  .event-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 10px;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: 5px;
  }

  .event-name { font-size: 13px; color: var(--text-primary); font-family: "Cascadia Code", "Consolas", monospace; }

  .event-count { font-size: 11px; color: var(--text-muted); font-variant-numeric: tabular-nums; }

  .empty-section { font-size: 12px; color: var(--text-faint); margin: 0; font-style: italic; }
</style>
