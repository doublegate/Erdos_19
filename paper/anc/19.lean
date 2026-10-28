/-
Copyright 2026 Lucas Parobek.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

# Erdős Problem #19 (Erdős–Faber–Lovász Conjecture) — Lean 4 Formal Verification Suite
Paper: "Three-Tier Pair-Budget Discharging Closure, Pairwise Balanced Reduction,
        and GPU Bitmask Certification for Erdős Problem #19" (Parobek, 2026)
Repo:  https://github.com/doublegate/Erdos_19

This self-contained Lean 4 module (zero `sorry`s, zero custom axioms) machine-checks the
discrete arithmetic identities and inequalities used in the Pairwise Balanced Design (PBD)
2-edge completion conservation law, the single-triple and two-triple parity-saturation capacity
bounds, the Vizing–Fournier shadow-slack degree relation, the Fournier 2-core forest degree-sum
bound, the Shadow-Slack decomposition identity, Edmonds' Blossom odd-set degree-sum bound,
resolvent parallel-class peeling slack, the large-block disjointness overflow bound, the
AG(2,q)^{+p} arithmetic slack identities, the Three-Tier Pair-Budget arithmetic conservation
relations, and the finite projective plane witnesses PG(2,2) and PG(2,3) of [Pa26].
-/

namespace Erdos19

local notation "ℕ" => Nat

/-! ## 1. Theorem 2.1(i): Exact Pairwise Balanced Design (PBD) Pair Conservation -/

/--
**Arithmetic Lemma for Theorem 2.1(i) (Additive Partition Conservation):**
For natural numbers `total_pairs`, `p_blocks`, and `e2` satisfying `p_blocks + e2 = total_pairs`,
`total_pairs - p_blocks = e2` and `p_blocks ≤ total_pairs` (used for the PBD 2-edge pair partition
`p_blocks + e2 = n * (n - 1) / 2` in Theorem 2.1(i) of [Pa26]).
-/
theorem erdos_19.variants.pbd_two_edge_completion_conservation (total_pairs p_blocks e2 : ℕ)
    (h_partition : p_blocks + e2 = total_pairs) :
    total_pairs - p_blocks = e2 ∧ p_blocks ≤ total_pairs := by
  omega

/-! ## 2. Theorem 3.2(ii) [2.2(b)]: Single-Triple Parity-Saturation Obstruction -/

/--
**Arithmetic Lemma for Theorem 3.2(ii) [2.2(b)] (Single-Triple Capacity Inequality):**
For any natural number `t ≥ 3`, if `m = 2 * t * t + t - 2` and `cap_nm1 = t + (2 * t - 1) * t`,
then `cap_nm1 = 2 * t * t < m`. In Theorem 3.2(ii) of [Pa26], `m` is the edge count of the
single-triple PBD `H_1(2t + 1)` and `cap_nm1` is the maximum hyperedge capacity of `2t = n - 1`
color classes when one class contains the 3-block (`t` edges) and `2t - 1` classes contain at
most `t` 2-edges each.
-/
theorem erdos_19.variants.single_triple_parity_saturation_chromatic (t m cap_nm1 : ℕ)
    (ht : 3 ≤ t)
    (hm : m = 2 * t * t + t - 2)
    (hcap : cap_nm1 = t + (2 * t - 1) * t) :
    cap_nm1 < m := by
  have h_mul : (2 * t - 1) * t + t = 2 * t * t := by
    have h1 : (2 * t - 1 + 1) * t = (2 * t - 1) * t + 1 * t := Nat.add_mul (2 * t - 1) 1 t
    have h2 : 2 * t - 1 + 1 = 2 * t := by omega
    rw [h2, Nat.one_mul] at h1
    exact h1.symm
  omega

/-! ## 3. Theorem 3.3(i)–(ii) [2.3(a)–(b)]: Two-Triple Disjoint Volume Starvation -/

/--
**Arithmetic Lemma for Theorem 3.3(i)–(ii) [2.3(a)–(b)] (Two-Triple Capacity Inequality at `t = 3`):**
For `t = 3`, `m = 2 * t * t + t - 4 = 17`, and `cap_disj = 2 * t + (2 * t - 2) * (t - 1) = 14`,
`cap_disj < m` holds on `ℕ`. In Theorem 3.3 of [Pa26], this compares the 6-color capacity bound
against the 17 hyperedges of the disjoint two-triple PBD `H_{2,disj}(7)`.
-/
theorem erdos_19.variants.two_triple_disjoint_volume_starvation (t m cap_disj : ℕ)
    (ht : t = 3)
    (hm : m = 2 * t * t + t - 4)
    (hcap : cap_disj = 2 * t + (2 * t - 2) * (t - 1)) :
    cap_disj < m := by
  subst ht
  omega

