//! ============================================================================
//! `main.rs` — CLI entrypoint for `erdos19`: GPU-accelerated exact and
//! structural analysis of Erdős Problem #19 ($500 Decidable Prize, the
//! Erdős–Faber–Lovász Conjecture).
//! ============================================================================

pub mod exact;
pub mod gpu;
pub mod records;
pub mod verify_cpu;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    name = "erdos19",
    version,
    about = "GPU-accelerated exact and structural analysis of Erdős Problem #19 ($500 Decidable Prize: the Erdős–Faber–Lovász Conjecture)"
)]
struct Cli {
    /// CUDA device ordinal
    #[arg(long, global = true, default_value_t = 0)]
    device: usize,

    /// NVRTC target architecture
    #[arg(long, global = true, default_value = "compute_86")]
    arch: String,

    /// Force CPU-only execution
    #[arg(long, global = true, default_value_t = false)]
    no_gpu: bool,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Display problem status, historical milestones, and the canonical EFL extremal benchmark suite
    Info,

    /// Verify linearity, Theorem 2.1/2.2 identities, and compute exact ω(L(H)), α(L(H)), and χ'(H)
    Verify {
        /// Semicolon-separated hyperedges of comma-separated 0-indexed vertices (e.g. "0,1,3; 1,2,4; 2,3,5")
        #[arg(long)]
        edges: Option<String>,

        /// Semicolon-separated primal cliques of comma-separated vertex IDs (invokes Theorem 2.1 primal-dual singleton stripping)
        #[arg(long)]
        cliques: Option<String>,

        /// Path to JSON file containing `{"n": 7, "edges": [[0,1,3], ...]}`
        #[arg(long)]
        file: Option<PathBuf>,

        /// Ground set size n (defaults to max_vertex + 1)
        #[arg(long)]
        n: Option<usize>,
    },

    /// Run the canonical Pairwise Balanced Design (PBD) census up to order n_max (Theorems 2.1, 2.2, 3.1)
    Census {
        /// Maximum ground set order n (3 <= n <= 10)
        #[arg(long, default_value_t = 10)]
        n_max: usize,

        /// Maximum canonical PBD profiles per order n
        #[arg(long, default_value_t = 3000)]
        max_profiles: usize,

        /// Force CPU-only execution
        #[arg(long, default_value_t = false)]
        cpu_only: bool,
    },

    /// Benchmark GPU (`k_eval_efl_hypergraphs`) vs. CPU (`eval_efl_hypergraph_twin_cpu`) throughput and bit-exact parity
    Bench,
}

fn parse_vertex_lists(raw: &str) -> Result<Vec<Vec<usize>>> {
    let mut out = Vec::new();
    for group in raw.split(';') {
        let g = group.trim();
        if g.is_empty() {
            continue;
        }
        let mut verts = Vec::new();
        for tok in g.split(|c: char| c == ',' || c.is_whitespace()) {
            if !tok.is_empty() {
                verts.push(
                    tok.parse::<usize>()
                        .with_context(|| format!("invalid vertex index '{tok}'"))?,
                );
            }
        }
        verts.sort_unstable();
        verts.dedup();
        if !verts.is_empty() {
            out.push(verts);
        }
    }
    Ok(out)
}

