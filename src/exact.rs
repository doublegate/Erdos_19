//! ============================================================================
//! `exact.rs` — Hybrid CPU + CUDA exact PBD census and bit-exact CPU twin of
//! `k_eval_efl_hypergraphs` (`src/kernels.cu`) for Erdős Problem #19.
//! ============================================================================

use crate::gpu::Gpu;
use crate::records;
use anyhow::Result;
use cudarc::driver::PushKernelArg;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_GPU_EDGES: usize = 64;

/// Summary of an exhaustive or canonical Pairwise Balanced Design (PBD) census at order `n`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbdCensusSummary {
    pub n: usize,
    pub total_pbds_evaluated: usize,
    pub vizing_fournier_certified: usize,
    pub fournier_kempe_certified: usize,
    pub q_le_2_unconditional: usize,
    pub max_chi_prime: usize,
    pub min_chi_prime: usize,
    pub tight_count: usize,
    pub efl_violations: usize,
    pub total_search_nodes: u64,
    pub tight_profiles: Vec<PbdProfileRecord>,
}

/// Certificate from Theorems 2.3 & 2.4 (Vizing–Fournier `2 d_{>=3}(v)`-Shadow Deficit Reduction
/// & Fournier 2-Core Forest / Kempe-Chain Discharging).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VizingFournierShadowCert {
    pub n: usize,
    pub num_large_blocks: usize,
    pub chi_prime_large: u32,
    pub max_residual_deg: u32,
    pub num_ge_n_minus_q: usize,
    pub q_le_2_unconditional: bool,
    pub shadow_deficit_certified: bool,
    pub fournier_kempe_certified: bool,
}

/// Record for an individual PBD profile in the census.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbdProfileRecord {
    pub n: usize,
    pub num_edges: usize,
    pub rank_hist: Vec<(u32, usize)>,
    pub omega: u32,
    pub chi_prime: u32,
    pub search_nodes: u64,
    pub edges: Vec<u64>,
}

/// Bit-exact CPU twin of `find_max_clique_dev` in `src/kernels.cu`.
pub fn find_max_clique_twin_cpu(adj: &[u64], m: u32) -> (u32, u64) {
    if m == 0 {
        return (0, 0);
    }
    let all_mask = if m == 64 { u64::MAX } else { (1u64 << m) - 1 };
    let mut best_size = 0u32;
    let mut best_mask = 0u64;

    let mut stk_cand = [0u64; 64];
    let mut stk_cur_mask = [0u64; 64];
    let mut stk_cur_size = [0u32; 64];

    stk_cand[0] = all_mask;
    stk_cur_mask[0] = 0;
    stk_cur_size[0] = 0;
    let mut sp: isize = 0;

    while sp >= 0 {
        let idx = sp as usize;
        let mut cand = stk_cand[idx];
        let cur_mask = stk_cur_mask[idx];
        let cur_size = stk_cur_size[idx];

        if cand == 0 {
            if cur_size > best_size {
                best_size = cur_size;
                best_mask = cur_mask;
            }
            sp -= 1;
            continue;
        }
        if cur_size + cand.count_ones() <= best_size {
            sp -= 1;
            continue;
        }

        let v = cand.trailing_zeros() as usize;
        let bit = 1u64 << v;
        cand &= !bit;
        stk_cand[idx] = cand;

        let next_cand = cand & adj[v];
        let next_mask = cur_mask | bit;
        let next_size = cur_size + 1;

        if next_cand == 0 {
            if next_size > best_size {
                best_size = next_size;
                best_mask = next_mask;
            }
            continue;
        }
        if next_size + next_cand.count_ones() > best_size {
            sp += 1;
            let nidx = sp as usize;
            stk_cand[nidx] = next_cand;
            stk_cur_mask[nidx] = next_mask;
            stk_cur_size[nidx] = next_size;
        }
    }

    (best_size, best_mask)
}

/// Bit-exact CPU twin of `parity_volume_lb_dev` in `src/kernels.cu`.
pub fn parity_volume_lb_twin_cpu(n: u32, edges: &[u64], m: u32) -> u32 {
    if m == 0 || n <= 1 {
        return 0;
    }
    let mut total_vol = 0u32;
    let mut m_odd = 0u32;
    let mut min_r = 64u32;
    for &e in edges.iter().take(m as usize) {
        let r = e.count_ones();
        total_vol += r;
        if (r & 1) != 0 {
            m_odd += 1;
        }
        if r > 0 && r < min_r {
            min_r = r;
        }
    }
    let max_per_color = if min_r >= 2 && min_r <= n {
        n / min_r
    } else {
        n
    };
    let min_r_lb = if max_per_color > 0 {
        m.div_ceil(max_per_color)
    } else {
        n
    };
    if (n & 1) == 0 {
        return total_vol.div_ceil(n).max(min_r_lb);
    }
    let max_c = m.max(n);
    for c in 1..=max_c {
        let odd_bonus = c.min(m_odd);
        if c * (n - 1) + odd_bonus >= total_vol {
            return c.max(min_r_lb);
        }
    }
    n.max(min_r_lb)
}

#[allow(clippy::too_many_arguments)]
fn check_pruning_and_pick_edge_twin_cpu(
    n: u32,
    min_r: u32,
    target_k: u32,
    uncolored_mask: u64,
    y_core_mask: u64,
    s_core_mask: u64,
    cross_edge_mask: u64,
    adj: &[u64],
    point_star_masks: &[u64],
    color_verts: &[u64],
    avail: &[u32],
) -> Option<u32> {
    let rem_edges = uncolored_mask.count_ones();
    let mut total_pair_cap = 0u32;
    let mut total_cross_cap = 0u32;
    for &cv in color_verts.iter().take(target_k as usize) {
        let used_v = cv.count_ones();
        let free_v = n.saturating_sub(used_v);
        total_pair_cap += free_v / min_r;
        if y_core_mask != 0 {
            let r_y = (y_core_mask & !cv).count_ones();
            let f_s = (s_core_mask & !cv).count_ones();
            let lim = r_y.min(f_s);
            if lim < (r_y & 1) {
                return None;
            }
            total_cross_cap += lim - ((lim ^ r_y) & 1);
        }
    }
    if total_pair_cap < rem_edges {
        return None;
    }
    if y_core_mask != 0 {
        let rem_cross = (cross_edge_mask & uncolored_mask).count_ones();
        if total_cross_cap < rem_cross {
            return None;
        }
    }

    for &star in point_star_masks.iter().take(n as usize) {
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
                return None;
            }
        }
    }

    let mut best_e = u32::MAX;
    let mut min_avail = u32::MAX;
    let mut max_deg = 0u32;
    let mut cand = uncolored_mask;
    while cand != 0 {
        let e = cand.trailing_zeros() as usize;
        cand &= cand - 1;
        let ac = avail[e].count_ones();
        if ac == 0 {
            return None;
        }
        let deg = (adj[e] & uncolored_mask).count_ones();
        if ac < min_avail || (ac == min_avail && deg > max_deg) {
            min_avail = ac;
            max_deg = deg;
            best_e = e as u32;
        }
    }

    Some(best_e)
}

/// Bit-exact CPU twin of `k_eval_efl_hypergraphs` (`src/kernels.cu`).
///
/// Returns `(is_linear, omega, chi_prime, total_nodes)`.
pub fn eval_efl_hypergraph_twin_cpu(n: u32, edges: &[u64]) -> (u32, u32, u32, u64) {
    let m = edges.len() as u32;
    assert!(m <= 64);

    let mut adj = [0u64; 64];
    let mut linear = 1u32;
    for a in 0..(m as usize) {
        for b in (a + 1)..(m as usize) {
            let inter = edges[a] & edges[b];
            if inter != 0 {
                if inter.count_ones() > 1 {
                    linear = 0;
                }
                adj[a] |= 1u64 << b;
                adj[b] |= 1u64 << a;
            }
        }
    }

    if m == 0 {
        return (linear, 0, 0, 0);
    }

    let mut point_star_masks = [0u64; 64];
    for (v, star_slot) in point_star_masks.iter_mut().enumerate().take(n as usize) {
        let mut star = 0u64;
        for (a, &e) in edges.iter().enumerate().take(m as usize) {
            if ((e >> v) & 1) != 0 {
                star |= 1u64 << a;
            }
        }
        *star_slot = star;
    }

    let (omega, clique_mask) = find_max_clique_twin_cpu(&adj, m);
    let vol_lb = parity_volume_lb_twin_cpu(n, edges, m);
    let lower_bound = omega.max(vol_lb).max(1);

    let all_edges_mask = if m == 64 { u64::MAX } else { (1u64 << m) - 1 };
    let all_verts_mask = if n == 64 { u64::MAX } else { (1u64 << n) - 1 };
    let mut v_in_large_mask = 0u64;
    let mut min_r = 64u32;
    for &e in edges.iter().take(m as usize) {
        let r = e.count_ones();
        if r >= 3 {
            v_in_large_mask |= e;
        }
        if r > 0 && r < min_r {
            min_r = r;
        }
    }
    if min_r < 2 || min_r > n {
        min_r = 2;
    }
    let mut total_nodes = 0u64;
    let max_k = m.max(n);

    let mut avail = [0u32; 64];
    let mut color_verts = [0u64; 32];

    let mut stk_best_e = [0u32; 64];
    let mut stk_c_mask = [0u32; 64];
    let mut stk_max_color = [0u32; 64];
    let mut stk_chosen_c = [0u32; 64];
    let mut stk_changed_nbrs = [0u64; 64];
    let mut stk_prev_cv = [0u64; 64];

    for target_k in lower_bound..=max_k {
        let mut y_core_mask = 0u64;
        for (v, &star) in point_star_masks.iter().enumerate().take(n as usize) {
            if ((v_in_large_mask >> v) & 1) == 0 && star.count_ones() == target_k {
                y_core_mask |= 1u64 << v;
            }
        }
        let s_core_mask = all_verts_mask & !y_core_mask;
        let mut cross_edge_mask = 0u64;
        if y_core_mask != 0 {
            for (a, &e) in edges.iter().enumerate().take(m as usize) {
                if (e & y_core_mask) != 0 && (e & s_core_mask) != 0 {
                    cross_edge_mask |= 1u64 << a;
                }
            }
        }
        let full_color_mask = if target_k == 32 {
            u32::MAX
        } else {
            (1u32 << target_k) - 1
        };
        for slot in avail.iter_mut().take(m as usize) {
            *slot = full_color_mask;
        }
        for slot in color_verts.iter_mut().take(target_k as usize) {
            *slot = 0;
        }

        let mut uncolored_mask = all_edges_mask;
        let mut cm = clique_mask;
        let mut c_idx = 0u32;
        while cm != 0 {
            let e_idx = cm.trailing_zeros() as usize;
            cm &= cm - 1;
            color_verts[c_idx as usize] |= edges[e_idx];
            uncolored_mask &= !(1u64 << e_idx);
            let c_bit = 1u32 << c_idx;
            let mut nbrs = adj[e_idx] & uncolored_mask;
            while nbrs != 0 {
                let nb = nbrs.trailing_zeros() as usize;
                nbrs &= nbrs - 1;
                avail[nb] &= !c_bit;
            }
            c_idx += 1;
        }

        if uncolored_mask == 0 {
            return (linear, omega, target_k, total_nodes);
        }

        let Some(init_best_e) = check_pruning_and_pick_edge_twin_cpu(
            n,
            min_r,
            target_k,
            uncolored_mask,
            y_core_mask,
            s_core_mask,
            cross_edge_mask,
            &adj,
            &point_star_masks,
            &color_verts,
            &avail,
        ) else {
            continue;
        };

        let mut sp: isize = 0;
        stk_best_e[0] = init_best_e;
        stk_c_mask[0] = avail[init_best_e as usize];
        stk_max_color[0] = omega.saturating_sub(1);
        stk_chosen_c[0] = u32::MAX;
        stk_changed_nbrs[0] = 0;
        stk_prev_cv[0] = 0;

        while sp >= 0 {
            let idx = sp as usize;
            let best_e = stk_best_e[idx] as usize;

            if stk_chosen_c[idx] != u32::MAX {
                let old_c = stk_chosen_c[idx] as usize;
                let c_bit = 1u32 << old_c;
                let mut restore = stk_changed_nbrs[idx];
                while restore != 0 {
                    let nb = restore.trailing_zeros() as usize;
                    restore &= restore - 1;
                    avail[nb] |= c_bit;
                }
                color_verts[old_c] = stk_prev_cv[idx];
                uncolored_mask |= 1u64 << best_e;
                stk_chosen_c[idx] = u32::MAX;
            }

            let c_mask = stk_c_mask[idx];
            if c_mask == 0 {
                sp -= 1;
                continue;
            }

            let c = c_mask.trailing_zeros();
            stk_c_mask[idx] = c_mask & (c_mask - 1);

            let color_limit = (stk_max_color[idx] + 1).min(target_k - 1);
            if c > color_limit {
                sp -= 1;
                continue;
            }

            total_nodes += 1;
            stk_chosen_c[idx] = c;
            stk_prev_cv[idx] = color_verts[c as usize];
            color_verts[c as usize] |= edges[best_e];
            uncolored_mask &= !(1u64 << best_e);

            let c_bit = 1u32 << c;
            let mut changed = 0u64;
            let mut nbrs = adj[best_e] & uncolored_mask;
            let mut dead = false;
            while nbrs != 0 {
                let nb = nbrs.trailing_zeros() as usize;
                nbrs &= nbrs - 1;
                if (avail[nb] & c_bit) != 0 {
                    avail[nb] &= !c_bit;
                    changed |= 1u64 << nb;
                    if avail[nb] == 0 {
                        dead = true;
                    }
                }
            }
            stk_changed_nbrs[idx] = changed;

            if dead {
                continue;
            }

            if uncolored_mask == 0 {
                return (linear, omega, target_k, total_nodes);
            }

            if let Some(next_best_e) = check_pruning_and_pick_edge_twin_cpu(
                n,
                min_r,
                target_k,
                uncolored_mask,
                y_core_mask,
                s_core_mask,
                cross_edge_mask,
                &adj,
                &point_star_masks,
                &color_verts,
                &avail,
            ) {
                let next_max_c = stk_max_color[idx].max(c);
                sp += 1;
                let nidx = sp as usize;
                stk_best_e[nidx] = next_best_e;
                stk_c_mask[nidx] = avail[next_best_e as usize];
                stk_max_color[nidx] = next_max_c;
                stk_chosen_c[nidx] = u32::MAX;
                stk_changed_nbrs[nidx] = 0;
                stk_prev_cv[nidx] = 0;
            }
        }
    }

    (linear, omega, max_k, total_nodes)
}

