//! ============================================================================
//! `records.rs` — Canonical linear hypergraph families, Pairwise Balanced
//! Designs (PBDs), Steiner Triple Systems, Projective/Affine Planes, and
//! historical bounds for Erdős Problem #19 ($500 Decidable Prize: the
//! Erdős–Faber–Lovász Conjecture).
//! ============================================================================

use serde::{Deserialize, Serialize};

/// Historical milestone in the resolution of Erdős Problem #19.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EflMilestone {
    pub year: u32,
    pub authors: &'static str,
    pub regime_or_bound: &'static str,
    pub significance: &'static str,
}

/// Canonical historical milestones from 1972 to 2026.
pub const EFL_MILESTONES: &[EflMilestone] = &[
    EflMilestone {
        year: 1972,
        authors: "Erdős, Faber & Lovász",
        regime_or_bound: "χ(G) <= n conjectured ($500 prize)",
        significance: "Original formulation at the Tea Party in Boulder, Colorado",
    },
    EflMilestone {
        year: 1981,
        authors: "Hindman",
        regime_or_bound: "n <= 10 (all linear hypergraphs)",
        significance: "First finite exhaustive verification up to n = 10",
    },
    EflMilestone {
        year: 1982,
        authors: "Seymour",
        regime_or_bound: "Every linear H has a hyperedge with d_{L(H)}(e) <= n",
        significance: "Proved maximum clique ω(L(H)) <= n and greedy (n+1)-list-colorability under minimum-rank conditions",
    },
    EflMilestone {
        year: 1992,
        authors: "Kahn",
        regime_or_bound: "χ'(H) <= n + o(n)",
        significance: "Asymptotic resolution via the Rödl Nibble ($100 consolation prize from Erdős)",
    },
    EflMilestone {
        year: 1992,
        authors: "Kahn & Seymour",
        regime_or_bound: "χ_f(L(H)) <= n for all n >= 1",
        significance: "Unconditional proof of the Fractional Erdős–Faber–Lovász Conjecture via LP duality",
    },
    EflMilestone {
        year: 2007,
        authors: "Romero & Sánchez-Arroyo",
        regime_or_bound: "n <= 12 (all linear hypergraphs)",
        significance: "Extended finite verification from n <= 10 to n <= 12",
    },
    EflMilestone {
        year: 2010,
        authors: "Faber",
        regime_or_bound: "Regular & uniform linear hypergraphs of rank >= 3",
        significance: "Eliminated broad classes of uniform / regular hypergraphs as minimal counterexamples",
    },
    EflMilestone {
        year: 2021,
        authors: "Kang, Kelly, Kühn, Methuku & Osthus (Ann. of Math. 2024)",
        regime_or_bound: "χ'(H) <= n for all n >= n_0",
        significance: "Proved EFL for all sufficiently large n via absorption, reducing Problem #19 to a finite decidable range",
    },
];

/// Classification of the tightness mechanism when χ'(H) = n (or slack when χ'(H) < n).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TightnessMechanism {
    /// De Bruijn–Erdős projective plane PG(2, q): all n = q^2 + q + 1 lines of size q + 1 intersect pairwise (ω = n, α = 1).
    DeBruijnErdosProjectivePlane,
    /// De Bruijn–Erdős near-pencil NP_n: 1 line of size n - 1 and n - 1 lines of size 2 (ω = n, α = 1).
    DeBruijnErdosNearPencil,
    /// Vizing Class-2 odd complete graph K_n: binom(n, 2) edges of size 2, ω = n - 1, α = (n - 1)/2, matching bound = n.
    VizingOddCompleteGraph,
    /// Cross-Edge Parity Saturation (Theorem 2.3): PBD on odd n >= 7 with small high-rank core S forcing χ'(H) = n despite ω = n - 1 and α = (n - 1)/2.
    CrossEdgeParitySaturation,
    /// Strictly sub-extremal linear hypergraph with χ'(H) <= n - 1.
    SubExtremalSlack,
}

impl TightnessMechanism {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeBruijnErdosProjectivePlane => "DeBruijn-Erdos PG(2,q)",
            Self::DeBruijnErdosNearPencil => "DeBruijn-Erdos Near-Pencil",
            Self::VizingOddCompleteGraph => "Vizing Odd K_n",
            Self::CrossEdgeParitySaturation => "Parity Saturation (Thm 2.3)",
            Self::SubExtremalSlack => "Sub-Extremal (χ' < n)",
        }
    }
}

/// Named benchmark linear hypergraph on `n <= 64` vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EflBenchmarkInstance {
    pub name: String,
    pub family: String,
    pub n: usize,
    pub edges: Vec<u64>,
    pub expected_chi_prime: usize,
    pub mechanism: TightnessMechanism,
}

/// Complete all uncovered vertex pairs `{u, v}` in `0..n` by size-2 hyperedges (Theorem 2.1 PBD completion).
pub fn pbd_complete(n: usize, blocks_ge3: &[u64]) -> Vec<u64> {
    let mut covered_pairs = vec![vec![false; n]; n];
    let mut out = Vec::new();
    for &b in blocks_ge3 {
        if b.count_ones() >= 2 {
            out.push(b);
            for (u, row) in covered_pairs.iter_mut().enumerate().take(n) {
                if ((b >> u) & 1) != 0 {
                    for (v, cell) in row.iter_mut().enumerate().take(n).skip(u + 1) {
                        if ((b >> v) & 1) != 0 {
                            *cell = true;
                        }
                    }
                }
            }
        }
    }
    for (u, row) in covered_pairs.iter().enumerate().take(n) {
        for (v, &cov) in row.iter().enumerate().take(n).skip(u + 1) {
            if !cov {
                out.push((1u64 << u) | (1u64 << v));
            }
        }
    }
    out
}

/// Complete graph `K_n` as a 2-uniform linear hypergraph on `n` vertices (`binom(n, 2)` edges).
pub fn build_complete_graph(n: usize) -> Vec<u64> {
    pbd_complete(n, &[])
}

/// Near-pencil `NP_n` on `n >= 3` vertices: one line `{0, ..., n-2}` of size `n - 1`
/// and `n - 1` lines `{i, n - 1}` of size 2.
pub fn build_near_pencil(n: usize) -> Vec<u64> {
    assert!((3..=63).contains(&n));
    let base_line = (1u64 << (n - 1)) - 1;
    pbd_complete(n, &[base_line])
}

/// Single-triple PBD `H_1(n)` on `n >= 4` vertices: one triple `{0, 1, 2}` and `binom(n, 2) - 3` pairs.
pub fn build_single_triple_pbd(n: usize) -> Vec<u64> {
    assert!((4..=63).contains(&n));
    let triple = 0b111u64;
    pbd_complete(n, &[triple])
}

/// Two-disjoint-triples PBD `H_{2,disj}(n)` on `n >= 6` vertices: `{0,1,2}`, `{3,4,5}`, and all remaining pairs.
pub fn build_two_disjoint_triples_pbd(n: usize) -> Vec<u64> {
    assert!((6..=63).contains(&n));
    pbd_complete(n, &[0b000_111u64, 0b111_000u64])
}

/// Two-intersecting-triples PBD `H_{2,star}(n)` on `n >= 5` vertices: `{0,1,2}`, `{0,3,4}`, and all remaining pairs.
pub fn build_two_intersecting_triples_pbd(n: usize) -> Vec<u64> {
    assert!((5..=63).contains(&n));
    pbd_complete(n, &[0b00111u64, 0b11001u64])
}

/// Fano plane `PG(2, 2) = STS(7)` on `n = 7` vertices (`7` lines of size `3`).
pub fn build_fano_pg2_2() -> Vec<u64> {
    // Cyclic difference set {0, 1, 3} mod 7
    let mut lines = Vec::with_capacity(7);
    for s in 0..7u32 {
        let mut mask = 0u64;
        for &d in &[0u32, 1, 3] {
            mask |= 1u64 << ((s + d) % 7);
        }
        lines.push(mask);
    }
    lines
}

/// Punctured Fano plane `PG(2, 2) \ {6}` on `n = 6` vertices (`4` triples and `3` disjoint pairs).
pub fn build_punctured_fano_n6() -> Vec<u64> {
    let fano = build_fano_pg2_2();
    let mask6 = (1u64 << 6) - 1;
    fano.into_iter().map(|e| e & mask6).collect()
}

