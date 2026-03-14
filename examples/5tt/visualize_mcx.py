"""
visualize_mcx.py
Loads and visualizes MCX simulation output (.jnii fluence volume).
Also diagnoses the source/detector placement against the 5tt volume.

Usage:
    python visualize_mcx.py <session_id.jnii> [--vol 5tt_2mm.bin --dim 43 43 43]

Example:
    python visualize_mcx.py fnirs_5tt.jnii
    python visualize_mcx.py fnirs_5tt.jnii --vol 5tt_2mm.bin --dim 43 43 43

Dependencies:
    pip install jdata numpy matplotlib nibabel
"""

import argparse
import json
import os
import sys

import matplotlib.gridspec as gridspec
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.colors import LogNorm
from matplotlib.widgets import Slider

try:
    import jdata as jd
except ImportError:
    print("[ERROR] jdata not installed. Run: pip install jdata")
    sys.exit(1)


TISSUE_NAMES = {0: "BG", 1: "WM", 2: "GM", 3: "CSF", 4: "Skull", 5: "Scalp"}
TISSUE_COLORS = {
    0: "#111",
    1: "#f5f5dc",
    2: "#c08080",
    3: "#4488ee",
    4: "#d4bb88",
    5: "#e8b898",
}


# ── Load helpers ──────────────────────────────────────────────────────────────


def load_jnii(path):
    """Load a .jnii file and return the fluence 4D array (x,y,z,tgate)."""
    data = jd.load(path)
    # The volumetric data lives under NIFTIData
    if "NIFTIData" in data:
        arr = np.array(data["NIFTIData"], dtype=np.float32)
    elif isinstance(data, np.ndarray):
        arr = data.astype(np.float32)
    else:
        # Try walking the dict for the largest numeric array
        def find_array(d):
            if isinstance(d, np.ndarray):
                return d
            if isinstance(d, dict):
                for v in d.values():
                    r = find_array(v)
                    if r is not None:
                        return r
            return None

        arr = find_array(data)
        if arr is None:
            raise ValueError("Could not locate fluence array in .jnii file")
        arr = arr.astype(np.float32)
    return arr


def load_bin(path, dim):
    """Load raw MCX binary volume (uint8, Fortran order)."""
    raw = np.fromfile(path, dtype=np.uint8)
    return raw.reshape(dim, order="F")


# ── Source/detector diagnostic ────────────────────────────────────────────────


def diagnose_placement(vol, sim_json_path):
    """Check what tissue labels are at source/detector positions."""
    try:
        with open(sim_json_path) as f:
            cfg = json.load(f)
        src = cfg["Optode"]["Source"]["Pos"]
        dets = cfg["Optode"]["Detector"]
        print("\n[DIAGNOSTIC] Source/Detector tissue labels:")
        sx, sy, sz = (
            int(src[0]) - 1,
            int(src[1]) - 1,
            int(src[2]) - 1,
        )  # MCX is 1-indexed
        sx = np.clip(sx, 0, vol.shape[0] - 1)
        sy = np.clip(sy, 0, vol.shape[1] - 1)
        sz = np.clip(sz, 0, vol.shape[2] - 1)
        lbl = vol[sx, sy, sz]
        print(
            f"  Source  [{src[0]},{src[1]},{src[2]}] → label {lbl} ({TISSUE_NAMES.get(lbl, '?')})"
        )
        for i, det in enumerate(dets):
            dx = np.clip(int(det["Pos"][0]) - 1, 0, vol.shape[0] - 1)
            dy = np.clip(int(det["Pos"][1]) - 1, 0, vol.shape[1] - 1)
            dz = np.clip(int(det["Pos"][2]) - 1, 0, vol.shape[2] - 1)
            lbl = vol[dx, dy, dz]
            print(
                f"  Detector {i + 1} [{det['Pos'][0]},{det['Pos'][1]},{det['Pos'][2]}]"
                f" → label {lbl} ({TISSUE_NAMES.get(lbl, '?')})"
            )
        if vol[sx, sy, sz] == 0:
            print(
                "\n  [WARNING] Source is in background (label 0)! Photons escape immediately."
            )
            print(
                "            Find a scalp voxel (label 5) near the surface and update Pos in simulation.json"
            )
            scalp = np.argwhere(vol == 5)
            if len(scalp):
                # Suggest top-centre scalp voxel
                mid = np.array(vol.shape) // 2
                dists = np.sqrt(((scalp[:, :2] - mid[:2]) ** 2).sum(axis=1))
                best = scalp[dists.argsort()[:5]]
                print(f"  Nearest scalp voxels (1-indexed for MCX):")
                for b in best:
                    print(f"    [{b[0] + 1}, {b[1] + 1}, {b[2] + 1}]")
    except Exception as e:
        print(f"  [WARN] Could not load simulation.json for diagnostic: {e}")


