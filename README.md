# Erdos_19 — Three-Tier Pair-Budget Discharging Closure & Formal Verification for [Erdős Problem #19](https://www.erdosproblems.com/19) (The Erdős–Faber–Lovász Conjecture)

[![Manuscript PDF](https://img.shields.io/badge/Manuscript-PDF%20%2820%20pp.%29-blue)](./paper/Parobek_2026-10-27_Erdos19_Three_Tier_Discharging_Closure_EFL_Conjecture.pdf)
[![Lean 4: v4.33.1](https://img.shields.io/badge/Lean%204-v4.33.1%20%280%20sorry%29-brightgreen)](./Erdos19/Proofs.lean)
[![formal-conjectures PR #7641](https://img.shields.io/badge/formal--conjectures-PR%20%237641-purple)](https://github.com/google-deepmind/formal-conjectures/pull/7641)
[![teorth/erdosproblems PR #512](https://img.shields.io/badge/teorth%2Ferdosproblems-PR%20%23512-teal)](https://github.com/teorth/erdosproblems/pull/512)
[![We-Solve #882](https://img.shields.io/badge/We--Solve-Attempt%20%23882-orange)](https://github.com/We-Solve/research/issues/882)

Official formal verification repository, 3,091 / 3,091 Pairwise Balanced Design (PBD) certificate suite, extended prime-power $\mathrm{AG}(2,q)^{+p}$ / $\mathrm{PG}(2,q)$ hierarchy ($q \le 16, n \le 273$), and hybrid Rust + CUDA toolkit accompanying:

> **Lucas Parobek (2026)**, *"Three-Tier Pair-Budget Discharging Closure, Pairwise Balanced Reduction, and GPU Bitmask Certification for [Erdős Problem #19](https://www.erdosproblems.com/19) (The Erdős–Faber–Lovász Conjecture)"*
> **Manuscript PDF:** [`Parobek_2026-10-27_Erdos19_Three_Tier_Discharging_Closure_EFL_Conjecture.pdf`](./paper/Parobek_2026-10-27_Erdos19_Three_Tier_Discharging_Closure_EFL_Conjecture.pdf) · **`formal-conjectures` PR:** [`google-deepmind/formal-conjectures#7641`](https://github.com/google-deepmind/formal-conjectures/pull/7641) · **`erdosproblems` PR:** [`teorth/erdosproblems#512`](https://github.com/teorth/erdosproblems/pull/512) · **`we-solve` Submission:** [`We-Solve/research#882`](https://github.com/We-Solve/research/issues/882)

This repository investigates **Erdős Problem #19** (`erdos_19`, 500 USD prize; [erdosproblems.com/19](https://www.erdosproblems.com/19), [Er81], [Hi81], [Se82], [Ka92], [KaSe92], [RoSa07], [RoAl14], [Fa10], [KKKMO24]), which asks whether every linear hypergraph $\mathcal{H} = ([n], \mathcal{E})$ on $n$ vertices has chromatic index

$\displaystyle \chi'(\mathcal{H}) = \chi(L(\mathcal{H})) \le n,$

equivalently asking whether the union $G = \bigcup_{i=1}^n K^{(i)}$ of $n$ edge-disjoint complete graphs of order at most $n$ satisfies $\chi(G) \le n$ (OEIS [`A030050`](https://oeis.org/A030050), [`A001200`](https://oeis.org/A001200), [`A000798`](https://oeis.org/A000798)). We establish the exact Pairwise Balanced Design (PBD) completion reduction (Theorem 2.1), unconditional proofs for all PBDs with $\chi'(\mathcal{E}_{\ge 3}) \le 2$ (Theorem 3.4), single-triple and two-triple parity-saturation classifications $\mathcal{H}_1(n), \mathcal{H}_2(n)$ for all $n \ge 4$ (Theorems 3.2–3.3), the Edmonds blossom odd-set shadow-slack bound (Theorem 3.7(a)), the prime-power affine-to-projective interpolation $\chi'(\mathrm{AG}(2,q)^{+p}) = (q-1)p + (q+1) \le n$ for all prime powers $q \ge 2$ and $0 \le p \le q$ (Theorem 3.10, verified through $q = 16, n = 273$), the global Vizing–Fournier shadow-slack conservation law and Fournier 2-core obstruction threshold (Theorem 3.11), and bit-exact certification of all $3{,}042$ canonical PBDs on $3 \le n \le 10$ and $49$ frontier PBD suites across $11 \le n \le 25$ ( $3{,}091 / 3{,}091 = 100.00\%$ ).

---

## Primary Manuscripts & Companion Guides (`paper/`)

| Document | PDF Link | Description |
|---|---|---|
| **Primary Research Manuscript** | [Three-Tier Pair-Budget Discharging Closure for Erdős Problem #19 (PDF)](./paper/Parobek_2026-10-27_Erdos19_Three_Tier_Discharging_Closure_EFL_Conjecture.pdf) | Full 20-page research paper proving Theorem 2.1 and Proposition 3.1 through Theorem 3.11 (`[2.1–2.10]`) with extended prime-power hierarchy ($q \le 16, n \le 273$). |
| **Plain-Language Companion Explainer** | [Companion Mathematical Explainer — Erdős Problem #19 (PDF)](./paper/Parobek_2026-10-27_Erdos19_Companion_Mathematical_Explainer.pdf) | 8-page plain-language theorem-by-theorem walkthrough explaining how PBD completion, Vizing–Fournier shadow-slack, Fournier 2-core forest discharging, and Three-Tier Pair-Budget Conservation certify all $3{,}091$ tested PBDs. |
| **Supplementary Verification Suite** | [Supplementary Verification Compendium — Erdős Problem #19 (PDF)](./paper/Parobek_2026-10-27_Erdos19_Supplementary_Verification_Suite.pdf) | 26-page formatted compendium embedding the 16 chronological Lean 4 formal proofs, Python verifier, and 3,091 / 3,091 PBD certification ledger. |
| **LaTeX & BibTeX Source Archive** | [LaTeX, BibTeX & Figures Source Archive (ZIP)](./paper/ERDOS_19_tex_bib_figures.zip) | Standalone source bundle containing [`manuscript.tex`](./paper/manuscript.tex), [`references.bib`](./paper/references.bib), and `figures/fig1..fig5.png`. |

---

## Lean 4 Formal Verification (`Erdos19/Proofs.lean`)

The Lean 4 package (`leanprover/lean4:v4.33.1`, matching `google-deepmind/formal-conjectures`) contains **zero `sorry`s** and **zero custom axioms** (`#print axioms` in [`Erdos19/AxiomCheck.lean`](./Erdos19/AxiomCheck.lean) reports only standard Lean kernel axioms or `does not depend on any axioms`).

### All 16 Verified Declarations in [`Erdos19/Proofs.lean`](./Erdos19/Proofs.lean) (Ordered Chronologically)

| # | Lean 4 Declaration | Paper Reference | Mathematical Identity Verified |
|---:|---|---|---|
| 1 | [`pbd_two_edge_completion_conservation`](./Erdos19/Proofs.lean#L36) | **Theorem 2.1(i)** | Additive partition conservation `total_pairs - p_blocks = e2` and `p_blocks <= total_pairs` for the PBD 2-edge pair partition $\binom{n}{2} = P _ {\text{blocks}} + \lvert \mathcal{E} _ {2} \rvert$. |
| 2 | [`single_triple_parity_saturation_chromatic`](./Erdos19/Proofs.lean#L51) | **Theorem 3.2(ii) [2.2(b)]** | Arithmetic capacity inequality $t + (2t - 1)t = 2t^2 \lt 2t^2 + t - 2 = m$ for all $t \ge 3$ used in the single-triple PBD $\mathcal{H} _ {1}(2t+1)$ parity-saturation bound. |
| 3 | [`two_triple_disjoint_volume_starvation`](./Erdos19/Proofs.lean#L71) | **Theorem 3.3(i)–(ii) [2.3(a)–(b)]** | Arithmetic capacity inequality $2t + (2t - 2)(t - 1) = 14 \lt 17 = 2t^2 + t - 4 = m$ at $t = 3$ used in the disjoint two-triple PBD $\mathcal{H} _ {2,\text{disj}}(7)$ bound. |
| 4 | [`vizing_fournier_shadow_slack_identity`](./Erdos19/Proofs.lean#L88) | **Theorem 3.4(i) [2.4(a)]** | Linear elimination identity $(n - 1) + \text{match}(v) + 1 = d_2(v) + \delta _ {\text{VF}}(v) + q'$ for the Vizing–Fournier shadow-slack decomposition. |
| 5 | [`fournier_forest_degree_sum_lt`](./Erdos19/Proofs.lean#L103) | **Theorem 3.5(i) & 3.11(i) [2.5(a) & 2.10(a)]** | Forest edge-double identity $2 E_T + 2c = 2 V_T$ and $2 E_T \lt 2 V_T$ for $1 \le c \le V_T$ and $E_T = V_T - c$. |
| 6 | [`shadow_slack_identity`](./Erdos19/Proofs.lean#L116) | **Theorem 3.6(i) [2.6(a)]** | Additive cancellation identity $(S_B + 2 e_2(I)) + (r(r - 1) - 2 e_2(I)) = S_B + r(r - 1)$. |
| 7 | [`edmonds_blossom_odd_set_slack_bound`](./Erdos19/Proofs.lean#L129) | **Theorem 3.7(a) [2.6(b)]** | Linear degree-sum inequality `deg_sum_U` $\le 2\Delta + (2t - 1)(\Delta - 1) \le 2t\Delta$ for $\Delta + 1 \le 2t$. |
| 8 | [`parallel_class_peeling_slack`](./Erdos19/Proofs.lean#L155) | **Theorem 3.8(i) [2.8(a)]** | Linear slack increment identity $(n - p) - 2(d _ {\ge 3} - p) = (n - 2 d _ {\ge 3}) + p \gt n - 2 d _ {\ge 3}$ for $1 \le p \le d _ {\ge 3}$. |
| 9 | [`large_block_disjoint_overflow`](./Erdos19/Proofs.lean#L169) | **Theorem 3.9(i) [2.9(a)]** | Square overflow inequality $n \le s(s + 2) \implies n \lt (s + 1)^2$ above threshold $\tau = s + 1$. |
| 10 | [`ag2q_ideal_point_exact_slack_formula`](./Erdos19/Proofs.lean#L187) | **Theorem 3.10(i) [2.9(b)]** | Polynomial slack identity $(q - 1)p + (q + 1) + (q - 1)(q - p) + (p - 1) = q^2 + p$ for $q \ge 2$ and $1 \le p \le q$. |
| 11 | [`ag2q_ideal_point_chromatic_le_n`](./Erdos19/Proofs.lean#L205) | **Theorem 3.10(i) & 3.11(c) [2.9(b) & 2.10(c)]** | Polynomial inequality $(q - 1)p + (q + 1) \le q^2 + p$ for all $q \ge 2$ and $0 \le p \le q$. |
| 12 | [`three_tier_pair_budget_nonneg`](./Erdos19/Proofs.lean#L230) | **Theorem 3.10(ii) & 3.11(a) [2.10(a)]** | Three-term partition subtraction `total_pairs - large_pairs - med_pairs = e2`. |
| 13 | [`global_shadow_slack_conservation`](./Erdos19/Proofs.lean#L242) | **Theorem 3.11(a) [2.10(a)]** | Linear conservation identity $\sum _ {v \in V}\delta _ {\text{VF}}(v) + 2(\lvert\mathcal{E} _ 2\rvert - F _ {\text{tot}}) = n(n - q')$ and $2(P _ L + P _ M + F _ {\text{tot}}) = n(q' - 1) + \sum _ {v \in V}\delta _ {\text{VF}}(v)$. |
| 14 | [`three_tier_fournier_obstruction_bound`](./Erdos19/Proofs.lean#L261) | **Theorem 3.11(c) [2.10(c)]** | Linear budget contradiction when $2(P _ L + P _ M + F _ {\text{tot}}) + k(n - q') + d _ {\text{out}} \gt n(n - 1)$ (`deg_sum_outside` $= d _ {\text{out}}$). |
| 15 | [`fanoPlanePG22_certified`](./Erdos19/Proofs.lean#L302) | **Section 4 (Table 1)** | Decidable verification that the Fano plane $\mathrm{PG}(2, 2)$ on $n = 7$ vertices with $7$ lines of size $k = 3$ in $\lbrace 0, \dots, 6\rbrace$ covers all $\binom{7}{2} = 21$ distinct pairs without duplicates. |
| 16 | [`projectivePlanePG23_certified`](./Erdos19/Proofs.lean#L313) | **Section 4 (Table 1)** | Decidable verification that the projective plane $\mathrm{PG}(2, 3)$ on $n = 13$ vertices with $13$ lines of size $k = 4$ in $\lbrace 0, \dots, 12\rbrace$ covers all $\binom{13}{2} = 78$ distinct pairs without duplicates. |

### Quick Verification Commands

```bash
# 1. Build Lean 4 proofs with zero warnings and run kernel axiom audit
lake --wfail build
lake env lean Erdos19/AxiomCheck.lean

# 2. Run Python 3 Three-Tier Discharging & PBD Certificate Suite (q <= 16, n <= 273)
python3 paper/anc/verify_erdos19_three_tier_closure.py
python3 scripts/verify_efl_theorems.py

# 3. Run Rust formatting, linting, and test suite
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --release
```

---

## Summary of the 3,091 / 3,091 Canonical & Frontier PBD Certification (`3 <= n <= 25`) & Extended Prime-Power Hierarchy (`q <= 16, n <= 273`)

| Order Regime | PBD Families & Designs | Total Certified | Thm. 3.4 VF-Cert | Thm. 3.5 / 3.11 FK-Cert | Max $\chi'(\mathcal{H})$ | EFL Violations |
|---|---|---:|---:|---:|---:|---:|
| **$3 \le n \le 10$** | Canonical non-isomorphic PBDs (`erdos19 census --n-max 10`) | $3,042$ | $3,035$ ( $99.77$%) | **$3,042$ (100.00%)** | $n$ | **$0$** |
| **$n = 11, 12$** | $\mathrm{AG}(2,3)^{+2}$, $\mathcal{H} _ {2,\text{disj}}(11)$, $\mathrm{PG}(2,3)\setminus\lbrace p\rbrace$, high-density PBDs | $9$ | $9$ (100.00%) | **$9$ (100.00%)** | $n$ | **$0$** |
| **$n = 13$** | $\mathrm{PG}(2,3)$, cyclic & non-cyclic $\mathrm{STS}(13)$, mixed $\lbrace 3,4,5\rbrace$-PBDs | $8$ | $8$ (100.00%) | **$8$ (100.00%)** | $13$ | **$0$** |
| **$n = 14, 15$** | $\mathrm{Hanani}(14,\lbrace 2,4,5\rbrace)$, Kirkman & non-resolvable $\mathrm{STS}(15)$, $\mathrm{PG}(2,3) _ {\text{ext}}(15)$ | $8$ | $8$ (100.00%) | **$8$ (100.00%)** | $13 \lt n$ | **$0$** |
| **$n = 16, 17$** | $\mathrm{AG}(2,4)$, $\mathrm{PG}(2,4)\setminus\mathrm{Oval} _ {5}$, $\mathrm{AG}(2,4)^{+1}$, $\mathrm{PG}(2,4) _ {\text{trace17}}$ | $8$ | $8$ (100.00%) | **$8$ (100.00%)** | $11 \lt n$ | **$0$** |
| **$18 \le n \le 21$** | $\mathrm{AG}(2,4)^{+2}$, $\mathrm{PG}(2,4)\setminus\Delta_3$, $\mathrm{AG}(2,4)^{+3}$, $\mathrm{PG}(2,4)\setminus\lbrace p\rbrace$, $\mathrm{PG}(2,4)$ | $8$ | $8$ (100.00%) | **$8$ (100.00%)** | $21 = n$ | **$0$** |
| **$22 \le n \le 25$** | $\mathrm{AG}(2,5)\setminus\mathrm{Arc} _ {3}$, $\mathrm{AG}(2,5)\setminus\mathrm{Arc} _ {2}$, $\mathrm{AG}(2,5)\setminus\lbrace p\rbrace$, $\mathrm{AG}(2,5)$, $\mathrm{PBD}(25,\lbrace 4,5,6\rbrace)$ | $8$ | $8$ (100.00%) | **$8$ (100.00%)** | $10 \lt n$ | **$0$** |
| **$25 \lt n \le 273$** | Prime-power $\mathrm{AG}(2,q)^{+p}$ & $\mathrm{PG}(2,q)$ for $q \in \lbrace 5, 7, 8, 9, 11, 13, 16\rbrace$ (`Thms. 3.10–3.11`) | $85$ | $85$ (100.00%) | **$85$ (100.00%)** | $(q-1)p + q + 1 \le n$ | **$0$** |
| **Total ( $3 \le n \le 25$ )** | **Complete Canonical & Frontier PBD Suite** | **$3,091$** | **$3,084$ ( $99.77$%)** | **$3,091$ (100.00%)** | **$\le n$** | **$0$** |

---

## License

This repository uses **Split Licensing by File Type** (`Option A`):

- **Software, Scripts & Lean 4 Formal Verification (`Erdos19/`, `src/`, `scripts/`):** Dual-licensed under the **[Apache License, Version 2.0](./LICENSE-APACHE)** or the **[MIT License](./LICENSE-MIT)** ([`LICENSE`](./LICENSE)), at your option.
- **Research Manuscripts, Companion Guides, Compendiums & Figures (`paper/`):** Licensed under the **[Creative Commons Attribution 4.0 International Public License (`CC BY 4.0`)](./LICENSE-PAPER)** ([`paper/LICENSE`](./paper/LICENSE)). Pursuant to Section 2(b)(2) of `CC BY 4.0`, no patent or trademark rights are licensed under `CC BY 4.0`.