/// Affine plane `AG(2, 3) = STS(9)` on `n = 9` vertices (`12` lines of size `3` in `4` parallel classes).
pub fn build_affine_ag2_3() -> Vec<u64> {
    build_ag2_3_ideal_extension(0)
}

/// Projective-to-Affine `AG(2, 3)^{+p}` (`PG(2, 3)` with `4 - p` collinear points on `L_∞` deleted
/// and remaining ideal pairs PBD-completed) on `n = 9 + p` vertices (`0 <= p <= 4`).
/// Satisfies `ω(L(H)) = χ'(H) = 2p + 4 = 2n - 14` for `0 <= p <= 3` (`n ∈ {9, 10, 11, 12}`).
pub fn build_ag2_3_ideal_extension(p: usize) -> Vec<u64> {
    assert!(p <= 4);
    let parallel_classes: &[[[u32; 3]; 3]; 4] = &[
        [[0, 1, 2], [3, 4, 5], [6, 7, 8]],
        [[0, 3, 6], [1, 4, 7], [2, 5, 8]],
        [[0, 4, 8], [1, 5, 6], [2, 3, 7]],
        [[0, 5, 7], [1, 3, 8], [2, 4, 6]],
    ];
    let mut blocks = Vec::with_capacity(13);
    for (c_idx, cls) in parallel_classes.iter().enumerate() {
        for t in cls {
            let mut mask = (1u64 << t[0]) | (1u64 << t[1]) | (1u64 << t[2]);
            if c_idx < p {
                mask |= 1u64 << (9 + c_idx);
            }
            blocks.push(mask);
        }
    }
    if p >= 2 {
        let mut inf_line = 0u64;
        for c_idx in 0..p {
            inf_line |= 1u64 << (9 + c_idx);
        }
        blocks.push(inf_line);
    }
    blocks
}

/// PBD completion of the 12 triples of `AG(2, 3)` embedded into `n` vertices (`9 <= n <= 12`).
pub fn build_ag2_3_pbd_completion(n: usize) -> Vec<u64> {
    assert!((9..=12).contains(&n));
    pbd_complete(n, &build_affine_ag2_3())
}

/// Projective plane `PG(2, 3)` on `n = 13` vertices (`13` lines of size `4`).
/// Generated by the Singer cyclic difference set `{0, 1, 3, 9} mod 13`.
pub fn build_projective_pg2_3() -> Vec<u64> {
    let mut lines = Vec::with_capacity(13);
    for s in 0..13u32 {
        let mut mask = 0u64;
        for &d in &[0u32, 1, 3, 9] {
            mask |= 1u64 << ((s + d) % 13);
        }
        lines.push(mask);
    }
    lines
}

/// Punctured Projective Plane `PG(2, 3) \ {12}` on `n = 12` vertices (`9` lines of size `4` and `4` disjoint triples).
pub fn build_punctured_pg2_3_n12() -> Vec<u64> {
    let pg = build_projective_pg2_3();
    let mask12 = (1u64 << 12) - 1;
    pg.into_iter().map(|e| e & mask12).collect()
}

/// Cyclic Steiner Triple System `STS(13)` on `n = 13` vertices (`26` triples).
/// Generated by the two base blocks `{0, 1, 4}` and `{0, 2, 7} mod 13` (differences `±1, ±3, ±4` and `±2, ±5, ±6`).
pub fn build_cyclic_sts13() -> Vec<u64> {
    let mut blocks = Vec::with_capacity(26);
    for s in 0..13u32 {
        let b1 = (1u64 << s) | (1u64 << ((s + 1) % 13)) | (1u64 << ((s + 4) % 13));
        let b2 = (1u64 << s) | (1u64 << ((s + 2) % 13)) | (1u64 << ((s + 7) % 13));
        blocks.push(b1);
        blocks.push(b2);
    }
    blocks
}

/// Non-Cyclic Steiner Triple System `STS(13)_noncyc` on `n = 13` vertices (`26` triples, `|Aut| = 6`).
/// Obtained from `build_cyclic_sts13()` by switching the Pasch configuration (4-triple trade)
/// `{0,1,4}, {0,2,7}, {1,7,9}, {2,4,9}` to `{0,1,7}, {0,2,4}, {1,4,9}, {2,7,9}` in-place.
pub fn build_noncyclic_sts13() -> Vec<u64> {
    let t1 = (1u64 << 0) | (1u64 << 1) | (1u64 << 4);
    let t2 = (1u64 << 0) | (1u64 << 2) | (1u64 << 7);
    let t3 = (1u64 << 1) | (1u64 << 7) | (1u64 << 9);
    let t4 = (1u64 << 2) | (1u64 << 4) | (1u64 << 9);
    let mut blocks: Vec<u64> = build_cyclic_sts13();
    let mut replaced = 0usize;
    for b in &mut blocks {
        if *b == t1 {
            *b = (1u64 << 0) | (1u64 << 1) | (1u64 << 7);
            replaced += 1;
        } else if *b == t2 {
            *b = (1u64 << 0) | (1u64 << 2) | (1u64 << 4);
            replaced += 1;
        } else if *b == t3 {
            *b = (1u64 << 1) | (1u64 << 4) | (1u64 << 9);
            replaced += 1;
        } else if *b == t4 {
            *b = (1u64 << 2) | (1u64 << 7) | (1u64 << 9);
            replaced += 1;
        }
    }
    assert_eq!(replaced, 4);
    blocks
}

/// 1-Line Broken Projective Plane `PG(2,3)-1line` on `n = 13` vertices
/// (`12` blocks of size `4` and `6` 2-edges = `18` total hyperedges).
pub fn build_broken_pg2_3_n13() -> Vec<u64> {
    let pg = build_projective_pg2_3();
    pbd_complete(13, &pg[1..])
}

/// Punctured Projective Plane + Star Completion `PG(2,3)\{p} + K_{1,12}` on `n = 13` vertices
/// (`9` blocks of size `4`, `4` triples on `{0..11}`, and `12` star 2-edges incident to vertex `12` = `25` hyperedges).
pub fn build_punctured_pg2_3_star_n13() -> Vec<u64> {
    let p_pg = build_punctured_pg2_3_n12();
    pbd_complete(13, &p_pg)
}

/// Affine Plane + Disjoint Quadruple `AG(2,3) + K_4` PBD on `n = 13` vertices
/// (`12` triples on `{0..8}`, one 4-block `{9,10,11,12}`, and `36` cross 2-edges = `49` hyperedges).
pub fn build_ag2_3_plus_k4_n13() -> Vec<u64> {
    let mut blocks = build_affine_ag2_3();
    blocks.push((1u64 << 9) | (1u64 << 10) | (1u64 << 11) | (1u64 << 12));
    pbd_complete(13, &blocks)
}

/// Mixed-Rank `(13, {2, 3, 4, 5}, 1)`-PBD on `n = 13` vertices with `Δ_{>=3}(H) = 3`:
/// one 5-block `{0,1,2,3,4}`, one 4-block `{0,5,6,7}`, and 7 triples (`50` total hyperedges).
pub fn build_pbd_13_345() -> Vec<u64> {
    let blocks: &[&[usize]] = &[
        &[0, 1, 2, 3, 4],
        &[0, 5, 6, 7],
        &[0, 8, 9],
        &[1, 5, 8],
        &[2, 6, 8],
        &[3, 7, 10],
        &[1, 6, 11],
        &[2, 7, 12],
        &[4, 5, 10],
    ];
    let masks: Vec<u64> = blocks
        .iter()
        .map(|b| b.iter().fold(0u64, |acc, &v| acc | (1u64 << v)))
        .collect();
    pbd_complete(13, &masks)
}

