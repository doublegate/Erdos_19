//! ============================================================================
//! `verify_cpu.rs` — Exact CPU verification, primal-dual conversion, line-graph
//! degree identities, De Bruijn–Erdős clique / matching bounds, and DSatur +
//! Star-Clique Hall-Capacity branch-and-bound chromatic index solver for
//! Erdős Problem #19 (the Erdős–Faber–Lovász Conjecture).
//! ============================================================================

use serde::{Deserialize, Serialize};

/// Complete structural and chromatic report for a linear hypergraph `H = ([n], E)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EflReport {
    /// Number of ground vertices `n = |X|`.
    pub n: usize,
    /// Number of hyperedges `m = |E|` of size >= 2.
    pub num_edges: usize,
    /// Number of singleton hyperedges stripped during primal-dual reduction.
    pub stripped_singletons: usize,
    /// True iff `|e_a ∩ e_b| <= 1` for all `0 <= a < b < m`.
    pub is_linear: bool,
    /// Maximum pairwise intersection `max_{a < b} |e_a ∩ e_b|`.
    pub max_pair_intersection: u32,
    /// Minimum hyperedge rank `min_{e in E} |e|`.
    pub min_edge_rank: u32,
    /// Maximum hyperedge rank `max_{e in E} |e|`.
    pub max_edge_rank: u32,
    /// Pair-packing sum `P(H) = sum_{e in E} binom(|e|, 2)`.
    pub pair_packing_sum: usize,
    /// Total vertex pairs `binom(n, 2)`.
    pub total_vertex_pairs: usize,
    /// True iff `H` is a Pairwise Balanced Design (`is_linear && P(H) == binom(n, 2)`).
    pub is_pbd: bool,
    /// Maximum point degree `Delta(H) = max_{i in [n]} d_H(i)`.
    pub max_point_degree: usize,
    /// Maximum line-graph degree `Delta(L(H)) = max_{e in E} d_{L(H)}(e)`.
    pub max_line_graph_degree: usize,
    /// True iff Theorem 2.2(i) line-graph degree identity holds for all `e in E`.
    pub degree_identity_verified: bool,
    /// Exact clique number `ω(L(H))` of the hyperedge line graph.
    pub clique_number_omega: usize,
    /// Exact matching number `α(L(H))` (maximum number of pairwise disjoint hyperedges).
    pub matching_number_alpha: usize,
    /// Matching ratio lower bound `ceil(|E| / α(L(H)))`.
    pub matching_ratio_bound: usize,
    /// Volume-parity lower bound from Theorem 2.2(iii).
    pub parity_volume_bound: usize,
    /// Kahn–Seymour fractional lower bound `max(ω(L(H)), |E| / α(L(H)))`.
    pub fractional_lower_bound: f64,
    /// Exact chromatic index `χ'(H) = χ(L(H))`.
    pub chromatic_index: usize,
    /// Valid proper hyperedge coloring in `0..chromatic_index` for each edge `0..m`.
    pub edge_coloring: Vec<usize>,
    /// Number of branch-and-bound search nodes explored.
    pub search_nodes: u64,
    /// True iff `χ'(H) <= n` (Erdős–Faber–Lovász property holds).
    pub efl_holds: bool,
    /// Slack `n - χ'(H)` (`0` when tight).
    pub efl_slack: isize,
}

/// Convert a primal EFL family of `n` cliques `K^{(0)}, ..., K^{(n-1)}` (given as lists of
/// primal vertex IDs) into its dual hypergraph on ground set `0..n`, stripping singleton
/// hyperedges (`|e_v| == 1`) per Theorem 2.1 (Singleton-Core Reduction).
///
/// Returns `(n, stripped_edges_ge2, stripped_singleton_count)`.
pub fn primal_cliques_to_dual_hypergraph(cliques: &[Vec<usize>]) -> (usize, Vec<u64>, usize) {
    let n = cliques.len();
    assert!(n <= 64, "ground set size n must be <= 64");
    let mut max_v = 0usize;
    let mut has_v = false;
    for c in cliques {
        for &v in c {
            has_v = true;
            if v > max_v {
                max_v = v;
            }
        }
    }
    if !has_v {
        return (n, Vec::new(), 0);
    }
    let mut vertex_incidence = vec![0u64; max_v + 1];
    for (clique_idx, c) in cliques.iter().enumerate() {
        for &v in c {
            vertex_incidence[v] |= 1u64 << clique_idx;
        }
    }
    let mut edges_ge2 = Vec::new();
    let mut singletons = 0usize;
    for mask in vertex_incidence {
        let sz = mask.count_ones();
        if sz >= 2 {
            edges_ge2.push(mask);
        } else if sz == 1 {
            singletons += 1;
        }
    }
    (n, edges_ge2, singletons)
}