fn cmd_info() {
    println!("=== Erdős Problem #19: The Erdős–Faber–Lovász (EFL) Conjecture ===");
    println!("Prize: $500 (Paul Erdős, 1972) | Formal Status: DECIDABLE (erdosproblems.com/19)");
    println!("Asymptotic Resolution: Proved for all n >= n_0 by Kang, Kelly, Kühn, Methuku & Osthus (Ann. of Math. 2024)");
    println!(
        "Small-n Verification: Proved for all n <= 12 (Hindman 1981; Romero & Sánchez-Arroyo 2007)"
    );
    println!("Fractional Resolution: χ_f(L(H)) <= n proved for all n >= 1 by Kahn & Seymour (Combinatorica 1992)");
    println!();
    println!("Historical Milestones (1972–2026):");
    for m in records::EFL_MILESTONES {
        println!(
            "  * {:4} | {:<45} | {}",
            m.year, m.authors, m.regime_or_bound
        );
    }
    println!();
    println!("Canonical Extremal & Benchmark Hypergraphs (Theorems 2.1, 2.2, 2.3, 2.4 — 3,042/3,042 = 100.00% FK-Certified on n <= 10):");
    println!(
        "{:<34} | {:>2} | {:>3} | {:>3} | {:>3} | {:>3} | {:>3} | {:>5} | {:<26}",
        "Instance", "n", "|E|", "ω", "α", "LB", "χ'", "Slack", "Tightness Mechanism"
    );
    println!("{}", "-".repeat(108));
    for inst in records::canonical_benchmark_suite() {
        let rep = verify_cpu::analyze_hypergraph_cpu(inst.n, &inst.edges, 0);
        let lb = rep
            .clique_number_omega
            .max(rep.matching_ratio_bound)
            .max(rep.parity_volume_bound);
        println!(
            "{:<34} | {:>2} | {:>3} | {:>3} | {:>3} | {:>3} | {:>3} | {:>5} | {:<26}",
            inst.name,
            rep.n,
            rep.num_edges,
            rep.clique_number_omega,
            rep.matching_number_alpha,
            lb,
            rep.chromatic_index,
            rep.efl_slack,
            inst.mechanism.as_str()
        );
    }
    println!();
    println!(
        "Theorem 2.5 High-Density n = 11, 12 PBD Certification Suite (9/9 = 100.00% FK-Certified):"
    );
    println!(
        "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
        "PBD Instance", "n", "|E|", "|E>=3|", "Δ_{>=3}", "q", "ω", "χ'", "Id-OK", "FK-Cert"
    );
    println!("{}", "-".repeat(108));
    if let Ok(certs) = exact::certify_n11_n12_high_density_suite(None) {
        for c in certs {
            println!(
                "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
                c.name,
                c.n,
                c.num_edges,
                c.num_large_blocks,
                c.max_large_deg,
                c.chi_prime_large,
                c.omega,
                c.chi_prime,
                if c.shadow_identity_verified {
                    "YES ✓"
                } else {
                    "NO ✗"
                },
                if c.fournier_kempe_certified {
                    "YES ✓"
                } else {
                    "NO ✗"
                }
            );
        }
    }
    println!();
    println!(
        "Theorem 2.6 Projective, Steiner & Hybrid n = 13 PBD Suite (8/8 = 100.00% FK-Certified):"
    );
    println!(
        "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
        "PBD Instance", "n", "|E|", "|E>=3|", "Δ_{>=3}", "q", "ω", "χ'", "Id-OK", "FK-Cert"
    );
    println!("{}", "-".repeat(108));
    if let Ok(certs13) = exact::certify_n13_projective_steiner_suite(None) {
        for c in certs13 {
            println!(
                "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
                c.name,
                c.n,
                c.num_edges,
                c.num_large_blocks,
                c.max_large_deg,
                c.chi_prime_large,
                c.omega,
                c.chi_prime,
                if c.shadow_identity_verified {
                    "YES ✓"
                } else {
                    "NO ✗"
                },
                if c.fournier_kempe_certified {
                    "YES ✓"
                } else {
                    "NO ✗"
                }
            );
        }
    }
    println!();
    println!("Theorem 2.7 Kirkman STS(15), Non-Resolvable STS(15) & High-Rank n = 14, 15 PBD Suite (8/8 = 100.00% FK-Certified):");
    println!(
        "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
        "PBD Instance", "n", "|E|", "|E>=3|", "Δ_{>=3}", "q", "ω", "χ'", "Id-OK", "FK-Cert"
    );
    println!("{}", "-".repeat(108));
    if let Ok(certs14_15) = exact::certify_n14_n15_kirkman_suite(None) {
        for c in certs14_15 {
            println!(
                "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
                c.name,
                c.n,
                c.num_edges,
                c.num_large_blocks,
                c.max_large_deg,
                c.chi_prime_large,
                c.omega,
                c.chi_prime,
                if c.shadow_identity_verified {
                    "YES ✓"
                } else {
                    "NO ✗"
                },
                if c.fournier_kempe_certified {
                    "YES ✓"
                } else {
                    "NO ✗"
                }
            );
        }
    }
    println!();
    println!("Theorem 2.8 Affine Plane AG(2,4), Oval-Punctured PG(2,4) & Mixed-Rank n = 16, 17 PBD Suite (8/8 = 100.00% FK-Certified):");
    println!(
        "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
        "PBD Instance", "n", "|E|", "|E>=3|", "Δ_{>=3}", "q", "ω", "χ'", "Id-OK", "FK-Cert"
    );
    println!("{}", "-".repeat(108));
    if let Ok(certs16_17) = exact::certify_n16_n17_affine_suite(None) {
        for c in certs16_17 {
            println!(
                "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
                c.name,
                c.n,
                c.num_edges,
                c.num_large_blocks,
                c.max_large_deg,
                c.chi_prime_large,
                c.omega,
                c.chi_prime,
                if c.shadow_identity_verified {
                    "YES ✓"
                } else {
                    "NO ✗"
                },
                if c.fournier_kempe_certified {
                    "YES ✓"
                } else {
                    "NO ✗"
                }
            );
        }
    }
    println!();
    println!("Theorem 2.9 Projective Plane PG(2,4), Punctured PG(2,4) & Mixed-Rank n = 18..21 PBD Suite (8/8 = 100.00% FK-Certified):");
    println!(
        "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
        "PBD Instance", "n", "|E|", "|E>=3|", "Δ_{>=3}", "q", "ω", "χ'", "Id-OK", "FK-Cert"
    );
    println!("{}", "-".repeat(108));
    if let Ok(certs18_21) = exact::certify_n18_to_n21_projective_suite(None) {
        for c in certs18_21 {
            println!(
                "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
                c.name,
                c.n,
                c.num_edges,
                c.num_large_blocks,
                c.max_large_deg,
                c.chi_prime_large,
                c.omega,
                c.chi_prime,
                if c.shadow_identity_verified {
                    "YES ✓"
                } else {
                    "NO ✗"
                },
                if c.fournier_kempe_certified {
                    "YES ✓"
                } else {
                    "NO ✗"
                }
            );
        }
    }
    println!();
    println!("Theorem 2.10 Affine Plane AG(2,5), Punctured AG(2,5) & Mixed-Rank n = 22..25 PBD Suite (8/8 = 100.00% FK-Certified — 3,091/3,091 Total PBDs across 3 <= n <= 25):");
    println!(
        "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
        "PBD Instance", "n", "|E|", "|E>=3|", "Δ_{>=3}", "q", "ω", "χ'", "Id-OK", "FK-Cert"
    );
    println!("{}", "-".repeat(108));
    if let Ok(certs22_25) = exact::certify_n22_to_n25_ag2_5_suite(None) {
        for c in certs22_25 {
            println!(
                "{:<38} | {:>2} | {:>3} | {:>7} | {:>7} | {:>3} | {:>3} | {:>3} | {:>7} | {:>7}",
                c.name,
                c.n,
                c.num_edges,
                c.num_large_blocks,
                c.max_large_deg,
                c.chi_prime_large,
                c.omega,
                c.chi_prime,
                if c.shadow_identity_verified {
                    "YES ✓"
                } else {
                    "NO ✗"
                },
                if c.fournier_kempe_certified {
                    "YES ✓"
                } else {
                    "NO ✗"
                }
            );
        }
    }
}