/// Complete `n = 13` Projective, Steiner, and Hybrid PBD Certification Suite for Theorem 2.6.
pub fn n13_projective_steiner_pbd_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "PG(2,3) (Projective Plane n=13)".to_string(),
            family: "Projective Plane".to_string(),
            n: 13,
            edges: build_projective_pg2_3(),
            expected_chi_prime: 13,
            mechanism: TightnessMechanism::DeBruijnErdosProjectivePlane,
        },
        EflBenchmarkInstance {
            name: "PG(2,3)-1line (Broken PG(2,3) n=13)".to_string(),
            family: "Broken PG(2,3)".to_string(),
            n: 13,
            edges: build_broken_pg2_3_n13(),
            expected_chi_prime: 12,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,3)\\{p}+K_{1,12} (Star-Blowup n=13)".to_string(),
            family: "Punctured PG(2,3)".to_string(),
            n: 13,
            edges: build_punctured_pg2_3_star_n13(),
            expected_chi_prime: 12,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(13)_cyc (Cyclic STS(13) n=13)".to_string(),
            family: "Steiner Triple System".to_string(),
            n: 13,
            edges: build_cyclic_sts13(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(13)_noncyc (Non-Cyclic STS(13))".to_string(),
            family: "Steiner Triple System".to_string(),
            n: 13,
            edges: build_noncyclic_sts13(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,3)+K_4 (Affine+4-Block n=13)".to_string(),
            family: "Disjoint-Union PBD".to_string(),
            n: 13,
            edges: build_ag2_3_plus_k4_n13(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(13,{3,4,5}) (Mixed Δ_{>=3}=3 n=13)".to_string(),
            family: "Mixed-Rank Δ_{>=3}<=3".to_string(),
            n: 13,
            edges: build_pbd_13_345(),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_13 (Near-Pencil n=13)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 13,
            edges: build_near_pencil(13),
            expected_chi_prime: 13,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
    ]
}

/// Kirkman's 7 parallel classes (each of 5 disjoint triples) partitioning the 35 triples
/// of `STS(15)_Kirkman` on `n = 15` vertices (`Z_7 × {0, 1} ∪ {14}`).
pub fn build_kirkman_sts15_parallel_classes() -> Vec<Vec<u64>> {
    let mut classes = Vec::with_capacity(7);
    for s in 0..7u32 {
        let t0 = (1u64 << s) | (1u64 << (7 + s)) | (1u64 << 14);
        let t1 = (1u64 << ((s + 1) % 7)) | (1u64 << ((s + 2) % 7)) | (1u64 << ((s + 4) % 7));
        let t2 =
            (1u64 << ((s + 3) % 7)) | (1u64 << (7 + ((s + 1) % 7))) | (1u64 << (7 + ((s + 5) % 7)));
        let t3 =
            (1u64 << ((s + 5) % 7)) | (1u64 << (7 + ((s + 4) % 7))) | (1u64 << (7 + ((s + 6) % 7)));
        let t4 =
            (1u64 << ((s + 6) % 7)) | (1u64 << (7 + ((s + 2) % 7))) | (1u64 << (7 + ((s + 3) % 7)));
        classes.push(vec![t0, t1, t2, t3, t4]);
    }
    classes
}

/// Resolvable Kirkman Steiner Triple System `STS(15)_Kirkman` on `n = 15` vertices
/// (`35` triples partitioned into `7` parallel classes of `5` triples each: `ω = 7, χ' = 7`).
pub fn build_kirkman_sts15() -> Vec<u64> {
    build_kirkman_sts15_parallel_classes()
        .into_iter()
        .flatten()
        .collect()
}

/// Non-Resolvable Pasch-Switched Steiner Triple System `STS(15)_nonres` on `n = 15` vertices
/// (`35` triples, obtained from `STS(15)_Kirkman` by switching the Fano-subplane Pasch quadrilateral
/// `{1,2,4}, {2,3,5}, {3,4,6}, {1,5,6}` to `{1,2,5}, {1,4,6}, {2,3,4}, {3,5,6}`, reducing the
/// parallel-class count from `7` to `4` and strictly increasing `χ'` from `7` to `8`).
pub fn build_nonresolvable_sts15() -> Vec<u64> {
    let p1 = (1u64 << 1) | (1u64 << 2) | (1u64 << 4);
    let p2 = (1u64 << 2) | (1u64 << 3) | (1u64 << 5);
    let p3 = (1u64 << 3) | (1u64 << 4) | (1u64 << 6);
    let p4 = (1u64 << 1) | (1u64 << 5) | (1u64 << 6);
    let mut blocks = build_kirkman_sts15();
    let mut replaced = 0usize;
    for b in &mut blocks {
        if *b == p1 {
            *b = (1u64 << 1) | (1u64 << 2) | (1u64 << 5);
            replaced += 1;
        } else if *b == p2 {
            *b = (1u64 << 1) | (1u64 << 4) | (1u64 << 6);
            replaced += 1;
        } else if *b == p3 {
            *b = (1u64 << 2) | (1u64 << 3) | (1u64 << 4);
            replaced += 1;
        } else if *b == p4 {
            *b = (1u64 << 3) | (1u64 << 5) | (1u64 << 6);
            replaced += 1;
        }
    }
    assert_eq!(replaced, 4);
    blocks
}

/// Punctured Resolvable Kirkman `STS(15)_Kirkman \ {14}` on `n = 14` vertices
/// (`28` triples and `7` disjoint 2-edges = `35` hyperedges, `ω = 7, χ' = 7`).
pub fn build_punctured_kirkman_sts15_n14() -> Vec<u64> {
    let mask14 = (1u64 << 14) - 1;
    build_kirkman_sts15()
        .into_iter()
        .map(|e| e & mask14)
        .collect()
}

/// Punctured Non-Resolvable `STS(15)_nonres \ {14}` on `n = 14` vertices
/// (`28` triples and `7` disjoint 2-edges = `35` hyperedges, `ω = 7, χ' = 8`).
pub fn build_punctured_nonres_sts15_n14() -> Vec<u64> {
    let mask14 = (1u64 << 14) - 1;
    build_nonresolvable_sts15()
        .into_iter()
        .map(|e| e & mask14)
        .collect()
}

/// Projective Plane + Apex Star `PG(2,3) + K_{1,13}` on `n = 14` vertices
/// (`13` lines of size `4` on `{0..12}` and `13` 2-edges incident to vertex `13` = `26` hyperedges, `ω = χ' = 13`).
pub fn build_pg2_3_plus_star_n14() -> Vec<u64> {
    pbd_complete(14, &build_projective_pg2_3())
}

/// Hanani Mixed-Rank `PBD(14, {2, 4, 5})` on `n = 14` vertices:
/// `PG(2,3)` with one line extended by vertex `13` to a 5-block, `12` lines of size `4`,
/// and `9` 2-edges (`22` total hyperedges, `ω = χ' = 13`).
pub fn build_hanani_pbd14_245() -> Vec<u64> {
    let mut pg = build_projective_pg2_3();
    pg[0] |= 1u64 << 13;
    pbd_complete(14, &pg)
}

/// Extended Projective Mixed-Rank `PG(2,3)_ext(15, {2, 4, 5})` on `n = 15` vertices:
/// `PG(2,3)` with line `L_0` extended by `13` and line `L_1` extended by `14` (`2` blocks of size `5`,
/// `11` blocks of size `4`, and `19` 2-edges = `32` total hyperedges, `ω = χ' = 13`).
pub fn build_pg2_3_ext_pbd15_245() -> Vec<u64> {
    let mut pg = build_projective_pg2_3();
    pg[0] |= 1u64 << 13;
    pg[1] |= 1u64 << 14;
    pbd_complete(15, &pg)
}

/// Complete `n = 14, 15` Kirkman, Non-Resolvable STS(15), and High-Rank PBD Suite for Theorem 2.7.
pub fn n14_n15_kirkman_pbd_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "STS(15)_Kirkman (Resolvable STS(15))".to_string(),
            family: "Kirkman STS(15)".to_string(),
            n: 15,
            edges: build_kirkman_sts15(),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(15)_nonres (Pasch-Switched STS(15))".to_string(),
            family: "Non-Resolvable STS(15)".to_string(),
            n: 15,
            edges: build_nonresolvable_sts15(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(15)_Kirkman\\{14} (Punctured n=14)".to_string(),
            family: "Punctured STS(15)".to_string(),
            n: 14,
            edges: build_punctured_kirkman_sts15_n14(),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(15)_nonres\\{14} (Punctured n=14)".to_string(),
            family: "Punctured STS(15)".to_string(),
            n: 14,
            edges: build_punctured_nonres_sts15_n14(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,3)+K_{1,13} (Star-Ext PG(2,3) n=14)".to_string(),
            family: "Extended PG(2,3)".to_string(),
            n: 14,
            edges: build_pg2_3_plus_star_n14(),
            expected_chi_prime: 13,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "Hanani_PBD(14,{2,4,5}) (1x5+12x4 n=14)".to_string(),
            family: "Hanani PBD(14)".to_string(),
            n: 14,
            edges: build_hanani_pbd14_245(),
            expected_chi_prime: 13,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,3)_ext(15,{2,4,5}) (2x5+11x4 n=15)".to_string(),
            family: "Extended PG(2,3)".to_string(),
            n: 15,
            edges: build_pg2_3_ext_pbd15_245(),
            expected_chi_prime: 13,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_15 (Near-Pencil n=15)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 15,
            edges: build_near_pencil(15),
            expected_chi_prime: 15,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
    ]
}

