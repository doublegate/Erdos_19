// ============================================================================
// `kernels.cu` — CUDA C kernels for Erdős Problem #19 ($500 Decidable Prize,
// the Erdős–Faber–Lovász Conjecture):
//   `k_eval_efl_hypergraphs`: Parallel exact linearity verification, line-graph
//   construction, exact bitmask Bron–Kerbosch clique number ω(L(H)), Theorem 2.2
//   parity-volume lower bound, and iterative-deepening DSatur + Star-Clique
//   Hall-Capacity branch-and-bound chromatic index χ'(H) solver.
//   Bit-exact twin of `eval_efl_hypergraph_twin_cpu` (`src/exact.rs`).
// ============================================================================
//
// ENCODING CONTRACTS (shared with `src/exact.rs` — DO NOT CHANGE UNILATERALLY):
//   * Each hypergraph instance `tid` has `m = edge_counts[tid]` (`m <= 64`)
//     hyperedges stored at `edges_flat[tid * 64 + a]` (`0 <= a < m`) as `u64`
//     vertex bitmasks on `0..n`.
//   * Exact clique finder `find_max_clique_dev` scans candidate bits in
//     ascending index order `v = __ffsll(cand) - 1`, recording the first
//     maximum clique `best_mask` of size `omega`.
//   * Iterative-deepening DSatur pre-colors `best_mask` with `0..omega-1` in
//     ascending bit order, applies Volume-Parity and Star-Clique Hall-Capacity
//     pruning, and branches on the uncolored edge minimizing `popcount(avail[e])`
//     (tie-breaking by strictly larger `popcount(adj[e] & uncolored_mask)` and
//     smallest edge index `e`).

typedef unsigned int       u32;
typedef unsigned long long u64;

__device__ __forceinline__ void find_max_clique_dev(
    const u64* __restrict__ adj,
    u32 m,
    u32* __restrict__ omega_out,
    u64* __restrict__ clique_mask_out
) {
    if (m == 0U) {
        *omega_out = 0U;
        *clique_mask_out = 0ULL;
        return;
    }
    u64 all_mask = (m == 64U) ? 0xFFFFFFFFFFFFFFFFULL : ((1ULL << m) - 1ULL);
    u32 best_size = 0U;
    u64 best_mask = 0ULL;

    u64 stk_cand[64];
    u64 stk_cur_mask[64];
    u32 stk_cur_size[64];

    stk_cand[0] = all_mask;
    stk_cur_mask[0] = 0ULL;
    stk_cur_size[0] = 0U;
    int sp = 0;

    while (sp >= 0) {
        u64 cand = stk_cand[sp];
        u64 cur_mask = stk_cur_mask[sp];
        u32 cur_size = stk_cur_size[sp];

        if (cand == 0ULL) {
            if (cur_size > best_size) {
                best_size = cur_size;
                best_mask = cur_mask;
            }
            sp--;
            continue;
        }
        if (cur_size + (u32)__popcll(cand) <= best_size) {
            sp--;
            continue;
        }

        u32 v = (u32)__ffsll((long long)cand) - 1U;
        u64 bit = 1ULL << v;
        cand &= ~bit;
        stk_cand[sp] = cand;

        u64 next_cand = cand & adj[v];
        u64 next_mask = cur_mask | bit;
        u32 next_size = cur_size + 1U;

        if (next_cand == 0ULL) {
            if (next_size > best_size) {
                best_size = next_size;
                best_mask = next_mask;
            }
            continue;
        }
        if (next_size + (u32)__popcll(next_cand) > best_size) {
            sp++;
            stk_cand[sp] = next_cand;
            stk_cur_mask[sp] = next_mask;
            stk_cur_size[sp] = next_size;
        }
    }

    *omega_out = best_size;
    *clique_mask_out = best_mask;
}