/// Convert a dual linear hypergraph `H = ([n], E)` (with `|e| >= 2`) into its canonical
/// primal union of `n` cliques `K^{(0)}, ..., K^{(n-1)}`, padding each clique `i` with
/// private singleton vertices so every clique has exact order `n`.
pub fn dual_hypergraph_to_primal_cliques(n: usize, edges: &[u64]) -> Vec<Vec<usize>> {
    let mut cliques = vec![Vec::new(); n];
    for (v_id, &mask) in edges.iter().enumerate() {
        for (i, clique) in cliques.iter_mut().enumerate().take(n) {
            if ((mask >> i) & 1) != 0 {
                clique.push(v_id);
            }
        }
    }
    let mut next_private_v = edges.len();
    for clique in cliques.iter_mut().take(n) {
        while clique.len() < n {
            clique.push(next_private_v);
            next_private_v += 1;
        }
    }
    cliques
}

/// Compute `max_{a < b} |e_a ∩ e_b|` across all hyperedge pairs.
pub fn max_pair_intersection_cpu(edges: &[u64]) -> u32 {
    let mut max_int = 0u32;
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            let c = (edges[i] & edges[j]).count_ones();
            if c > max_int {
                max_int = c;
            }
        }
    }
    max_int
}

/// Check whether `edges` forms a linear hypergraph (`|e_a ∩ e_b| <= 1` for all `a < b`).
pub fn is_linear_hypergraph_cpu(edges: &[u64]) -> bool {
    max_pair_intersection_cpu(edges) <= 1
}

/// Compute the pair-packing sum `P(H) = sum_{e in E} binom(|e|, 2)`.
pub fn pair_packing_sum_cpu(edges: &[u64]) -> usize {
    edges
        .iter()
        .map(|&e| {
            let r = e.count_ones() as usize;
            r.saturating_sub(1) * r / 2
        })
        .sum()
}

/// Construct the 64-bit adjacency masks of the line graph `L(H)` for `m <= 64` hyperedges.
pub fn build_line_graph_adj64(edges: &[u64]) -> Vec<u64> {
    let m = edges.len();
    assert!(
        m <= 64,
        "line graph bitmask solver supports up to 64 hyperedges"
    );
    let mut adj = vec![0u64; m];
    for a in 0..m {
        for b in (a + 1)..m {
            if (edges[a] & edges[b]) != 0 {
                adj[a] |= 1u64 << b;
                adj[b] |= 1u64 << a;
            }
        }
    }
    adj
}

/// Verify Theorem 2.2(i) exact line-graph degree identity and upper bound:
/// For every linear hypergraph `H` and every `e in E`,
/// `d_{L(H)}(e) == sum_{i in e} (d_H(i) - 1) <= sum_{i in e} floor((n - |e|) / (r_min(i) - 1))`.
pub fn verify_degree_identity_cpu(n: usize, edges: &[u64], adj: &[u64]) -> bool {
    if !is_linear_hypergraph_cpu(edges) {
        return false;
    }
    let mut point_deg = vec![0usize; n];
    let mut point_rmin = vec![usize::MAX; n];
    for &e in edges {
        let r = e.count_ones() as usize;
        if r < 2 {
            return false;
        }
        for i in 0..n {
            if ((e >> i) & 1) != 0 {
                point_deg[i] += 1;
                if r < point_rmin[i] {
                    point_rmin[i] = r;
                }
            }
        }
    }
    for (a, &e) in edges.iter().enumerate() {
        let deg_lh = adj[a].count_ones() as usize;
        let r = e.count_ones() as usize;
        let mut sum_pt = 0usize;
        let mut sum_ub = 0usize;
        for i in 0..n {
            if ((e >> i) & 1) != 0 {
                sum_pt += point_deg[i].saturating_sub(1);
                let rmin_i = point_rmin[i].max(2);
                sum_ub += (n.saturating_sub(r)) / (rmin_i - 1);
            }
        }
        if deg_lh != sum_pt || deg_lh > sum_ub {
            return false;
        }
    }
    true
}