# ── Main viewer ───────────────────────────────────────────────────────────────


def visualize(fluence_4d, vol=None, session_name="MCX"):
    """
    Interactive viewer:
      Left  — log10 fluence slices (axial, coronal, sagittal)
      Right — CW fluence (time-integrated) with optional tissue overlay
    """
    # CW fluence = sum over time gates
    # MCX can output (Nx,Ny,Nz,Nt) or (Nx,Ny,Nz,Nt,Ns) with trailing source dim
    arr = np.squeeze(
        fluence_4d
    )  # remove any size-1 dims e.g. (43,43,43,10,1)->(43,43,43,10)
    if arr.ndim == 4:
        cw = arr.sum(axis=3)  # sum time gates -> (Nx,Ny,Nz)
    elif arr.ndim == 3:
        cw = arr
    else:
        cw = arr.reshape(arr.shape[0], arr.shape[1], arr.shape[2], -1).sum(axis=3)

    # Mask zeros for log display
    cw_safe = np.where(cw > 0, cw, np.nan)
    vmin, vmax = np.nanpercentile(cw_safe, 1), np.nanpercentile(cw_safe, 99)

    shape = cw.shape
    cx, cy, cz = shape[0] // 2, shape[1] // 2, shape[2] // 2

    fig = plt.figure(figsize=(16, 9), facecolor="#1a1a2e")
    fig.suptitle(
        f"MCX Fluence — {session_name}",
        color="#b4befe",
        fontsize=13,
        fontweight="bold",
        y=0.97,
    )

    gs = gridspec.GridSpec(
        3,
        4,
        figure=fig,
        left=0.05,
        right=0.97,
        top=0.91,
        bottom=0.08,
        wspace=0.3,
        hspace=0.4,
    )

    ax_ax = fig.add_subplot(gs[0, 0], facecolor="#0d0d1a")
    ax_cor = fig.add_subplot(gs[0, 1], facecolor="#0d0d1a")
    ax_sag = fig.add_subplot(gs[0, 2], facecolor="#0d0d1a")
    ax_cw = fig.add_subplot(gs[1:, :2], facecolor="#0d0d1a")
    ax_info = fig.add_subplot(gs[0, 3], facecolor="#0d0d1a")
    ax_prof = fig.add_subplot(gs[1:, 2:], facecolor="#0d0d1a")

    # Colourmap — deep blue → cyan → yellow → red (like NIRFASTer)
    from matplotlib.colors import LinearSegmentedColormap

    nirs_cmap = LinearSegmentedColormap.from_list(
        "nirs", ["#000033", "#0033ff", "#00ffff", "#ffff00", "#ff0000"]
    )

    norm = LogNorm(vmin=max(vmin, 1e-20), vmax=vmax)

    def slice_rgb(arr_2d):
        arr_2d = np.where(arr_2d > 0, arr_2d, np.nan)
        return arr_2d

    ims = {}

    def draw_slices():
        for ax in [ax_ax, ax_cor, ax_sag]:
            ax.cla()
            ax.set_facecolor("#0d0d1a")
            ax.set_xticks([])
            ax.set_yticks([])

        ims["ax"] = ax_ax.imshow(
            slice_rgb(cw[:, :, cz].T),
            origin="lower",
            cmap=nirs_cmap,
            norm=norm,
            aspect="equal",
        )
        ims["cor"] = ax_cor.imshow(
            slice_rgb(cw[:, cy, :].T),
            origin="lower",
            cmap=nirs_cmap,
            norm=norm,
            aspect="equal",
        )
        ims["sag"] = ax_sag.imshow(
            slice_rgb(cw[cx, :, :].T),
            origin="lower",
            cmap=nirs_cmap,
            norm=norm,
            aspect="equal",
        )

        ax_ax.set_title(f"Axial z={cz}", color="#b4befe", fontsize=8)
        ax_cor.set_title(f"Coronal y={cy}", color="#b4befe", fontsize=8)
        ax_sag.set_title(f"Sagittal x={cx}", color="#b4befe", fontsize=8)

        # Crosshairs
        for ax, h, v in [(ax_ax, cy, cx), (ax_cor, cz, cx), (ax_sag, cz, cy)]:
            ax.axhline(h, color="#f38ba8", lw=0.5, alpha=0.7)
            ax.axvline(v, color="#f38ba8", lw=0.5, alpha=0.7)

        fig.canvas.draw_idle()

    # CW max-intensity projection
    ax_cw.cla()
    mip = np.nanmax(cw_safe, axis=2)
    ax_cw.imshow(mip.T, origin="lower", cmap=nirs_cmap, norm=norm, aspect="equal")
    ax_cw.set_title("Max-Intensity Projection (Z-axis)", color="#b4befe", fontsize=9)
    ax_cw.set_xlabel("X voxel", color="#888", fontsize=7)
    ax_cw.set_ylabel("Y voxel", color="#888", fontsize=7)
    ax_cw.tick_params(colors="#555", labelsize=6)

    # Depth profile through source position
    ax_prof.cla()
    profile = cw[cx, cy, :]
    z_mm = np.arange(len(profile)) * 6.0  # 6mm voxels
    valid = profile > 0
    if valid.any():
        ax_prof.semilogy(z_mm[valid], profile[valid], color="#89b4fa", lw=1.5)
        ax_prof.set_xlabel("Depth (mm)", color="#888", fontsize=8)
        ax_prof.set_ylabel("Fluence (a.u.)", color="#888", fontsize=8)
        ax_prof.set_title(
            f"Depth profile at x={cx}, y={cy}", color="#b4befe", fontsize=9
        )
        ax_prof.tick_params(colors="#555", labelsize=7)
        ax_prof.set_facecolor("#0d0d1a")
        ax_prof.spines["bottom"].set_color("#333")
        ax_prof.spines["left"].set_color("#333")
        ax_prof.spines["top"].set_visible(False)
        ax_prof.spines["right"].set_visible(False)
        ax_prof.yaxis.label.set_color("#888")
        ax_prof.xaxis.label.set_color("#888")

    # Info panel
    ax_info.set_axis_off()
    total_fluence = float(np.nansum(cw_safe))
    max_fluence = float(np.nanmax(cw_safe)) if not np.all(np.isnan(cw_safe)) else 0
    nonzero = int(np.sum(cw > 0))
    info = (
        f"Volume: {shape[0]}×{shape[1]}×{shape[2]}\n"
        f"Voxel size: 6.0 mm\n"
        f"Time gates: {fluence_4d.shape[-1] if fluence_4d.ndim == 4 else 1}\n\n"
        f"Max fluence:\n  {max_fluence:.3e}\n\n"
        f"Total fluence:\n  {total_fluence:.3e}\n\n"
        f"Non-zero voxels:\n  {nonzero:,}\n\n"
        f"Dynamic range:\n  {np.log10(max_fluence / max(vmin, 1e-20)):.1f} decades"
        if max_fluence > 0
        else "No fluence detected.\nCheck source placement."
    )
    ax_info.text(
        0.05,
        0.95,
        info,
        transform=ax_info.transAxes,
        va="top",
        color="#a0a0c0",
        fontsize=7.5,
        fontfamily="monospace",
    )
    ax_info.set_title("Simulation Info", color="#b4befe", fontsize=8)

    # Sliders
    sl_ax = Slider(
        fig.add_axes([0.05, 0.03, 0.17, 0.015]),
        "Z",
        0,
        shape[2] - 1,
        valinit=cz,
        valstep=1,
        color="#3a3a5c",
        track_color="#1a1a2e",
    )
    sl_cor = Slider(
        fig.add_axes([0.26, 0.03, 0.17, 0.015]),
        "Y",
        0,
        shape[1] - 1,
        valinit=cy,
        valstep=1,
        color="#3a3a5c",
        track_color="#1a1a2e",
    )
    sl_sag = Slider(
        fig.add_axes([0.47, 0.03, 0.17, 0.015]),
        "X",
        0,
        shape[0] - 1,
        valinit=cx,
        valstep=1,
        color="#3a3a5c",
        track_color="#1a1a2e",
    )

    def on_slider(val):
        nonlocal cx, cy, cz
        cz = int(sl_ax.val)
        cy = int(sl_cor.val)
        cx = int(sl_sag.val)
        draw_slices()

    sl_ax.on_changed(on_slider)
    sl_cor.on_changed(on_slider)
    sl_sag.on_changed(on_slider)

    for sl in [sl_ax, sl_cor, sl_sag]:
        sl.label.set_color("#b4befe")
        sl.valtext.set_color("#e0e0e0")

    draw_slices()
    plt.show()


