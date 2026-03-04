import { writable } from 'svelte/store';

// Default transform: identity
const defaultTransform = () => ({ position: [0, 0, 0], rotation: [0, 0, 0], scale: [1, 1, 1] });

// cortexState / scalpState: null until the object is loaded
// Shape: { position:[x,y,z], rotation:[x,y,z], scale:[x,y,z], opacity:f, visible:bool }
export const cortexState = writable(null);
export const scalpState  = writable(null);

export const defaultCortexState = () => ({
  ...defaultTransform(),
  opacity: 0.7,
  visible: true,
});

export const defaultScalpState = () => ({
  ...defaultTransform(),
  opacity: 0.3,
  visible: true,
});

// optodeState: null until probe is loaded
// Shape: { transform:{position,rotation,scale}, settings:{spread_factor,optode_radius} }
export const optodeState = writable(null);

export const defaultOptodeState = () => ({
  transform: defaultTransform(),
  settings: {
    spread_factor: 1.0,
    optode_radius: 0.005,
  },
});