/// Compute the exact maximum clique mask in `L(H)` (size `ω(L(H))`) via bitmask branch-and-bound.
pub fn max_clique_mask_cpu(adj: &[u64]) -> (usize, u64) {
    let m = adj.len();
    if m == 0 {
        return (0, 0);
    }
    // Order vertices by non-increasing degree for rapid lower-bound discovery
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by_key(|&v| std::cmp::Reverse(adj[v].count_ones()));

    let mut best_size = 0usize;
    let mut best_mask = 0u64;

    fn expand(
        adj: &[u64],
        candidates: u64,
        current_mask: u64,
        current_size: usize,
        best_size: &mut usize,
        best_mask: &mut u64,
    ) {
        if candidates == 0 {
            if current_size > *best_size {
                *best_size = current_size;
                *best_mask = current_mask;
            }
            return;
        }
        if current_size + (candidates.count_ones() as usize) <= *best_size {
            return;
        }
        let mut cand = candidates;
        while cand != 0 {
            if current_size + (cand.count_ones() as usize) <= *best_size {
                break;
            }
            let v = cand.trailing_zeros() as usize;
            let bit = 1u64 << v;
            cand &= !bit;
            expand(
                adj,
                cand & adj[v],
                current_mask | bit,
                current_size + 1,
                best_size,
                best_mask,
            );
        }
    }

    let mut init_cand = if m == 64 { u64::MAX } else { (1u64 << m) - 1 };
    for &v in &order {
        let bit = 1u64 << v;
        init_cand &= !bit;
        expand(
            adj,
            init_cand & adj[v],
            bit,
            1,
            &mut best_size,
            &mut best_mask,
        );
    }

    (best_size, best_mask)
}

/// Compute the exact matching number `α(L(H))` (maximum number of pairwise vertex-disjoint
/// hyperedges in `H`) using ground-vertex bitmask packing on `[n]`.
pub fn max_matching_alpha_cpu(n: usize, edges: &[u64]) -> usize {
    let m = edges.len();
    if m == 0 {
        return 0;
    }
    // Sort edges by ascending size |e| so smaller hyperedges pack first
    let mut sorted_edges = edges.to_vec();
    sorted_edges.sort_by_key(|e| e.count_ones());

    let mut best_alpha = 1usize;

    fn dfs_pack(
        n: usize,
        edges: &[u64],
        idx: usize,
        used_verts: u64,
        cur_count: usize,
        best_alpha: &mut usize,
    ) {
        if cur_count > *best_alpha {
            *best_alpha = cur_count;
        }
        let free_verts = n.saturating_sub(used_verts.count_ones() as usize);
        // Since edges are sorted by ascending size, remaining edges have size >= edges[idx].count_ones()
        if idx >= edges.len() {
            return;
        }
        let min_rem_sz = (edges[idx].count_ones() as usize).max(2);
        let max_more = (free_verts / min_rem_sz).min(edges.len() - idx);
        if cur_count + max_more <= *best_alpha {
            return;
        }
        for i in idx..edges.len() {
            let e = edges[i];
            let sz = e.count_ones() as usize;
            if cur_count + (free_verts / sz.max(2)).min(edges.len() - i) <= *best_alpha {
                break;
            }
            if (used_verts & e) == 0 {
                dfs_pack(n, edges, i + 1, used_verts | e, cur_count + 1, best_alpha);
            }
        }
    }

    dfs_pack(n, &sorted_edges, 0, 0, 0, &mut best_alpha);
    best_alpha
}

/// Compute the Theorem 2.2(iii) volume-parity lower bound on `χ'(H)`.
pub fn parity_volume_lower_bound(n: usize, edges: &[u64]) -> usize {
    if edges.is_empty() || n <= 1 {
        return 0;
    }
    let total_vol: usize = edges.iter().map(|e| e.count_ones() as usize).sum();
    if n.is_multiple_of(2) {
        total_vol.div_ceil(n)
    } else {
        let m_odd = edges
            .iter()
            .filter(|&&e| !e.count_ones().is_multiple_of(2))
            .count();
        // From Theorem 2.2(iii): sum_{e in E} |e| <= C * (n - 1) + min(C, m_odd)
        for c in 1..=edges.len().max(n) {
            if c * (n - 1) + c.min(m_odd) >= total_vol {
                return c;
            }
        }
        n
    }
}