/-! ## 4. Theorem 3.4(i) [2.4(a)]: Vizing–Fournier Shadow-Slack Decomposition -/

/--
**Arithmetic Lemma for Theorem 3.4(i) [2.4(a)] (Linear Elimination for Shadow-Slack):**
For natural numbers satisfying `n_minus_1 = d_2 + 2 * d_ge3 + rank_excess` and
`delta_vf + q_prime = 2 * d_ge3 + match_v + rank_excess + 1`, eliminating `2 * d_ge3 + rank_excess`
yields `n_minus_1 + match_v + 1 = d_2 + delta_vf + q_prime` (the arithmetic elimination step in
the Vizing–Fournier shadow-slack identity of Theorem 3.4(i) in [Pa26]).
-/
theorem erdos_19.variants.vizing_fournier_shadow_slack_identity
    (n_minus_1 d_2 d_ge3 rank_excess match_v q_prime delta_vf : ℕ)
    (h_pbd_deg : n_minus_1 = d_2 + 2 * d_ge3 + rank_excess)
    (h_delta : delta_vf + q_prime = 2 * d_ge3 + match_v + rank_excess + 1) :
    n_minus_1 + match_v + 1 = d_2 + delta_vf + q_prime := by
  omega

/-! ## 5. Theorem 3.5(i) & 3.11(i) [2.5(a) & 2.10(a)]: Fournier 2-Core Forest Leaf Discharging -/

/--
**Arithmetic Lemma for Theorem 3.5(i) & 3.11(i) (Forest Edge-Double Identity):**
For natural numbers `1 ≤ c ≤ V_T` and `E_T = V_T - c`, `2 * E_T + 2 * c = 2 * V_T` and
`2 * E_T < 2 * V_T` (the arithmetic step relating forest edge count `E_T = V_T - c` to twice
the vertex count `2 * V_T` in Theorem 3.5(i) of [Pa26]).
-/
theorem erdos_19.variants.fournier_forest_degree_sum_lt (V_T E_T c : ℕ)
    (_hV : 1 ≤ V_T) (hc : 1 ≤ c) (hc_le : c ≤ V_T) (h_forest : E_T = V_T - c) :
    2 * E_T + 2 * c = 2 * V_T ∧ 2 * E_T < 2 * V_T := by
  omega

/-! ## 6. Theorem 3.6(i) [2.6(a)]: Exact Shadow-Slack Identity -/

/--
**Arithmetic Lemma for Theorem 3.6(i) [2.6(a)] (Additive Cancellation Identity):**
For natural numbers `S_B`, `pair_twice`, and `e2_I` with `2 * e2_I ≤ pair_twice`,
`(S_B + 2 * e2_I) + (pair_twice - 2 * e2_I) = S_B + pair_twice` (the arithmetic cancellation step
in the Shadow-Slack identity of Theorem 3.6(i) in [Pa26]).
-/
theorem erdos_19.variants.shadow_slack_identity (S_B pair_twice e2_I : ℕ)
    (h_le : 2 * e2_I ≤ pair_twice) :
    (S_B + 2 * e2_I) + (pair_twice - 2 * e2_I) = S_B + pair_twice := by
  omega

/-! ## 7. Theorem 3.7(a) [2.6(b)]: Edmonds Blossom Odd-Set Shadow-Slack Bound -/

