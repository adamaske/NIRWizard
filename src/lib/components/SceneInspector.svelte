<script>
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { cortexState, scalpState, optodeState,
           defaultCortexState, defaultScalpState, defaultOptodeState } from '../stores/sceneState.js';
  import SceneObjectEditor from './SceneObjectEditor.svelte';
  import OptodeLayoutEditor from './OptodeLayoutEditor.svelte';

  let hasCortex = false;
  let hasScalp  = false;
  let hasProbe  = false;

  let cortexExpanded = true;
  let scalpExpanded  = true;
  let probeExpanded  = true;

  let localCortex = null;
  let localScalp  = null;
  let localProbe  = null;

  const unlistenFns = [];

  // Keep local copies in sync with stores (for display)
  const unsubCortex = cortexState.subscribe(s => { localCortex = s; });
  const unsubScalp  = scalpState.subscribe(s => { localScalp  = s; });
  const unsubProbe  = optodeState.subscribe(s => { localProbe  = s; });

  onMount(async () => {
    unlistenFns.push(await listen('cortex-loaded', () => { hasCortex = true; }));
    unlistenFns.push(await listen('scalp-loaded',  () => { hasScalp  = true; }));
    unlistenFns.push(await listen('snirf-loaded',  () => { hasProbe  = true; }));
  });

  onDestroy(() => {
    for (const u of unlistenFns) u();
    unsubCortex();
    unsubScalp();
    unsubProbe();
  });

  function onCortexChange(e) {
    const s = e.detail;
    cortexState.set(s);
    invoke('set_cortex_transform', {
      position: s.position,
      rotation: s.rotation,
      scale: s.scale,
    }).catch(console.error);
    invoke('set_cortex_opacity', {
      opacity: s.opacity,
      visible: s.visible,
    }).catch(console.error);
  }

  function onScalpChange(e) {
    const s = e.detail;
    scalpState.set(s);
    invoke('set_scalp_transform', {
      position: s.position,
      rotation: s.rotation,
      scale: s.scale,
    }).catch(console.error);
    invoke('set_scalp_opacity', {
      opacity: s.opacity,
      visible: s.visible,
    }).catch(console.error);
  }

  function onOptodeChange(e) {
    const s = e.detail;
    optodeState.set(s);
    invoke('set_optode_layout_transform', {
      position: s.transform.position,
      rotation: s.transform.rotation,
      scale: s.transform.scale,
    }).catch(console.error);
    invoke('set_optode_layout_settings', {
      spreadFactor: s.settings.spread_factor,
      optodeRadius: s.settings.optode_radius,
    }).catch(console.error);
  }
</script>

<aside class="scene-inspector">
  <div class="inspector-title">Scene Inspector</div>

  {#if hasCortex && localCortex}
    <section>
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <div class="section-header" on:click={() => (cortexExpanded = !cortexExpanded)}>
        <span class="chevron">{cortexExpanded ? '▾' : '▸'}</span>
        Cortex
      </div>
      {#if cortexExpanded}
        <div class="section-body">
          <SceneObjectEditor state={localCortex} on:change={onCortexChange} />
        </div>
      {/if}
    </section>
  {/if}

  {#if hasScalp && localScalp}
    <section>
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <div class="section-header" on:click={() => (scalpExpanded = !scalpExpanded)}>
        <span class="chevron">{scalpExpanded ? '▾' : '▸'}</span>
        Scalp
      </div>
      {#if scalpExpanded}
        <div class="section-body">
          <SceneObjectEditor state={localScalp} on:change={onScalpChange} />
        </div>
      {/if}
    </section>
  {/if}

  {#if hasProbe && localProbe}
    <section>
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <div class="section-header" on:click={() => (probeExpanded = !probeExpanded)}>
        <span class="chevron">{probeExpanded ? '▾' : '▸'}</span>
        Probe
      </div>
      {#if probeExpanded}
        <div class="section-body">
          <OptodeLayoutEditor state={localProbe} on:change={onOptodeChange} />
        </div>
      {/if}
    </section>
  {/if}

  {#if !hasCortex && !hasScalp && !hasProbe}
    <div class="empty">Load a SNIRF or OBJ file to inspect scene objects.</div>
  {/if}
</aside>

<style>
  .scene-inspector {
    width: 220px;
    flex-shrink: 0;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    overflow-x: hidden;
    font-size: 11px;
    color: var(--text-primary);
  }

  .inspector-title {
    padding: 8px 10px 6px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  section {
    border-bottom: 1px solid var(--border-subtle);
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    user-select: none;
    color: var(--text-secondary);
    transition: background 0.1s;
  }

  .section-header:hover {
    background: var(--bg-base);
  }

  .chevron {
    font-size: 9px;
    color: var(--text-muted);
    width: 10px;
  }

  .section-body {
    padding: 6px 10px 10px;
  }

  .empty {
    padding: 16px 10px;
    font-size: 10px;
    color: var(--text-muted);
    line-height: 1.5;
  }
</style>