# ── Entry point ───────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(description="Visualize MCX .jnii fluence output")
    parser.add_argument(
        "jnii",
        nargs="?",
        help="Path to .jnii fluence file. Auto-discovered if omitted.",
    )
    parser.add_argument(
        "--vol", help="Path to .bin volume (for source/detector diagnostic)"
    )
    parser.add_argument(
        "--dim",
        nargs=3,
        type=int,
        default=[43, 43, 43],
        metavar=("NX", "NY", "NZ"),
        help="Volume dimensions (default: 43 43 43)",
    )
    args = parser.parse_args()

    # Auto-discover .jnii if not given
    jnii_path = args.jnii
    if not jnii_path:
        for f in os.listdir("."):
            if f.endswith(".jnii"):
                jnii_path = f
                print(f"[INFO] Found: {jnii_path}")
                break

    if not jnii_path or not os.path.exists(jnii_path):
        print("[ERROR] No .jnii file found. Run MCX first, then:")
        print("        python visualize_mcx.py fnirs_5tt.jnii")
        sys.exit(1)

    session = os.path.splitext(os.path.basename(jnii_path))[0]
    print(f"[INFO] Loading {jnii_path} ...")
    fluence = load_jnii(jnii_path)
    print(f"[INFO] Fluence shape: {fluence.shape}")
    print(f"[INFO] Non-zero voxels: {(fluence > 0).sum():,}")
    print(f"[INFO] Max fluence: {fluence.max():.4e}")

    # Optional: load tissue volume for diagnostic
    vol = None
    if args.vol and os.path.exists(args.vol):
        dim = tuple(args.dim)
        vol = load_bin(args.vol, dim)
        print(f"[INFO] Loaded tissue volume: {vol.shape}")
        # Look for simulation.json in same dir
        sim_json = os.path.join(os.path.dirname(args.vol), "simulation.json")
        if not os.path.exists(sim_json):
            sim_json = "simulation.json"
        diagnose_placement(vol, sim_json)

    if fluence.max() == 0:
        print(
            "\n[WARNING] All fluence values are zero — no photons were detected inside the volume."
        )
        print(
            "          Most likely cause: source is placed in a background (label 0) voxel."
        )
        print(
            "          Run with --vol 5tt_2mm.bin --dim 43 43 43 to see a diagnostic."
        )
        if vol is None:
            print(
                "          Then update 'Pos' in simulation.json to a scalp (label 5) voxel."
            )

    print("\n[INFO] Opening viewer...")
    visualize(fluence, vol=vol, session_name=session)


if __name__ == "__main__":
    main()
