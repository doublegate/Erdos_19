# Changelog — Erdos_19

All notable changes to the `Erdos_19` public verification repository for Erdős Problem #19 ($500 Erdős–Faber–Lovász Conjecture) are documented in this file.

## [1.0.0] — 2026-10-27

### Added
- **Lean 4 (`v4.33.1`) Formal Verification Suite (`Erdos19/Proofs.lean`, `Erdos19/AxiomCheck.lean`):**
  - 14 zero-`sorry`, zero-warning declarations ordered chronologically from Theorem 2.1 (`pbd_two_edge_completion_conservation`) through Theorem 3.11 (`three_tier_discharging_closure`) plus decidable projective plane witnesses `fanoPlanePG22_certified` ($\mathrm{PG}(2,2)$) and `projectivePlanePG23_certified` ($\mathrm{PG}(2,3)$).
- **Primary Research Manuscript, Companion Explainer, and Supplementary Compendium (`paper/`):**
  - `Parobek_2026-10-27_Erdos19_Three_Tier_Discharging_Closure_EFL_Conjecture.pdf` (16 pages, 0 missing-glyph boxes).
  - `Parobek_2026-10-27_Erdos19_Companion_Mathematical_Explainer.pdf` (6 pages, plain-language theorem-by-theorem walkthrough) and `Companion_Explainer_Erdos19.md`.
  - `Parobek_2026-10-27_Erdos19_Supplementary_Verification_Suite.pdf` (15 pages, `DejaVu Sans Mono` formatted code & certificates) and `.zip`.
  - `ERDOS_19_tex_bib_figures.zip` (`.tex`, `.bib`, and `figures/fig1..fig4.png` source archive).
- **Hybrid Rust + CUDA Solver & Python Verification Suites (`src/`, `scripts/`, `paper/anc/`):**
  - 64-bit bitmask branch-and-bound chromatic solver with Star-Clique Hall-Capacity pruning, Fournier 2-core forest discharging, and Three-Tier Pair-Budget verification across all $3,091 / 3,091$ (100.00%) canonical and frontier PBDs on $3 \le n \le 25$.
- **Three-Stage GitHub Actions CI (`.github/workflows/ci.yml`):**
  - Automated CI for `lean4-verification` (`lake --wfail build` & `AxiomCheck.lean`), `python-verification`, and `rust-suite` (`cargo fmt`, `cargo clippy`, `cargo test --release`).
