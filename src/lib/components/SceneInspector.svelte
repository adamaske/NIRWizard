<script>
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { anatomyLayerStates, optodeState, defaultOptodeState } from '../stores/sceneState.js';
  import SceneObjectEditor from './SceneObjectEditor.svelte';
  import OptodeLayoutEditor from './OptodeLayoutEditor.svelte';

  // Display order and labels for anatomy layers
  const LAYER_ORDER  = ['skull', 'csf', 'grey_matter', 'white_matter'];
  const LAYER_LABELS = {
    skull:        'Skull',
    csf:          'CSF',
    grey_matter:  'Grey Matter',
    white_matter: 'White Matter',
  };

  let hasProbe = false;
  let probeExpanded = true;
  let layerExpanded = {};  // layer → bool
  let localProbe = null;

  const unlistenFns = [];
  const unsubProbe = optodeState.subscribe(s => { localProbe = s; });

  // Derive ordered list of loaded layers from store
  $: loadedLayers = LAYER_ORDER.filter(l => $anatomyLayerStates[l] != null);

  onMount(async () => {
    // anatomy-loaded just tells us layers are ready; the store is populated by Viewport3D
    unlistenFns.push(await listen('anatomy-loaded', (e) => {
      for (const layer of e.payload.layers) {
        if (layerExpanded[layer] === undefined) layerExpanded[layer] = true;
        // Reassign to trigger Svelte reactivity on the object
        layerExpanded = layerExpanded;
      }
    }));
    unlistenFns.push(await listen('snirf-loaded', () => { hasProbe = true; }));
  });

  onDestroy(() => {
    for (const u of unlistenFns) u();
    unsubProbe();
  });

  function onLayerChange(layer, e) {
    const s = e.detail;
    anatomyLayerStates.update(m => ({ ...m, [layer]: s }));
    invoke('set_anatomy_transform', {
      layer,
      position: s.position,
      rotation: s.rotation,
      scale: s.scale,
    }).catch(console.error);
    invoke('set_anatomy_opacity', {
      layer,
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

  {#each loadedLayers as layer (layer)}
    <section>
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <div class="section-header" on:click={() => { layerExpanded[layer] = !layerExpanded[layer]; layerExpanded = layerExpanded; }}>
        <span class="chevron">{layerExpanded[layer] ? '▾' : '▸'}</span>
        {LAYER_LABELS[layer] ?? layer}
      </div>
      {#if layerExpanded[layer]}
        <div class="section-body">
          <SceneObjectEditor state={$anatomyLayerStates[layer]} on:change={e => onLayerChange(layer, e)} />
        </div>
      {/if}
    </section>
  {/each}

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

  {#if loadedLayers.length === 0 && !hasProbe}
    <div class="empty">Load a SNIRF or MRI file to inspect scene objects.</div>
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