/--
**Arithmetic Lemma for Theorem 3.7(a) [2.6(b)] (Linear Degree-Sum Inequality):**
For natural numbers `t ≥ 1` and `delta ≥ 1` with `delta + 1 ≤ 2 * t`, any `deg_sum_U` satisfying
`deg_sum_U ≤ 2 * delta + (2 * t - 1) * (delta - 1)` also satisfies `deg_sum_U ≤ 2 * t * delta`
(the arithmetic inequality used in the odd-set degree-sum bound of Theorem 3.7(a) in [Pa26]).
-/
theorem erdos_19.variants.edmonds_blossom_odd_set_slack_bound
    (t delta deg_sum_U : ℕ)
    (ht : 1 ≤ t) (hdelta : 1 ≤ delta)
    (h_large_odd : delta + 1 ≤ 2 * t)
    (h_deg_bound : deg_sum_U ≤ 2 * delta + (2 * t - 1) * (delta - 1)) :
    deg_sum_U ≤ 2 * t * delta := by
  have h_expand : (2 * t - 1) * (delta - 1) + (2 * t - 1) + delta = 2 * t * delta := by
    have h1 : (2 * t - 1) * (delta - 1 + 1) = (2 * t - 1) * (delta - 1) + (2 * t - 1) * 1 :=
      Nat.mul_add (2 * t - 1) (delta - 1) 1
    have h2 : delta - 1 + 1 = delta := by omega
    rw [h2, Nat.mul_one] at h1
    have h3 : (2 * t - 1 + 1) * delta = (2 * t - 1) * delta + 1 * delta :=
      Nat.add_mul (2 * t - 1) 1 delta
    have h4 : 2 * t - 1 + 1 = 2 * t := by omega
    rw [h4, Nat.one_mul] at h3
    omega
  omega

/-! ## 8. Theorem 3.8(i) [2.8(a)]: Resolvent Parallel-Class Peeling Slack Gain -/

/--
**Arithmetic Lemma for Theorem 3.8(i) [2.8(a)] (Linear Slack Increment Identity):**
For natural numbers `1 ≤ p ≤ d_ge3` and `2 * d_ge3 ≤ n`, `(n - p) - 2 * (d_ge3 - p) = (n - 2 * d_ge3) + p`
and `n - 2 * d_ge3 < (n - p) - 2 * (d_ge3 - p)` (the arithmetic step showing a `+p` slack increase
when `d_ge3` decreases by `p` and `n` decreases by `p` in Theorem 3.8(i) of [Pa26]).
-/
theorem erdos_19.variants.parallel_class_peeling_slack (n d_ge3 p : ℕ)
    (hp : 1 ≤ p) (hd : p ≤ d_ge3) (hn : 2 * d_ge3 ≤ n) :
    (n - p) - 2 * (d_ge3 - p) = (n - 2 * d_ge3) + p ∧
      n - 2 * d_ge3 < (n - p) - 2 * (d_ge3 - p) := by
  omega

/-! ## 9. Theorem 3.9(i) [2.9(a)]: Large-Block Disjointness Overflow above `⌊√n⌋ + 1` -/

/--
**Arithmetic Lemma for Theorem 3.9(i) [2.9(a)] (Square Overflow Inequality):**
For natural numbers `n, s` with `n ≤ s * (s + 2)`, `n < (s + 1) * (s + 1)` since
`(s + 1)^2 = s(s + 2) + 1` (the arithmetic step used in Theorem 3.9(i) of [Pa26] to show that
`s + 1` disjoint blocks of size `≥ s + 1` require more than `s(s + 2) ≥ n` vertices).
-/
theorem erdos_19.variants.large_block_disjoint_overflow (n s : ℕ)
    (hn : n ≤ s * (s + 2)) :
    n < (s + 1) * (s + 1) := by
  have h_expand : (s + 1) * (s + 1) = s * (s + 2) + 1 := by
    calc
      (s + 1) * (s + 1) = s * (s + 1) + 1 * (s + 1) := Nat.add_mul s 1 (s + 1)
      _ = (s * s + s * 1) + (s + 1) := by rw [Nat.mul_add, Nat.one_mul]
      _ = (s * s + s * 2) + 1 := by omega
      _ = s * (s + 2) + 1 := by rw [Nat.mul_add]
  omega

/-! ## 10. Theorem 3.10(i) & 3.11(ii) [2.9(b) & 2.10(b)]: Prime-Power AG(2,q)^{+p} Chromatic Interpolation -/

