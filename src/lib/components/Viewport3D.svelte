<script>
  import { onMount, onDestroy } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let containerEl;
  let renderer, scene, camera, controls, animId, ro;
  const unlistenFns = [];

  // Retained scene object references for removal/replacement
  let cortexMesh = null;
  let scalpMesh = null;
  let optodeGroup = null;
  let cameraFitted = false;

  function buildBufferGeometry(payload) {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(payload.positions), 3),
    );
    geo.setAttribute(
      "normal",
      new THREE.BufferAttribute(new Float32Array(payload.normals), 3),
    );
    geo.setIndex(
      new THREE.BufferAttribute(new Uint32Array(payload.indices), 1),
    );
    return geo;
  }

  function autoFitCamera() {
    if (cameraFitted) return;
    const box = new THREE.Box3();
    for (const obj of [cortexMesh, scalpMesh, optodeGroup]) {
      if (obj) box.expandByObject(obj);
    }
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

  async function loadCortexIntoScene() {
    const payload = await invoke("get_cortex_geometry");
    if (!payload) return;
    if (cortexMesh) scene.remove(cortexMesh);
    cortexMesh = new THREE.Mesh(
      buildBufferGeometry(payload),
      new THREE.MeshStandardMaterial({
        color: 0x886644,
        transparent: true,
        opacity: 0.7,
      }),
    );
    scene.add(cortexMesh);
    autoFitCamera();
  }

  async function loadScalpIntoScene() {
    const payload = await invoke("get_scalp_geometry");
    if (!payload) return;
    if (scalpMesh) scene.remove(scalpMesh);
    scalpMesh = new THREE.Mesh(
      buildBufferGeometry(payload),
      new THREE.MeshStandardMaterial({
        color: 0xddbbaa,
        transparent: true,
        opacity: 0.3,
      }),
    );
    scene.add(scalpMesh);
    autoFitCamera();
  }

  async function loadOptodeLayoutIntoScene() {
    const layout = await invoke("get_optode_layout_3d");
    if (!layout) return;

    if (optodeGroup) scene.remove(optodeGroup);
    optodeGroup = new THREE.Group();

    const sf = layout.settings.spread_factor;
    const r = layout.settings.optode_radius;
    const [tx, ty, tz] = layout.settings.projection_target;
    const target = new THREE.Vector3(tx, ty, tz);
    const sphereGeo = new THREE.SphereGeometry(r, 8, 8);

    for (const o of layout.sources) {
      const mesh = new THREE.Mesh(
        sphereGeo,
        new THREE.MeshStandardMaterial({ color: 0xdd3333 }),
      );
      const [x, y, z] = o.position;
      mesh.position.set(x * sf, y * sf, z * sf);
      mesh.lookAt(target);
      optodeGroup.add(mesh);
    }

    for (const o of layout.detectors) {
      const mesh = new THREE.Mesh(
        sphereGeo,
        new THREE.MeshStandardMaterial({ color: 0x3355dd }),
      );
      const [x, y, z] = o.position;
      mesh.position.set(x * sf, y * sf, z * sf);
      mesh.lookAt(target);
      optodeGroup.add(mesh);
    }

    // Channel lines as a single LineSegments draw call
    const linePoints = [];
    for (const ch of layout.channels) {
      const src = layout.sources[ch.source_idx];
      const det = layout.detectors[ch.detector_idx];
      if (!src || !det) continue;
      const [sx, sy, sz] = src.position;
      const [dx, dy, dz] = det.position;
      linePoints.push(sx * sf, sy * sf, sz * sf, dx * sf, dy * sf, dz * sf);
    }

    if (linePoints.length > 0) {
      const lineGeo = new THREE.BufferGeometry();
      lineGeo.setAttribute(
        "position",
        new THREE.BufferAttribute(new Float32Array(linePoints), 3),
      );
      optodeGroup.add(
        new THREE.LineSegments(
          lineGeo,
          new THREE.LineBasicMaterial({ color: 0x6e6e8a }),
        ),
      );
    }

    scene.add(optodeGroup);
    autoFitCamera();
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

    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    containerEl.appendChild(renderer.domElement);

    const { clientWidth: w, clientHeight: h } = containerEl;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();

    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;

    ro = new ResizeObserver(() => {
      const { clientWidth: rw, clientHeight: rh } = containerEl;
      if (rw > 0 && rh > 0) {
        renderer.setSize(rw, rh);
        camera.aspect = rw / rh;
        camera.updateProjectionMatrix();
      }
    });
    ro.observe(containerEl);

    // Register event listeners; store unlisten fns for cleanup
    unlistenFns.push(await listen("cortex-loaded", () => loadCortexIntoScene()));
    unlistenFns.push(await listen("scalp-loaded", () => loadScalpIntoScene()));
    unlistenFns.push(
      await listen("snirf-loaded", () => loadOptodeLayoutIntoScene()),
    );

    function animate() {
      animId = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    }
    animate();
  });

  onDestroy(() => {
    cancelAnimationFrame(animId);
    ro?.disconnect();
    controls?.dispose();
    renderer?.dispose();
    for (const unlisten of unlistenFns) unlisten();
  });
</script>

<div class="viewport3d" bind:this={containerEl}></div>

<style>
  .viewport3d {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: #0f0f1a;
  }

  .viewport3d :global(canvas) {
    display: block;
  }
</style>
