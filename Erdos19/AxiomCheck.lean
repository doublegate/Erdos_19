/-
Copyright 2026 Lucas Parobek.
Licensed under the Apache License, Version 2.0.

Kernel axiom audit for all theorems in `Erdos19.Proofs`.
Running `lean Erdos19/AxiomCheck.lean` or `lake build` verifies that no theorem
depends on `sorryAx` or any custom axiom.
-/
import Erdos19.Proofs

#print axioms Erdos19.erdos_19.variants.pbd_two_edge_completion_conservation
#print axioms Erdos19.erdos_19.variants.single_triple_parity_saturation_chromatic
#print axioms Erdos19.erdos_19.variants.two_triple_disjoint_volume_starvation
#print axioms Erdos19.erdos_19.variants.vizing_fournier_shadow_slack_identity
#print axioms Erdos19.erdos_19.variants.fournier_forest_degree_sum_lt
#print axioms Erdos19.erdos_19.variants.shadow_slack_identity
#print axioms Erdos19.erdos_19.variants.parallel_class_peeling_slack
#print axioms Erdos19.erdos_19.variants.large_block_disjoint_overflow
#print axioms Erdos19.erdos_19.variants.ag2q_ideal_point_exact_slack_formula
#print axioms Erdos19.erdos_19.variants.ag2q_ideal_point_chromatic_le_n
#print axioms Erdos19.erdos_19.variants.three_tier_pair_budget_nonneg
#print axioms Erdos19.erdos_19.variants.three_tier_discharging_closure
#print axioms Erdos19.fanoPlanePG22_certified
#print axioms Erdos19.projectivePlanePG23_certified