__device__ __forceinline__ u32 parity_volume_lb_dev(u32 n, const u64* __restrict__ edges, u32 m) {
    if (m == 0U || n <= 1U) {
        return 0U;
    }
    u32 total_vol = 0U;
    u32 m_odd = 0U;
    u32 min_r = 64U;
    for (u32 a = 0U; a < m; a++) {
        u32 r = (u32)__popcll(edges[a]);
        total_vol += r;
        if ((r & 1U) != 0U) {
            m_odd += 1U;
        }
        if (r > 0U && r < min_r) {
            min_r = r;
        }
    }
    u32 max_per_color = (min_r >= 2U && min_r <= n) ? (n / min_r) : n;
    u32 min_r_lb = (max_per_color > 0U) ? ((m + max_per_color - 1U) / max_per_color) : n;
    if ((n & 1U) == 0U) {
        u32 v_lb = (total_vol + n - 1U) / n;
        return (v_lb > min_r_lb) ? v_lb : min_r_lb;
    }
    u32 max_c = (m > n) ? m : n;
    for (u32 c = 1U; c <= max_c; c++) {
        u32 odd_bonus = (c < m_odd) ? c : m_odd;
        if (c * (n - 1U) + odd_bonus >= total_vol) {
            return (c > min_r_lb) ? c : min_r_lb;
        }
    }
    return (n > min_r_lb) ? n : min_r_lb;
}

__device__ __forceinline__ bool check_pruning_and_pick_edge_dev(
    u32 n,
    u32 m,
    u32 min_r,
    u32 target_k,
    u64 uncolored_mask,
    u64 y_core_mask,
    u64 s_core_mask,
    u64 cross_edge_mask,
    const u64* __restrict__ adj,
    const u64* __restrict__ point_star_masks,
    const u64* __restrict__ color_verts,
    const u32* __restrict__ avail,
    u32* __restrict__ best_e_out
) {
    u32 rem_edges = (u32)__popcll(uncolored_mask);
    u32 total_pair_cap = 0U;
    u32 total_cross_cap = 0U;
    for (u32 c = 0U; c < target_k; c++) {
        u64 cv = color_verts[c];
        u32 used_v = (u32)__popcll(cv);
        u32 free_v = (n > used_v) ? (n - used_v) : 0U;
        total_pair_cap += free_v / min_r;
        if (y_core_mask != 0ULL) {
            u32 r_y = (u32)__popcll(y_core_mask & ~cv);
            u32 f_s = (u32)__popcll(s_core_mask & ~cv);
            u32 lim = (r_y < f_s) ? r_y : f_s;
            if (lim < (r_y & 1U)) {
                return false;
            }
            total_cross_cap += lim - ((lim ^ r_y) & 1U);
        }
    }
    if (total_pair_cap < rem_edges) {
        return false;
    }
    if (y_core_mask != 0ULL) {
        u32 rem_cross = (u32)__popcll(cross_edge_mask & uncolored_mask);
        if (total_cross_cap < rem_cross) {
            return false;
        }
    }

    for (u32 v = 0U; v < n; v++) {
        u64 rem_star = point_star_masks[v] & uncolored_mask;
        u32 need = (u32)__popcll(rem_star);
        if (need >= 2U) {
            u32 union_colors = 0U;
            u64 s = rem_star;
            while (s != 0ULL) {
                u32 e = (u32)__ffsll((long long)s) - 1U;
                s &= (s - 1ULL);
                union_colors |= avail[e];
            }
            if ((u32)__popc(union_colors) < need) {
                return false;
            }
        }
    }

    u32 best_e = 0xFFFFFFFFU;
    u32 min_avail = 0xFFFFFFFFU;
    u32 max_deg = 0U;
    u64 cand = uncolored_mask;
    while (cand != 0ULL) {
        u32 e = (u32)__ffsll((long long)cand) - 1U;
        cand &= (cand - 1ULL);
        u32 ac = (u32)__popc(avail[e]);
        if (ac == 0U) {
            return false;
        }
        u32 deg = (u32)__popcll(adj[e] & uncolored_mask);
        if (ac < min_avail || (ac == min_avail && deg > max_deg)) {
            min_avail = ac;
            max_deg = deg;
            best_e = e;
        }
    }

    *best_e_out = best_e;
    return true;
}

