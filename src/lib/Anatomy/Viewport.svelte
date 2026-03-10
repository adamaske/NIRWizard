<script>
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import {
    anatomyLayerStates,
    defaultLayerState,
    optodeState,
    defaultOptodeState,
    voxelVolumeStates,
    defaultVoxelState,
  } from "../stores/sceneState.js";
  import { labelToColor } from "../utils/colormap.js";

  let containerEl;
  let renderer, scene, camera, controls, animId, ro;
  const unlistenFns = [];
  const storeUnsubs = [];

  const layerMeshes  = new Map();
  const voxelObjects = new Map();
  let optodeGroup    = null;
  let channelLines   = null;
  let cachedLayout   = null;
  let cameraFitted   = false;
  let selectedChannelIds = new Set();

  const LAYER_MATERIAL = {
    white_matter: { color: 0xeeeecc, opacity: 0.9,  renderOrder: 0, visibleByDefault: false },
    grey_matter:  { color: 0x886644, opacity: 1.0,  renderOrder: 1, visibleByDefault: true  },
    csf:          { color: 0x8888ff, opacity: 0.4,  renderOrder: 2, visibleByDefault: false },
    skull:        { color: 0xf0e0d0, opacity: 0.35, renderOrder: 3, visibleByDefault: true  },
  };

  function buildBufferGeometry(payload) {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(payload.positions), 3));
    geo.setIndex(new THREE.BufferAttribute(new Uint32Array(payload.indices), 1));
    geo.computeVertexNormals();
    return geo;
  }

  function applyTransformToObject(obj, t) {
    obj.position.set(...t.position);
    obj.rotation.set(
      THREE.MathUtils.degToRad(t.rotation[0]),
      THREE.MathUtils.degToRad(t.rotation[1]),
      THREE.MathUtils.degToRad(t.rotation[2]),
    );
    obj.scale.set(...t.scale);
  }

  function autoFitCamera() {
    if (cameraFitted) return;
    const box = new THREE.Box3();
    for (const mesh of layerMeshes.values()) box.expandByObject(mesh);
    for (const { mesh } of voxelObjects.values()) if (mesh.count > 0) box.expandByObject(mesh);
    if (optodeGroup) box.expandByObject(optodeGroup);
    if (box.isEmpty()) return;
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    const maxDim = Math.max(size.x, size.y, size.z);
    if (maxDim < 0.0001) return;
    camera.position.copy(center);
    camera.position.z += maxDim * 2;
    controls.target.copy(center);
    controls.update();
    cameraFitted = true;
  }

  function buildOptodeGroup(settings) {
    if (!cachedLayout) return;
    if (optodeGroup) scene.remove(optodeGroup);
    optodeGroup = new THREE.Group();
    const sf = settings.spread_factor;
    const r = settings.optode_radius;
    const [tx, ty, tz] = cachedLayout.settings.projection_target;
    const target = new THREE.Vector3(tx, ty, tz);
    const cylUp  = new THREE.Vector3(0, 1, 0);
    const height = r * 3;
    const cylGeo = new THREE.CylinderGeometry(r, r, height, 12);

    function makeCylinder(x, y, z, color) {
      const pos     = new THREE.Vector3(x * sf, y * sf, z * sf);
      const outward = pos.clone().sub(target).normalize();
      const mesh    = new THREE.Mesh(cylGeo, new THREE.MeshPhongMaterial({ color, shininess: 60 }));
      mesh.position.copy(pos).addScaledVector(outward, height / 2);
      mesh.quaternion.setFromUnitVectors(cylUp, outward);
      mesh.renderOrder = 1;
      return mesh;
    }

    for (const o of cachedLayout.sources) {
      const [x, y, z] = o.position;
      optodeGroup.add(makeCylinder(x, y, z, 0xdd3333));
    }
    for (const o of cachedLayout.detectors) {
      const [x, y, z] = o.position;
      optodeGroup.add(makeCylinder(x, y, z, 0x3355dd));
    }

    const linePoints = [], lineColors = [];
    const GREY   = [0x6e / 255, 0x6e / 255, 0x8a / 255];
    const YELLOW = [1.0, 0xdd / 255, 0.0];

    for (const ch of cachedLayout.channels) {
      const src = cachedLayout.sources[ch.source_idx];
      const det = cachedLayout.detectors[ch.detector_idx];
      if (!src || !det) continue;
      const [sx, sy, sz] = src.position;
      const [dx, dy, dz] = det.position;
      linePoints.push(sx * sf, sy * sf, sz * sf, dx * sf, dy * sf, dz * sf);
      const c = selectedChannelIds.has(ch.id) ? YELLOW : GREY;
      lineColors.push(...c, ...c);
    }

    if (linePoints.length > 0) {
      const lineGeo = new THREE.BufferGeometry();
      lineGeo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(linePoints), 3));
      lineGeo.setAttribute("color",    new THREE.BufferAttribute(new Float32Array(lineColors), 3));
      channelLines = new THREE.LineSegments(lineGeo, new THREE.LineBasicMaterial({ vertexColors: true }));
      optodeGroup.add(channelLines);
    }

    scene.add(optodeGroup);
    autoFitCamera();
  }

  async function loadLayerIntoScene(layer) {
    const payload = await invoke("get_anatomy_geometry", { layer }).catch((e) => {
      console.warn(`[Viewport] get_anatomy_geometry(${layer}) failed:`, e);
      return null;
    });
    if (!payload) return;

    const existing = layerMeshes.get(layer);
    if (existing) scene.remove(existing);

    const mat = LAYER_MATERIAL[layer] ?? { color: 0x888888, opacity: 1.0, renderOrder: 0, visibleByDefault: true };
    const isTransparent = mat.opacity < 1.0;
    const mesh = new THREE.Mesh(
      buildBufferGeometry(payload),
      new THREE.MeshPhongMaterial({
        color: mat.color, transparent: isTransparent, opacity: mat.opacity,
        shininess: 40, side: isTransparent ? THREE.DoubleSide : THREE.FrontSide,
        depthWrite: !isTransparent, depthTest: true, premultipliedAlpha: false,
      }),
    );
    mesh.renderOrder = mat.renderOrder ?? 0;
    mesh.visible = mat.visibleByDefault ?? true;
    layerMeshes.set(layer, mesh);
    scene.add(mesh);

    const current = get(anatomyLayerStates);
    if (!current[layer]) {
      const state = defaultLayerState(layer);
      state.visible = mat.visibleByDefault ?? true;
      anatomyLayerStates.update((m) => ({ ...m, [layer]: state }));
    }
    autoFitCamera();
  }

  async function loadAnatomyLayers(layers) {
    cameraFitted = false;
    for (const layer of layers) await loadLayerIntoScene(layer);
  }

  async function initVoxelVolume(name) {
    const info = await invoke("get_voxel_volume_info", { name });
    const m = new THREE.Matrix4().fromArray(info.vox2ras);
    const dx = new THREE.Vector3(m.elements[0], m.elements[1], m.elements[2]).length();
    const dy = new THREE.Vector3(m.elements[4], m.elements[5], m.elements[6]).length();
    const dz = new THREE.Vector3(m.elements[8], m.elements[9], m.elements[10]).length();
    const maxVoxels = info.dims[0] * info.dims[1];
    const geo = new THREE.BoxGeometry(dx, dy, dz);
    const mat = new THREE.MeshPhongMaterial({ vertexColors: false, shininess: 20 });
    const mesh = new THREE.InstancedMesh(geo, mat, maxVoxels);
    mesh.count = 0;
    mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    scene.add(mesh);
    voxelObjects.set(name, { mesh, info, m });
    const current = get(voxelVolumeStates);
    if (!current[name]) voxelVolumeStates.update(s => ({ ...s, [name]: defaultVoxelState(info) }));
  }

  async function renderVoxelSlice(name, state) {
    const entry = voxelObjects.get(name);
    if (!entry || !state || !state.visible) { if (entry) entry.mesh.count = 0; return; }
    const { mesh, info, m } = entry;
    const sliceData = await invoke("get_voxel_slice", { name, axis: state.axis, index: state.sliceIndex });
    const maxLabel = Math.max(...info.labels_present, 1);
    const dummy = new THREE.Object3D(), color = new THREE.Color();
    let count = 0;
    for (let row = 0; row < sliceData.height; row++) {
      for (let col = 0; col < sliceData.width; col++) {
        const label = sliceData.labels[row * sliceData.width + col];
        if (label === 0 || !state.visibleLabels.has(label)) continue;
        let vx, vy, vz;
        if (state.axis === 'x')      { [vx, vy, vz] = [state.sliceIndex, col, row]; }
        else if (state.axis === 'y') { [vx, vy, vz] = [col, state.sliceIndex, row]; }
        else                         { [vx, vy, vz] = [col, row, state.sliceIndex]; }
        dummy.position.copy(new THREE.Vector3(vx, vy, vz).applyMatrix4(m));
        dummy.updateMatrix();
        mesh.setMatrixAt(count, dummy.matrix);
        color.setHex(labelToColor(label, maxLabel));
        mesh.setColorAt(count, color);
        count++;
      }
    }
    mesh.count = count;
    mesh.instanceMatrix.needsUpdate = true;
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
  }

  function updateChannelColors() {
    if (!channelLines || !cachedLayout) return;
    const GREY   = [0x6e / 255, 0x6e / 255, 0x8a / 255];
    const YELLOW = [1.0, 0xdd / 255, 0.0];
    const colors = [];
    for (const ch of cachedLayout.channels) {
      const src = cachedLayout.sources[ch.source_idx];
      const det = cachedLayout.detectors[ch.detector_idx];
      if (!src || !det) continue;
      const c = selectedChannelIds.has(ch.id) ? YELLOW : GREY;
      colors.push(...c, ...c);
    }
    const attr = channelLines.geometry.getAttribute("color");
    attr.array.set(colors);
    attr.needsUpdate = true;
  }

  async function loadOptodeLayoutIntoScene() {
    const layout = await invoke("get_optode_layout_3d");
    if (!layout) return;
    cachedLayout = layout;
    const current = get(optodeState);
    const settings = current ? current.settings : layout.settings;
    buildOptodeGroup(settings);
    if (current !== null) applyTransformToObject(optodeGroup, current.transform);
    else optodeState.set(defaultOptodeState());
  }

  onMount(async () => {
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0f0f1a);
    camera = new THREE.PerspectiveCamera(60, 1, 0.01, 1000);
    camera.position.set(0, 0, 3);
    const light = new THREE.DirectionalLight(0xffffff, 1.2);
    camera.add(light);
    scene.add(camera);
    scene.add(new THREE.AmbientLight(0x404060, 0.5));
    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.sortObjects = true;
    containerEl.appendChild(renderer.domElement);
    const { clientWidth: w, clientHeight: h } = containerEl;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    ro = new ResizeObserver(() => {
      const { clientWidth: rw, clientHeight: rh } = containerEl;
      if (rw > 0 && rh > 0) { renderer.setSize(rw, rh); camera.aspect = rw / rh; camera.updateProjectionMatrix(); }
    });
    ro.observe(containerEl);

    unlistenFns.push(await listen("anatomy-loaded", async (e) => {
      loadAnatomyLayers(e.payload.layers);
      for (const name of (e.payload.voxel_volumes ?? [])) {
        await initVoxelVolume(name);
        const state = get(voxelVolumeStates)[name];
        if (state) renderVoxelSlice(name, state);
      }
    }));
    unlistenFns.push(await listen("snirf-loaded", () => loadOptodeLayoutIntoScene()));
    unlistenFns.push(await listen("channels-selected", (e) => { selectedChannelIds = new Set(e.payload.channel_ids); updateChannelColors(); }));

    storeUnsubs.push(anatomyLayerStates.subscribe((states) => {
      for (const [layer, s] of Object.entries(states)) {
        const mesh = layerMeshes.get(layer);
        if (!mesh || !s) continue;
        applyTransformToObject(mesh, s);
        const isTransparent = s.opacity < 1.0;
        mesh.material.opacity = s.opacity;
        mesh.material.transparent = isTransparent;
        mesh.material.depthWrite = !isTransparent;
        mesh.material.side = isTransparent ? THREE.DoubleSide : THREE.FrontSide;
        mesh.material.needsUpdate = true;
        mesh.visible = s.visible;
      }
    }));
    storeUnsubs.push(voxelVolumeStates.subscribe((states) => {
      for (const [name, state] of Object.entries(states)) renderVoxelSlice(name, state);
    }));
    storeUnsubs.push(optodeState.subscribe((s) => {
      if (!s) return;
      if (optodeGroup) { applyTransformToObject(optodeGroup, s.transform); optodeGroup.visible = s.visible; }
      if (cachedLayout) buildOptodeGroup(s.settings);
    }));

    function animate() { animId = requestAnimationFrame(animate); controls.update(); renderer.render(scene, camera); }
    animate();
  });

  onDestroy(() => {
    cancelAnimationFrame(animId);
    ro?.disconnect();
    controls?.dispose();
    renderer?.dispose();
    for (const { mesh } of voxelObjects.values()) mesh.dispose();
    for (const unlisten of unlistenFns) unlisten();
    for (const unsub of storeUnsubs) unsub();
  });
</script>

<div class="viewport" bind:this={containerEl}></div>

<style>
  .viewport {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: #0f0f1a;
  }

  .viewport :global(canvas) {
    display: block;
  }
</style>
