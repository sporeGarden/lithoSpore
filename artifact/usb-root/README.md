# lithoSpore

A portable science validation tool. Plug it in, run `./validate`, and
watch it reproduce published results from the longest-running evolution
experiment in history.

---

## What's on this drive

This USB contains **7 science modules** that reproduce results from the
Lenski Lab's **Long-Term Evolution Experiment** (LTEE) — 50,000+
generations of *E. coli* evolving in continuous culture since 1988.

Each module validates a published result:

| # | What it checks | Paper |
|---|---------------|-------|
| 1 | Fitness follows a power law | Wiser 2013, *Science* |
| 2 | Mutations accumulate like a clock | Barrick 2009, *Nature* |
| 3 | Competing lineages interfere | Good 2017, *Nature* |
| 4 | Citrate use evolved once in 50k generations | Blount 2008/2012 |
| 5 | Synthetic parts impose a metabolic burden | Barrick 2024, *Nat Comms* |
| 6 | 264 genomes tell a consistent story | Tenaillon 2016, *Nature* |
| 7 | Fitness landscapes show disorder (Anderson-QS) | New prediction |

Every result is checked against the original publication. If the math
matches, you get `[PASS]`. If it doesn't, you get `[FAIL]` and the
exact values that disagreed.

---

## Quick start

```bash
# Run all 7 modules (~5 seconds, no internet needed)
./validate

# Check data integrity (BLAKE3 hashes)
./verify

# See your deployment status
./spore tier
```

That's it. No install, no dependencies, no internet. The binary and all
data are on the drive.

---

## If you want to go deeper

| Document | What it covers |
|----------|---------------|
| `SCIENCE.md` | The scientific story connecting all 7 modules |
| `papers/READING_ORDER.md` | Guided path through the LTEE literature |
| `GETTING_STARTED.md` | Technical walkthrough: tiers, data model, building |
| `PROTOTYPE.md` | Honest status report: what's solid, what's evolving |
| `notebooks/` | Python code for every module (numpy/scipy, readable) |

---

## Exploring with an AI

This artifact is designed to be explored with an AI assistant. Open a
chat (ChatGPT, Claude, Cursor, Copilot — whatever you have) and try:

> "I plugged in a USB called lithoSpore. It has a file called SCIENCE.md
> and Python notebooks in notebooks/. Can you help me understand what
> Module 1 does? Here's the Python code..."

Then paste the contents of `notebooks/module1_fitness/power_law_fitness.py`.
The AI can walk you through the curve fitting, explain AIC/BIC model
comparison, and help you modify the code to test your own ideas.

**Good questions to ask an AI about this artifact:**

- "What is a power law and why does LTEE fitness follow one?"
- "This notebook uses `scipy.optimize.minimize` — what is Nelder-Mead
  and why use it instead of linear regression?"
- "Module 3 simulates 100,000 individuals. How does clonal interference
  change fixation probability?"
- "What is the Anderson localization prediction in Module 7 and how
  would you test it in a wet lab?"
- "Can you explain the BLAKE3 hashes in `data_manifest.toml`? Why does
  scientific data need integrity checking?"

The Python notebooks are intentionally simple. A second-year biology
student who's taken intro programming can read them. The Rust code
(`bin/litho`) does the same math faster — comparing them teaches you
about implementation vs. algorithm.

---

## For biology students

You don't need to know Rust or systems programming. Start with:

1. Read `SCIENCE.md` — it tells the story of 50,000 generations
2. Open `notebooks/module1_fitness/power_law_fitness.py` in any editor
3. The Python code loads 12 data points and fits three curves
4. Ask an AI to explain each section

The LTEE is one of biology's most important ongoing experiments. This
tool lets you reproduce its key results on your laptop — something that
would have required months of manual computation in 2009 when Barrick
first published the mutation accumulation data.

## For CS students

The interesting part is the architecture:

- **Same math, two languages**: Python notebooks and Rust binaries
  produce identical results. `litho parity` verifies this automatically.
- **Three tiers**: Python (readable) → Rust (fast) → Primal network
  (provenance). Each tier adds capability without changing the science.
- **Provenance chain**: Every result links back to a published paper,
  a BLAKE3 hash, and an SRA accession. `provenance/braids/` contains
  computation records from the upstream pipeline.
- **No unsafe code**: The entire Rust codebase forbids `unsafe` at the
  workspace level. 125 tests, zero C dependencies in the critical path.

Look at `artifact/scope.toml` to see how the chassis separates
"lithoSpore the framework" from "LTEE the first instance."

---

## How it works

```
USB Drive
├── validate           → runs all 7 science modules
├── verify             → checks data integrity
├── bin/litho          → the validation engine (single Rust binary)
├── notebooks/         → Python baselines (7 modules, readable)
├── artifact/data/     → datasets (BLAKE3-verified)
├── papers/            → paper registry + reading order
├── figures/           → SVG visualizations (7 modules)
├── provenance/braids/ → upstream computation records
├── SCIENCE.md         → the scientific narrative
├── GETTING_STARTED.md → technical walkthrough
└── PROTOTYPE.md       → honest status + what's next
```

---

## The bigger picture

This USB is one artifact from **ecoPrimals** — a sovereign compute
ecosystem that validates science across 8 domains, 175+ papers, and
20,695+ checks. The LTEE modules are the first instance of the
**lithoSpore** chassis, which can be reused for any body of science
with quantitative claims and source data.

- Website: [primals.eco](https://primals.eco)
- Glossary: [primals.eco/glossary](https://primals.eco/glossary/)
- Source: [github.com/sporeGarden/lithoSpore](https://github.com/sporeGarden/lithoSpore)

---

**License**: AGPL-3.0-or-later (code), CC-BY-SA 4.0 (docs)
**Contact**: Kevin Mok — mokkevin@msu.edu