/// Solve the exact chromatic index `χ'(H) = χ(L(H))` using DSatur + Clique Pre-Coloring +
/// Star-Clique Hall-Capacity Pruning.
///
/// Returns `(chromatic_index, coloring, total_search_nodes)`.
pub fn solve_chromatic_index_cpu(n: usize, edges: &[u64]) -> (usize, Vec<usize>, u64) {
    let m = edges.len();
    if m == 0 {
        return (0, Vec::new(), 0);
    }
    let adj = build_line_graph_adj64(edges);
    let (omega, clique_mask) = max_clique_mask_cpu(&adj);
    let alpha = max_matching_alpha_cpu(n, edges);
    let match_lb = m.div_ceil(alpha.max(1));
    let vol_lb = parity_volume_lower_bound(n, edges);
    let lower_bound = omega.max(match_lb).max(vol_lb).max(1);

    // Precompute for each ground vertex v in 0..n the bitmask of incident hyperedges S_v
    let mut point_star_masks = vec![0u64; n];
    for (a, &e) in edges.iter().enumerate() {
        for (v, star) in point_star_masks.iter_mut().enumerate().take(n) {
            if ((e >> v) & 1) != 0 {
                *star |= 1u64 << a;
            }
        }
    }

    // Extract clique edges for root symmetry breaking
    let mut clique_edges = Vec::with_capacity(omega);
    for a in 0..m {
        if ((clique_mask >> a) & 1) != 0 {
            clique_edges.push(a);
        }
    }

    let all_edges_mask = if m == 64 { u64::MAX } else { (1u64 << m) - 1 };
    let mut total_nodes = 0u64;

    for target_k in lower_bound..=m.max(n) {
        assert!(target_k <= 32, "color bitmask supports up to 32 colors");
        let full_color_mask: u32 = if target_k == 32 {
            u32::MAX
        } else {
            (1u32 << target_k) - 1
        };

        let mut colors = vec![usize::MAX; m];
        let mut color_verts = vec![0u64; target_k];
        let mut avail = vec![full_color_mask; m];
        let mut uncolored_mask = all_edges_mask;

        // Pre-color the maximum clique C_0 with colors 0..omega-1
        for (c, &e_idx) in clique_edges.iter().enumerate() {
            colors[e_idx] = c;
            color_verts[c] |= edges[e_idx];
            uncolored_mask &= !(1u64 << e_idx);
            let c_bit = 1u32 << c;
            let mut nbrs = adj[e_idx] & uncolored_mask;
            while nbrs != 0 {
                let nb = nbrs.trailing_zeros() as usize;
                nbrs &= nbrs - 1;
                avail[nb] &= !c_bit;
            }
        }

        let max_color_used = omega.saturating_sub(1);
        if dfs_color_k(
            n,
            edges,
            &adj,
            &point_star_masks,
            target_k,
            uncolored_mask,
            max_color_used,
            &mut colors,
            &mut color_verts,
            &mut avail,
            &mut total_nodes,
        ) {
            return (target_k, colors, total_nodes);
        }
    }

    unreachable!("m colors always suffice for m hyperedges")
}

