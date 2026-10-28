================================================================================
Ancillary Verification Suite — Erdős Problem #19 ($500 Prize — EFL)
Paper: "Three-Tier Pair-Budget Discharging Closure, Pairwise Balanced Reduction,
        and GPU Bitmask Certification for Erdős Problem #19
        (The Erdős–Faber–Lovász Conjecture)"
Author: Lucas Parobek (parobek@gmail.com)
Paper PDF: https://github.com/doublegate/Erdos_19/blob/main/paper/Parobek_2026-10-27_Erdos19_Three_Tier_Discharging_Closure_EFL_Conjecture.pdf
Repository: https://github.com/doublegate/Erdos_19
formal-conjectures PR: https://github.com/google-deepmind/formal-conjectures/pull/7641
teorth/erdosproblems PR: https://github.com/teorth/erdosproblems/pull/512
We-Solve/research Issue: https://github.com/We-Solve/research/issues/882

Contents of anc/:
  1. 19.lean (and 19.lean.txt for plain-text viewers)
     Self-contained Lean 4 (v4.33.1) formal verification suite (16 declarations
     in strictly chronological manuscript order: 14 `erdos_19.variants.*`
     theorems + 2 `decide` projective plane witness theorems
     `fanoPlanePG22_certified` and `projectivePlanePG23_certified`),
     zero `sorry`s, zero warnings, zero custom axioms. Matches `Erdos19/Proofs.lean`
     and `google-deepmind/formal-conjectures#7641`.
     Run via:
       lake --wfail build && lake env lean Erdos19/AxiomCheck.lean
       (or standalone: lean paper/anc/19.lean)

  2. verify_erdos19_three_tier_closure.py
     Standalone zero-dependency Python 3 verification script that checks:
       - Theorem 3.2(ii) [2.2(b)]: Single-Triple PBD Parity-Saturation Law
         (chi'(H_1(5)) = 4, chi'(H_1(n)) = n for n = 4 and all n >= 6).
       - Theorem 3.4, 3.5 & 3.6 [2.4, 2.5 & 2.6]: Vizing-Fournier Shadow-Slack
         Identity delta_VF(v) = 2*d_{>=3}(v) + match(v) + sum(|E|-3) + 1 - q'
         and Fournier 2-Core Forest Degree-Sum Bound 2*E_T + 2*c = 2*V_T.
       - Theorem 3.7(a) [2.6(b)]: Edmonds Blossom Odd-Set Shadow-Slack Bound
         2*|E(G_res[U])| <= 2*Delta + (2*t - 1)*(Delta - 1) <= 2*t*Delta.
       - Theorem 3.8 [2.8]: Resolvent Parallel-Class Peeling Gain Delta_peel(v) >= p.
       - Theorem 3.9 [2.9]: Large-Block tau = floor(sqrt(n)) + 1 Clique/Matching
         Dichotomy.
       - Theorem 3.10 & 3.11 [2.9(b) & 2.10(a)-(c)]: Exact Prime-Power
         Affine-to-Projective Chromatic Interpolation
         chi'(AG(2,q)^{+p}) = (q-1)*p + (q+1) < q^2 + p across prime powers
         q in {2, 3, 4, 5, 7, 8, 9, 11, 13, 16} (n <= 273) and 0 <= p <= q,
         Global Shadow-Slack & Residual 2-Edge Conservation
         sum_v delta_VF(v) + 2*(e_2 - F_tot) = n*(n - q'), Three-Tier Fournier
         2-Core Obstruction Bound, and the complete 3,091 / 3,091 (100.00%)
         canonical PBD certification ledger across 3 <= n <= 25 (plus extended
         prime-power verification up to n <= 273).
     Run via:
       python3 paper/anc/verify_erdos19_three_tier_closure.py

  3. pbd_3091_certification_summary.json
     Machine-readable JSON certificate output containing the n=3..10 census,
     AG(2,q)^{+p} chromatic interpolation table, and three-tier pair-budget
     certificates across n=3..25.
================================================================================
