// Viridis colormap — 256 [r, g, b] entries in 0–255 range.
// Interpolated from the 5 canonical control points.
const VIRIDIS_STOPS = [
  [0.000, [68,   1,  84]],
  [0.250, [59,  82, 139]],
  [0.500, [33, 145, 140]],
  [0.750, [94, 201,  98]],
  [1.000, [253, 231,  37]],
];

function lerp(a, b, t) { return a + (b - a) * t; }

export const VIRIDIS = Array.from({ length: 256 }, (_, i) => {
  const t = i / 255;
  for (let s = 0; s < VIRIDIS_STOPS.length - 1; s++) {
    const [t0, c0] = VIRIDIS_STOPS[s];
    const [t1, c1] = VIRIDIS_STOPS[s + 1];
    if (t <= t1) {
      const u = (t - t0) / (t1 - t0);
      return [
        Math.round(lerp(c0[0], c1[0], u)),
        Math.round(lerp(c0[1], c1[1], u)),
        Math.round(lerp(c0[2], c1[2], u)),
      ];
    }
  }
  return VIRIDIS_STOPS.at(-1)[1];
});

/**
 * Map a label value to a THREE.js hex color using viridis.
 * maxLabel: the highest label value in the volume (for normalisation).
 */
export function labelToColor(label, maxLabel) {
  if (label === 0) return null; // background — skip
  const idx = Math.round((label / Math.max(maxLabel, 1)) * 255);
  const [r, g, b] = VIRIDIS[Math.min(idx, 255)];
  return (r << 16) | (g << 8) | b;
}

/**
 * Return a CSS rgb() string for a label (for UI swatches).
 */
export function labelToCss(label, maxLabel) {
  if (label === 0) return 'transparent';
  const idx = Math.round((label / Math.max(maxLabel, 1)) * 255);
  const [r, g, b] = VIRIDIS[Math.min(idx, 255)];
  return `rgb(${r},${g},${b})`;
}