#[allow(clippy::too_many_arguments)]
fn dfs_color_k(
    n: usize,
    edges: &[u64],
    adj: &[u64],
    point_star_masks: &[u64],
    target_k: usize,
    uncolored_mask: u64,
    max_color_used: usize,
    colors: &mut [usize],
    color_verts: &mut [u64],
    avail: &mut [u32],
    nodes: &mut u64,
) -> bool {
    if uncolored_mask == 0 {
        return true;
    }

    // 1. Global volume-parity capacity check: each color class c has (n - popcount(color_verts[c]))
    //    free vertices, so it can accommodate at most floor((n - used_c) / 2) additional hyperedges.
    let rem_edges = uncolored_mask.count_ones() as usize;
    let mut total_pair_cap = 0usize;
    for &cv in color_verts.iter().take(target_k) {
        let free_v = n.saturating_sub(cv.count_ones() as usize);
        total_pair_cap += free_v / 2;
    }
    if total_pair_cap < rem_edges {
        return false;
    }

    // 2. Star-Clique Hall-Capacity Prune: for every vertex v in 0..n, the uncolored edges
    //    incident to v form a clique in L(H), so the union of their available colors must
    //    have cardinality at least |S_v ∩ uncolored_mask|.
    for &star in point_star_masks.iter().take(n) {
        let rem_star = star & uncolored_mask;
        let need = rem_star.count_ones();
        if need >= 2 {
            let mut union_colors = 0u32;
            let mut s = rem_star;
            while s != 0 {
                let e = s.trailing_zeros() as usize;
                s &= s - 1;
                union_colors |= avail[e];
            }
            if union_colors.count_ones() < need {
                return false;
            }
        }
    }

    // 3. DSatur variable selection: pick uncolored edge with fewest available colors,
    //    breaking ties by largest uncolored degree in L(H).
    let mut best_e = usize::MAX;
    let mut min_avail = u32::MAX;
    let mut max_deg = 0u32;
    let mut cand = uncolored_mask;
    while cand != 0 {
        let e = cand.trailing_zeros() as usize;
        cand &= cand - 1;
        let ac = avail[e].count_ones();
        if ac == 0 {
            return false;
        }
        let deg = (adj[e] & uncolored_mask).count_ones();
        if ac < min_avail || (ac == min_avail && deg > max_deg) {
            min_avail = ac;
            max_deg = deg;
            best_e = e;
        }
    }

    let next_uncolored = uncolored_mask & !(1u64 << best_e);
    let color_limit = (max_color_used + 1).min(target_k - 1);
    let mut c_mask = avail[best_e];

    while c_mask != 0 {
        let c = c_mask.trailing_zeros() as usize;
        c_mask &= c_mask - 1;
        if c > color_limit {
            break;
        }

        *nodes += 1;
        colors[best_e] = c;
        let prev_cv = color_verts[c];
        color_verts[c] = prev_cv | edges[best_e];

        // Record neighbors whose availability changes so we can restore in O(deg)
        let c_bit = 1u32 << c;
        let mut changed_nbrs = 0u64;
        let mut nbrs = adj[best_e] & next_uncolored;
        let mut dead = false;
        while nbrs != 0 {
            let nb = nbrs.trailing_zeros() as usize;
            nbrs &= nbrs - 1;
            if (avail[nb] & c_bit) != 0 {
                avail[nb] &= !c_bit;
                changed_nbrs |= 1u64 << nb;
                if avail[nb] == 0 {
                    dead = true;
                }
            }
        }

        if !dead
            && dfs_color_k(
                n,
                edges,
                adj,
                point_star_masks,
                target_k,
                next_uncolored,
                max_color_used.max(c),
                colors,
                color_verts,
                avail,
                nodes,
            )
        {
            return true;
        }

        // Restore neighbor availabilities
        let mut restore = changed_nbrs;
        while restore != 0 {
            let nb = restore.trailing_zeros() as usize;
            restore &= restore - 1;
            avail[nb] |= c_bit;
        }
        color_verts[c] = prev_cv;
        colors[best_e] = usize::MAX;
    }

    false
}

/// Verify that a hyperedge coloring `colors[0..m]` is a valid proper `k`-edge-coloring of `H`.
pub fn is_valid_edge_coloring_cpu(edges: &[u64], colors: &[usize], max_colors: usize) -> bool {
    if edges.len() != colors.len() {
        return false;
    }
    for i in 0..edges.len() {
        if colors[i] >= max_colors {
            return false;
        }
        for j in (i + 1)..edges.len() {
            if (edges[i] & edges[j]) != 0 && colors[i] == colors[j] {
                return false;
            }
        }
    }
    true
}