/// Evaluate a batch of hypergraphs on the CUDA GPU via `k_eval_efl_hypergraphs`.
pub fn eval_efl_batch_gpu(
    gpu: &Gpu,
    n: usize,
    batch: &[Vec<u64>],
) -> Result<Vec<(u32, u32, u32, u64)>> {
    if batch.is_empty() {
        return Ok(Vec::new());
    }
    let num_instances = batch.len();
    let mut edges_flat = vec![0u64; num_instances * MAX_GPU_EDGES];
    let mut edge_counts = vec![0u32; num_instances];

    for (i, edges) in batch.iter().enumerate() {
        assert!(edges.len() <= MAX_GPU_EDGES);
        edge_counts[i] = edges.len() as u32;
        for (a, &e) in edges.iter().enumerate() {
            edges_flat[i * MAX_GPU_EDGES + a] = e;
        }
    }

    let d_edges = gpu.stream.clone_htod(&edges_flat)?;
    let d_counts = gpu.stream.clone_htod(&edge_counts)?;
    let mut d_linear = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_omega = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_chi = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_nodes = gpu.stream.alloc_zeros::<u64>(num_instances)?;

    let num_u32 = num_instances as u32;
    let n_u32 = n as u32;
    let cfg = gpu.launch_cfg(num_u32);

    unsafe {
        gpu.stream
            .launch_builder(&gpu.fn_eval_efl_hypergraphs)
            .arg(&d_edges)
            .arg(&d_counts)
            .arg(&num_u32)
            .arg(&n_u32)
            .arg(&mut d_linear)
            .arg(&mut d_omega)
            .arg(&mut d_chi)
            .arg(&mut d_nodes)
            .launch(cfg)?;
    }

    let h_linear = gpu.stream.clone_dtoh(&d_linear)?;
    let h_omega = gpu.stream.clone_dtoh(&d_omega)?;
    let h_chi = gpu.stream.clone_dtoh(&d_chi)?;
    let h_nodes = gpu.stream.clone_dtoh(&d_nodes)?;

    let mut out = Vec::with_capacity(num_instances);
    for i in 0..num_instances {
        out.push((h_linear[i], h_omega[i], h_chi[i], h_nodes[i]));
    }
    Ok(out)
}

/// GPU output tuple `(tau, p_med, p_large, psi_3tier, closure_ok)` for Theorem 2.10(a).
pub type ThreeTierClosureRecord = (u32, u32, u32, u32, u32);

/// Theorem 2.10(a): Evaluate a batch of `(n, edges)` hypergraph instances on the CUDA GPU via
/// `k_verify_three_tier_discharging_closure`, returning `(tau, p_med, p_large, psi_3tier, closure_ok)`.
pub fn verify_three_tier_batch_gpu(
    gpu: &Gpu,
    batch: &[(usize, Vec<u64>)],
) -> Result<Vec<ThreeTierClosureRecord>> {
    if batch.is_empty() {
        return Ok(Vec::new());
    }
    let num_instances = batch.len();
    let mut edges_flat = vec![0u64; num_instances * MAX_GPU_EDGES];
    let mut edge_counts = vec![0u32; num_instances];
    let mut n_vals = vec![0u32; num_instances];

    for (i, (n, edges)) in batch.iter().enumerate() {
        assert!(edges.len() <= MAX_GPU_EDGES && *n <= 64);
        edge_counts[i] = edges.len() as u32;
        n_vals[i] = *n as u32;
        for (a, &e) in edges.iter().enumerate() {
            edges_flat[i * MAX_GPU_EDGES + a] = e;
        }
    }

    let d_edges = gpu.stream.clone_htod(&edges_flat)?;
    let d_counts = gpu.stream.clone_htod(&edge_counts)?;
    let d_n_vals = gpu.stream.clone_htod(&n_vals)?;
    let mut d_tau = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_p_med = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_p_large = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_psi = gpu.stream.alloc_zeros::<u32>(num_instances)?;
    let mut d_ok = gpu.stream.alloc_zeros::<u32>(num_instances)?;

    let num_u32 = num_instances as u32;
    let cfg = gpu.launch_cfg(num_u32);

    unsafe {
        gpu.stream
            .launch_builder(&gpu.fn_verify_three_tier_discharging_closure)
            .arg(&d_edges)
            .arg(&d_counts)
            .arg(&d_n_vals)
            .arg(&num_u32)
            .arg(&mut d_tau)
            .arg(&mut d_p_med)
            .arg(&mut d_p_large)
            .arg(&mut d_psi)
            .arg(&mut d_ok)
            .launch(cfg)?;
    }

    let h_tau = gpu.stream.clone_dtoh(&d_tau)?;
    let h_p_med = gpu.stream.clone_dtoh(&d_p_med)?;
    let h_p_large = gpu.stream.clone_dtoh(&d_p_large)?;
    let h_psi = gpu.stream.clone_dtoh(&d_psi)?;
    let h_ok = gpu.stream.clone_dtoh(&d_ok)?;

    let mut out = Vec::with_capacity(num_instances);
    for i in 0..num_instances {
        out.push((h_tau[i], h_p_med[i], h_p_large[i], h_psi[i], h_ok[i]));
    }
    Ok(out)
}

/// Find an explicit proper `q`-coloring of `large_blocks` (returning the vertex bitmask
/// `V(M_c)` covered by each color class `0 <= c < q`).
fn color_large_blocks_explicit(n: usize, large_blocks: &[u64], q: usize) -> Vec<u64> {
    let m = large_blocks.len();
    let mut color_verts = vec![0u64; q];
    if m == 0 || q == 0 {
        return color_verts;
    }
    let mut adj = [0u64; 64];
    for a in 0..m {
        for b in (a + 1)..m {
            if (large_blocks[a] & large_blocks[b]) != 0 {
                adj[a] |= 1u64 << b;
                adj[b] |= 1u64 << a;
            }
        }
    }
    let (omega, clique_mask) = find_max_clique_twin_cpu(&adj, m as u32);
    let mut uncolored = if m == 64 { u64::MAX } else { (1u64 << m) - 1 };
    let mut cm = clique_mask;
    let mut c_idx = 0usize;
    while cm != 0 {
        let e_idx = cm.trailing_zeros() as usize;
        cm &= cm - 1;
        color_verts[c_idx] |= large_blocks[e_idx];
        uncolored &= !(1u64 << e_idx);
        c_idx += 1;
    }

    fn dfs_mrv(
        uncolored: u64,
        blocks: &[u64],
        q: usize,
        max_used: usize,
        color_verts: &mut [u64],
    ) -> bool {
        if uncolored == 0 {
            return true;
        }
        let limit = (max_used + 1).min(q - 1);
        let mut best_e = 0usize;
        let mut min_avail = usize::MAX;
        let mut best_r = 0u32;
        let mut cand = uncolored;
        while cand != 0 {
            let e = cand.trailing_zeros() as usize;
            cand &= cand - 1;
            let b = blocks[e];
            let mut ac = 0usize;
            for cv in color_verts.iter().take(limit + 1) {
                if (cv & b) == 0 {
                    ac += 1;
                }
            }
            if ac == 0 {
                return false;
            }
            let r = b.count_ones();
            if ac < min_avail || (ac == min_avail && r > best_r) {
                min_avail = ac;
                best_r = r;
                best_e = e;
            }
        }
        let b = blocks[best_e];
        let next_uncolored = uncolored & !(1u64 << best_e);
        for c in 0..=limit {
            if (color_verts[c] & b) == 0 {
                color_verts[c] |= b;
                if dfs_mrv(next_uncolored, blocks, q, max_used.max(c), color_verts) {
                    return true;
                }
                color_verts[c] &= !b;
            }
        }
        false
    }

    let _ = n;
    let init_max = (omega as usize).saturating_sub(1);
    let ok = dfs_mrv(uncolored, large_blocks, q, init_max, &mut color_verts);
    assert!(ok, "large_blocks must be properly {q}-colorable");
    color_verts
}

