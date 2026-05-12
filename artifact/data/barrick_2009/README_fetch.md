# Barrick 2009 — Full Data Fetch Instructions

The complete dataset requires SRA Toolkit:

```bash
# Install SRA Toolkit
# See: https://github.com/ncbi/sra-tools/wiki

# Fetch reads for the 20,000-generation evolved genome
prefetch SRR000868
fastq-dump --split-files SRR000868

# Fetch ancestor reference genome (REL606)
# GenBank: U00096 (E. coli K-12 MG1655, close reference)
# LTEE ancestor: NC_012967 (REL606)
```

For Tier 1 (Python) validation, the mutation parameter JSON is sufficient.
Full SRA data is needed for Tier 2 (Rust) breseq-style analysis.
