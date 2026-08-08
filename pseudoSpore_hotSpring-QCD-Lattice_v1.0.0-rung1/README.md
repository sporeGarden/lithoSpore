# pseudoSpore: hotSpring-QCD-Lattice v1.0.0-rung1

**External maturity**: publication-ready
<!-- metrics: 2026-08-08 -->

Content-addressed scientific artifact for the arXiv paper:
*"Sovereign GPU Lattice QCD: SU(N) on Consumer Hardware via WGSL"*

## What This Contains

SU(3) lattice gauge theory plaquette measurements produced by **barraCuda**
(pure Rust/WGSL compute shaders via wgpu) on consumer GPUs:

| Module | Checks | Data |
|--------|--------|------|
| `su3_beta_scan` | 6/6 | 8 β points on 8⁴, compared to published strong-coupling + MC references |
| `su3_dual_gpu_cross_vendor` | 3/3 | NVIDIA RTX 3090 vs AMD RX 6950 XT at β=5.9, 6.0, 6.2 on 16⁴ |
| `su3_performance_scaling` | 6/6 | GPU trajectory throughput (ms/traj) across architectures |

**Total**: 15/15 checks PASS, 3/3 modules COMPLETE.

## Key Results

- **Cross-vendor plaquette agreement**: <1σ separation at all β (NVIDIA vs AMD)
- **β=6.0 on 16⁴**: ⟨P⟩ = 0.5935 ± 0.0002 (published: 0.593) — sub-permille agreement
- **AMD GPU throughput**: 275 ms/traj on 16⁴ SU(3) — consumer hardware viability demonstrated

## Verification

```bash
cd pseudoSpore_hotSpring-QCD-Lattice_v1.0.0-rung1
b3sum --check receipts/checksums.blake3
litho audit --path . --json
```

## License

AGPL-3.0-or-later

## Provenance

- **Origin**: ecoPrimals/springs/hotSpring (barraCuda engine)
- **Compute**: strandGate (NVIDIA RTX 3090 + AMD RX 6950 XT)
- **Pipeline**: litho emit-pseudospore → init-validation → populate-validation → promote-spore → pack