/// Theorem 2.4: Vizing Adjacency Lemma & Fournier 2-Core Forest Peeling check
/// certifying `chi'(G_res) <= target_deg` in polynomial time.
pub fn check_fournier_vizing_class1(n: usize, e2_adj: &[u64], target_deg: u32) -> bool {
    let mut deg = [0u32; 64];
    let mut max_d = 0u32;
    for v in 0..n {
        deg[v] = e2_adj[v].count_ones();
        if deg[v] > max_d {
            max_d = deg[v];
        }
    }
    if max_d == 0 {
        return true;
    }
    if max_d > target_deg {
        return false;
    }
    if max_d < target_deg {
        // Vizing's Theorem: chi'(G_res) <= max_d + 1 <= target_deg
        return true;
    }

    // max_d == target_deg: apply iterative Vizing Adjacency Lemma / Fournier 2-core peeling
    // on the set D of vertices of degree target_deg.
    let mut d_mask = 0u64;
    for (v, &d_v) in deg.iter().enumerate().take(n) {
        if d_v == target_deg {
            d_mask |= 1u64 << v;
        }
    }
    loop {
        let mut removed = 0u64;
        let mut m = d_mask;
        while m != 0 {
            let u = m.trailing_zeros() as usize;
            m &= m - 1;
            let nbrs_in_d = e2_adj[u] & d_mask;
            if nbrs_in_d.count_ones() < 2 {
                removed |= 1u64 << u;
                continue;
            }
            // Vizing's Adjacency Lemma: for every neighbor w of u in G_res,
            // u must have at least (target_deg - deg[w] + 1) neighbors in D \ {w}.
            let mut adj_u = e2_adj[u];
            while adj_u != 0 {
                let w = adj_u.trailing_zeros() as usize;
                adj_u &= adj_u - 1;
                let req = target_deg - deg[w] + 1;
                let d_without_w = (nbrs_in_d & !(1u64 << w)).count_ones();
                if d_without_w < req {
                    removed |= 1u64 << u;
                    break;
                }
            }
        }
        if removed == 0 {
            break;
        }
        d_mask &= !removed;
    }
    d_mask == 0
}

/// Find the exact maximum-weight matching in `e2_adj` restricted to `free_mask` (`n <= 13`),
/// maximizing `(total_deficit_gain, total_matched_edges)`.
fn best_augmenting_matching_mask(
    e2_adj: &[u64],
    free_mask: u64,
    sigma_plus_match: &[i32],
    q_i32: i32,
) -> Vec<(usize, usize)> {
    #[allow(clippy::too_many_arguments)]
    fn dfs_match(
        e2_adj: &[u64],
        rem_mask: u64,
        sigma_plus_match: &[i32],
        q_i32: i32,
        cur_edges: &mut Vec<(usize, usize)>,
        cur_score: (i32, i32, usize),
        best_score: &mut (i32, i32, usize),
        best_edges: &mut Vec<(usize, usize)>,
    ) {
        if cur_score > *best_score {
            *best_score = cur_score;
            *best_edges = cur_edges.clone();
        }
        if rem_mask.count_ones() < 2 {
            return;
        }
        let u = rem_mask.trailing_zeros() as usize;
        let rem_without_u = rem_mask & !(1u64 << u);
        // Option 1: leave u unmatched
        dfs_match(
            e2_adj,
            rem_without_u,
            sigma_plus_match,
            q_i32,
            cur_edges,
            cur_score,
            best_score,
            best_edges,
        );
        // Option 2: match u with an available neighbor v > u
        let mut avail_v = e2_adj[u] & rem_without_u;
        while avail_v != 0 {
            let v = avail_v.trailing_zeros() as usize;
            avail_v &= avail_v - 1;
            let def_u = (q_i32 - sigma_plus_match[u]).max(0);
            let def_v = (q_i32 - sigma_plus_match[v]).max(0);
            let gain = def_u + def_v;
            let deg_sum = sigma_plus_match[u] + sigma_plus_match[v];
            cur_edges.push((u, v));
            dfs_match(
                e2_adj,
                rem_without_u & !(1u64 << v),
                sigma_plus_match,
                q_i32,
                cur_edges,
                (cur_score.0 + gain, cur_score.1 - deg_sum, cur_score.2 + 1),
                best_score,
                best_edges,
            );
            cur_edges.pop();
        }
    }

    let mut cur_edges = Vec::new();
    let mut best_score = (0i32, 0i32, 0usize);
    let mut best_edges = Vec::new();
    dfs_match(
        e2_adj,
        free_mask,
        sigma_plus_match,
        q_i32,
        &mut cur_edges,
        (0, 0, 0),
        &mut best_score,
        &mut best_edges,
    );
    best_edges
}

/// Enumerate up to `max_colorings` proper `q_try`-colorings of `large_blocks`.
fn enumerate_large_block_colorings(
    large_blocks: &[u64],
    q_try: usize,
    max_colorings: usize,
) -> Vec<Vec<u64>> {
    let m = large_blocks.len();
    if m == 0 || q_try == 0 {
        return vec![vec![0u64; q_try]];
    }
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(large_blocks[i].count_ones()));

    #[allow(clippy::too_many_arguments)]
    fn dfs(
        idx: usize,
        order: &[usize],
        blocks: &[u64],
        q_try: usize,
        max_used: usize,
        color_verts: &mut Vec<u64>,
        out: &mut Vec<Vec<u64>>,
        max_colorings: usize,
    ) {
        if out.len() >= max_colorings {
            return;
        }
        if idx == order.len() {
            out.push(color_verts.clone());
            return;
        }
        let b = blocks[order[idx]];
        let limit = (max_used + 1).min(q_try - 1);
        for c in (0..=limit).rev() {
            if (color_verts[c] & b) == 0 {
                color_verts[c] |= b;
                dfs(
                    idx + 1,
                    order,
                    blocks,
                    q_try,
                    max_used.max(c),
                    color_verts,
                    out,
                    max_colorings,
                );
                color_verts[c] &= !b;
                if out.len() >= max_colorings {
                    return;
                }
            }
        }
    }

    let mut color_verts = vec![0u64; q_try];
    color_verts[0] = large_blocks[order[0]];
    let mut out = Vec::new();
    dfs(
        1,
        &order,
        large_blocks,
        q_try,
        0,
        &mut color_verts,
        &mut out,
        max_colorings,
    );
    out
}

/// Theorems 2.3 & 2.4: Certify a PBD `H = (V, E_{>=3} ∪ E_2)` on `n` vertices via the
/// Vizing–Fournier `2 d_{>=3}(v)`-Shadow Deficit Reduction (Theorem 2.3) and
/// Fournier 2-Core Forest / Kempe-Chain Discharging (Theorem 2.4).
pub fn certify_vizing_fournier_shadow(n: usize, edges: &[u64]) -> VizingFournierShadowCert {
    let large_blocks: Vec<u64> = edges
        .iter()
        .copied()
        .filter(|&e| e.count_ones() >= 3)
        .collect();
    if large_blocks.is_empty() {
        return VizingFournierShadowCert {
            n,
            num_large_blocks: 0,
            chi_prime_large: 0,
            max_residual_deg: (n.saturating_sub(1)) as u32,
            num_ge_n_minus_q: 0,
            q_le_2_unconditional: true,
            shadow_deficit_certified: true,
            fournier_kempe_certified: true,
        };
    }

    let (_, _, q_u32, _) = eval_efl_hypergraph_twin_cpu(n as u32, &large_blocks);
    let q = q_u32 as usize;
    if large_blocks.len() == edges.len() {
        let q_le_2 = q <= 2;
        let ok = q <= n;
        return VizingFournierShadowCert {
            n,
            num_large_blocks: large_blocks.len(),
            chi_prime_large: q_u32,
            max_residual_deg: 0,
            num_ge_n_minus_q: 0,
            q_le_2_unconditional: q_le_2,
            shadow_deficit_certified: ok,
            fournier_kempe_certified: ok,
        };
    }
    let color_verts = color_large_blocks_explicit(n, &large_blocks, q);

    let mut sigma_ge3 = vec![0i32; n];
    let mut base_e2_adj = vec![0u64; n];
    for &e in edges {
        let r = e.count_ones();
        if r >= 3 {
            for (v, slot) in sigma_ge3.iter_mut().enumerate().take(n) {
                if ((e >> v) & 1) != 0 {
                    *slot += (r - 1) as i32;
                }
            }
        } else if r == 2 {
            let u = e.trailing_zeros() as usize;
            let v = (e & (e - 1)).trailing_zeros() as usize;
            if u < n && v < n {
                base_e2_adj[u] |= 1u64 << v;
                base_e2_adj[v] |= 1u64 << u;
            }
        }
    }

    let mut e2_adj = base_e2_adj.clone();
    let mut match_count = vec![0i32; n];
    let q_i32 = q as i32;

    // Theorem 2.3 greedy augmenting matchings F_c subseteq E_2 on V \ V(M_c)
    for &cv in &color_verts {
        let mut free_mask = ((1u64 << n) - 1) & !cv;
        loop {
            let mut best_pair: Option<(usize, usize, i32, i32)> = None;
            for u in 0..n {
                if ((free_mask >> u) & 1) == 0 {
                    continue;
                }
                let avail_v = e2_adj[u] & free_mask;
                for v in (u + 1)..n {
                    if ((avail_v >> v) & 1) == 0 {
                        continue;
                    }
                    let def_u = (q_i32 - sigma_ge3[u] - match_count[u]).max(0);
                    let def_v = (q_i32 - sigma_ge3[v] - match_count[v]).max(0);
                    let gain = def_u + def_v;
                    if gain <= 0 {
                        continue;
                    }
                    let max_end = def_u.max(def_v);
                    let cand = (u, v, gain, max_end);
                    if best_pair
                        .map(|(_, _, bg, bm)| (gain, max_end) > (bg, bm))
                        .unwrap_or(true)
                    {
                        best_pair = Some(cand);
                    }
                }
            }
            if let Some((u, v, _, _)) = best_pair {
                free_mask &= !((1u64 << u) | (1u64 << v));
                e2_adj[u] &= !(1u64 << v);
                e2_adj[v] &= !(1u64 << u);
                match_count[u] += 1;
                match_count[v] += 1;
            } else {
                break;
            }
        }
    }

    let mut max_res_deg = 0u32;
    let mut num_ge = 0usize;
    let target_deg = n.saturating_sub(q) as u32;
    for v in 0..n {
        let rem = ((n as i32) - 1 - sigma_ge3[v] - match_count[v]).max(0) as u32;
        if rem > max_res_deg {
            max_res_deg = rem;
        }
        if rem >= target_deg {
            num_ge += 1;
        }
    }

    let q_le_2 = q <= 2;
    let shadow_ok = q_le_2
        || (q <= n
            && (max_res_deg == 0
                || (max_res_deg <= target_deg && (max_res_deg < target_deg || num_ge <= 1))));

    // Theorem 2.4: Fournier 2-Core Forest / Vizing Adjacency Lemma + Kempe-Chain Discharging
    let mut fournier_kempe_ok =
        shadow_ok || (q <= n && check_fournier_vizing_class1(n, &e2_adj, target_deg));

    if !fournier_kempe_ok && q <= n {
        // Search across proper q'-colorings (q' in q..=n) with exact maximum-weight
        // augmenting matchings F_1..F_{q'} on V \ V(M_c) + Fournier 2-core peeling.
        'search_q: for q_try in q..=n {
            let t_deg = (n - q_try) as u32;
            let colorings = enumerate_large_block_colorings(&large_blocks, q_try, 64);
            for col in colorings {
                let mut cur_e2 = base_e2_adj.clone();
                let mut spm = sigma_ge3.clone();
                for &cv in &col {
                    let free_mask = ((1u64 << n) - 1) & !cv;
                    let m_edges =
                        best_augmenting_matching_mask(&cur_e2, free_mask, &spm, q_try as i32);
                    for (u, v) in m_edges {
                        cur_e2[u] &= !(1u64 << v);
                        cur_e2[v] &= !(1u64 << u);
                        spm[u] += 1;
                        spm[v] += 1;
                    }
                }
                if check_fournier_vizing_class1(n, &cur_e2, t_deg) {
                    fournier_kempe_ok = true;
                    break 'search_q;
                }
            }
        }
    }

    VizingFournierShadowCert {
        n,
        num_large_blocks: large_blocks.len(),
        chi_prime_large: q_u32,
        max_residual_deg: max_res_deg,
        num_ge_n_minus_q: num_ge,
        q_le_2_unconditional: q_le_2,
        shadow_deficit_certified: shadow_ok,
        fournier_kempe_certified: fournier_kempe_ok,
    }
}