/// Finite field `GF(4) = {0, 1, 2, 3}` multiplication (`x^2 + x + 1 = 0`).
#[inline]
fn gf4_mul(a: usize, b: usize) -> usize {
    match (a, b) {
        (0, _) | (_, 0) => 0,
        (1, x) | (x, 1) => x,
        (2, 2) => 3,
        (2, 3) | (3, 2) => 1,
        (3, 3) => 2,
        _ => unreachable!(),
    }
}

/// The `5` parallel classes of the affine plane `AG(2, 4)` on `n = 16` vertices (`GF(4)^2`),
/// each consisting of `4` vertex-disjoint lines of size `4` (`20` lines total).
pub fn build_affine_ag2_4_parallel_classes() -> Vec<Vec<u64>> {
    let mut classes = Vec::with_capacity(5);
    // 4 slope classes y = m * x + c for m in 0..4
    for m in 0..4usize {
        let mut cls = Vec::with_capacity(4);
        for c in 0..4usize {
            let mut mask = 0u64;
            for x in 0..4usize {
                let y = gf4_mul(m, x) ^ c;
                let pt = 4 * x + y;
                mask |= 1u64 << pt;
            }
            cls.push(mask);
        }
        classes.push(cls);
    }
    // 5th parallel class: vertical lines x = c for c in 0..4
    let mut vert_cls = Vec::with_capacity(4);
    for x in 0..4usize {
        let mut mask = 0u64;
        for y in 0..4usize {
            let pt = 4 * x + y;
            mask |= 1u64 << pt;
        }
        vert_cls.push(mask);
    }
    classes.push(vert_cls);
    classes
}

/// Affine plane `AG(2, 4)` (`2-(16, 4, 1)` design) on `n = 16` vertices
/// (`20` lines of size `4` in `5` parallel classes of `4` lines each, `ω = 5, α = 4, χ' = 5`).
pub fn build_affine_ag2_4() -> Vec<u64> {
    build_affine_ag2_4_parallel_classes()
        .into_iter()
        .flatten()
        .collect()
}

/// Projective plane `PG(2, 4)` (`2-(21, 5, 1)` design) on `n = 21` vertices
/// (`21` lines of size `5`, obtained by adjoining `5` ideal points `{16..20}` on `L_∞`
/// to the `5` parallel classes of `AG(2, 4)`).
pub fn build_projective_pg2_4() -> Vec<u64> {
    let classes = build_affine_ag2_4_parallel_classes();
    let mut lines = Vec::with_capacity(21);
    for (j, cls) in classes.into_iter().enumerate() {
        let ideal_bit = 1u64 << (16 + j);
        for line in cls {
            lines.push(line | ideal_bit);
        }
    }
    let l_inf = (16..21).fold(0u64, |acc, v| acc | (1u64 << v));
    lines.push(l_inf);
    lines
}

