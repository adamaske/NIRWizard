import { writable } from 'svelte/store';

const defaultTransform = () => ({ position: [0, 0, 0], rotation: [0, 0, 0], scale: [1, 1, 1] });

// Per-layer default opacity
const LAYER_OPACITY = {
  skull:        0.5,
  csf:          0.4,
  grey_matter:  0.7,
  white_matter: 0.9,
};

export function defaultLayerState(layer) {
  return {
    ...defaultTransform(),
    opacity: LAYER_OPACITY[layer] ?? 1.0,
    visible: true,
  };
}

// voxelVolumeStates: plain object keyed by volume name
// Entry shape: { axis:'z', sliceIndex:128, visibleLabels:Set<number>, visible:true }
export const voxelVolumeStates = writable({});

export function defaultVoxelState(info) {
  return {
    axis: 'z',
    sliceIndex: Math.floor(info.dims[2] / 2),
    visibleLabels: new Set(info.labels_present),
    visible: true,
  };
}

// anatomyLayerStates: plain object keyed by layer name, null until anatomy is loaded
// Shape: { [layer]: { position, rotation, scale, opacity, visible } }
export const anatomyLayerStates = writable({});

// optodeState: null until probe is loaded
// Shape: { transform:{position,rotation,scale}, settings:{spread_factor,optode_radius} }
export const optodeState = writable(null);

export const defaultOptodeState = () => ({
  transform: defaultTransform(),
  settings: {
    spread_factor: 1.0,
    optode_radius: 2.0,
  },
  visible: true,
});