/// Compute a permutation-invariant structural signature of a PBD on `n` vertices to
/// deduplicate isomorphic block extensions during census enumeration.
fn pbd_structural_signature(n: usize, edges: &[u64]) -> Vec<u64> {
    let m = edges.len();
    let mut ranks: Vec<u32> = edges.iter().map(|e| e.count_ones()).collect();
    ranks.sort_unstable();

    let mut pt_profiles: Vec<(u32, u32)> = (0..n)
        .map(|v| {
            let mut deg = 0u32;
            let mut rank_sum = 0u32;
            for &e in edges {
                if ((e >> v) & 1) != 0 {
                    deg += 1;
                    rank_sum += e.count_ones();
                }
            }
            (deg, rank_sum)
        })
        .collect();
    pt_profiles.sort_unstable();

    let mut lg_degs = vec![0u32; m];
    for a in 0..m {
        for b in (a + 1)..m {
            if (edges[a] & edges[b]) != 0 {
                lg_degs[a] += 1;
                lg_degs[b] += 1;
            }
        }
    }
    let mut edge_profiles: Vec<(u32, u32)> = edges
        .iter()
        .enumerate()
        .map(|(a, e)| (e.count_ones(), lg_degs[a]))
        .collect();
    edge_profiles.sort_unstable();

    let mut sig = Vec::with_capacity(1 + ranks.len() + pt_profiles.len() + edge_profiles.len());
    sig.push(m as u64);
    for r in ranks {
        sig.push(r as u64);
    }
    for (d, rs) in pt_profiles {
        sig.push(((d as u64) << 32) | (rs as u64));
    }
    for (r, ld) in edge_profiles {
        sig.push(((r as u64) << 32) | (ld as u64));
    }
    sig
}

/// Enumerate canonical Pairwise Balanced Designs `PBD(n, K, 1)` on `n` vertices
/// by generating linear block systems `B` of blocks of size `>= 3` (with root
/// symmetry breaking on the first block `B_0 = {0, ..., r_0 - 1}`) and completing
/// all uncovered vertex pairs with 2-edges per Theorem 2.1.
pub fn enumerate_canonical_pbds(n: usize, max_profiles: usize) -> Vec<Vec<u64>> {
    assert!((3..=13).contains(&n));
    let mut candidate_blocks: Vec<u64> = Vec::new();
    let total_masks = 1u64 << n;
    for mask in 0..total_masks {
        let r = mask.count_ones() as usize;
        if r >= 3 && r <= n {
            candidate_blocks.push(mask);
        }
    }
    // Sort candidate blocks by descending size then ascending bitmask
    candidate_blocks.sort_by_key(|&b| (std::cmp::Reverse(b.count_ones()), b));

    let mut seen_sigs: HashSet<Vec<u64>> = HashSet::new();
    let mut pbds: Vec<Vec<u64>> = Vec::new();

    // 1. Empty block system B = ∅ -> Complete graph K_n
    let kn = records::pbd_complete(n, &[]);
    if kn.len() <= MAX_GPU_EDGES {
        seen_sigs.insert(pbd_structural_signature(n, &kn));
        pbds.push(kn);
    }

    // 2. For each maximum block size r_0 in (3..=n).rev(), fix B_0 = (1 << r_0) - 1
    //    by vertex-relabelling symmetry.
    #[allow(clippy::too_many_arguments)]
    fn dfs_blocks(
        n: usize,
        cand_blocks: &[u64],
        idx: usize,
        current_blocks: &mut Vec<u64>,
        seen_sigs: &mut HashSet<Vec<u64>>,
        pbds: &mut Vec<Vec<u64>>,
        max_profiles: usize,
    ) {
        if pbds.len() >= max_profiles {
            return;
        }
        let completed = records::pbd_complete(n, current_blocks);
        if completed.len() <= MAX_GPU_EDGES {
            let sig = pbd_structural_signature(n, &completed);
            if seen_sigs.insert(sig) {
                pbds.push(completed);
                if pbds.len() >= max_profiles {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        }

        for i in idx..cand_blocks.len() {
            let b = cand_blocks[i];
            if current_blocks
                .iter()
                .all(|&prev| (prev & b).count_ones() <= 1)
            {
                current_blocks.push(b);
                dfs_blocks(
                    n,
                    cand_blocks,
                    i + 1,
                    current_blocks,
                    seen_sigs,
                    pbds,
                    max_profiles,
                );
                current_blocks.pop();
                if pbds.len() >= max_profiles {
                    return;
                }
            }
        }
    }

    for r0 in (3..=n).rev() {
        let b0 = (1u64 << r0) - 1;
        // Only consider subsequent blocks of size <= r0 that intersect b0 in <= 1 vertex
        let filtered: Vec<u64> = candidate_blocks
            .iter()
            .copied()
            .filter(|&b| b != b0 && (b.count_ones() as usize) <= r0 && (b & b0).count_ones() <= 1)
            .collect();
        let mut current = vec![b0];
        dfs_blocks(
            n,
            &filtered,
            0,
            &mut current,
            &mut seen_sigs,
            &mut pbds,
            max_profiles,
        );
        if pbds.len() >= max_profiles {
            break;
        }
    }

    pbds
}

/// Classification record for each of the 7 critical PBDs on `n <= 10` where the
/// single-vertex Theorem 2.3 greedy check (`num_ge_n_minus_q <= 1`) fails and
/// Theorem 2.4 (Fournier 2-Core Forest / Kempe-Chain Discharging) resolves `chi'(H) <= n`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionalPbdProfile {
    pub n: usize,
    pub num_large_blocks: usize,
    pub max_large_deg: u32,
    pub chi_prime_large: u32,
    pub max_residual_deg: u32,
    pub num_ge_n_minus_q: usize,
    pub large_blocks: Vec<Vec<usize>>,
    pub fournier_kempe_certified: bool,
}

/// Theorem 2.5: Verify the exact Vizing–Fournier Shadow-Slack Equality Frontier Identity
/// for a Pairwise Balanced Design `H = ([n], E_{>=3} ∪ E_2)`:
///
///     sigma_{>=3}(v) = 2 * d_{>=3}(v) + sum_{E in E_{>=3}, v in E} (|E| - 3),
///     delta_VF(v) := (n - q') - d_{G_res}(v)
///                  = 2 * d_{>=3}(v) + match(v) + sum_{E in E_{>=3}, v in E} (|E| - 3) + 1 - q'.
///
/// Also verifies that every vertex with `d_{>=3}(v) >= ceil(q' / 2)` satisfies `delta_VF(v) >= 1`
/// even when `match(v) = 0`. Returns `(max_d_ge3, identity_verified)`.
pub fn verify_shadow_slack_equality_frontier(n: usize, edges: &[u64]) -> (u32, bool) {
    let mut d_ge3 = vec![0i32; n];
    let mut rank_excess = vec![0i32; n];
    let mut sigma_ge3 = vec![0i32; n];
    let mut e2_deg = vec![0i32; n];

    for &e in edges {
        let r = e.count_ones() as i32;
        if r >= 3 {
            for v in 0..n {
                if ((e >> v) & 1) != 0 {
                    d_ge3[v] += 1;
                    rank_excess[v] += r - 3;
                    sigma_ge3[v] += r - 1;
                }
            }
        } else if r == 2 {
            for (v, slot) in e2_deg.iter_mut().enumerate().take(n) {
                if ((e >> v) & 1) != 0 {
                    *slot += 1;
                }
            }
        }
    }

    let mut max_d_ge3 = 0u32;
    let mut ok = true;
    for v in 0..n {
        max_d_ge3 = max_d_ge3.max(d_ge3[v] as u32);
        // Identity 1: PBD pair-partition degree identity (n - 1) - sigma_ge3[v] == e2_deg[v]
        if (n as i32 - 1) - sigma_ge3[v] != e2_deg[v] {
            ok = false;
        }
        // Identity 2: Shadow-slack decomposition sigma_ge3[v] == 2 * d_ge3[v] + rank_excess[v]
        if sigma_ge3[v] != 2 * d_ge3[v] + rank_excess[v] {
            ok = false;
        }
        // Identity 3: For every q' in 1..=n and match(v) = 0,
        // delta_VF(v) = (n - q') - e2_deg[v] == 2 * d_ge3[v] + rank_excess[v] + 1 - q',
        // and d_ge3[v] >= ceil(q'/2) implies delta_VF(v) >= 1.
        for q_prime in 1..=(n as i32) {
            let delta_lhs = (n as i32 - q_prime) - e2_deg[v];
            let delta_rhs = 2 * d_ge3[v] + rank_excess[v] + 1 - q_prime;
            if delta_lhs != delta_rhs {
                ok = false;
            }
            if 2 * d_ge3[v] >= q_prime && delta_lhs < 1 {
                ok = false;
            }
        }
    }
    (max_d_ge3, ok)
}

/// Theorem 2.5 certification record for a high-density `n = 11` or `n = 12` PBD instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighDensityPbdCertRecord {
    pub name: String,
    pub family: String,
    pub n: usize,
    pub num_edges: usize,
    pub num_large_blocks: usize,
    pub max_large_deg: u32,
    pub chi_prime_large: u32,
    pub omega: u32,
    pub chi_prime: u32,
    pub shadow_identity_verified: bool,
    pub fournier_kempe_certified: bool,
}