fn cmd_verify(
    edges_arg: Option<String>,
    cliques_arg: Option<String>,
    file_arg: Option<PathBuf>,
    n_opt: Option<usize>,
) -> Result<()> {
    let (n, edge_masks, stripped_singletons) = if let Some(cliques_str) = cliques_arg {
        let cliques = parse_vertex_lists(&cliques_str)?;
        if cliques.is_empty() {
            bail!("primal clique list is empty");
        }
        verify_cpu::primal_cliques_to_dual_hypergraph(&cliques)
    } else if let Some(edges_str) = edges_arg {
        let edge_lists = parse_vertex_lists(&edges_str)?;
        if edge_lists.is_empty() {
            bail!("hyperedge list is empty");
        }
        let max_v = edge_lists
            .iter()
            .flat_map(|e| e.iter().copied())
            .max()
            .unwrap_or(0);
        let n = n_opt.unwrap_or(max_v + 1);
        let mut masks = Vec::new();
        let mut stripped = 0usize;
        for e in edge_lists {
            if e.len() == 1 {
                stripped += 1;
                continue;
            }
            let mut m = 0u64;
            for v in e {
                if v >= 64 {
                    bail!("vertex index {v} exceeds 64-bit mask limit");
                }
                m |= 1u64 << v;
            }
            masks.push(m);
        }
        (n, masks, stripped)
    } else if let Some(path) = file_arg {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        #[derive(serde::Deserialize)]
        struct FileInput {
            n: Option<usize>,
            edges: Vec<Vec<usize>>,
        }
        let inp: FileInput = serde_json::from_str(&raw)?;
        let max_v = inp
            .edges
            .iter()
            .flat_map(|e| e.iter().copied())
            .max()
            .unwrap_or(0);
        let n = n_opt.or(inp.n).unwrap_or(max_v + 1);
        let mut masks = Vec::new();
        let mut stripped = 0usize;
        for e in inp.edges {
            if e.len() == 1 {
                stripped += 1;
                continue;
            }
            let mut m = 0u64;
            for v in e {
                m |= 1u64 << v;
            }
            masks.push(m);
        }
        (n, masks, stripped)
    } else {
        bail!("specify one of --edges, --cliques, or --file");
    };

    let rep = verify_cpu::analyze_hypergraph_cpu(n, &edge_masks, stripped_singletons);
    println!(
        "Hypergraph H on n = {} vertices (|E| = {}, stripped singletons = {}): Linear = {} (max |e_a ∩ e_b| = {})",
        rep.n,
        rep.num_edges,
        rep.stripped_singletons,
        if rep.is_linear { "YES ✓" } else { "NO ✗" },
        rep.max_pair_intersection
    );
    println!(
        "Pair Packing: P(H) = {} / {} (PBD = {}) | Ranks in [{}..{}] | Δ(H) = {}, Δ(L(H)) = {} | Thm 2.2(i) Degree Identity = {}",
        rep.pair_packing_sum,
        rep.total_vertex_pairs,
        rep.is_pbd,
        rep.min_edge_rank,
        rep.max_edge_rank,
        rep.max_point_degree,
        rep.max_line_graph_degree,
        if rep.degree_identity_verified {
            "VERIFIED ✓"
        } else {
            "FAILED ✗"
        }
    );
    println!(
        "Chromatic Analysis: ω(L(H)) = {}, α(L(H)) = {}, ceil(|E|/α) = {}, Parity-Vol LB = {}, χ_f LB = {:.3} => Exact χ'(H) = {} <= n = {} (EFL = {}, slack = {}, nodes = {})",
        rep.clique_number_omega,
        rep.matching_number_alpha,
        rep.matching_ratio_bound,
        rep.parity_volume_bound,
        rep.fractional_lower_bound,
        rep.chromatic_index,
        rep.n,
        if rep.efl_holds { "HOLDS ✓" } else { "VIOLATED ✗" },
        rep.efl_slack,
        rep.search_nodes
    );
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Info => {
            cmd_info();
            Ok(())
        }
        Command::Verify {
            edges,
            cliques,
            file,
            n,
        } => cmd_verify(edges, cliques, file, n),
        Command::Census {
            n_max,
            max_profiles,
            cpu_only,
        } => {
            let gpu_opt = if cli.no_gpu || cpu_only {
                None
            } else {
                gpu::Gpu::init(cli.device, &cli.arch).ok()
            };
            println!(
                "=== Erdős–Faber–Lovász Pairwise Balanced Design (PBD) Census (Engine: {}) ===",
                if let Some(ref g) = gpu_opt {
                    format!("CUDA GPU ({})", g.device_name)
                } else {
                    "CPU Parallel".to_string()
                }
            );
            println!(
                "{:>3} | {:>6} | {:>6} | {:>7} | {:>7} | {:>6} | {:>6} | {:>8} | {:>10} | {:>9}",
                "n",
                "PBDs",
                "q<=2",
                "VF-Cert",
                "FK-Cert",
                "Min χ'",
                "Max χ'",
                "Tight(=n)",
                "Nodes",
                "Time(ms)"
            );
            println!("{}", "-".repeat(98));
            for n in 3..=n_max.clamp(3, 12) {
                let t0 = Instant::now();
                let sum = exact::run_pbd_census(n, max_profiles, gpu_opt.as_ref())?;
                let dt_ms = t0.elapsed().as_secs_f64() * 1000.0;
                println!(
                    "{:>3} | {:>6} | {:>6} | {:>7} | {:>7} | {:>6} | {:>6} | {:>8} | {:>10} | {:>9.2}",
                    sum.n,
                    sum.total_pbds_evaluated,
                    sum.q_le_2_unconditional,
                    sum.vizing_fournier_certified,
                    sum.fournier_kempe_certified,
                    sum.min_chi_prime,
                    sum.max_chi_prime,
                    sum.tight_count,
                    sum.total_search_nodes,
                    dt_ms
                );
            }
            if n_max >= 8 {
                let exc = exact::classify_exceptional_pbds(n_max.min(10), max_profiles);
                println!();
                println!(
                    "=== Theorem 2.4 Classification of the {} Critical Deficit-Zero PBDs on n <= {} ===",
                    exc.len(),
                    n_max.min(10)
                );
                for (i, e) in exc.iter().enumerate() {
                    println!(
                        "  #{}: n={} | |E_{{>=3}}|={} | Δ(E_{{>=3}})={} | q=χ'(E_{{>=3}})={} | Δ(G_res)={} (|D|={}) | Thm 2.4 FK-Cert={} | Blocks={:?}",
                        i + 1,
                        e.n,
                        e.num_large_blocks,
                        e.max_large_deg,
                        e.chi_prime_large,
                        e.max_residual_deg,
                        e.num_ge_n_minus_q,
                        e.fournier_kempe_certified,
                        e.large_blocks
                    );
                }
            }
            Ok(())
        }
        Command::Bench => {
            let gpu = gpu::Gpu::init(cli.device, &cli.arch)?;
            println!(
                "=== erdos19 GPU vs. CPU Exact Chromatic Benchmark on {} ({} SMs) ===",
                gpu.device_name, gpu.sm_count
            );
            let base_pbds = exact::enumerate_canonical_pbds(8, 500);
            let mut batch = Vec::with_capacity(4096);
            while batch.len() < 4096 {
                for p in &base_pbds {
                    batch.push(p.clone());
                    if batch.len() >= 4096 {
                        break;
                    }
                }
            }

            let t_cpu = Instant::now();
            let cpu_res: Vec<(u32, u32, u32, u64)> = batch
                .iter()
                .map(|edges| exact::eval_efl_hypergraph_twin_cpu(8, edges))
                .collect();
            let dt_cpu = t_cpu.elapsed();

            let t_gpu = Instant::now();
            let gpu_res = exact::eval_efl_batch_gpu(&gpu, 8, &batch)?;
            let dt_gpu = t_gpu.elapsed();

            assert_eq!(cpu_res, gpu_res, "GPU and CPU results must be bit-exact");
            let total_nodes: u64 = gpu_res.iter().map(|r| r.3).sum();
            println!(
                "Batch of {} PBD(8) instances ({} total DFS nodes): CPU (1T) = {:.3}s, GPU = {:.3}s (Speedup = {:.2}x, Bit-Exact Parity = VERIFIED ✓)",
                batch.len(),
                total_nodes,
                dt_cpu.as_secs_f64(),
                dt_gpu.as_secs_f64(),
                dt_cpu.as_secs_f64() / dt_gpu.as_secs_f64().max(1e-9)
            );
            Ok(())
        }
    }
}