/// Theorem 2.10(a): Bit-exact CPU twin of `k_verify_three_tier_discharging_closure` (`src/kernels.cu`).
///
/// Partitions a reduced linear hypergraph `H = ([n], E)` into `E = E_2 ⊔ M ⊔ L` around
/// `tau = floor(sqrt(n)) + 1` (`M = {E : 3 <= |E| <= tau - 1}`, `L = {E : |E| >= tau}`)
/// and verifies:
///   1. The Local-to-Global Intermediate Shadow Identity at every vertex `v in 0..n`:
///      `s_M(v) == sigma(v) - d_2(v) - d_{>=3}(v) - s_L(v)`,
///      where `s_M(v) = sum_{M ni v} (|M| - 2)`, `s_L(v) = sum_{L ni v} (|L| - 2)`,
///      and `sigma(v) = sum_{E ni v} (|E| - 1) <= n - 1` (`== n - 1` on every PBD).
///   2. Global Three-Tier Pair Conservation:
///      `sum_{v=0}^{n-1} (d_M(v) + s_M(v)) == 2 * P(M)`,
///      `sum_{v=0}^{n-1} (d_L(v) + s_L(v)) == 2 * P(L)`,
///      and `Psi_{3-tier}(H) := 2 * binom(n, 2) - sum_{v=0}^{n-1} sigma(v) == 2 * (binom(n, 2) - P(L) - P(M) - |E_2|) >= 0`.
///
/// Returns `(tau, p_med, p_large, psi_3tier, closure_ok)`.
pub fn verify_three_tier_discharging_closure_cpu(
    n: u32,
    edges: &[u64],
) -> (u32, u32, u32, u32, u32) {
    let m = edges.len();
    assert!(m <= 64 && n <= 64);

    let mut root = 0u32;
    while (root + 1) * (root + 1) <= n {
        root += 1;
    }
    let tau = root + 1;

    let mut linear = 1u32;
    for a in 0..m {
        let ea = edges[a];
        if ea.count_ones() < 2 {
            linear = 0;
        }
        for &eb in edges.iter().take(m).skip(a + 1) {
            if (ea & eb).count_ones() > 1 {
                linear = 0;
            }
        }
    }

    let mut p_e2 = 0u32;
    let mut p_med = 0u32;
    let mut p_large = 0u32;

    let mut d2 = [0u32; 64];
    let mut d_med = [0u32; 64];
    let mut d_large = [0u32; 64];
    let mut s_med = [0u32; 64];
    let mut s_large = [0u32; 64];

    for &e in edges.iter().take(m) {
        let r = e.count_ones();
        let pairs = (r * r.saturating_sub(1)) / 2;
        if r == 2 {
            p_e2 += 1;
            for (v, slot) in d2.iter_mut().enumerate().take(n as usize) {
                if ((e >> v) & 1) != 0 {
                    *slot += 1;
                }
            }
        } else if r >= 3 && r < tau {
            p_med += pairs;
            for v in 0..(n as usize) {
                if ((e >> v) & 1) != 0 {
                    d_med[v] += 1;
                    s_med[v] += r - 2;
                }
            }
        } else if r >= tau {
            p_large += pairs;
            for v in 0..(n as usize) {
                if ((e >> v) & 1) != 0 {
                    d_large[v] += 1;
                    s_large[v] += r - 2;
                }
            }
        }
    }

    let total_pairs = (n * n.saturating_sub(1)) / 2;
    let mut sum_sigma = 0u32;
    let mut sum_med_inc = 0u32;
    let mut sum_large_inc = 0u32;
    let mut ok = linear;

    for v in 0..(n as usize) {
        let sigma_m = d_med[v] + s_med[v];
        let sigma_l = d_large[v] + s_large[v];
        let sigma_v = d2[v] + sigma_m + sigma_l;
        let d_ge3 = d_med[v] + d_large[v];
        if sigma_v > n.saturating_sub(1) {
            ok = 0;
        }
        if s_med[v] + d2[v] + d_ge3 + s_large[v] != sigma_v {
            ok = 0;
        }
        sum_sigma += sigma_v;
        sum_med_inc += sigma_m;
        sum_large_inc += sigma_l;
    }

    if sum_med_inc != 2 * p_med || sum_large_inc != 2 * p_large {
        ok = 0;
    }
    if p_e2 + p_med + p_large > total_pairs {
        ok = 0;
    }
    if sum_sigma != 2 * (p_e2 + p_med + p_large) {
        ok = 0;
    }

    let psi_3tier = (2 * total_pairs).saturating_sub(sum_sigma);
    if p_e2 + p_med + p_large == total_pairs && psi_3tier != 0 {
        ok = 0;
    }

    (tau, p_med, p_large, psi_3tier, ok)
}