/// Evaluate and certify all 9 high-density `n = 11, 12` PBD instances from
/// `records::n11_n12_high_density_pbd_suite()` under Theorem 2.5.
pub fn certify_n11_n12_high_density_suite(
    gpu_opt: Option<&Gpu>,
) -> Result<Vec<HighDensityPbdCertRecord>> {
    let suite = records::n11_n12_high_density_pbd_suite();
    let mut out = Vec::with_capacity(suite.len());
    for inst in suite {
        let (is_lin, omega, chi, _nodes) = if let Some(gpu) = gpu_opt {
            eval_efl_batch_gpu(gpu, inst.n, std::slice::from_ref(&inst.edges))?[0]
        } else {
            eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges)
        };
        assert_eq!(is_lin, 1, "PBD instance {} must be linear", inst.name);
        let (max_d_ge3, id_ok) = verify_shadow_slack_equality_frontier(inst.n, &inst.edges);
        let (num_large_blocks, chi_prime_large, fk_ok) =
            if inst.edges.iter().all(|&e| e.count_ones() >= 3) {
                (inst.edges.len(), chi, (chi as usize) <= inst.n)
            } else {
                let cert = certify_vizing_fournier_shadow(inst.n, &inst.edges);
                (
                    cert.num_large_blocks,
                    cert.chi_prime_large,
                    cert.fournier_kempe_certified,
                )
            };
        out.push(HighDensityPbdCertRecord {
            name: inst.name,
            family: inst.family,
            n: inst.n,
            num_edges: inst.edges.len(),
            num_large_blocks,
            max_large_deg: max_d_ge3,
            chi_prime_large,
            omega,
            chi_prime: chi,
            shadow_identity_verified: id_ok,
            fournier_kempe_certified: fk_ok,
        });
    }
    Ok(out)
}

/// Theorem 2.6(a): Compute the exact Edmonds fractional chromatic index `chi'_f(G_res)`
/// of a 2-uniform simple graph `G_res` on `n <= 13` vertices:
///
///     chi'_f(G_res) = max( Delta(G_res), max_{U subseteq [n], |U| >= 3 odd} 2 * |E(G_res[U])| / (|U| - 1) ).
pub fn compute_edmonds_residual_fractional_index(n: usize, e2_adj: &[u64]) -> f64 {
    let mut best = 0.0f64;
    for &adj in e2_adj.iter().take(n) {
        best = best.max(adj.count_ones() as f64);
    }
    if (3..=17).contains(&n) {
        let total_masks = 1u64 << n;
        for u_mask in 0..total_masks {
            let sz = u_mask.count_ones();
            if sz >= 3 && sz % 2 == 1 {
                let mut twice_edges = 0u32;
                let mut m = u_mask;
                while m != 0 {
                    let v = m.trailing_zeros() as usize;
                    m &= m - 1;
                    twice_edges += (e2_adj[v] & u_mask).count_ones();
                }
                let ratio = (twice_edges as f64) / ((sz - 1) as f64);
                if ratio > best {
                    best = ratio;
                }
            }
        }
    }
    best
}

/// Evaluate and certify all 8 `n = 13` Projective, Steiner, and Hybrid PBD instances
/// from `records::n13_projective_steiner_pbd_suite()` under Theorem 2.6.
pub fn certify_n13_projective_steiner_suite(
    gpu_opt: Option<&Gpu>,
) -> Result<Vec<HighDensityPbdCertRecord>> {
    let suite = records::n13_projective_steiner_pbd_suite();
    let mut out = Vec::with_capacity(suite.len());
    for inst in suite {
        let (is_lin, omega, chi, _nodes) = if let Some(gpu) = gpu_opt {
            eval_efl_batch_gpu(gpu, inst.n, std::slice::from_ref(&inst.edges))?[0]
        } else {
            eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges)
        };
        assert_eq!(is_lin, 1, "PBD instance {} must be linear", inst.name);
        let (max_d_ge3, id_ok) = verify_shadow_slack_equality_frontier(inst.n, &inst.edges);
        let (num_large_blocks, chi_prime_large, fk_ok) =
            if inst.edges.iter().all(|&e| e.count_ones() >= 3) {
                (inst.edges.len(), chi, (chi as usize) <= inst.n)
            } else {
                let cert = certify_vizing_fournier_shadow(inst.n, &inst.edges);
                (
                    cert.num_large_blocks,
                    cert.chi_prime_large,
                    cert.fournier_kempe_certified,
                )
            };
        out.push(HighDensityPbdCertRecord {
            name: inst.name,
            family: inst.family,
            n: inst.n,
            num_edges: inst.edges.len(),
            num_large_blocks,
            max_large_deg: max_d_ge3,
            chi_prime_large,
            omega,
            chi_prime: chi,
            shadow_identity_verified: id_ok,
            fournier_kempe_certified: fk_ok,
        });
    }
    Ok(out)
}

/// Theorem 2.7(a): Verify Resolvent Parallel-Class Peeling & Residual Shadow-Slack Discharging.
///
/// Given a PBD `H = ([n], edges)` and `p` edge-disjoint partial parallel classes `P_1, ..., P_p`
/// of blocks of size `>= 3`, verifies that:
///   1. Every `P_j` is a valid matching of blocks of size `>= 3` in `edges`, and `P_1, ..., P_p` are disjoint.
///   2. Peeling `P_1, ..., P_p` as `p` dedicated color classes reduces the residual color budget `q' -> q' - p`
///      while reducing each covered vertex's residual pair degree by `|E_{j,v}| - 1 >= 2`, thereby strictly
///      increasing the Vizing–Fournier shadow slack `delta_VF(v)` at `v` by
///      `Delta_peel(v) = sum_{j: v in V(P_j)} (|E_{j,v}| - 2) >= |{j : v in V(P_j)}|`.
///
/// Returns `(num_classes_p, min_vertex_slack_gain, verified)`.
pub fn verify_parallel_class_peeling_slack(
    n: usize,
    edges: &[u64],
    parallel_classes: &[Vec<u64>],
) -> (usize, i32, bool) {
    let mut rem_edges = edges.to_vec();
    let mut vertex_gain = vec![0i32; n];
    let mut vertex_cov = vec![0i32; n];

    for cls in parallel_classes {
        let mut used_v = 0u64;
        for &b in cls {
            let r = b.count_ones() as i32;
            if r < 3 || (used_v & b) != 0 {
                return (parallel_classes.len(), 0, false);
            }
            if let Some(pos) = rem_edges.iter().position(|&e| e == b) {
                rem_edges.swap_remove(pos);
            } else {
                return (parallel_classes.len(), 0, false);
            }
            used_v |= b;
            for v in 0..n {
                if ((b >> v) & 1) != 0 {
                    vertex_cov[v] += 1;
                    vertex_gain[v] += r - 2;
                }
            }
        }
    }

    let mut ok = true;
    let mut min_gain = i32::MAX;
    for v in 0..n {
        if vertex_gain[v] < vertex_cov[v] {
            ok = false;
        }
        min_gain = min_gain.min(vertex_gain[v]);
    }
    (
        parallel_classes.len(),
        if min_gain == i32::MAX { 0 } else { min_gain },
        ok,
    )
}

/// Evaluate and certify all 8 `n = 14, 15` Kirkman, Non-Resolvable STS(15), and High-Rank PBD
/// instances from `records::n14_n15_kirkman_pbd_suite()` under Theorem 2.7.
pub fn certify_n14_n15_kirkman_suite(
    gpu_opt: Option<&Gpu>,
) -> Result<Vec<HighDensityPbdCertRecord>> {
    let suite = records::n14_n15_kirkman_pbd_suite();
    let mut out = Vec::with_capacity(suite.len());
    for inst in suite {
        let (is_lin, omega, chi, _nodes) = if let Some(gpu) = gpu_opt {
            eval_efl_batch_gpu(gpu, inst.n, std::slice::from_ref(&inst.edges))?[0]
        } else {
            eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges)
        };
        assert_eq!(is_lin, 1, "PBD instance {} must be linear", inst.name);
        let (max_d_ge3, id_ok) = verify_shadow_slack_equality_frontier(inst.n, &inst.edges);
        let (num_large_blocks, chi_prime_large, fk_ok) =
            if inst.edges.iter().all(|&e| e.count_ones() >= 3) {
                (inst.edges.len(), chi, (chi as usize) <= inst.n)
            } else {
                let cert = certify_vizing_fournier_shadow(inst.n, &inst.edges);
                (
                    cert.num_large_blocks,
                    cert.chi_prime_large,
                    cert.fournier_kempe_certified,
                )
            };
        out.push(HighDensityPbdCertRecord {
            name: inst.name,
            family: inst.family,
            n: inst.n,
            num_edges: inst.edges.len(),
            num_large_blocks,
            max_large_deg: max_d_ge3,
            chi_prime_large,
            omega,
            chi_prime: chi,
            shadow_identity_verified: id_ok,
            fournier_kempe_certified: fk_ok,
        });
    }
    Ok(out)
}

/// Theorem 2.8(a): Verify the Large-Block Clique/Matching Dichotomy at threshold `tau = floor(sqrt(n))`
/// for a linear hypergraph `H = ([n], edges)`.
///
/// Let `E_{>tau} := {E in edges : |E| >= tau + 1}`. Verifies that:
///   1. Every vertex `v in [n]` lies in at most `floor((n - 1) / tau)` blocks of `E_{>tau}`.
///   2. The matching number `alpha(E_{>tau})` (maximum number of mutually vertex-disjoint blocks in `E_{>tau}`)
///      satisfies `alpha(E_{>tau}) <= floor(n / (tau + 1)) <= tau`.
///   3. (Clique/Matching Dichotomy):
///      - If `alpha(E_{>tau}) <= 1`, then `E_{>tau}` is pairwise intersecting, so by De Bruijn–Erdős / Hindman,
///        `omega(E_{>tau}) == chi'(E_{>tau}) == |E_{>tau}| <= n`.
///      - If `alpha(E_{>tau}) >= 2`, then `chi'(E_{>tau}) <= |E_{>tau}| - (alpha(E_{>tau}) - 1) < |E_{>tau}|`,
///        and any maximum matching `M subseteq E_{>tau}` covers `>= alpha(E_{>tau}) * (tau + 1)` vertices while
///        increasing the Vizing–Fournier shadow slack `delta_VF(v)` by `|E| - 2 >= tau - 1` at every `v in V(M)`.
///
/// Returns `(tau, num_gt_tau, omega_gt_tau, alpha_gt_tau, chi_gt_tau, verified)`.
pub fn verify_large_block_tau_dichotomy(
    n: usize,
    edges: &[u64],
) -> (usize, usize, u32, u32, u32, bool) {
    let tau = (n as f64).sqrt().floor() as usize;
    if tau == 0 {
        return (0, 0, 0, 0, 0, true);
    }
    let e_gt_tau: Vec<u64> = edges
        .iter()
        .copied()
        .filter(|&e| (e.count_ones() as usize) > tau)
        .collect();
    let m_gt = e_gt_tau.len();
    if m_gt == 0 {
        return (tau, 0, 0, 0, 0, true);
    }

    let mut ok = true;
    let max_deg_bound = (n - 1) / tau;
    for v in 0..n {
        let d_v = e_gt_tau.iter().filter(|&&e| ((e >> v) & 1) != 0).count();
        if d_v > max_deg_bound {
            ok = false;
        }
    }

    let (is_lin, omega_gt, chi_gt, _) = eval_efl_hypergraph_twin_cpu(n as u32, &e_gt_tau);
    if is_lin != 1 {
        ok = false;
    }

    let mut disj_adj = vec![0u64; 64];
    for a in 0..m_gt {
        for b in (a + 1)..m_gt {
            if (e_gt_tau[a] & e_gt_tau[b]) == 0 {
                disj_adj[a] |= 1u64 << b;
                disj_adj[b] |= 1u64 << a;
            }
        }
    }
    let (alpha_gt, match_mask) = find_max_clique_twin_cpu(&disj_adj, m_gt as u32);
    if (alpha_gt as usize) > n / (tau + 1) || (alpha_gt as usize) > tau {
        ok = false;
    }

    if alpha_gt <= 1 {
        if omega_gt != m_gt as u32 || chi_gt != m_gt as u32 || m_gt > n {
            ok = false;
        }
    } else {
        let ub = (m_gt as u32) - (alpha_gt - 1);
        if chi_gt > ub {
            ok = false;
        }
        let mut cov_verts = 0u32;
        let mut mm = match_mask;
        while mm != 0 {
            let idx = mm.trailing_zeros() as usize;
            mm &= mm - 1;
            let r = e_gt_tau[idx].count_ones();
            cov_verts += r;
            if (r as usize) <= tau {
                ok = false;
            }
        }
        if (cov_verts as usize) < (alpha_gt as usize) * (tau + 1) {
            ok = false;
        }
    }

    (tau, m_gt, omega_gt, alpha_gt, chi_gt, ok)
}