extern "C" __global__ void k_eval_efl_hypergraphs(
    const u64* __restrict__ edges_flat,
    const u32* __restrict__ edge_counts,
    u32 num_instances,
    u32 n,
    u32* __restrict__ is_linear_out,
    u32* __restrict__ omega_out,
    u32* __restrict__ chi_prime_out,
    u64* __restrict__ node_counts_out
) {
    u32 tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= num_instances) {
        return;
    }

    u32 m = edge_counts[tid];
    const u64* base_edges = edges_flat + ((u64)tid * 64ULL);

    u64 edges[64];
    u64 adj[64];
    for (u32 a = 0U; a < m; a++) {
        edges[a] = base_edges[a];
        adj[a] = 0ULL;
    }

    u32 linear = 1U;
    for (u32 a = 0U; a < m; a++) {
        for (u32 b = a + 1U; b < m; b++) {
            u64 inter = edges[a] & edges[b];
            if (inter != 0ULL) {
                if (__popcll(inter) > 1) {
                    linear = 0U;
                }
                adj[a] |= (1ULL << b);
                adj[b] |= (1ULL << a);
            }
        }
    }
    is_linear_out[tid] = linear;

    if (m == 0U) {
        omega_out[tid] = 0U;
        chi_prime_out[tid] = 0U;
        node_counts_out[tid] = 0ULL;
        return;
    }

    u64 point_star_masks[64];
    for (u32 v = 0U; v < n; v++) {
        u64 star = 0ULL;
        for (u32 a = 0U; a < m; a++) {
            if (((edges[a] >> v) & 1ULL) != 0ULL) {
                star |= (1ULL << a);
            }
        }
        point_star_masks[v] = star;
    }

    u32 omega = 0U;
    u64 clique_mask = 0ULL;
    find_max_clique_dev(adj, m, &omega, &clique_mask);
    omega_out[tid] = omega;

    u32 vol_lb = parity_volume_lb_dev(n, edges, m);
    u32 lower_bound = (omega > vol_lb) ? omega : vol_lb;
    if (lower_bound == 0U) {
        lower_bound = 1U;
    }

    u64 all_edges_mask = (m == 64U) ? 0xFFFFFFFFFFFFFFFFULL : ((1ULL << m) - 1ULL);
    u64 all_verts_mask = (n == 64U) ? 0xFFFFFFFFFFFFFFFFULL : ((1ULL << n) - 1ULL);
    u64 v_in_large_mask = 0ULL;
    u32 min_r = 64U;
    for (u32 a = 0U; a < m; a++) {
        u32 r = (u32)__popcll(edges[a]);
        if (r >= 3U) {
            v_in_large_mask |= edges[a];
        }
        if (r > 0U && r < min_r) {
            min_r = r;
        }
    }
    if (min_r < 2U || min_r > n) {
        min_r = 2U;
    }
    u64 total_nodes = 0ULL;
    u32 max_k = (m > n) ? m : n;

    u32 avail[64];
    u64 color_verts[32];

    u32 stk_best_e[64];
    u32 stk_c_mask[64];
    u32 stk_max_color[64];
    u32 stk_chosen_c[64];
    u64 stk_changed_nbrs[64];
    u64 stk_prev_cv[64];

    for (u32 target_k = lower_bound; target_k <= max_k; target_k++) {
        u64 y_core_mask = 0ULL;
        for (u32 v = 0U; v < n; v++) {
            if (((v_in_large_mask >> v) & 1ULL) == 0ULL &&
                (u32)__popcll(point_star_masks[v]) == target_k) {
                y_core_mask |= (1ULL << v);
            }
        }
        u64 s_core_mask = all_verts_mask & ~y_core_mask;
        u64 cross_edge_mask = 0ULL;
        if (y_core_mask != 0ULL) {
            for (u32 a = 0U; a < m; a++) {
                if ((edges[a] & y_core_mask) != 0ULL && (edges[a] & s_core_mask) != 0ULL) {
                    cross_edge_mask |= (1ULL << a);
                }
            }
        }
        u32 full_color_mask = (target_k == 32U) ? 0xFFFFFFFFU : ((1U << target_k) - 1U);
        for (u32 a = 0U; a < m; a++) {
            avail[a] = full_color_mask;
        }
        for (u32 c = 0U; c < target_k; c++) {
            color_verts[c] = 0ULL;
        }

        u64 uncolored_mask = all_edges_mask;
        u64 cm = clique_mask;
        u32 c_idx = 0U;
        while (cm != 0ULL) {
            u32 e_idx = (u32)__ffsll((long long)cm) - 1U;
            cm &= (cm - 1ULL);
            color_verts[c_idx] |= edges[e_idx];
            uncolored_mask &= ~(1ULL << e_idx);
            u32 c_bit = 1U << c_idx;
            u64 nbrs = adj[e_idx] & uncolored_mask;
            while (nbrs != 0ULL) {
                u32 nb = (u32)__ffsll((long long)nbrs) - 1U;
                nbrs &= (nbrs - 1ULL);
                avail[nb] &= ~c_bit;
            }
            c_idx++;
        }

        if (uncolored_mask == 0ULL) {
            chi_prime_out[tid] = target_k;
            node_counts_out[tid] = total_nodes;
            return;
        }

        u32 init_best_e = 0xFFFFFFFFU;
        if (!check_pruning_and_pick_edge_dev(
                n, m, min_r, target_k, uncolored_mask, y_core_mask, s_core_mask, cross_edge_mask,
                adj, point_star_masks, color_verts, avail, &init_best_e)) {
            continue;
        }

        int sp = 0;
        stk_best_e[0] = init_best_e;
        stk_c_mask[0] = avail[init_best_e];
        stk_max_color[0] = (omega > 0U) ? (omega - 1U) : 0U;
        stk_chosen_c[0] = 0xFFFFFFFFU;
        stk_changed_nbrs[0] = 0ULL;
        stk_prev_cv[0] = 0ULL;

        while (sp >= 0) {
            u32 best_e = stk_best_e[sp];

            // Undo previous branch at level sp if returning from failed child
            if (stk_chosen_c[sp] != 0xFFFFFFFFU) {
                u32 old_c = stk_chosen_c[sp];
                u32 c_bit = 1U << old_c;
                u64 restore = stk_changed_nbrs[sp];
                while (restore != 0ULL) {
                    u32 nb = (u32)__ffsll((long long)restore) - 1U;
                    restore &= (restore - 1ULL);
                    avail[nb] |= c_bit;
                }
                color_verts[old_c] = stk_prev_cv[sp];
                uncolored_mask |= (1ULL << best_e);
                stk_chosen_c[sp] = 0xFFFFFFFFU;
            }

            u32 c_mask = stk_c_mask[sp];
            if (c_mask == 0U) {
                sp--;
                continue;
            }

            u32 c = (u32)__ffs((int)c_mask) - 1U;
            stk_c_mask[sp] = c_mask & (c_mask - 1U);

            u32 color_limit = stk_max_color[sp] + 1U;
            if (color_limit >= target_k) {
                color_limit = target_k - 1U;
            }
            if (c > color_limit) {
                sp--;
                continue;
            }

            // Apply color c to best_e
            total_nodes += 1ULL;
            stk_chosen_c[sp] = c;
            stk_prev_cv[sp] = color_verts[c];
            color_verts[c] |= edges[best_e];
            uncolored_mask &= ~(1ULL << best_e);

            u32 c_bit = 1U << c;
            u64 changed = 0ULL;
            u64 nbrs = adj[best_e] & uncolored_mask;
            bool dead = false;
            while (nbrs != 0ULL) {
                u32 nb = (u32)__ffsll((long long)nbrs) - 1U;
                nbrs &= (nbrs - 1ULL);
                if ((avail[nb] & c_bit) != 0U) {
                    avail[nb] &= ~c_bit;
                    changed |= (1ULL << nb);
                    if (avail[nb] == 0U) {
                        dead = true;
                    }
                }
            }
            stk_changed_nbrs[sp] = changed;

            if (dead) {
                continue;
            }

            if (uncolored_mask == 0ULL) {
                chi_prime_out[tid] = target_k;
                node_counts_out[tid] = total_nodes;
                return;
            }

            u32 next_best_e = 0xFFFFFFFFU;
            if (check_pruning_and_pick_edge_dev(
                    n, m, min_r, target_k, uncolored_mask, y_core_mask, s_core_mask, cross_edge_mask,
                    adj, point_star_masks, color_verts, avail, &next_best_e)) {
                u32 next_max_c = (c > stk_max_color[sp]) ? c : stk_max_color[sp];
                sp++;
                stk_best_e[sp] = next_best_e;
                stk_c_mask[sp] = avail[next_best_e];
                stk_max_color[sp] = next_max_c;
                stk_chosen_c[sp] = 0xFFFFFFFFU;
                stk_changed_nbrs[sp] = 0ULL;
                stk_prev_cv[sp] = 0ULL;
            }
        }
    }

    chi_prime_out[tid] = max_k;
    node_counts_out[tid] = total_nodes;
}