/// Generate a complete `EflReport` for a linear hypergraph `H = ([n], edges)`.
pub fn analyze_hypergraph_cpu(n: usize, edges: &[u64], stripped_singletons: usize) -> EflReport {
    let m = edges.len();
    let max_int = max_pair_intersection_cpu(edges);
    let is_linear = max_int <= 1;
    let min_rank = edges.iter().map(|e| e.count_ones()).min().unwrap_or(0);
    let max_rank = edges.iter().map(|e| e.count_ones()).max().unwrap_or(0);
    let pair_sum = pair_packing_sum_cpu(edges);
    let total_pairs = n.saturating_sub(1) * n / 2;
    let is_pbd = is_linear && pair_sum == total_pairs;

    let mut max_pt_deg = 0usize;
    for i in 0..n {
        let d = edges.iter().filter(|&&e| ((e >> i) & 1) != 0).count();
        if d > max_pt_deg {
            max_pt_deg = d;
        }
    }

    let adj = build_line_graph_adj64(edges);
    let max_lg_deg = adj
        .iter()
        .map(|a| a.count_ones() as usize)
        .max()
        .unwrap_or(0);
    let deg_id_ok = verify_degree_identity_cpu(n, edges, &adj);

    let (omega, _) = max_clique_mask_cpu(&adj);
    let alpha = max_matching_alpha_cpu(n, edges);
    let match_lb = if m == 0 { 0 } else { m.div_ceil(alpha.max(1)) };
    let vol_lb = parity_volume_lower_bound(n, edges);
    let frac_lb = (omega as f64).max(if m == 0 {
        0.0
    } else {
        (m as f64) / (alpha.max(1) as f64)
    });

    let (chi_prime, coloring, nodes) = solve_chromatic_index_cpu(n, edges);
    assert!(is_valid_edge_coloring_cpu(edges, &coloring, chi_prime));

    EflReport {
        n,
        num_edges: m,
        stripped_singletons,
        is_linear,
        max_pair_intersection: max_int,
        min_edge_rank: min_rank,
        max_edge_rank: max_rank,
        pair_packing_sum: pair_sum,
        total_vertex_pairs: total_pairs,
        is_pbd,
        max_point_degree: max_pt_deg,
        max_line_graph_degree: max_lg_deg,
        degree_identity_verified: deg_id_ok,
        clique_number_omega: omega,
        matching_number_alpha: alpha,
        matching_ratio_bound: match_lb,
        parity_volume_bound: vol_lb,
        fractional_lower_bound: frac_lb,
        chromatic_index: chi_prime,
        edge_coloring: coloring,
        search_nodes: nodes,
        efl_holds: chi_prime <= n,
        efl_slack: (n as isize) - (chi_prime as isize),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records;

    #[test]
    fn test_primal_dual_singleton_reduction_bijection() {
        let fano = records::build_fano_pg2_2();
        let primal = dual_hypergraph_to_primal_cliques(7, &fano);
        assert_eq!(primal.len(), 7);
        for c in &primal {
            assert_eq!(c.len(), 7);
        }
        let (n_back, dual_back, stripped) = primal_cliques_to_dual_hypergraph(&primal);
        assert_eq!(n_back, 7);
        assert_eq!(dual_back, fano);
        // Each of the 7 cliques had 3 Fano vertices + 4 private singletons = 28 singletons
        assert_eq!(stripped, 28);
    }

    #[test]
    fn test_canonical_benchmark_suite_exact_chromatic_indices() {
        let suite = records::canonical_benchmark_suite();
        for inst in suite {
            let rep = analyze_hypergraph_cpu(inst.n, &inst.edges, 0);
            assert!(rep.is_linear, "{} must be linear", inst.name);
            assert!(
                rep.degree_identity_verified,
                "{} must satisfy Theorem 2.2(i) degree identity",
                inst.name
            );
            assert!(rep.efl_holds, "{} must satisfy EFL χ'(H) <= n", inst.name);
            assert_eq!(
                rep.chromatic_index, inst.expected_chi_prime,
                "mismatch on {}: got χ'={}, expected {}",
                inst.name, rep.chromatic_index, inst.expected_chi_prime
            );
        }
    }

    #[test]
    fn test_theorem_2_3_single_triple_parity_saturation() {
        // For n = 5, H_1(5) drops to n - 1 = 4.
        let rep5 = analyze_hypergraph_cpu(5, &records::build_single_triple_pbd(5), 0);
        assert_eq!(rep5.chromatic_index, 4);

        // For all n = 4 and all n >= 6 (both even n = 6, 8 and odd n = 7, 9),
        // Theorem 2.3 proves χ'(H_1(n)) = n despite ω(L(H_1(n))) = n - 1 for n >= 6!
        for &n in &[6usize, 7, 8, 9] {
            let rep = analyze_hypergraph_cpu(n, &records::build_single_triple_pbd(n), 0);
            assert_eq!(rep.clique_number_omega, n - 1);
            assert_eq!(rep.chromatic_index, n);
        }
    }
}