/// Evaluate and certify all 8 `n = 16, 17` Affine Plane `AG(2, 4)`, Bruck Net, Punctured `PG(2, 4)`,
/// and Hanani PBD instances from `records::n16_n17_affine_projective_pbd_suite()` under Theorem 2.8.
pub fn certify_n16_n17_affine_suite(
    gpu_opt: Option<&Gpu>,
) -> Result<Vec<HighDensityPbdCertRecord>> {
    let suite = records::n16_n17_affine_projective_pbd_suite();
    let mut out = Vec::with_capacity(suite.len());
    for inst in suite {
        let (is_lin, omega, chi, _nodes) = if let Some(gpu) = gpu_opt {
            eval_efl_batch_gpu(gpu, inst.n, std::slice::from_ref(&inst.edges))?[0]
        } else {
            eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges)
        };
        assert_eq!(is_lin, 1, "PBD instance {} must be linear", inst.name);
        let (max_d_ge3, id_ok) = verify_shadow_slack_equality_frontier(inst.n, &inst.edges);
        let (_tau, _m_gt, _om_gt, _al_gt, _ch_gt, tau_ok) =
            verify_large_block_tau_dichotomy(inst.n, &inst.edges);
        assert!(
            tau_ok,
            "Theorem 2.8(a) tau-dichotomy failed on {}",
            inst.name
        );
        let (num_large_blocks, chi_prime_large, fk_ok) =
            if inst.edges.iter().all(|&e| e.count_ones() >= 3) {
                (inst.edges.len(), chi, (chi as usize) <= inst.n)
            } else {
                let cert = certify_vizing_fournier_shadow(inst.n, &inst.edges);
                (
                    cert.num_large_blocks,
                    cert.chi_prime_large,
                    cert.fournier_kempe_certified,
                )
            };
        out.push(HighDensityPbdCertRecord {
            name: inst.name,
            family: inst.family,
            n: inst.n,
            num_edges: inst.edges.len(),
            num_large_blocks,
            max_large_deg: max_d_ge3,
            chi_prime_large,
            omega,
            chi_prime: chi,
            shadow_identity_verified: id_ok && tau_ok,
            fournier_kempe_certified: fk_ok,
        });
    }
    Ok(out)
}

/// Theorem 2.9(a): Verify Cross-Tier Pair-Budget Conservation Between `E_{>tau}` and `E_{3..tau}`
/// and Post-Peeling Residual Degree Contraction at threshold `tau = floor(sqrt(n))`.
///
/// Partitioning `edges = E_{>tau} ∪ E_{3..tau} ∪ E_2` with `tau = floor(sqrt(n))`, verifies that:
///   1. `P_{3..tau} + |E_2| <= binom(n, 2) - P_{>tau} <= binom(n, 2) - |E_{>tau}| * binom(tau + 1, 2)`,
///      with exact equality `P_{3..tau} + |E_2| == binom(n, 2) - P_{>tau}` whenever `H` is a PBD.
///   2. `|E_{3..tau}| <= (binom(n, 2) - P_{>tau}) / 3 <= (binom(n, 2) - |E_{>tau}| * binom(tau + 1, 2)) / 3`.
///   3. At every vertex `v in [n]`, `sigma_{3..tau}(v) + d_2(v) <= (n - 1) - sigma_{>tau}(v) <= (n - 1) - tau * d_{>tau}(v)`
///      (with exact equality on the first relation for every PBD), and
///      `sum_{v=0}^{n-1} (sigma_{3..tau}(v) + d_2(v)) == 2 * (P_{3..tau} + |E_2|) <= n * (n - 1) - |E_{>tau}| * tau * (tau + 1)`.
///
/// Returns `(tau, num_gt_tau, p_gt_tau, p_med_plus_2, rem_pair_budget, verified)`.
pub fn verify_cross_tier_pair_budget_contraction(
    n: usize,
    edges: &[u64],
) -> (usize, usize, u64, u64, u64, bool) {
    let tau = (n as f64).sqrt().floor() as usize;
    let total_pairs = (n * n.saturating_sub(1) / 2) as u64;
    let min_large_pairs = (tau * (tau + 1) / 2) as u64;

    let mut num_gt_tau = 0usize;
    let mut num_med = 0usize;
    let mut p_gt_tau = 0u64;
    let mut p_med = 0u64;
    let mut p_e2 = 0u64;

    let mut d_gt_tau = vec![0u64; n];
    let mut sigma_gt_tau = vec![0u64; n];
    let mut sigma_med = vec![0u64; n];
    let mut d_e2 = vec![0u64; n];

    for &e in edges {
        let r = e.count_ones() as usize;
        let pairs = (r * r.saturating_sub(1) / 2) as u64;
        if r > tau {
            num_gt_tau += 1;
            p_gt_tau += pairs;
            for v in 0..n {
                if ((e >> v) & 1) != 0 {
                    d_gt_tau[v] += 1;
                    sigma_gt_tau[v] += (r - 1) as u64;
                }
            }
        } else if r >= 3 {
            num_med += 1;
            p_med += pairs;
            for (v, slot) in sigma_med.iter_mut().enumerate().take(n) {
                if ((e >> v) & 1) != 0 {
                    *slot += (r - 1) as u64;
                }
            }
        } else if r == 2 {
            p_e2 += 1;
            for (v, slot) in d_e2.iter_mut().enumerate().take(n) {
                if ((e >> v) & 1) != 0 {
                    *slot += 1;
                }
            }
        }
    }

    let p_med_plus_2 = p_med + p_e2;
    let rem_pair_budget = total_pairs.saturating_sub(p_gt_tau);
    let coarse_rem_budget =
        total_pairs.saturating_sub((num_gt_tau as u64).saturating_mul(min_large_pairs));
    let is_pbd = p_gt_tau + p_med_plus_2 == total_pairs;

    let mut ok = p_gt_tau >= (num_gt_tau as u64) * min_large_pairs
        && p_med_plus_2 <= rem_pair_budget
        && rem_pair_budget <= coarse_rem_budget
        && (num_med as u64) <= rem_pair_budget / 3
        && (num_med as u64) <= coarse_rem_budget / 3;

    let mut sum_res_inc = 0u64;
    for v in 0..n {
        let res_v = sigma_med[v] + d_e2[v];
        sum_res_inc += res_v;
        if sigma_gt_tau[v] < (tau as u64) * d_gt_tau[v] {
            ok = false;
        }
        if sigma_gt_tau[v] + res_v > (n.saturating_sub(1)) as u64 {
            ok = false;
        }
        if is_pbd && sigma_gt_tau[v] + res_v != (n.saturating_sub(1)) as u64 {
            ok = false;
        }
    }

    if sum_res_inc != 2 * p_med_plus_2 || sum_res_inc > 2 * coarse_rem_budget {
        ok = false;
    }

    (tau, num_gt_tau, p_gt_tau, p_med_plus_2, rem_pair_budget, ok)
}

/// Evaluate and certify all 8 `n = 18, 19, 20, 21` Projective Plane `PG(2, 4)`, Punctured `PG(2, 4)`
/// (`AG(2, 4)^{+p}`), and Mixed-Rank PBD instances from `records::n18_to_n21_projective_pbd_suite()` under Theorem 2.9.
pub fn certify_n18_to_n21_projective_suite(
    gpu_opt: Option<&Gpu>,
) -> Result<Vec<HighDensityPbdCertRecord>> {
    let suite = records::n18_to_n21_projective_pbd_suite();
    let mut out = Vec::with_capacity(suite.len());
    for inst in suite {
        let (is_lin, omega, chi, _nodes) = if let Some(gpu) = gpu_opt {
            eval_efl_batch_gpu(gpu, inst.n, std::slice::from_ref(&inst.edges))?[0]
        } else {
            eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges)
        };
        assert_eq!(is_lin, 1, "PBD instance {} must be linear", inst.name);
        let (max_d_ge3, id_ok) = verify_shadow_slack_equality_frontier(inst.n, &inst.edges);
        let (_tau, _m_gt, _om_gt, _al_gt, _ch_gt, tau_ok) =
            verify_large_block_tau_dichotomy(inst.n, &inst.edges);
        let (_tau2, _m_gt2, _p_gt, _p_m2, _rem_b, budget_ok) =
            verify_cross_tier_pair_budget_contraction(inst.n, &inst.edges);
        assert!(
            tau_ok && budget_ok,
            "Theorem 2.8(a)/2.9(a) invariants failed on {}",
            inst.name
        );
        let (num_large_blocks, chi_prime_large, fk_ok) =
            if inst.edges.iter().all(|&e| e.count_ones() >= 3) {
                (inst.edges.len(), chi, (chi as usize) <= inst.n)
            } else {
                let cert = certify_vizing_fournier_shadow(inst.n, &inst.edges);
                (
                    cert.num_large_blocks,
                    cert.chi_prime_large,
                    cert.fournier_kempe_certified,
                )
            };
        out.push(HighDensityPbdCertRecord {
            name: inst.name,
            family: inst.family,
            n: inst.n,
            num_edges: inst.edges.len(),
            num_large_blocks,
            max_large_deg: max_d_ge3,
            chi_prime_large,
            omega,
            chi_prime: chi,
            shadow_identity_verified: id_ok && tau_ok && budget_ok,
            fournier_kempe_certified: fk_ok,
        });
    }
    Ok(out)
}