/--
**Arithmetic Lemma for Theorem 3.10(i) [2.9(b)] (Polynomial Slack Identity for `1 ≤ p ≤ q`):**
Whenever `1 ≤ p ≤ q` and `q ≥ 2`, `(q - 1) * p + (q + 1) + (q - 1) * (q - p) + (p - 1) = q * q + p`
(the arithmetic identity comparing `(q - 1)p + (q + 1)` and `q^2 + p` in Theorem 3.10(i) of [Pa26]).
-/
theorem erdos_19.variants.ag2q_ideal_point_exact_slack_formula (q p : ℕ)
    (hq : 2 ≤ q) (hp_pos : 1 ≤ p) (hp : p ≤ q) :
    (q - 1) * p + (q + 1) + (q - 1) * (q - p) + (p - 1) = q * q + p := by
  have h_dist : (q - 1) * p + (q - 1) * (q - p) = (q - 1) * q := by
    rw [← Nat.mul_add, Nat.add_sub_of_le hp]
  have h_sq : (q - 1) * q + q = q * q := by
    have h1 : (q - 1 + 1) * q = (q - 1) * q + 1 * q := Nat.add_mul (q - 1) 1 q
    have h2 : q - 1 + 1 = q := by omega
    rw [h2, Nat.one_mul] at h1
    exact h1.symm
  omega

/--
**Arithmetic Lemma for Theorem 3.10(i) & 3.11(ii) (Polynomial Inequality `(q - 1)p + (q + 1) ≤ q^2 + p`):**
For every natural number `q ≥ 2` and `0 ≤ p ≤ q`, `(q - 1) * p + (q + 1) ≤ q * q + p`
(the arithmetic inequality showing that the formula `(q - 1)p + (q + 1)` does not exceed `q^2 + p`
in Theorem 3.10(i) of [Pa26]).
-/
theorem erdos_19.variants.ag2q_ideal_point_chromatic_le_n (q p : ℕ)
    (hq : 2 ≤ q) (hp : p ≤ q) :
    (q - 1) * p + (q + 1) ≤ q * q + p := by
  cases p with
  | zero =>
    have h_sq : (q - 1) * q + q = q * q := by
      have h1 : (q - 1 + 1) * q = (q - 1) * q + 1 * q := Nat.add_mul (q - 1) 1 q
      have h2 : q - 1 + 1 = q := by omega
      rw [h2, Nat.one_mul] at h1
      exact h1.symm
    have h_ge : q ≤ (q - 1) * q := by
      have h3 : 1 * q ≤ (q - 1) * q := Nat.mul_le_mul_right q (by omega)
      simpa using h3
    omega
  | succ m =>
    have h := erdos_19.variants.ag2q_ideal_point_exact_slack_formula q (m + 1) hq (by omega) hp
    omega

/-! ## 11. Theorem 3.10(ii) & 3.11(i)–(iii) [2.10(a)–(c)]: Universal Three-Tier Pair-Budget Discharging Closure -/

/--
**Arithmetic Lemma for Theorem 3.10(ii) & 3.11(i) (Three-Term Partition Subtraction):**
For natural numbers `total_pairs`, `large_pairs`, `med_pairs`, and `e2` satisfying
`large_pairs + med_pairs + e2 = total_pairs`, `total_pairs - large_pairs - med_pairs = e2`.
-/
theorem erdos_19.variants.three_tier_pair_budget_nonneg (total_pairs large_pairs med_pairs e2 : ℕ)
    (h_pbd : large_pairs + med_pairs + e2 = total_pairs) :
    total_pairs - large_pairs - med_pairs = e2 := by
  omega

