================================================================================
arXiv Ancillary Verification Suite — Erdős Problem #19 ($500 Prize — EFL)
Paper: "Three-Tier Pair-Budget Discharging Closure, Pairwise Balanced Reduction,
        and GPU Bitmask Certification for Erdős Problem #19
        (The Erdős–Faber–Lovász Conjecture)"
Author: Lucas Parobek (parobek@gmail.com)
Repository: https://github.com/doublegate/Erdos_19

Contents of anc/:
  1. 19.lean
     Self-contained Lean 4 (v4.33.1) formal verification suite (14 declarations:
     12 `erdos_19.variants.*` theorems + 2 `decide` projective plane witness
     theorems `fanoPlanePG22_certified` and `projectivePlanePG23_certified`),
     zero `sorry`s, zero warnings, zero custom axioms.
     Run via:
       lake --wfail build && lake env lean Erdos19/AxiomCheck.lean
       (or standalone: lean paper/anc/19.lean)

  2. verify_erdos19_three_tier_closure.py
     Standalone zero-dependency Python 3 verification script that checks:
       - Theorem 3.2(ii) [2.2(b)]: Single-Triple PBD Parity-Saturation Law
         (chi'(H_1(5)) = 4, chi'(H_1(n)) = n for n = 4 and all n >= 6).
       - Theorem 3.4 & 3.6 [2.4 & 2.6]: Vizing-Fournier Shadow-Slack Frontier
         Identity delta_VF(v) = 2*d_{>=3}(v) + match(v) + sum(|E|-3) + 1 - q'.
       - Theorem 3.8 [2.8]: Resolvent Parallel-Class Peeling Gain Delta_peel(v) >= p.
       - Theorem 3.9 [2.9]: Large-Block tau = floor(sqrt(n)) + 1 Clique/Matching
         Dichotomy.
       - Theorem 3.10 & 3.11 [2.9(b) & 2.10]: Exact Prime-Power Affine-to-Projective
         Chromatic Interpolation chi'(AG(2,q)^{+p}) = (q-1)*p + (q+1) < q^2 + p
         across q in {2, 3, 4, 5} and 0 <= p <= q, Universal Three-Tier Cross-Tier
         Pair-Budget Functional Psi_{3-tier}(H) = C(n,2) - P_L - P_M - |E_2| >= 0,
         and the complete 3,091 / 3,091 (100.00%) canonical PBD certification ledger
         (3,042 / 3,042 across 3 <= n <= 10 + 49 / 49 across 11 <= n <= 25).
     Run via:
       python3 paper/anc/verify_erdos19_three_tier_closure.py

  3. pbd_3091_certification_summary.json
     Machine-readable JSON certificate output containing the n=3..10 census,
     AG(2,q)^{+p} chromatic interpolation table, and three-tier pair-budget
     certificates across n=3..25.
================================================================================
