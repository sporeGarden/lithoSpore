#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Shared figure generation for lithoSpore LTEE module baselines.

Produces publication-quality SVG figures from module data.
Falls back gracefully if matplotlib is not installed.
"""
from __future__ import annotations

import json
import os
from pathlib import Path

HAS_MPL = False
try:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    HAS_MPL = True
except ImportError:
    pass

STYLE = {
    "font.size": 10,
    "axes.titlesize": 12,
    "axes.labelsize": 10,
    "figure.figsize": (8, 5),
    "figure.dpi": 150,
    "savefig.bbox": "tight",
    "savefig.pad_inches": 0.15,
}


def ensure_output_dir(output_dir: str | Path) -> Path:
    p = Path(output_dir)
    p.mkdir(parents=True, exist_ok=True)
    return p


def can_generate() -> bool:
    return HAS_MPL


def apply_style():
    if HAS_MPL:
        plt.rcParams.update(STYLE)


def save_figure(fig, output_dir: Path, name: str):
    svg_path = output_dir / f"{name}.svg"
    fig.savefig(str(svg_path), format="svg")
    plt.close(fig)
    print(f"  Figure: {svg_path}")
    return svg_path