/// Oval-Punctured Projective Plane `PG(2, 4)\Oval5` on `n = 16` vertices (`PBD(16, {3, 4, 5})`):
/// obtained by deleting the 5-point conic oval `{0, 5, 11, 14, 20}` (no 3 collinear) from `PG(2, 4)`
/// while retaining its nucleus `16`, yielding `6` exterior lines of size `5`, `5` tangent lines of size `4`
/// concurring at the nucleus, and `10` secant lines of size `3` (`21` hyperedges total, `ω = 11, χ' = 11`).
pub fn build_oval_punctured_pg2_4_n16() -> Vec<u64> {
    let deleted = [0usize, 5, 11, 14, 20];
    let kept: Vec<usize> = (0..21).filter(|v| !deleted.contains(v)).collect();
    assert_eq!(kept.len(), 16);
    let mut out = Vec::with_capacity(21);
    for line in build_projective_pg2_4() {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// Punctured Projective Plane `PG(2, 4)_punct16` on `n = 16` vertices (`PBD(16, {3, 4, 5})`):
/// obtained by restricting `PG(2, 4)` to `15` affine points `{0..14}` and `1` ideal point `16`
/// (relabeled as vertex `15`), yielding `3` blocks of size `5`, `13` blocks of size `4`, and
/// `4` blocks of size `3` (`20` hyperedges total, `ω = 8, χ' = 8`).
pub fn build_punctured_pg2_4_n16() -> Vec<u64> {
    let kept: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16];
    let mut out = Vec::new();
    for line in build_projective_pg2_4() {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// Mixed-Rank 2-Ideal-Point Projective Restriction `PBD(16, {2, 3, 4, 5})` on `n = 16` vertices:
/// obtained by restricting `PG(2, 4)` to `14` affine points `{0..13}` and `2` ideal points `{16, 17}`
/// (deleting `{14, 15, 18, 19, 20}`), yielding `4` blocks of size `5`, `11` blocks of size `4`,
/// `4` blocks of size `3`, and `2` 2-edges (`21` hyperedges total, `ω = 9, χ' = 9`).
pub fn build_pbd16_mixed_2345() -> Vec<u64> {
    let deleted = [14usize, 15, 18, 19, 20];
    let kept: Vec<usize> = (0..21).filter(|v| !deleted.contains(v)).collect();
    assert_eq!(kept.len(), 16);
    let mut out = Vec::with_capacity(21);
    for line in build_projective_pg2_4() {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// 1-Ideal-Point Extended Affine Plane `AG(2, 4)^{+1}` on `n = 17` vertices (`PBD(17, {4, 5})`):
/// `4` lines of size `5` (parallel class `P_0` extended by ideal point `16`) and `16` lines of size `4`
/// (`P_1, ..., P_4`), totaling `20` hyperedges (`ω = 8, χ' = 8`).
pub fn build_ag2_4_ideal_ext1_n17() -> Vec<u64> {
    let classes = build_affine_ag2_4_parallel_classes();
    let mut out = Vec::with_capacity(20);
    for (j, cls) in classes.into_iter().enumerate() {
        for line in cls {
            if j == 0 {
                out.push(line | (1u64 << 16));
            } else {
                out.push(line);
            }
        }
    }
    out
}

/// Quadrangle-Punctured Projective Plane `PG(2, 4)_trace17` on `n = 17` vertices (`PBD(17, {3, 4, 5})`):
/// obtained by deleting a 4-point projective frame `{0, 5, 11, 16}` (no 3 collinear) from `PG(2, 4)`,
/// yielding `7` exterior lines of size `5`, `8` tangent lines of size `4`, and `6` secant lines of size `3`
/// (`21` hyperedges total, `ω = 11, χ' = 11`).
pub fn build_pg2_4_quadrangle_trace_n17() -> Vec<u64> {
    let deleted = [0usize, 5, 11, 16];
    let kept: Vec<usize> = (0..21).filter(|v| !deleted.contains(v)).collect();
    assert_eq!(kept.len(), 17);
    let mut out = Vec::with_capacity(21);
    for line in build_projective_pg2_4() {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// Mixed-Rank 3-Ideal-Point Projective Restriction `PBD(17, {2, 3, 4, 5})` on `n = 17` vertices:
/// obtained by restricting `PG(2, 4)` to `14` affine points `{0..13}` and `3` ideal points `{16, 17, 18}`
/// (deleting `{14, 15, 19, 20}`), yielding `6` blocks of size `5`, `11` blocks of size `4`,
/// `3` blocks of size `3`, and `1` 2-edge (`21` hyperedges total, `ω = 10, χ' = 10`).
pub fn build_hanani_pbd17_2345() -> Vec<u64> {
    let deleted = [14usize, 15, 19, 20];
    let kept: Vec<usize> = (0..21).filter(|v| !deleted.contains(v)).collect();
    assert_eq!(kept.len(), 17);
    let mut out = Vec::with_capacity(21);
    for line in build_projective_pg2_4() {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// Complete `n = 16, 17` Affine Plane `AG(2, 4)`, Oval-Punctured `PG(2, 4)`, and Mixed-Rank PBD Suite for Theorem 2.8.
pub fn n16_n17_affine_projective_pbd_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "AG(2,4) (Affine Plane 2-(16,4,1))".to_string(),
            family: "Affine Plane AG(2,4)".to_string(),
            n: 16,
            edges: build_affine_ag2_4(),
            expected_chi_prime: 5,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4)\\Oval5 (Oval-Punctured PG(2,4) n=16)".to_string(),
            family: "Oval-Punctured PG(2,4)".to_string(),
            n: 16,
            edges: build_oval_punctured_pg2_4_n16(),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4)_punct16 (Punctured PG(2,4) n=16)".to_string(),
            family: "Punctured PG(2,4)".to_string(),
            n: 16,
            edges: build_punctured_pg2_4_n16(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(16,{2,3,4,5}) (2-Ideal PG(2,4) n=16)".to_string(),
            family: "Mixed-Rank PBD(16)".to_string(),
            n: 16,
            edges: build_pbd16_mixed_2345(),
            expected_chi_prime: 9,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,4)^{+1} (1-Ideal-Pt AG(2,4) n=17)".to_string(),
            family: "Extended AG(2,4)".to_string(),
            n: 17,
            edges: build_ag2_4_ideal_ext1_n17(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4)_trace17 (Quadrangle-Trace PG(2,4) n=17)".to_string(),
            family: "Punctured PG(2,4)".to_string(),
            n: 17,
            edges: build_pg2_4_quadrangle_trace_n17(),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(17,{2,3,4,5}) (2-Ideal PG(2,4) n=17)".to_string(),
            family: "Mixed-Rank PBD(17)".to_string(),
            n: 17,
            edges: build_hanani_pbd17_2345(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_17 (Near-Pencil n=17)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 17,
            edges: build_near_pencil(17),
            expected_chi_prime: 17,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
    ]
}

/// General `p`-ideal-point extension `AG(2, 4)^{+p}` on `n = 16 + p` vertices for `0 <= p <= 5`:
/// extends the first `p` parallel classes `P_0, ..., P_{p-1}` of `AG(2, 4)` by ideal points
/// `16, ..., 16 + p - 1`, keeps the remaining `5 - p` parallel classes of size `4`, and adds
/// the collinear ideal line `L_∞^{(p)} = {16, ..., 16 + p - 1}` whenever `p >= 2`.
///
/// By Theorem 2.9(a), for `0 <= p <= 4` (`n = 16..20`),
/// `ω(L(AG(2, 4)^{+p})) = χ'(AG(2, 4)^{+p}) = 3p + 5 in {5, 8, 11, 14, 17}`,
/// whereas at `p = 5` (`PG(2, 4)`, `n = 21`), `ω = χ' = 21` (a `+4` projective jump).
pub fn build_ag2_4_ideal_extension(p: usize) -> Vec<u64> {
    assert!(p <= 5);
    let classes = build_affine_ag2_4_parallel_classes();
    let mut out = Vec::with_capacity(21);
    for (j, cls) in classes.into_iter().enumerate() {
        for line in cls {
            if j < p {
                out.push(line | (1u64 << (16 + j)));
            } else {
                out.push(line);
            }
        }
    }
    if p >= 2 {
        let l_inf_p = (16..(16 + p)).fold(0u64, |acc, v| acc | (1u64 << v));
        out.push(l_inf_p);
    }
    out
}

/// Triangle-Punctured Projective Plane `PG(2, 4)\Triangle3` on `n = 18` vertices (`PBD(18, {3, 4, 5})`):
/// obtained by deleting the 3 non-collinear triangle vertices `{0, 5, 16}` from `PG(2, 4)`,
/// yielding `9` exterior lines of size `5`, `9` tangent lines of size `4`, and `3` secant lines of size `3`
/// (`21` hyperedges total, `ω = 12, α = 5, χ' = 12`).
pub fn build_triangle_punctured_pg2_4_n18() -> Vec<u64> {
    let deleted = [0usize, 5, 16];
    let kept: Vec<usize> = (0..21).filter(|v| !deleted.contains(v)).collect();
    assert_eq!(kept.len(), 18);
    let mut out = Vec::with_capacity(21);
    for line in build_projective_pg2_4() {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// Mixed-Rank Projective PBD `PBD(19, {2, 3, 4, 5})` on `n = 19` vertices:
/// obtained from `AG(2, 4)^{+3}` (`n = 19`) by shattering the unextended 4-block `{12, 13, 14, 15}`
/// into the triple `{12, 13, 14}` and the three 2-edges `{12, 15}, {13, 15}, {14, 15}`, yielding
/// `12` blocks of size `5`, `7` blocks of size `4`, `2` blocks of size `3`, and `3` blocks of size `2`
/// (`24` hyperedges total, `ω = 14, α = 5, χ' = 14`).
pub fn build_pbd19_mixed_2345() -> Vec<u64> {
    let target_b4 = (1u64 << 12) | (1u64 << 13) | (1u64 << 14) | (1u64 << 15);
    let mut out = Vec::with_capacity(24);
    let mut found = false;
    for b in build_ag2_4_ideal_extension(3) {
        if b == target_b4 {
            found = true;
            out.push((1u64 << 12) | (1u64 << 13) | (1u64 << 14));
            out.push((1u64 << 12) | (1u64 << 15));
            out.push((1u64 << 13) | (1u64 << 15));
            out.push((1u64 << 14) | (1u64 << 15));
        } else {
            out.push(b);
        }
    }
    assert!(found);
    out
}

/// Mixed-Rank Projective PBD `PBD(20, {2, 3, 4, 5})` on `n = 20` vertices:
/// obtained from `PG(2, 4)\{p} = AG(2, 4)^{+4}` (`n = 20`) by shattering the ideal 4-block
/// `{16, 17, 18, 19}` into the triple `{16, 17, 18}` and the three 2-edges `{16, 19}, {17, 19}, {18, 19}`,
/// yielding `16` blocks of size `5`, `4` blocks of size `4`, `1` block of size `3`, and `3` blocks of size `2`
/// (`24` hyperedges total, `ω = 17, α = 5, χ' = 17`).
pub fn build_pbd20_mixed_2345() -> Vec<u64> {
    let target_b4 = (1u64 << 16) | (1u64 << 17) | (1u64 << 18) | (1u64 << 19);
    let mut out = Vec::with_capacity(24);
    let mut found = false;
    for b in build_ag2_4_ideal_extension(4) {
        if b == target_b4 {
            found = true;
            out.push((1u64 << 16) | (1u64 << 17) | (1u64 << 18));
            out.push((1u64 << 16) | (1u64 << 19));
            out.push((1u64 << 17) | (1u64 << 19));
            out.push((1u64 << 18) | (1u64 << 19));
        } else {
            out.push(b);
        }
    }
    assert!(found);
    out
}

/// Complete `n = 18, 19, 20, 21` Projective Plane `PG(2, 4)`, Punctured `PG(2, 4)` (`AG(2, 4)^{+p}`),
/// and Mixed-Rank PBD Suite for Theorem 2.9.
pub fn n18_to_n21_projective_pbd_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "AG(2,4)^{+2} (2-Ideal-Pt AG(2,4) n=18)".to_string(),
            family: "Extended AG(2,4)".to_string(),
            n: 18,
            edges: build_ag2_4_ideal_extension(2),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4)\\Triangle3 (Triangle-Trace n=18)".to_string(),
            family: "Punctured PG(2,4)".to_string(),
            n: 18,
            edges: build_triangle_punctured_pg2_4_n18(),
            expected_chi_prime: 12,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,4)^{+3} (Doubly-Punctured PG(2,4) n=19)".to_string(),
            family: "Punctured PG(2,4)".to_string(),
            n: 19,
            edges: build_ag2_4_ideal_extension(3),
            expected_chi_prime: 14,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(19,{2,3,4,5}) (Mixed-Rank PG(2,4) n=19)".to_string(),
            family: "Mixed-Rank PBD(19)".to_string(),
            n: 19,
            edges: build_pbd19_mixed_2345(),
            expected_chi_prime: 14,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4)\\{p} (Punctured PG(2,4) n=20)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 20,
            edges: build_ag2_4_ideal_extension(4),
            expected_chi_prime: 17,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(20,{2,3,4,5}) (Mixed-Rank PG(2,4) n=20)".to_string(),
            family: "Mixed-Rank PBD(20)".to_string(),
            n: 20,
            edges: build_pbd20_mixed_2345(),
            expected_chi_prime: 17,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4) (Projective Plane 2-(21,5,1) n=21)".to_string(),
            family: "Projective Plane".to_string(),
            n: 21,
            edges: build_projective_pg2_4(),
            expected_chi_prime: 21,
            mechanism: TightnessMechanism::DeBruijnErdosProjectivePlane,
        },
        EflBenchmarkInstance {
            name: "NP_21 (Near-Pencil n=21)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 21,
            edges: build_near_pencil(21),
            expected_chi_prime: 21,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
    ]
}

/// The `6` parallel classes of the affine plane `AG(2, 5)` on `n = 25` vertices (`GF(5)^2`),
/// each consisting of `5` vertex-disjoint lines of size `5` (`30` lines total).
pub fn build_affine_ag2_5_parallel_classes() -> Vec<Vec<u64>> {
    let mut classes = Vec::with_capacity(6);
    // 5 slope classes y = (m * x + c) mod 5 for m in 0..5
    for m in 0..5usize {
        let mut cls = Vec::with_capacity(5);
        for c in 0..5usize {
            let mut mask = 0u64;
            for x in 0..5usize {
                let y = (m * x + c) % 5;
                let pt = 5 * x + y;
                mask |= 1u64 << pt;
            }
            cls.push(mask);
        }
        classes.push(cls);
    }
    // 6th parallel class: vertical lines x = c for c in 0..5
    let mut vert_cls = Vec::with_capacity(5);
    for x in 0..5usize {
        let mut mask = 0u64;
        for y in 0..5usize {
            let pt = 5 * x + y;
            mask |= 1u64 << pt;
        }
        vert_cls.push(mask);
    }
    classes.push(vert_cls);
    classes
}

/// Affine plane `AG(2, 5)` (`2-(25, 5, 1)` design, `PG(2, 5)\L_∞`) on `n = 25` vertices
/// (`30` lines of size `5` in `6` parallel classes of `5` lines each, `ω = 6, α = 5, χ' = 6`).
pub fn build_affine_ag2_5() -> Vec<u64> {
    build_affine_ag2_5_parallel_classes()
        .into_iter()
        .flatten()
        .collect()
}

/// General `p`-ideal-point extension `AG(2, 5)^{+p}` on `n = 25 + p` vertices for `0 <= p <= 6`:
/// extends the first `p` parallel classes `P_0, ..., P_{p-1}` of `AG(2, 5)` by ideal points
/// `25, ..., 25 + p - 1`, keeps the remaining `6 - p` parallel classes of size `5`, and adds
/// the collinear ideal line `L_∞^{(p)} = {25, ..., 25 + p - 1}` whenever `p >= 2`.
///
/// By Theorem 2.10(b), for `0 <= p <= 5` (`n = 25..30`),
/// `ω(L(AG(2, 5)^{+p})) = χ'(AG(2, 5)^{+p}) = 4p + 6 in {6, 10, 14, 18, 22, 26} < 25 + p`,
/// whereas at `p = 6` (`PG(2, 5)`, `n = 31`), `ω = χ' = 31` (a `+5` projective jump).
pub fn build_ag2_5_ideal_extension(p: usize) -> Vec<u64> {
    assert!(p <= 6);
    let classes = build_affine_ag2_5_parallel_classes();
    let mut out = Vec::with_capacity(31);
    for (j, cls) in classes.into_iter().enumerate() {
        for line in cls {
            if j < p {
                out.push(line | (1u64 << (25 + j)));
            } else {
                out.push(line);
            }
        }
    }
    if p >= 2 {
        let l_inf_p = (25..(25 + p)).fold(0u64, |acc, v| acc | (1u64 << v));
        out.push(l_inf_p);
    }
    out
}

/// Projective plane `PG(2, 5)` (`2-(31, 6, 1)` design) on `n = 31` vertices
/// (`31` lines of size `6`, `ω = 31, α = 1, χ' = 31`).
pub fn build_projective_pg2_5() -> Vec<u64> {
    build_ag2_5_ideal_extension(6)
}

/// Helper to restrict a hypergraph on `0..orig_n` by deleting `deleted` vertices and
/// relabeling the remaining vertices densely into `0..(orig_n - deleted.len())`.
fn restrict_hypergraph_by_deletion(orig_n: usize, base: &[u64], deleted: &[usize]) -> Vec<u64> {
    let kept: Vec<usize> = (0..orig_n).filter(|v| !deleted.contains(v)).collect();
    let mut out = Vec::with_capacity(base.len());
    for &line in base {
        let mut mask = 0u64;
        for (new_idx, &old_v) in kept.iter().enumerate() {
            if ((line >> old_v) & 1) != 0 {
                mask |= 1u64 << new_idx;
            }
        }
        if mask.count_ones() >= 2 {
            out.push(mask);
        }
    }
    out
}

/// 3-Arc-Punctured Affine Plane `AG(2, 5)\Arc3` on `n = 22` vertices (`PBD(22, {3, 4, 5})`):
/// obtained by deleting the 3 non-collinear conic points `{6, 14, 19}` (`(1,1), (2,4), (3,4)`)
/// from `AG(2, 5)`, yielding `12` exterior lines of size `5`, `15` tangent lines of size `4`,
/// and `3` secant lines of size `3` (`30` hyperedges total, `ω = 6, α = 5, χ' = 6`).
pub fn build_ag2_5_punct_arc3_n22() -> Vec<u64> {
    restrict_hypergraph_by_deletion(25, &build_affine_ag2_5(), &[6, 14, 19])
}

/// 2-Arc-Punctured Affine Plane `AG(2, 5)\Arc2` on `n = 23` vertices (`PBD(23, {3, 4, 5})`):
/// obtained by deleting `{6, 14}` from `AG(2, 5)`, yielding `19` exterior lines of size `5`,
/// `10` tangent lines of size `4`, and `1` secant line of size `3` (`30` hyperedges, `ω = 6, χ' = 6`).
pub fn build_ag2_5_punct_arc2_n23() -> Vec<u64> {
    restrict_hypergraph_by_deletion(25, &build_affine_ag2_5(), &[6, 14])
}

/// Singly-Punctured Affine Plane `AG(2, 5)\{p}` on `n = 24` vertices (`PBD(24, {4, 5})`):
/// obtained by deleting vertex `24` (`(4, 4)`) from `AG(2, 5)`, yielding `24` lines of size `5`
/// and `6` lines of size `4` (`30` hyperedges, `ω = 6, χ' = 6`).
pub fn build_ag2_5_punct1_n24() -> Vec<u64> {
    restrict_hypergraph_by_deletion(25, &build_affine_ag2_5(), &[24])
}

/// Mixed-Rank 1-Ideal-Point Restriction `PBD(22, {2, 4, 5, 6})` on `n = 22` vertices:
/// obtained by deleting `{9, 14, 19, 24}` (4 affine points on horizontal line `y = 4`, preserving
/// base point `4 = (0, 4)` and ideal point `25`) from `AG(2, 5)^{+1}` (`n = 26`), yielding
/// `4` blocks of size `6`, `5` blocks of size `5`, `20` blocks of size `4`, and `1` block of size `2`
/// (`30` hyperedges total, `ω = 10, χ' = 10`).
pub fn build_pbd22_mixed_ag2_5_ext1() -> Vec<u64> {
    restrict_hypergraph_by_deletion(26, &build_ag2_5_ideal_extension(1), &[9, 14, 19, 24])
}

/// Mixed-Rank 1-Ideal-Point Restriction `PBD(23, {3, 4, 5, 6})` on `n = 23` vertices:
/// obtained by deleting `{14, 19, 24}` from `AG(2, 5)^{+1}` (`n = 26`), yielding `4` blocks of size `6`,
/// `10` blocks of size `5`, `15` blocks of size `4`, and `1` block of size `3` (`30` hyperedges, `ω = 10, χ' = 10`).
pub fn build_pbd23_mixed_ag2_5_ext1() -> Vec<u64> {
    restrict_hypergraph_by_deletion(26, &build_ag2_5_ideal_extension(1), &[14, 19, 24])
}

/// Mixed-Rank 1-Ideal-Point Restriction `PBD(24, {4, 5, 6})` on `n = 24` vertices:
/// obtained by deleting `{19, 24}` from `AG(2, 5)^{+1}` (`n = 26`), yielding `4` blocks of size `6`,
/// `15` blocks of size `5`, and `11` blocks of size `4` (`30` hyperedges, `ω = 10, χ' = 10`).
pub fn build_pbd24_mixed_ag2_5_ext1() -> Vec<u64> {
    restrict_hypergraph_by_deletion(26, &build_ag2_5_ideal_extension(1), &[19, 24])
}

/// Mixed-Rank 1-Ideal-Point Restriction `PBD(25, {4, 5, 6})` on `n = 25` vertices:
/// obtained by deleting `{24}` from `AG(2, 5)^{+1}` (`n = 26`), yielding `4` blocks of size `6`,
/// `21` blocks of size `5`, and `5` blocks of size `4` (`30` hyperedges, `ω = 10, χ' = 10`).
pub fn build_pbd25_mixed_ag2_5_ext1() -> Vec<u64> {
    restrict_hypergraph_by_deletion(26, &build_ag2_5_ideal_extension(1), &[24])
}

/// Complete `n = 22, 23, 24, 25` Affine Plane `AG(2, 5)` (`PG(2, 5)\L_∞`), Arc-Punctured `AG(2, 5)`,
/// and Mixed-Rank `(3, 4, 5, 6)`-Block PBD Suite for Theorem 2.10.
pub fn n22_to_n25_ag2_5_pbd_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "AG(2,5)\\Arc3 (Arc-Punctured AG(2,5) n=22)".to_string(),
            family: "Punctured AG(2,5)".to_string(),
            n: 22,
            edges: build_ag2_5_punct_arc3_n22(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(22,{2,4,5,6}) (1-Ideal AG(2,5) n=22)".to_string(),
            family: "Mixed-Rank PBD(22)".to_string(),
            n: 22,
            edges: build_pbd22_mixed_ag2_5_ext1(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,5)\\Arc2 (Arc-Punctured AG(2,5) n=23)".to_string(),
            family: "Punctured AG(2,5)".to_string(),
            n: 23,
            edges: build_ag2_5_punct_arc2_n23(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(23,{3,4,5,6}) (1-Ideal AG(2,5) n=23)".to_string(),
            family: "Mixed-Rank PBD(23)".to_string(),
            n: 23,
            edges: build_pbd23_mixed_ag2_5_ext1(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,5)\\{p} (Punctured AG(2,5) n=24)".to_string(),
            family: "Punctured AG(2,5)".to_string(),
            n: 24,
            edges: build_ag2_5_punct1_n24(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(24,{4,5,6}) (1-Ideal AG(2,5) n=24)".to_string(),
            family: "Mixed-Rank PBD(24)".to_string(),
            n: 24,
            edges: build_pbd24_mixed_ag2_5_ext1(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,5) (Affine Plane 2-(25,5,1) n=25)".to_string(),
            family: "Affine Plane AG(2,5)".to_string(),
            n: 25,
            edges: build_affine_ag2_5(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(25,{4,5,6}) (1-Ideal AG(2,5) n=25)".to_string(),
            family: "Mixed-Rank PBD(25)".to_string(),
            n: 25,
            edges: build_pbd25_mixed_ag2_5_ext1(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
    ]
}

/// Punctured Cyclic Steiner Triple System `STS(13) \ {12}` on `n = 12` vertices
/// (`20` triples and `6` disjoint 2-edges = `26` total hyperedges).
pub fn build_punctured_sts13_n12() -> Vec<u64> {
    let mask12 = (1u64 << 12) - 1;
    build_cyclic_sts13()
        .into_iter()
        .map(|e| e & mask12)
        .filter(|&e| e.count_ones() >= 2)
        .collect()
}

/// Doubly-Punctured Cyclic Steiner Triple System `STS(13) \ {11, 12}` on `n = 11` vertices
/// (`15` triples and `10` 2-edges = `25` total hyperedges).
pub fn build_doubly_punctured_sts13_n11() -> Vec<u64> {
    let mask11 = (1u64 << 11) - 1;
    build_cyclic_sts13()
        .into_iter()
        .map(|e| e & mask11)
        .filter(|&e| e.count_ones() >= 2)
        .collect()
}

/// Three-disjoint-triples PBD `H_{3,disj}(n)` on `n >= 9` vertices:
/// `{0,1,2}`, `{3,4,5}`, `{6,7,8}`, and all remaining 2-edges.
pub fn build_three_disjoint_triples_pbd(n: usize) -> Vec<u64> {
    assert!((9..=63).contains(&n));
    pbd_complete(n, &[0b000_000_111u64, 0b000_111_000u64, 0b111_000_000u64])
}

/// Mixed-rank `(11, {2, 3, 4, 5}, 1)`-PBD on `n = 11` vertices with `Δ_{>=3}(H) = 3`:
/// one 5-block `{0,1,2,3,4}`, one 4-block `{0,5,6,7}`, and 7 linear triples with `d_{>=3}(v) <= 3`.
pub fn build_pbd_11_345() -> Vec<u64> {
    let blocks: &[&[usize]] = &[
        &[0, 1, 2, 3, 4],
        &[0, 5, 6, 7],
        &[0, 8, 9],
        &[1, 5, 8],
        &[2, 6, 8],
        &[3, 7, 10],
        &[1, 6, 9],
        &[2, 7, 9],
        &[4, 5, 10],
    ];
    let masks: Vec<u64> = blocks
        .iter()
        .map(|b| b.iter().fold(0u64, |acc, &v| acc | (1u64 << v)))
        .collect();
    pbd_complete(11, &masks)
}

/// High-density `n = 11` and `n = 12` PBD certification testbench for Theorem 2.5.
pub fn n11_n12_high_density_pbd_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "AG(2,3)^{+2} (PG(2,3)\\2pts n=11)".to_string(),
            family: "Punctured PG(2,3)".to_string(),
            n: 11,
            edges: build_ag2_3_ideal_extension(2),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(13)\\{11,12} (Doubly Punctured n=11)".to_string(),
            family: "Punctured STS(13)".to_string(),
            n: 11,
            edges: build_doubly_punctured_sts13_n11(),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PBD(11,{3,4,5}) (Mixed Δ_{>=3}=3 n=11)".to_string(),
            family: "Mixed-Rank Δ_{>=3}<=3".to_string(),
            n: 11,
            edges: build_pbd_11_345(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,3)+2pts (Completed AG(2,3) n=11)".to_string(),
            family: "Completed AG(2,3)".to_string(),
            n: 11,
            edges: build_ag2_3_pbd_completion(11),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "H_{3,disj}(11) (3 Disj Triples n=11)".to_string(),
            family: "Disjoint-Block PBD".to_string(),
            n: 11,
            edges: build_three_disjoint_triples_pbd(11),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,3)\\{p} (Punctured PG(2,3) n=12)".to_string(),
            family: "Punctured PG(2,3)".to_string(),
            n: 12,
            edges: build_punctured_pg2_3_n12(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(13)\\{p} (Punctured STS(13) n=12)".to_string(),
            family: "Punctured STS(13)".to_string(),
            n: 12,
            edges: build_punctured_sts13_n12(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,3)+3pts (Completed AG(2,3) n=12)".to_string(),
            family: "Completed AG(2,3)".to_string(),
            n: 12,
            edges: build_ag2_3_pbd_completion(12),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "H_{3,disj}(12) (3 Disj Triples n=12)".to_string(),
            family: "Disjoint-Block PBD".to_string(),
            n: 12,
            edges: build_three_disjoint_triples_pbd(12),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
    ]
}

/// Canonical suite of benchmark EFL linear hypergraphs across `3 <= n <= 21`.
pub fn canonical_benchmark_suite() -> Vec<EflBenchmarkInstance> {
    vec![
        EflBenchmarkInstance {
            name: "K_3 (Triangle)".to_string(),
            family: "Complete K_n".to_string(),
            n: 3,
            edges: build_complete_graph(3),
            expected_chi_prime: 3,
            mechanism: TightnessMechanism::VizingOddCompleteGraph,
        },
        EflBenchmarkInstance {
            name: "K_4 (Tetrahedron)".to_string(),
            family: "Complete K_n".to_string(),
            n: 4,
            edges: build_complete_graph(4),
            expected_chi_prime: 3,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_4 (Near-Pencil n=4)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 4,
            edges: build_near_pencil(4),
            expected_chi_prime: 4,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
        EflBenchmarkInstance {
            name: "K_5 (Complete n=5)".to_string(),
            family: "Complete K_n".to_string(),
            n: 5,
            edges: build_complete_graph(5),
            expected_chi_prime: 5,
            mechanism: TightnessMechanism::VizingOddCompleteGraph,
        },
        EflBenchmarkInstance {
            name: "H_1(5) (1-Triple PBD n=5)".to_string(),
            family: "Single-Triple PBD".to_string(),
            n: 5,
            edges: build_single_triple_pbd(5),
            expected_chi_prime: 4,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_5 (Near-Pencil n=5)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 5,
            edges: build_near_pencil(5),
            expected_chi_prime: 5,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
        EflBenchmarkInstance {
            name: "K_6 (Complete n=6)".to_string(),
            family: "Complete K_n".to_string(),
            n: 6,
            edges: build_complete_graph(6),
            expected_chi_prime: 5,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,2)\\{p} (Punctured Fano n=6)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 6,
            edges: build_punctured_fano_n6(),
            expected_chi_prime: 5,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_6 (Near-Pencil n=6)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 6,
            edges: build_near_pencil(6),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
        EflBenchmarkInstance {
            name: "PG(2,2) (Fano Plane STS(7))".to_string(),
            family: "Projective Plane".to_string(),
            n: 7,
            edges: build_fano_pg2_2(),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::DeBruijnErdosProjectivePlane,
        },
        EflBenchmarkInstance {
            name: "K_7 (Complete n=7)".to_string(),
            family: "Complete K_n".to_string(),
            n: 7,
            edges: build_complete_graph(7),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::VizingOddCompleteGraph,
        },
        EflBenchmarkInstance {
            name: "H_1(7) (1-Triple PBD n=7)".to_string(),
            family: "Single-Triple PBD".to_string(),
            n: 7,
            edges: build_single_triple_pbd(7),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::CrossEdgeParitySaturation,
        },
        EflBenchmarkInstance {
            name: "NP_7 (Near-Pencil n=7)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 7,
            edges: build_near_pencil(7),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
        EflBenchmarkInstance {
            name: "K_8 (Complete n=8)".to_string(),
            family: "Complete K_n".to_string(),
            n: 8,
            edges: build_complete_graph(8),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_8 (Near-Pencil n=8)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 8,
            edges: build_near_pencil(8),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
        EflBenchmarkInstance {
            name: "AG(2,3) (Affine Plane STS(9))".to_string(),
            family: "Affine Plane / STS".to_string(),
            n: 9,
            edges: build_affine_ag2_3(),
            expected_chi_prime: 4,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "K_9 (Complete n=9)".to_string(),
            family: "Complete K_n".to_string(),
            n: 9,
            edges: build_complete_graph(9),
            expected_chi_prime: 9,
            mechanism: TightnessMechanism::VizingOddCompleteGraph,
        },
        EflBenchmarkInstance {
            name: "H_1(9) (1-Triple PBD n=9)".to_string(),
            family: "Single-Triple PBD".to_string(),
            n: 9,
            edges: build_single_triple_pbd(9),
            expected_chi_prime: 9,
            mechanism: TightnessMechanism::CrossEdgeParitySaturation,
        },
        EflBenchmarkInstance {
            name: "H_{2,disj}(7) (2 Disj Triples n=7)".to_string(),
            family: "Two-Triple PBD".to_string(),
            n: 7,
            edges: build_two_disjoint_triples_pbd(7),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::CrossEdgeParitySaturation,
        },
        EflBenchmarkInstance {
            name: "H_{2,star}(7) (2 Star Triples n=7)".to_string(),
            family: "Two-Triple PBD".to_string(),
            n: 7,
            edges: build_two_intersecting_triples_pbd(7),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "H_{2,disj}(9) (2 Disj Triples n=9)".to_string(),
            family: "Two-Triple PBD".to_string(),
            n: 9,
            edges: build_two_disjoint_triples_pbd(9),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "NP_9 (Near-Pencil n=9)".to_string(),
            family: "Near-Pencil NP_n".to_string(),
            n: 9,
            edges: build_near_pencil(9),
            expected_chi_prime: 9,
            mechanism: TightnessMechanism::DeBruijnErdosNearPencil,
        },
        EflBenchmarkInstance {
            name: "AG(2,3)^{+1} (PG(2,3)\\3pts n=10)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 10,
            edges: build_ag2_3_ideal_extension(1),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,3)^{+2} (PG(2,3)\\2pts n=11)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 11,
            edges: build_ag2_3_ideal_extension(2),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "H_{2,disj}(11) (2 Disj Triples n=11)".to_string(),
            family: "Two-Triple PBD".to_string(),
            n: 11,
            edges: build_two_disjoint_triples_pbd(11),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::CrossEdgeParitySaturation,
        },
        EflBenchmarkInstance {
            name: "PG(2,3)\\{p} (Punctured PG(2,3) n=12)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 12,
            edges: build_punctured_pg2_3_n12(),
            expected_chi_prime: 10,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,3) (Projective Plane n=13)".to_string(),
            family: "Projective Plane".to_string(),
            n: 13,
            edges: build_projective_pg2_3(),
            expected_chi_prime: 13,
            mechanism: TightnessMechanism::DeBruijnErdosProjectivePlane,
        },
        EflBenchmarkInstance {
            name: "STS(13) (Cyclic Steiner Triple System)".to_string(),
            family: "Steiner Triple System".to_string(),
            n: 13,
            edges: build_cyclic_sts13(),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "STS(15)_Kirkman (Resolvable STS(15))".to_string(),
            family: "Steiner Triple System".to_string(),
            n: 15,
            edges: build_kirkman_sts15(),
            expected_chi_prime: 7,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,4) (Affine Plane 2-(16,4,1))".to_string(),
            family: "Affine Plane AG(2,q)".to_string(),
            n: 16,
            edges: build_affine_ag2_4(),
            expected_chi_prime: 5,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,4)^{+1} (PG(2,4)\\4pts n=17)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 17,
            edges: build_ag2_4_ideal_extension(1),
            expected_chi_prime: 8,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,4)^{+2} (PG(2,4)\\3pts n=18)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 18,
            edges: build_ag2_4_ideal_extension(2),
            expected_chi_prime: 11,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,4)^{+3} (PG(2,4)\\2pts n=19)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 19,
            edges: build_ag2_4_ideal_extension(3),
            expected_chi_prime: 14,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4)\\{p} (Punctured PG(2,4) n=20)".to_string(),
            family: "Punctured PG(2,q)".to_string(),
            n: 20,
            edges: build_ag2_4_ideal_extension(4),
            expected_chi_prime: 17,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "PG(2,4) (Projective Plane n=21)".to_string(),
            family: "Projective Plane".to_string(),
            n: 21,
            edges: build_projective_pg2_4(),
            expected_chi_prime: 21,
            mechanism: TightnessMechanism::DeBruijnErdosProjectivePlane,
        },
        EflBenchmarkInstance {
            name: "AG(2,5)\\Arc3 (Punctured AG(2,5) n=22)".to_string(),
            family: "Punctured AG(2,q)".to_string(),
            n: 22,
            edges: build_ag2_5_punct_arc3_n22(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,5)\\Arc2 (Punctured AG(2,5) n=23)".to_string(),
            family: "Punctured AG(2,q)".to_string(),
            n: 23,
            edges: build_ag2_5_punct_arc2_n23(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,5)\\{p} (Punctured AG(2,5) n=24)".to_string(),
            family: "Punctured AG(2,q)".to_string(),
            n: 24,
            edges: build_ag2_5_punct1_n24(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
        EflBenchmarkInstance {
            name: "AG(2,5) (Affine Plane 2-(25,5,1) n=25)".to_string(),
            family: "Affine Plane AG(2,q)".to_string(),
            n: 25,
            edges: build_affine_ag2_5(),
            expected_chi_prime: 6,
            mechanism: TightnessMechanism::SubExtremalSlack,
        },
    ]
}