/// Evaluate and certify all 8 `n = 22, 23, 24, 25` Affine Plane `AG(2, 5)`, Arc-Punctured `AG(2, 5)`,
/// and Mixed-Rank `(3, 4, 5, 6)`-Block PBD instances from `records::n22_to_n25_ag2_5_pbd_suite()` under Theorem 2.10.
pub fn certify_n22_to_n25_ag2_5_suite(
    gpu_opt: Option<&Gpu>,
) -> Result<Vec<HighDensityPbdCertRecord>> {
    let suite = records::n22_to_n25_ag2_5_pbd_suite();
    let mut out = Vec::with_capacity(suite.len());
    for inst in suite {
        let (is_lin, omega, chi, _nodes) = if let Some(gpu) = gpu_opt {
            eval_efl_batch_gpu(gpu, inst.n, std::slice::from_ref(&inst.edges))?[0]
        } else {
            eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges)
        };
        assert_eq!(is_lin, 1, "PBD instance {} must be linear", inst.name);
        let (max_d_ge3, id_ok) = verify_shadow_slack_equality_frontier(inst.n, &inst.edges);
        let (_tau, _m_gt, _om_gt, _al_gt, _ch_gt, tau_ok) =
            verify_large_block_tau_dichotomy(inst.n, &inst.edges);
        let (_tau2, _m_gt2, _p_gt, _p_m2, _rem_b, budget_ok) =
            verify_cross_tier_pair_budget_contraction(inst.n, &inst.edges);
        let (_tau3, _pm, _pl, psi_3tier, closure_ok) =
            crate::verify_cpu::verify_three_tier_discharging_closure_cpu(
                inst.n as u32,
                &inst.edges,
            );
        assert!(
            tau_ok && budget_ok && closure_ok == 1 && psi_3tier == 0,
            "Theorem 2.8/2.9/2.10 invariants failed on {}",
            inst.name
        );
        let (num_large_blocks, chi_prime_large, fk_ok) =
            if inst.edges.iter().all(|&e| e.count_ones() >= 3) {
                (inst.edges.len(), chi, (chi as usize) <= inst.n)
            } else {
                let cert = certify_vizing_fournier_shadow(inst.n, &inst.edges);
                (
                    cert.num_large_blocks,
                    cert.chi_prime_large,
                    cert.fournier_kempe_certified,
                )
            };
        out.push(HighDensityPbdCertRecord {
            name: inst.name,
            family: inst.family,
            n: inst.n,
            num_edges: inst.edges.len(),
            num_large_blocks,
            max_large_deg: max_d_ge3,
            chi_prime_large,
            omega,
            chi_prime: chi,
            shadow_identity_verified: id_ok && tau_ok && budget_ok && closure_ok == 1,
            fournier_kempe_certified: fk_ok,
        });
    }
    Ok(out)
}

/// Extract and classify all PBDs across `3 <= n <= n_max` where `shadow_deficit_certified`
/// is false, verifying that 100% are resolved by Theorem 2.4 (`fournier_kempe_certified`).
pub fn classify_exceptional_pbds(n_max: usize, max_profiles: usize) -> Vec<ExceptionalPbdProfile> {
    let mut out = Vec::new();
    for n in 3..=n_max {
        let pbds = enumerate_canonical_pbds(n, max_profiles);
        let certs: Vec<VizingFournierShadowCert> = pbds
            .par_iter()
            .map(|edges| certify_vizing_fournier_shadow(n, edges))
            .collect();
        for (edges, cert) in pbds.iter().zip(certs.iter()) {
            if !cert.shadow_deficit_certified {
                let mut large_blocks = Vec::new();
                let mut deg_ge3 = vec![0u32; n];
                for &e in edges {
                    if e.count_ones() >= 3 {
                        let mut blk = Vec::new();
                        for (v, slot) in deg_ge3.iter_mut().enumerate().take(n) {
                            if ((e >> v) & 1) != 0 {
                                blk.push(v);
                                *slot += 1;
                            }
                        }
                        large_blocks.push(blk);
                    }
                }
                let max_large_deg = deg_ge3.into_iter().max().unwrap_or(0);
                out.push(ExceptionalPbdProfile {
                    n,
                    num_large_blocks: cert.num_large_blocks,
                    max_large_deg,
                    chi_prime_large: cert.chi_prime_large,
                    max_residual_deg: cert.max_residual_deg,
                    num_ge_n_minus_q: cert.num_ge_n_minus_q,
                    large_blocks,
                    fournier_kempe_certified: cert.fournier_kempe_certified,
                });
            }
        }
    }
    out
}

