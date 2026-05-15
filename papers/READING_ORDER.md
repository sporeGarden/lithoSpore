# LTEE Paper Reading Order — lithoSpore GuideStone

A guided path through the Long-Term Evolution Experiment literature,
following the lithoSpore module escalation order. Each module targets
specific figures and tables from these papers.

## Foundation

Start here to understand the LTEE itself:

1. **Lenski et al. 1991** — *Long-term experimental evolution in E. coli. I.*
   The founding paper. 12 populations, identical ancestors, glucose minimal
   medium, daily 1:100 transfers. Everything builds on this protocol.
   DOI: `10.1086/285289`

## Module 1: Fitness Trajectories

2. **Wiser, Ribeck & Lenski 2013** — *Long-term dynamics of adaptation in asexual populations*
   Science 342:1364–1367. Power-law fitness dynamics: w(t) ~ t^β.
   lithoSpore reproduces Fig 1 (trajectories) and Table 1 (exponent β ≈ 0.12).
   Data: [Dryad doi:10.5061/dryad.0hc2m](https://datadryad.org/stash/dataset/doi:10.5061/dryad.0hc2m)
   DOI: `10.1126/science.1243357`

## Module 2: Mutation Accumulation

3. **Barrick et al. 2009** — *Genome evolution and adaptation in a long-term experiment*
   Nature 461:1243–1247. First whole-genome resequencing at 20K generations.
   lithoSpore reproduces Fig 1 (accumulation curve) and validates ~8.9e-11 mutation rate.
   Data: [BioProject PRJNA29543](https://www.ncbi.nlm.nih.gov/bioproject/PRJNA29543)
   DOI: `10.1038/nature08480`

## Module 3: Allele Dynamics

4. **Good et al. 2017** — *The dynamics of molecular evolution over 60,000 generations*
   Nature 551:45–50. Deep metagenomic sequencing reveals clonal interference.
   lithoSpore computes fixation probabilities and interference ratios from allele trajectories.
   Data: [BioProject PRJNA380528](https://www.ncbi.nlm.nih.gov/bioproject/PRJNA380528)
   DOI: `10.1038/nature24287`

5. **Maddamsetti, Lenski & Barrick 2015** — *Adaptation, clonal interference, and frequency-dependent interactions*
   Genetics 200:619–631. Context for interference dynamics in module 3.
   DOI: `10.1534/genetics.115.178962`

## Module 4: Citrate Innovation

6. **Blount, Borland & Lenski 2008** — *Historical contingency and the evolution of a key innovation*
   PNAS 105:7899–7906. The Cit+ innovation and replay experiment.
   lithoSpore validates the potentiation window (~2000 generations before Cit+).
   DOI: `10.1073/pnas.0803151105`

7. **Blount et al. 2012** — *Genomic analysis of a key innovation*
   Nature 489:513–518. Genomic dissection of the citrate innovation.
   lithoSpore validates replay probabilities and two-hit model ordering.
   Data: [BioProject PRJNA188627](https://www.ncbi.nlm.nih.gov/bioproject/PRJNA188627)
   DOI: `10.1038/nature11514`

## Module 5: BioBrick Burden

8. **Barrick et al. 2024** — *Measuring the burden of hundreds of BioBricks*
   Nature Communications 15:6242. Growth burden for 301 plasmids.
   lithoSpore computes burden distribution from plate reader CSVs.
   Data: [GitHub barricklab/igem2019 v1.0.2](https://github.com/barricklab/igem2019/releases/tag/v1.0.2)
   DOI: `10.1038/s41467-024-50639-9`

## Module 6: Genome-Wide Evolution (breseq Comparison)

9. **Tenaillon et al. 2016** — *Tempo and mode of genome evolution in a 50,000-generation experiment*
   Nature 536:165–170. 264 whole-genome sequences across all 12 populations.
   lithoSpore validates mutation accumulation curves, Ts/Tv ratios, and parallel evolution.
   Data: [BioProject PRJNA294072](https://www.ncbi.nlm.nih.gov/bioproject/PRJNA294072)
   DOI: `10.1038/nature18959`

10. **Deatherage & Barrick 2014** — *Identification of mutations using breseq*
    Methods in Molecular Biology 1151:165–188. The breseq methodology paper.
    Module 6 benchmarks against breseq reference calls.
    Software: [github.com/barricklab/breseq](https://github.com/barricklab/breseq)
    DOI: `10.1186/s12864-014-1160-7`

## Module 7: Anderson-QS Predictions

11. **Anderson 1958** — *Absence of diffusion in certain random lattices*
    Physical Review 109:1492–1505. The foundational Anderson localization paper.
    Module 7 maps LTEE fitness landscapes to the disorder parameter W/V.
    DOI: `10.1103/PhysRev.109.1492`

## Broader Context

12. **Lenski 2017** — *Experimental evolution and the dynamics of adaptation*
    ISME Journal 11:2181–2194. Comprehensive LTEE review after 25+ years.
    DOI: `10.1086/680530`

13. **Woods et al. 2011** — *Second-order selection for evolvability*
    Science 331:1433–1436. Epistasis and marker divergence in LTEE.
    DOI: `10.1126/science.1203801`

---

## How lithoSpore Uses These Papers

Each module has a `--expected` JSON file containing values ported from the
papers above. The `validation/expected/module{N}_*.json` files cite the
specific table rows and figure panels. Run `litho validate --json` to see
which claims pass, and `litho validate --targets` to see T01–T14 coverage.

See `papers/registry.toml` for the machine-readable version of this guide.