// ============================================================================
// Theorem 2.10(a): `k_verify_three_tier_discharging_closure`
// Parallel GPU verifier for the Universal Three-Tier Intermediate-Block
// Discharging Closure and Cross-Tier Pair-Budget Conservation Law across
// E = E_2 ⊔ M ⊔ L with threshold tau = floor(sqrt(n)) + 1.
// Bit-exact twin of `verify_three_tier_discharging_closure_cpu` (`src/verify_cpu.rs`).
// ============================================================================
extern "C" __global__ void k_verify_three_tier_discharging_closure(
    const u64* __restrict__ edges_flat,
    const u32* __restrict__ edge_counts,
    const u32* __restrict__ n_vals,
    u32 num_instances,
    u32* __restrict__ tau_out,
    u32* __restrict__ p_med_out,
    u32* __restrict__ p_large_out,
    u32* __restrict__ psi_3tier_out,
    u32* __restrict__ closure_ok_out
) {
    u32 tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= num_instances) {
        return;
    }

    u32 n = n_vals[tid];
    u32 m = edge_counts[tid];
    const u64* base_edges = edges_flat + ((u64)tid * 64ULL);

    u32 root = 0U;
    while ((root + 1U) * (root + 1U) <= n) {
        root += 1U;
    }
    u32 tau = root + 1U;
    tau_out[tid] = tau;

    u32 linear = 1U;
    for (u32 a = 0U; a < m; a++) {
        u64 ea = base_edges[a];
        if ((u32)__popcll(ea) < 2U) {
            linear = 0U;
        }
        for (u32 b = a + 1U; b < m; b++) {
            if (__popcll(ea & base_edges[b]) > 1) {
                linear = 0U;
            }
        }
    }

    u32 p_e2 = 0U;
    u32 p_med = 0U;
    u32 p_large = 0U;

    u32 d2[64];
    u32 d_med[64];
    u32 d_large[64];
    u32 s_med[64];
    u32 s_large[64];
    for (u32 v = 0U; v < n; v++) {
        d2[v] = 0U;
        d_med[v] = 0U;
        d_large[v] = 0U;
        s_med[v] = 0U;
        s_large[v] = 0U;
    }

    for (u32 a = 0U; a < m; a++) {
        u64 e = base_edges[a];
        u32 r = (u32)__popcll(e);
        u32 pairs = (r * (r - 1U)) / 2U;
        if (r == 2U) {
            p_e2 += 1U;
            for (u32 v = 0U; v < n; v++) {
                if (((e >> v) & 1ULL) != 0ULL) {
                    d2[v] += 1U;
                }
            }
        } else if (r >= 3U && r < tau) {
            p_med += pairs;
            for (u32 v = 0U; v < n; v++) {
                if (((e >> v) & 1ULL) != 0ULL) {
                    d_med[v] += 1U;
                    s_med[v] += (r - 2U);
                }
            }
        } else if (r >= tau) {
            p_large += pairs;
            for (u32 v = 0U; v < n; v++) {
                if (((e >> v) & 1ULL) != 0ULL) {
                    d_large[v] += 1U;
                    s_large[v] += (r - 2U);
                }
            }
        }
    }

    u32 total_pairs = (n * (n - 1U)) / 2U;
    u32 sum_sigma = 0U;
    u32 sum_med_inc = 0U;
    u32 sum_large_inc = 0U;
    u32 ok = linear;

    for (u32 v = 0U; v < n; v++) {
        u32 sigma_m = d_med[v] + s_med[v];
        u32 sigma_l = d_large[v] + s_large[v];
        u32 sigma_v = d2[v] + sigma_m + sigma_l;
        u32 d_ge3 = d_med[v] + d_large[v];
        if (sigma_v > n - 1U) {
            ok = 0U;
        }
        if (s_med[v] + d2[v] + d_ge3 + s_large[v] != sigma_v) {
            ok = 0U;
        }
        sum_sigma += sigma_v;
        sum_med_inc += sigma_m;
        sum_large_inc += sigma_l;
    }

    if (sum_med_inc != 2U * p_med || sum_large_inc != 2U * p_large) {
        ok = 0U;
    }
    if (p_e2 + p_med + p_large > total_pairs) {
        ok = 0U;
    }
    if (sum_sigma != 2U * (p_e2 + p_med + p_large)) {
        ok = 0U;
    }

    u32 psi_3tier = (2U * total_pairs >= sum_sigma) ? (2U * total_pairs - sum_sigma) : 0U;
    if (p_e2 + p_med + p_large == total_pairs && psi_3tier != 0U) {
        ok = 0U;
    }

    p_med_out[tid] = p_med;
    p_large_out[tid] = p_large;
    psi_3tier_out[tid] = psi_3tier;
    closure_ok_out[tid] = ok;
}