/// Run the PBD census for order `n`, evaluating all canonical PBD configurations on GPU (if
/// provided) or in parallel on CPU via `eval_efl_hypergraph_twin_cpu`.
pub fn run_pbd_census(
    n: usize,
    max_profiles: usize,
    gpu_opt: Option<&Gpu>,
) -> Result<PbdCensusSummary> {
    let pbds = enumerate_canonical_pbds(n, max_profiles);
    let results: Vec<(u32, u32, u32, u64)> = if let Some(gpu) = gpu_opt {
        eval_efl_batch_gpu(gpu, n, &pbds)?
    } else {
        pbds.par_iter()
            .map(|edges| eval_efl_hypergraph_twin_cpu(n as u32, edges))
            .collect()
    };

    let certs: Vec<VizingFournierShadowCert> = pbds
        .par_iter()
        .map(|edges| certify_vizing_fournier_shadow(n, edges))
        .collect();

    let mut max_chi = 0usize;
    let mut min_chi = usize::MAX;
    let mut tight_count = 0usize;
    let mut vf_certified = 0usize;
    let mut fk_certified = 0usize;
    let mut q_le_2_count = 0usize;
    let mut violations = 0usize;
    let mut total_nodes = 0u64;
    let mut tight_profiles = Vec::new();

    for ((edges, &(is_lin, omega, chi, nodes)), cert) in
        pbds.iter().zip(results.iter()).zip(certs.iter())
    {
        assert_eq!(is_lin, 1, "all generated PBDs must be linear");
        if cert.q_le_2_unconditional {
            q_le_2_count += 1;
        }
        if cert.shadow_deficit_certified {
            vf_certified += 1;
            assert!(
                (chi as usize) <= n,
                "Theorem 2.3 certified PBD must have chi' <= n"
            );
        }
        if cert.fournier_kempe_certified {
            fk_certified += 1;
            assert!(
                (chi as usize) <= n,
                "Theorem 2.4 certified PBD must have chi' <= n"
            );
        }
        let chi_usize = chi as usize;
        if chi_usize > max_chi {
            max_chi = chi_usize;
        }
        if chi_usize < min_chi {
            min_chi = chi_usize;
        }
        if chi_usize > n {
            violations += 1;
        }
        total_nodes += nodes;
        if chi_usize == n {
            tight_count += 1;
            let mut r_counts = std::collections::BTreeMap::new();
            for &e in edges {
                *r_counts.entry(e.count_ones()).or_insert(0usize) += 1;
            }
            tight_profiles.push(PbdProfileRecord {
                n,
                num_edges: edges.len(),
                rank_hist: r_counts.into_iter().collect(),
                omega,
                chi_prime: chi,
                search_nodes: nodes,
                edges: edges.clone(),
            });
        }
    }

    Ok(PbdCensusSummary {
        n,
        total_pbds_evaluated: pbds.len(),
        vizing_fournier_certified: vf_certified,
        fournier_kempe_certified: fk_certified,
        q_le_2_unconditional: q_le_2_count,
        max_chi_prime: max_chi,
        min_chi_prime: if min_chi == usize::MAX { 0 } else { min_chi },
        tight_count,
        efl_violations: violations,
        total_search_nodes: total_nodes,
        tight_profiles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twin_cpu_matches_verify_cpu_on_canonical_suite() {
        for inst in records::canonical_benchmark_suite() {
            let (is_lin, omega, chi, _nodes) =
                eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges);
            assert_eq!(is_lin, 1);
            assert_eq!(
                chi as usize, inst.expected_chi_prime,
                "twin CPU mismatch on {}",
                inst.name
            );
            let rep = crate::verify_cpu::analyze_hypergraph_cpu(inst.n, &inst.edges, 0);
            assert_eq!(omega as usize, rep.clique_number_omega);
            assert_eq!(chi as usize, rep.chromatic_index);
        }
    }

    #[test]
    fn test_exhaustive_pbd_census_n3_to_n8_cpu() {
        for (n, expected_tight) in [(3, 1), (4, 1), (5, 2), (6, 2), (7, 6), (8, 3)] {
            let summary = run_pbd_census(n, 500, None).unwrap();
            assert_eq!(summary.efl_violations, 0);
            assert_eq!(summary.max_chi_prime, n);
            assert_eq!(summary.tight_count, expected_tight);
        }
    }

    #[test]
    fn test_two_triple_and_ag23_deletion_families() {
        // Theorem 2.2: Two-Triple PBD dichotomy H_{2,disj}(n) and H_{2,star}(n)
        for (n, exp_disj) in [(6, 4), (7, 7), (8, 7), (9, 8), (10, 9), (11, 11), (12, 11)] {
            let edges = records::build_two_disjoint_triples_pbd(n);
            let (_, _, chi, _) = eval_efl_hypergraph_twin_cpu(n as u32, &edges);
            assert_eq!(chi as usize, exp_disj, "H_2,disj({n}) mismatch");
        }
        for (n, exp_star) in [
            (5, 4),
            (6, 5),
            (7, 6),
            (8, 7),
            (9, 8),
            (10, 9),
            (11, 11),
            (12, 11),
        ] {
            let edges = records::build_two_intersecting_triples_pbd(n);
            let (_, _, chi, _) = eval_efl_hypergraph_twin_cpu(n as u32, &edges);
            assert_eq!(chi as usize, exp_star, "H_2,star({n}) mismatch");
        }
        // Theorem 2.2: AG(2,3)^{+p} projective-to-affine deletion hierarchy
        for p in 0..=3 {
            let n = 9 + p;
            let edges = records::build_ag2_3_ideal_extension(p);
            let (_, omega, chi, _) = eval_efl_hypergraph_twin_cpu(n as u32, &edges);
            assert_eq!(omega as usize, 2 * p + 4);
            assert_eq!(chi as usize, 2 * p + 4);
        }

        // Theorems 2.3 & 2.4: Vizing-Fournier Shadow Deficit & Fournier-Kempe 100% Certification across n = 3..8
        for n in 3..=8 {
            let summary = run_pbd_census(n, 500, None).unwrap();
            assert_eq!(summary.efl_violations, 0);
            assert!(summary.vizing_fournier_certified >= summary.q_le_2_unconditional);
            assert_eq!(
                summary.fournier_kempe_certified,
                summary.total_pbds_evaluated
            );
            assert!(summary.q_le_2_unconditional > 0);
        }
        let exc8 = classify_exceptional_pbds(8, usize::MAX);
        assert_eq!(exc8.len(), 2);
        assert!(exc8.iter().all(|e| e.fournier_kempe_certified));

        // Theorem 2.5: Shadow-Slack Equality Frontier & n = 11, 12 High-Density PBD Suite
        for n in 3..=8 {
            for edges in enumerate_canonical_pbds(n, 500) {
                let (_, id_ok) = verify_shadow_slack_equality_frontier(n, &edges);
                assert!(id_ok, "Shadow-slack identity failed at n={n}");
            }
        }
        let suite_11_12 = records::n11_n12_high_density_pbd_suite();
        let certs_11_12 = certify_n11_n12_high_density_suite(None).unwrap();
        assert_eq!(certs_11_12.len(), 9);
        for (inst, rec) in suite_11_12.iter().zip(certs_11_12.iter()) {
            assert!(
                rec.shadow_identity_verified,
                "Shadow identity failed on {}",
                rec.name
            );
            assert!(
                rec.fournier_kempe_certified,
                "Fournier-Kempe certification failed on {}",
                rec.name
            );
            assert_eq!(
                rec.chi_prime as usize, inst.expected_chi_prime,
                "Chromatic index mismatch on {}",
                rec.name
            );
            assert!((rec.chi_prime as usize) <= rec.n);
        }

        // Theorem 2.6: Fractional Blossom Shadow-Slack & Complete n = 13 Projective/Steiner PBD Suite
        for n in 3..=8 {
            for edges in enumerate_canonical_pbds(n, 500) {
                let mut e2_adj = vec![0u64; n];
                for &e in &edges {
                    if e.count_ones() == 2 {
                        let u = e.trailing_zeros() as usize;
                        let v = (e & (e - 1)).trailing_zeros() as usize;
                        e2_adj[u] |= 1u64 << v;
                        e2_adj[v] |= 1u64 << u;
                    }
                }
                let chi_f = compute_edmonds_residual_fractional_index(n, &e2_adj);
                assert!(chi_f <= n as f64 + 1e-9);
            }
        }
        let suite_13 = records::n13_projective_steiner_pbd_suite();
        let certs_13 = certify_n13_projective_steiner_suite(None).unwrap();
        assert_eq!(certs_13.len(), 8);
        for (inst, rec) in suite_13.iter().zip(certs_13.iter()) {
            assert!(
                rec.shadow_identity_verified,
                "Shadow identity failed on {}",
                rec.name
            );
            assert!(
                rec.fournier_kempe_certified,
                "Fournier-Kempe certification failed on {}",
                rec.name
            );
            assert_eq!(
                rec.chi_prime as usize, inst.expected_chi_prime,
                "Chromatic index mismatch on {}",
                rec.name
            );
            assert!((rec.chi_prime as usize) <= rec.n);
        }

        // Theorem 2.7: Resolvent Parallel-Class Peeling & Complete n = 14, 15 Kirkman/PBD Suite
        let kirkman = records::build_kirkman_sts15();
        let kirkman_classes = records::build_kirkman_sts15_parallel_classes();
        let (p_cnt, min_gain, peel_ok) =
            verify_parallel_class_peeling_slack(15, &kirkman, &kirkman_classes);
        assert_eq!(p_cnt, 7);
        assert_eq!(min_gain, 7);
        assert!(peel_ok);

        let ag23 = records::build_affine_ag2_3();
        let ag23_classes = vec![
            vec![ag23[0], ag23[1], ag23[2]],
            vec![ag23[3], ag23[4], ag23[5]],
            vec![ag23[6], ag23[7], ag23[8]],
            vec![ag23[9], ag23[10], ag23[11]],
        ];
        let (p_ag, gain_ag, ok_ag) = verify_parallel_class_peeling_slack(9, &ag23, &ag23_classes);
        assert_eq!(p_ag, 4);
        assert_eq!(gain_ag, 4);
        assert!(ok_ag);

        let suite_14_15 = records::n14_n15_kirkman_pbd_suite();
        let certs_14_15 = certify_n14_n15_kirkman_suite(None).unwrap();
        assert_eq!(certs_14_15.len(), 8);
        for (inst, rec) in suite_14_15.iter().zip(certs_14_15.iter()) {
            assert!(
                rec.shadow_identity_verified,
                "Shadow identity failed on {}",
                rec.name
            );
            assert!(
                rec.fournier_kempe_certified,
                "Fournier-Kempe certification failed on {}",
                rec.name
            );
            assert_eq!(
                rec.chi_prime as usize, inst.expected_chi_prime,
                "Chromatic index mismatch on {}",
                rec.name
            );
            assert!((rec.chi_prime as usize) <= rec.n);
        }

        // Theorem 2.8: Large-Block Clique/Matching Dichotomy & Complete n = 16, 17 Affine/Projective PBD Suite
        let ag24 = records::build_affine_ag2_4();
        let ag24_classes = records::build_affine_ag2_4_parallel_classes();
        let (p_ag24, gain_ag24, ok_ag24) =
            verify_parallel_class_peeling_slack(16, &ag24, &ag24_classes);
        assert_eq!(p_ag24, 5);
        assert_eq!(gain_ag24, 10);
        assert!(ok_ag24);

        for n in 3..=8 {
            for edges in enumerate_canonical_pbds(n, 500) {
                let (_, _, _, _, _, tau_ok) = verify_large_block_tau_dichotomy(n, &edges);
                assert!(tau_ok, "Large-block tau-dichotomy failed at n={n}");
            }
        }

        let suite_16_17 = records::n16_n17_affine_projective_pbd_suite();
        let certs_16_17 = certify_n16_n17_affine_suite(None).unwrap();
        assert_eq!(certs_16_17.len(), 8);
        for (inst, rec) in suite_16_17.iter().zip(certs_16_17.iter()) {
            assert!(
                rec.shadow_identity_verified,
                "Shadow/tau identity failed on {}",
                rec.name
            );
            assert!(
                rec.fournier_kempe_certified,
                "Fournier-Kempe certification failed on {}",
                rec.name
            );
            assert_eq!(
                rec.chi_prime as usize, inst.expected_chi_prime,
                "Chromatic index mismatch on {}",
                rec.name
            );
            assert!((rec.chi_prime as usize) <= rec.n);
        }

        // Theorem 2.9(a): Cross-Tier Pair-Budget Conservation & AG(2,4)^{+p} Exact Chromatic Law
        for n in 3..=8 {
            for edges in enumerate_canonical_pbds(n, 500) {
                let (_, _, p_gt, p_m2, rem_b, budget_ok) =
                    verify_cross_tier_pair_budget_contraction(n, &edges);
                assert!(
                    budget_ok && p_m2 == rem_b && p_gt + p_m2 == (n * (n - 1) / 2) as u64,
                    "Cross-tier pair budget failed at n={n}"
                );
            }
        }
        for p in 0..=4 {
            let n = 16 + p;
            let edges = records::build_ag2_4_ideal_extension(p);
            let (_, omega, chi, _) = eval_efl_hypergraph_twin_cpu(n as u32, &edges);
            assert_eq!(omega as usize, 3 * p + 5);
            assert_eq!(chi as usize, 3 * p + 5);
        }
        let pg24 = records::build_ag2_4_ideal_extension(5);
        let (_, omega_pg24, chi_pg24, _) = eval_efl_hypergraph_twin_cpu(21, &pg24);
        assert_eq!(omega_pg24, 21);
        assert_eq!(chi_pg24, 21);

        // Theorem 2.9(b): Complete n = 18..21 Projective & Punctured PG(2,4) PBD Suite
        let suite_18_21 = records::n18_to_n21_projective_pbd_suite();
        let certs_18_21 = certify_n18_to_n21_projective_suite(None).unwrap();
        assert_eq!(certs_18_21.len(), 8);
        for (inst, rec) in suite_18_21.iter().zip(certs_18_21.iter()) {
            assert!(
                rec.shadow_identity_verified,
                "Shadow/tau/budget identity failed on {}",
                rec.name
            );
            assert!(
                rec.fournier_kempe_certified,
                "Fournier-Kempe certification failed on {}",
                rec.name
            );
            assert_eq!(
                rec.chi_prime as usize, inst.expected_chi_prime,
                "Chromatic index mismatch on {}",
                rec.name
            );
            assert!((rec.chi_prime as usize) <= rec.n);
        }

        // Theorem 2.10(a)-(b): Universal Three-Tier Intermediate-Block Discharging Closure,
        // AG(2,5)^{+p} Interpolation (p = 0..6, n = 25..31), and Complete n = 22..25 PBD Suite
        for n in 3..=8 {
            for edges in enumerate_canonical_pbds(n, 500) {
                let (tau, _pm, _pl, psi, ok) =
                    crate::verify_cpu::verify_three_tier_discharging_closure_cpu(n as u32, &edges);
                assert_eq!(tau as usize, ((n as f64).sqrt().floor() as usize) + 1);
                assert_eq!(psi, 0);
                assert_eq!(ok, 1);
            }
        }
        for p in 0..=5 {
            let n = 25 + p;
            let edges = records::build_ag2_5_ideal_extension(p);
            let (_, omega, chi, _) = eval_efl_hypergraph_twin_cpu(n as u32, &edges);
            assert_eq!(omega as usize, 4 * p + 6);
            assert_eq!(chi as usize, 4 * p + 6);
            let (_, _, _, _, ok) =
                crate::verify_cpu::verify_three_tier_discharging_closure_cpu(n as u32, &edges);
            assert_eq!(ok, 1);
        }
        let pg25 = records::build_projective_pg2_5();
        let (_, omega_pg25, chi_pg25, _) = eval_efl_hypergraph_twin_cpu(31, &pg25);
        assert_eq!(omega_pg25, 31);
        assert_eq!(chi_pg25, 31);

        let suite_22_25 = records::n22_to_n25_ag2_5_pbd_suite();
        let certs_22_25 = certify_n22_to_n25_ag2_5_suite(None).unwrap();
        assert_eq!(certs_22_25.len(), 8);
        for (inst, rec) in suite_22_25.iter().zip(certs_22_25.iter()) {
            assert!(
                rec.shadow_identity_verified,
                "Theorem 2.10 three-tier closure failed on {}",
                rec.name
            );
            assert!(
                rec.fournier_kempe_certified,
                "Fournier-Kempe certification failed on {}",
                rec.name
            );
            assert_eq!(
                rec.chi_prime as usize, inst.expected_chi_prime,
                "Chromatic index mismatch on {}",
                rec.name
            );
            assert!((rec.chi_prime as usize) <= rec.n);
        }
    }

    #[test]
    #[ignore]
    fn gpu_cpu_parity_canonical_benchmark_suite() {
        let gpu = Gpu::init(0, "compute_86").expect("CUDA GPU required for parity test");
        let suite = records::canonical_benchmark_suite();
        let batch_3tier: Vec<(usize, Vec<u64>)> =
            suite.iter().map(|s| (s.n, s.edges.clone())).collect();
        let gpu_3tier = verify_three_tier_batch_gpu(&gpu, &batch_3tier).unwrap();

        // Group by n or evaluate instance by instance since n varies across the suite
        for (i, inst) in suite.iter().enumerate() {
            let cpu_res = eval_efl_hypergraph_twin_cpu(inst.n as u32, &inst.edges);
            let gpu_res =
                eval_efl_batch_gpu(&gpu, inst.n, std::slice::from_ref(&inst.edges)).unwrap()[0];
            assert_eq!(
                cpu_res, gpu_res,
                "GPU/CPU twin parity mismatch on {}",
                inst.name
            );
            let cpu_3tier = crate::verify_cpu::verify_three_tier_discharging_closure_cpu(
                inst.n as u32,
                &inst.edges,
            );
            assert_eq!(
                cpu_3tier, gpu_3tier[i],
                "GPU/CPU 3-tier closure parity mismatch on {}",
                inst.name
            );
            assert_eq!(gpu_3tier[i].4, 1);
        }
    }

    #[test]
    #[ignore]
    fn gpu_cpu_parity_exhaustive_pbd_census_n7() {
        let gpu = Gpu::init(0, "compute_86").expect("CUDA GPU required for parity test");
        let pbds = enumerate_canonical_pbds(7, 500);
        let gpu_results = eval_efl_batch_gpu(&gpu, 7, &pbds).unwrap();
        for (i, edges) in pbds.iter().enumerate() {
            let cpu_res = eval_efl_hypergraph_twin_cpu(7, edges);
            assert_eq!(
                cpu_res, gpu_results[i],
                "GPU/CPU parity mismatch at PBD index {i} for n=7"
            );
        }
    }
}
