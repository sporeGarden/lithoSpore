// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::Path;

/// Try to generate figures from outputs/ using Python + matplotlib.
/// Embeds a minimal figure-generation script and runs it if matplotlib is importable.
pub(super) fn try_generate_figures(root: &Path) {
    let outputs_dir = root.join("outputs");
    if !outputs_dir.exists() {
        return;
    }

    // Check if Python + matplotlib are available
    let check = std::process::Command::new("python3")
        .args(["-c", "import matplotlib; import numpy"])
        .output();

    let has_deps = check.is_ok_and(|o| o.status.success());
    if !has_deps {
        println!("  [~] figures/ skipped (matplotlib/numpy not available)");
        return;
    }

    let figures_dir = root.join("figures");
    fs::create_dir_all(&figures_dir).ok();

    // Inline minimal figure generation script
    let script = r"
import sys, numpy as np
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap

root = Path(sys.argv[1])
outputs = root / 'outputs'
figures = root / 'figures'
figures.mkdir(exist_ok=True)

def parse_fes_1d(path):
    xs, ys = [], []
    for line in open(path):
        if line.startswith('#') or not line.strip(): continue
        p = line.split()
        if len(p) >= 2: xs.append(float(p[0])); ys.append(float(p[1]))
    return np.array(xs), np.array(ys)

def parse_fes_2d(path):
    xs, ys, zs = [], [], []
    for line in open(path):
        if line.startswith('#') or not line.strip(): continue
        p = line.split()
        if len(p) >= 3: xs.append(float(p[0])); ys.append(float(p[1])); zs.append(float(p[2]))
    xs, ys, zs = np.array(xs), np.array(ys), np.array(zs)
    uy = len(set(np.round(ys, 8)))
    ux = len(xs) // uy if uy > 0 else 1
    return xs.reshape(uy, ux), ys.reshape(uy, ux), zs.reshape(uy, ux)

count = 0

# 1D comparison
fes_files_1d = list(outputs.glob('*/fes_theta.dat'))
if len(fes_files_1d) >= 2:
    fig, ax = plt.subplots(figsize=(8,5))
    for f in sorted(fes_files_1d):
        x, y = parse_fes_1d(f)
        label = f.parent.name.replace('-', ' ').replace('_', ' ')
        ax.plot(np.degrees(x), y, linewidth=2, label=label)
    ax.set_xlabel('Cremer-Pople θ (degrees)')
    ax.set_ylabel('Free energy (kJ/mol)')
    ax.set_title('1D Puckering Free Energy Landscapes')
    ax.legend()
    ax.grid(True, alpha=0.3)
    plt.tight_layout()
    fig.savefig(figures / 'fel_1d_comparison.png', dpi=300)
    plt.close()
    count += 1

# 2D heatmaps
for fes_2d in sorted(outputs.glob('*/fes_2d.dat')):
    try:
        X, Y, Z = parse_fes_2d(fes_2d)
        fig, ax = plt.subplots(figsize=(7,6))
        Z_viz = np.clip(Z, 0, min(Z.max(), 60))
        cmap = LinearSegmentedColormap.from_list('fel',
            ['#000033','#0000aa','#0066ff','#00cccc','#66ff66','#ffff00','#ff6600','#ff0000','#ffffff'])
        im = ax.pcolormesh(X*10, Y*10, Z_viz, cmap=cmap, shading='auto')
        plt.colorbar(im, ax=ax, label='Free energy (kJ/mol)')
        name = fes_2d.parent.name.replace('-',' ')
        ax.set_title(f'2D FEL — {name}')
        ax.set_xlabel('qx'); ax.set_ylabel('qy')
        ax.set_aspect('equal')
        plt.tight_layout()
        fig.savefig(figures / f'fel_2d_{fes_2d.parent.name}.png', dpi=300)
        plt.close()
        count += 1
    except: pass

print(count)
";

    let result = std::process::Command::new("python3")
        .args(["-c", script, root.to_str().unwrap_or(".")])
        .output();

    match result {
        Ok(o) if o.status.success() => {
            let count = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let n: usize = count.parse().unwrap_or(0);
            if n > 0 {
                println!("  [+] figures/ ({n} plots generated)");
            } else {
                println!("  [~] figures/ (no plottable outputs found)");
            }
        }
        _ => {
            println!("  [~] figures/ skipped (generation failed)");
        }
    }
}
