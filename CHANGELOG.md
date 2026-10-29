# Changelog — Erdos_19

All notable changes to the `Erdos_19` public verification repository for Erdős Problem #19 ($500 Erdős–Faber–Lovász Conjecture) are documented in this file.

## [1.1.1] — 2026-10-29

### Fixed & Remediated
- **`formal-conjectures#7641` Final Attribution Polish & Manuscript/Archive Sync (`FormalConjectures_19.lean`, `submissions/formal_conjectures_lean4/19.lean`, `paper/`, `src/`):**
  - Removed the parenthetical in `erdos_19.variants.le_twelve` (`FormalConjectures/ErdosProblems/19.lean` in `google-deepmind/formal-conjectures#7641`, approved by `bocowgill`) and dropped the unused `[RoSa07]` header tag so `le_twelve` solely credits Romero & Alonso-Pecina `[RoAl14]` (2014) for $n \le 12$ while `lt_ten` credits Hindman `[Hi81]` (1981) for $n < 10$.
  - Updated `paper/manuscript.tex` (Section 3, line 256), `paper/Companion_Explainer_Erdos19.md`, `paper/Companion_Explainer_Erdos19.typ` (Lean line anchors `#L36`–`#L313`), `src/records.rs`, `src/main.rs`, and all compiled PDFs (`manuscript.pdf`, `Companion_Explainer_Erdos19.pdf`, `Supplementary_Verification_Suite_Erdos19.pdf`) and `.zip`/`.tar.gz` bundles to consistently distinguish Romero & Sánchez-Arroyo (2007, special infinite class of dense linear hypergraphs) from Romero & Alonso-Pecina (2014, computational verification of $n \le 12$).

## [1.1.0] — 2026-10-27

### Added & Remediated
- **We-Solve `#882` & `formal-conjectures#7641` Reviewer Audit Remediations, Chronological 16-Theorem Lean 4 Suite, Restored Expository Prose (`20 pp.`), and Extended Prime-Power ($q \le 16, n \le 273$) Hierarchy (`Erdos19/Proofs.lean`, `paper/anc/verify_erdos19_three_tier_closure.py`, `paper/`):**
  - Expanded and reordered `Erdos19/Proofs.lean` (`paper/anc/19.lean`, `FormalConjectures_19.lean`, `google-deepmind/formal-conjectures#7641`) into **16 strictly chronological zero-`sorry` declarations** (`#L36`–`#L313`), including `edmonds_blossom_odd_set_slack_bound` (Theorem 3.7(a) [2.6(b)] at `#L129`), `global_shadow_slack_conservation` (Theorem 3.11(a) [2.10(a)] at `#L242`), and `three_tier_fournier_obstruction_bound` (Theorem 3.11(c) [2.10(c)] at `#L261`).
  - Refined all 14 arithmetic helper declarations (`category API` in `formal-conjectures#7641`) so their docstrings state the exact arithmetic identity/inequality proved on $\mathbb{N}$, added the Romero & Alonso-Pecina (2014) citation (`[RoAl14]`) for $n \le 12$, and strengthened `isCertifiedProjectivePlane` with `listNoDups` and `v < n` vertex-range verification.
  - Extended exact prime-power affine-to-projective $\mathrm{AG}(2,q)^{+p}$ and $\mathrm{PG}(2,q)$ chromatic interpolation and Three-Tier Pair-Budget verification across **$q \in \{2, 3, 4, 5, 7, 8, 9, 11, 13, 16\}$ ($n \le 273$, pair budget $\binom{n}{2} \le 37{,}128$)**.
  - Added Figure 5 (`paper/figures/fig5_three_tier_shadow_slack_conservation.{pdf,png}`), "Countering Potential Methodological Objections" subsection, restored rich mathematical prose, intuition, and architectural motivation across Sections 1–5 of `paper/manuscript.tex` and `paper/manuscript.typ`, and rebuilt all manuscripts (`20 pp.`), companion explainers (`8 pp.`), supplementary verification compendiums (`26 pp.`), and archives.

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