/--
**Arithmetic Lemma for Theorem 3.11(a) [2.10(a)] (Global Linear Conservation Identity):**
For natural numbers `1 ≤ q' ≤ n` and `F_tot ≤ e2` satisfying `2 * (P_L + P_M + e2) = n * (n - 1)`
and `sum_delta_vf + n * (q' - 1) = 2 * (P_L + P_M + F_tot)`, we have
`sum_delta_vf + 2 * (e2 - F_tot) = n * (n - q')` and `2 * (P_L + P_M + F_tot) = n * (q' - 1) + sum_delta_vf`
(the linear elimination step in Theorem 3.11(a) of [Pa26]).
-/
theorem erdos_19.variants.global_shadow_slack_conservation
    (n q' P_L P_M e2 F_tot sum_delta_vf : ℕ)
    (hq_pos : 1 ≤ q') (hq_le : q' ≤ n) (hF_le : F_tot ≤ e2)
    (h_pbd_twice : 2 * (P_L + P_M + e2) = n * (n - 1))
    (h_slack_sum : sum_delta_vf + n * (q' - 1) = 2 * (P_L + P_M + F_tot)) :
    sum_delta_vf + 2 * (e2 - F_tot) = n * (n - q') ∧
    2 * (P_L + P_M + F_tot) = n * (q' - 1) + sum_delta_vf := by
  have h1 : n * (n - q') + n * (q' - 1) = n * (n - 1) := by
    rw [← Nat.mul_add]
    have h_add : (n - q') + (q' - 1) = n - 1 := by omega
    rw [h_add]
  omega

/--
**Arithmetic Lemma for Theorem 3.11(c) [2.10(c)] (Linear Budget Contradiction):**
For natural numbers satisfying `F_tot ≤ e2`, `2 * (P_L + P_M + e2) = n * (n - 1)`, and
`2 * (e2 - F_tot) = k * (n - q') + deg_sum_outside`, the strict inequality
`n * (n - 1) < 2 * (P_L + P_M + F_tot) + k * (n - q') + deg_sum_outside` is impossible (`False`).
-/
theorem erdos_19.variants.three_tier_fournier_obstruction_bound
    (n q' k P_L P_M e2 F_tot deg_sum_outside : ℕ)
    (_hk : 3 ≤ k) (_hk_le : k ≤ n) (_hq_le : q' ≤ n) (hF_le : F_tot ≤ e2)
    (h_pbd_twice : 2 * (P_L + P_M + e2) = n * (n - 1))
    (h_core_deg : 2 * (e2 - F_tot) = k * (n - q') + deg_sum_outside)
    (h_budget_exceeds : n * (n - 1) < 2 * (P_L + P_M + F_tot) + k * (n - q') + deg_sum_outside) :
    False := by
  omega

/-! ## 12. Section 4: Certified Extremal Projective Plane Witnesses PG(2,2) & PG(2,3) -/

/--
Checks that a family of hyperedges on `{0, ..., n-1}` is a `(n, k, 1)` Steiner 2-design /
projective plane: there are `n` blocks, every block has `k` distinct vertices in `{0, ..., n-1}`,
and the `n` blocks cover all `n * (n - 1) / 2` distinct unordered vertex pairs without duplicates.
-/
def pairsOfBlock : List ℕ → List (ℕ × ℕ)
  | [] => []
  | u :: rest => (rest.map (fun v => if u < v then (u, v) else (v, u))) ++ pairsOfBlock rest

def allBlockPairs (blocks : List (List ℕ)) : List (ℕ × ℕ) :=
  blocks.foldr (fun b acc => pairsOfBlock b ++ acc) []

def listNoDups : List ℕ → Bool
  | [] => true
  | v :: rest => !(rest.contains v) && listNoDups rest

def pairListNoDups : List (ℕ × ℕ) → Bool
  | [] => true
  | p :: rest => !(rest.contains p) && pairListNoDups rest

def isCertifiedProjectivePlane (n k : ℕ) (blocks : List (List ℕ)) : Bool :=
  (blocks.length == n) &&
  (blocks.all (fun b => b.length == k && listNoDups b && b.all (fun v => v < n))) &&
  (let pairs := allBlockPairs blocks
   (pairs.length == n * (n - 1) / 2) && pairListNoDups pairs)

/-- Fano plane `PG(2, 2)` on `n = 7` vertices with `7` lines of size `k = 3`. -/
def fanoPlanePG22 : List (List ℕ) :=
  [[0, 1, 3], [1, 2, 4], [2, 3, 5], [3, 4, 6], [4, 5, 0], [5, 6, 1], [6, 0, 2]]

theorem fanoPlanePG22_certified :
    isCertifiedProjectivePlane 7 3 fanoPlanePG22 = true := by
  decide

/-- Projective plane `PG(2, 3)` on `n = 13` vertices with `13` lines of size `k = 4`. -/
def projectivePlanePG23 : List (List ℕ) :=
  [[0, 1, 3, 9], [1, 2, 4, 10], [2, 3, 5, 11], [3, 4, 6, 12],
   [4, 5, 7, 0], [5, 6, 8, 1],  [6, 7, 9, 2],  [7, 8, 10, 3],
   [8, 9, 11, 4], [9, 10, 12, 5], [10, 11, 0, 6], [11, 12, 1, 7],
   [12, 0, 2, 8]]

theorem projectivePlanePG23_certified :
    isCertifiedProjectivePlane 13 4 projectivePlanePG23 = true := by
  decide

end Erdos19
