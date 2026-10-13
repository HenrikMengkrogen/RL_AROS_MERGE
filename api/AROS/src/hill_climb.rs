use rand::RngExt;
use rand::seq::IndexedRandom;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::config::{RIBOSOMAL_RNA, USE_ML_MODEL};
use crate::folding::{bp_distance_to_target, compute_pf_defect, score_candidate};
use crate::mutation::{mutate_ribosome, mutate_seq};
use crate::pseudoknot::pk_pair_mismatches;
use crate::structure::{
    expand_positions_near_mismatches, find_hard_loops, find_mismatched_positions, get_pair_map,
    reduce_positions_by_mismatch_proximity,
};
use crate::types::DesignResult;

pub fn hill_climb_design(
    seq_in: &str,
    target_structure: &str,
    n_positions: Vec<usize>,
    max_steps: i64,
    wobble_frequency: f64,
    n_keep: usize,
    initial_temp: f64,
    cooling_rate: f64,
    reheat_after: i64,
    reheat_temp: f64,
    if_slices: bool,
    last_global: bool,
    seed_seq: &str,
) -> Vec<DesignResult> {
    let pair_map = get_pair_map(target_structure);

    let slice_len = target_structure.len();
    let n_designable = n_positions.len();

    let mut n_positions = n_positions;
    let original_designable_positions: HashSet<usize> = n_positions.iter().copied().collect();

    let mut designable_positions: HashSet<usize> = n_positions.iter().copied().collect();

    let n_fixed = slice_len.saturating_sub(n_designable);
    let fixed_fraction = n_fixed as f64 / slice_len.max(1) as f64;

    // let dist_threshold: i64 = ((n_fixed as f64) / 12.0).ceil() as i64;

    let dist_threshold: i64 = if RIBOSOMAL_RNA {
        ((n_fixed as f64) / 6.0).ceil() as i64
    } else if if_slices {
        0
    } else {
        0
    };

    let hard_loops = find_hard_loops(&target_structure);

    let dist_threshold = if if_slices {
        ((n_fixed as f64) / 12.0).ceil() as i64
    } else {
        dist_threshold.max(0)
    };

    let p_threshold: f64 = if fixed_fraction > 0.6 && !last_global && if_slices {
        0.25
    } else if hard_loops && !last_global {
        0.25
    } else {
        if !last_global { 0.20 } else { 0.10 }
    };

    println!(
        "  [hill_climb] slice_len={} designable={} fixed={} ({:.0}% fixed) → dist_threshold={}, p_threshold={:.2}",
        slice_len,
        n_designable,
        n_fixed,
        fixed_fraction * 100.0,
        dist_threshold,
        p_threshold,
    );

    let nucleotides = vec!['A', 'U', 'G', 'C'];
    let pair_options: Vec<(char, char)> = vec![
        ('A', 'U'),
        ('U', 'A'),
        ('G', 'C'),
        ('C', 'G'),
        ('G', 'U'),
        ('U', 'G'),
    ];
    let seq_in_chars: Vec<char> = seq_in.chars().collect();
    let comp_dict = HashMap::from([('A', 'U'), ('U', 'A'), ('G', 'C'), ('C', 'G')]);

    let mut current = if RIBOSOMAL_RNA {
        mutate_ribosome(seq_in, target_structure, &n_positions)
    } else {
        mutate_seq(
            seq_in,
            target_structure,
            &n_positions,
            wobble_frequency,
            last_global,
        )
    };
    let mut rng = rand::rng();
    if rng.random::<f64>() <= 0.5 && if_slices && USE_ML_MODEL {
        current = seed_seq.to_string();
    }

    //let mut current = mutate_seq(seq_in, target_structure, wobble_frequency);

    let mut current_chars: Vec<char> = current.chars().collect();

    let mut stem_pairs: Vec<(usize, usize)> = n_positions
        .iter()
        .filter_map(|&pos| {
            pair_map.get(&pos).map(|&partner| {
                if pos < partner {
                    (pos, partner)
                } else {
                    (partner, pos)
                }
            })
        })
        .collect();
    stem_pairs.sort_by_key(|&(i, _)| i);
    stem_pairs.dedup();

    for &(pos, partner) in &stem_pairs {
        if seq_in_chars[pos] != 'N' && seq_in_chars[pos] != 'K' && seq_in_chars[pos] != 'S' {
            continue;
        }

        if pos == 0 || partner + 1 >= seq_in_chars.len() {
            continue;
        }
        let outer_5 = pos - 1;
        let outer_3 = partner + 1;

        if pair_map.get(&outer_5) != Some(&outer_3) {
            continue;
        }

        let outer_5_nuc = seq_in_chars[outer_5];
        let outer_3_nuc = seq_in_chars[outer_3];

        if outer_5_nuc == 'N' || outer_3_nuc == 'N' {
            continue;
        }

        let is_weak = matches!(
            (outer_5_nuc, outer_3_nuc),
            ('G', 'U') | ('U', 'G') | ('A', 'U') | ('U', 'A')
        );

        if is_weak {
            current_chars[pos] = 'G';
            current_chars[partner] = 'C';
            const VERBOSE: bool = false;

            if VERBOSE {
                println!(
                    "  Option B: forcing GC at ({}, {}) — outer pair {}{} is weak",
                    pos, partner, outer_5_nuc, outer_3_nuc,
                );
            }
        }
    }
    current = current_chars.into_iter().collect();

    let initial_score = score_candidate(&current, target_structure, &pair_map);

    let mut current_dist = initial_score.total_distance;
    let mut current_structure = initial_score.structure;
    let mut current_mfe = initial_score.mfe;
    let mut current_energy_gap = initial_score.energy_gap;

    let mut best_candidates = vec![(
        current_dist,
        current.clone(),
        current_structure.clone(),
        current_mfe,
    )];

    let mut temp = initial_temp;
    let mut rng = rand::rng();
    let mut last_improvement_step: i64 = 0;

    // ################## Temporarily --> This is purely for testing
    let mut n_iterations_for_testing: Vec<i64> = vec![0];
    let mut temperature_for_testing: Vec<f64> = vec![temp];
    let mut current_dist_for_testing: Vec<i64> = vec![current_dist];

    // ##################

    let mut last_expand_step: i64 = -1000;
    let mut last_double_step: i64 = -1000;
    for step in 0..max_steps {
        if best_candidates[0].0 == 0 && best_candidates.len() >= n_keep {
            break;
        }

        if step - last_improvement_step > reheat_after {
            temp = reheat_temp;
            last_improvement_step = step;
            println!("step {step}: REHEATING to temp={temp:.2}");
        }

        let mismatched =
            find_mismatched_positions(&current_structure, target_structure, &n_positions);

        let stuck = step - last_improvement_step > 100;
        let use_double = stuck && current_dist <= 2 && step - last_double_step > 25;
        if use_double { last_double_step = step; }

        let mut candidate: Vec<char> = current.chars().collect();

        let target_bytes = target_structure.as_bytes();
        let stuck_positions: Vec<usize> = n_positions
            .iter()
            .filter(|&&p| {
                p < current_structure.len() && current_structure.as_bytes()[p] != target_bytes[p]
            })
            .copied()
            .collect();

        if use_double {
            let mut search_positions: Vec<usize> = if stuck_positions.len() >= 2 {
                stuck_positions.clone()
            } else {
                n_positions.clone()
            };

            if search_positions.len() > 30 {
                use rand::seq::SliceRandom;

                search_positions.shuffle(&mut rng);
                search_positions.truncate(30);
            }

            let mut best_trial = candidate.clone();
            let mut best_cost = f64::MAX;
            const MAX_DOUBLE_TRIALS: usize = 300;
            let mut trials = 0usize;

            'outer: for idx1 in 0..search_positions.len() {
                for idx2 in (idx1 + 1)..search_positions.len() {
                    let i1 = search_positions[idx1];
                    let i2 = search_positions[idx2];

                    if pair_map.get(&i1) == Some(&i2) {
                        continue;
                    }
                    

                    let opts1: Vec<char> = if let Some(&j1) = pair_map.get(&i1) {
                        if designable_positions.contains(&j1) {
                            nucleotides.clone()
                        } else {
                            let fixed_partner = candidate[j1];

                            vec![
                                *comp_dict
                                    .get(&fixed_partner)
                                    .expect("fixed paired position must contain A, U, G, or C"),
                            ]
                        }
                    } else {
                        nucleotides.clone()
                    };

                    let opts2: Vec<char> = if let Some(&j2) = pair_map.get(&i2) {
                        if designable_positions.contains(&j2) {
                            nucleotides.clone()
                        } else {
                            let fixed_partner = candidate[j2];

                            vec![
                                *comp_dict
                                    .get(&fixed_partner)
                                    .expect("fixed paired position must contain A, U, G, or C"),
                            ]
                        }
                    } else {
                        nucleotides.clone()
                    };

                    for &n1 in &opts1 {
                        for &n2 in &opts2 {
                            let mut trial = candidate.clone();

                            trial[i1] = n1;

                            if let Some(&j1) = pair_map.get(&i1) {
                                if designable_positions.contains(&j1) {
                                    trial[j1] = comp_dict[&n1];
                                }
                            }

                            trial[i2] = n2;

                            if let Some(&j2) = pair_map.get(&i2) {
                                if designable_positions.contains(&j2) {
                                    trial[j2] = comp_dict[&n2];
                                }
                            }

                            let trial_str: String = trial.iter().collect();

                            let trial_result = bp_distance_to_target(&trial_str, target_structure);

                            let pk_mismatches =
                                pk_pair_mismatches(&trial_str, &pair_map, target_structure);

                            let trial_cost =
                                trial_result.bp_distance as f64 + 2.0 * pk_mismatches as f64;

                            if trial_cost < best_cost { best_cost = trial_cost; best_trial = trial.clone(); }
                            trials += 1;
                            if best_cost == 0.0 || trials >= MAX_DOUBLE_TRIALS { break 'outer; }

                            if trial_cost < best_cost {
                                best_cost = trial_cost;
                                best_trial = trial;
                            }
                        }
                    }
                
                
                }
            }

            candidate = best_trial;
        } else {
            // === SINGLE MUTATION ===
            let i: usize = if !mismatched.is_empty() && rng.random::<f64>() < 0.7 {
                *mismatched.choose(&mut rng).unwrap()
            } else {
                *n_positions.choose(&mut rng).unwrap()
            };

            if let Some(&j) = pair_map.get(&i) {
                if !designable_positions.contains(&j) {
                    let current_nuc = candidate[i];
                    let mut best_nuc = current_nuc;
                    let mut best_cost = f64::MAX;

                    let fixed_partner = candidate[j];

                    let allowed_nucs = vec![
                        *comp_dict
                            .get(&fixed_partner)
                            .expect("fixed paired position must contain A, U, G, or C"),
                    ];

                    for &nuc in &nucleotides {
                        let mut trial = candidate.clone();
                        trial[i] = nuc;

                        let trial_str: String = trial.iter().collect();

                        let trial_result = bp_distance_to_target(&trial_str, target_structure);

                        let pk_mismatches =
                            pk_pair_mismatches(&trial_str, &pair_map, target_structure);

                        let trial_cost =
                            trial_result.bp_distance as f64 + 2.0 * pk_mismatches as f64;

                        if trial_cost < best_cost {
                            best_cost = trial_cost;
                            best_nuc = nuc;
                        }
                    }

                    if best_nuc == current_nuc {
                        best_nuc = allowed_nucs[0];
                    }

                    candidate[i] = best_nuc;
                } else {
                    let current_pair = (candidate[i], candidate[j]);
                    let mut best_pair = current_pair;
                    let mut best_cost = f64::MAX;

                    for &(ni, nj) in &pair_options {
                        let mut trial = candidate.clone();
                        trial[i] = ni;
                        trial[j] = nj;

                        let trial_str: String = trial.iter().collect();

                        let trial_result = bp_distance_to_target(&trial_str, target_structure);

                        let pk_mismatches =
                            pk_pair_mismatches(&trial_str, &pair_map, target_structure);

                        let trial_cost =
                            trial_result.bp_distance as f64 + 2.0 * pk_mismatches as f64;

                        if trial_cost < best_cost {
                            best_cost = trial_cost;
                            best_pair = (ni, nj);
                        }
                    }

                    if best_pair == current_pair {
                        best_pair = pair_options[rng.random_range(0..pair_options.len())];
                    }

                    candidate[i] = best_pair.0;
                    candidate[j] = best_pair.1;
                }
            } else {
                let current_nuc = candidate[i];
                let mut best_nuc = current_nuc;
                let mut best_cost = f64::MAX;

                for &nuc in &nucleotides {
                    let mut trial = candidate.clone();
                    trial[i] = nuc;

                    let trial_str: String = trial.iter().collect();

                    let trial_result = bp_distance_to_target(&trial_str, target_structure);

                    let pk_mismatches = pk_pair_mismatches(&trial_str, &pair_map, target_structure);

                    let trial_cost = trial_result.bp_distance as f64 + 2.0 * pk_mismatches as f64;

                    if trial_cost < best_cost {
                        best_cost = trial_cost;
                        best_nuc = nuc;
                    }
                }

                if best_nuc == current_nuc {
                    best_nuc = nucleotides[rng.random_range(0..nucleotides.len())];
                }

                candidate[i] = best_nuc;
            }
        }

        let candidate_string: String = candidate.into_iter().collect();

        let candidate_score = score_candidate(&candidate_string, target_structure, &pair_map);

        let cand_dist = candidate_score.total_distance;
        let cand_energy_gap = candidate_score.energy_gap;

        let accept = if cand_dist < current_dist {
            true
        } else if cand_dist == current_dist {
            cand_energy_gap < current_energy_gap
                || rng.random::<f64>()
                    < (-(cand_energy_gap - current_energy_gap) / temp.max(1e-6)).exp()
        } else {
            rng.random::<f64>() < (-(cand_dist - current_dist) as f64 / temp.max(1e-6)).exp()
        };

        n_iterations_for_testing.push(step);
        temperature_for_testing.push(temp);
        current_dist_for_testing.push(current_dist);

        

        if accept {
            let improved = cand_dist < current_dist;

            current = candidate_string.clone();
            current_dist = candidate_score.total_distance;
            current_energy_gap = candidate_score.energy_gap;
            current_structure = candidate_score.structure.clone();
            current_mfe = candidate_score.mfe;

            if improved {
                last_improvement_step = step;
            }

            let existing_seqs: HashSet<&String> = best_candidates.iter().map(|c| &c.1).collect();
            if !existing_seqs.contains(&candidate_string) {
                best_candidates.push((
                    current_dist,
                    candidate_string,
                    current_structure.clone(),
                    current_mfe,
                ));
                best_candidates.sort_by(|a, b| a.0.cmp(&b.0));
                best_candidates.truncate(n_keep);
            }
        }

        

        if cand_dist <= dist_threshold && step % 50 == 0 {
            let current_bytes = current_structure.as_bytes();

            let fixed_mismatches: Vec<usize> = (0..current_bytes.len())
                .filter(|&i| {
                    current_bytes[i] != target_bytes[i]
                        && !original_designable_positions.contains(&i)
                })
                .collect();

            let max_p_paired = if fixed_mismatches.is_empty() {
                1.0
            } else {
                let (_cost, defects, _mfe, _e_target) =
                    compute_pf_defect(&current, target_structure);

                fixed_mismatches
                    .iter()
                    .map(|&i| 1.0 - defects[i])
                    .fold(0.0, f64::max)
            };

            if max_p_paired < p_threshold {
                println!(
                    "  early exit: dist={} but remaining mismatches are weakly paired \
                    (max p≈{:.2})",
                    cand_dist, max_p_paired,
                );
                break;
            }

            if if_slices {
                println!("Early exit for slices at dist={cand_dist}");
                break;
            }
        }

        const MIN_ACTIVE_POSITIONS: usize = 15;
        const STAGNATION_STEPS: i64 = 12;
        const CLOSE_TARGET_DISTANCE: i64 = 6; // Originally 4
        let expansion_radius: usize = (seq_in.len() as usize)*0.2 as usize;
        

        if step > 0 && step % 20 == 0 && current_dist > 0 {
            let stagnation_steps = step - last_improvement_step;
            let is_stagnating_near_target =
                current_dist <= CLOSE_TARGET_DISTANCE && stagnation_steps >= STAGNATION_STEPS;

            if is_stagnating_near_target {
                let expanded = expand_positions_near_mismatches(
                    &current_structure,
                    target_structure,
                    &n_positions,
                    &original_designable_positions,
                    &pair_map,
                    expansion_radius,
                );

                if expanded.len() > n_positions.len() {
                    println!(
                        "stagnation at dist {}: expanding {} -> {} positions",
                        current_dist,
                        n_positions.len(),
                        expanded.len()
                    );
                    n_positions = expanded;
                    designable_positions = n_positions.iter().copied().collect();
                    last_expand_step = step;
                }
            } else if step - last_expand_step >= 60 {
                let reduced = reduce_positions_by_mismatch_proximity(
                    &current_structure,
                    target_structure,
                    &n_positions,
                    MIN_ACTIVE_POSITIONS,
                );
                if reduced.len() < n_positions.len() {
                    n_positions = reduced;
                    designable_positions = n_positions.iter().copied().collect();
                }
            }
        }

        let stall_limit: i64 = if current_dist <= CLOSE_TARGET_DISTANCE {
            200
        } else {
            100
        };

        if step - last_improvement_step >= stall_limit {
            println!(
                "  stopping stagnant restart at step {step}: \
                dist={current_dist}, best_dist={}",
                best_candidates[0].0,
            );
            break;
        }

        temp *= cooling_rate;
        temp = temp.max(0.05);

        if step % 500 == 0 {
            println!(
                "step {step}: dist={current_dist}, gap={current_energy_gap:.2}, best_dist={}, temp={temp:.4}, mode={}",
                best_candidates[0].0,
                if use_double { "DOUBLE" } else { "single" }
            );
        }
    }

    best_candidates
        .into_iter()
        .map(|(bp_distance, sequence, structure, mfe)| DesignResult {
            bp_distance,
            structure,
            mfe,
            sequence,
        })
        .collect()
}

pub fn multi_start_hill_climb_design(
    seq_in: &str,
    target_structure: &str,
    n_position: Vec<usize>,
    n_runs: i64,
    max_steps: i64,
    wobble_frequency: f64,
    if_slices: bool,
    last_global: bool,
    slice_start: Option<&usize>,
    slice_end: Option<&usize>,
    seed: &str,
) -> Vec<DesignResult> {
    let mut all_candidates: Vec<DesignResult> = Vec::new();
    let n_keep_per_run: usize = 1;
    let temp: f64 = 1.0;
    let final_temp: f64 = 0.01;
    let cooling_rate: f64 = (final_temp / temp).powf(1.0 / max_steps as f64);
    //let cooling_rate: f64 = 0.999; // <-- optional override for testing
    let reheat_after: i64 = 500; // number of stagnant steps before reheating
    let reheat_temp: f64 = 1.0; // temperature to jump back

    let seed_seq: &str = if if_slices && USE_ML_MODEL{
        match (slice_start, slice_end) {
            (Some(start), Some(end)) => {
                if *start > *end || *end > seed.len() {
                    eprintln!(
                        "[seed warning] invalid global slice range {}..{} for seed length {}; \
                        using local input sequence instead",
                        start,
                        end,
                        seed.len()
                    );

                    seq_in
                } else {
                    &seed[*start..*end]
                }
            }

            (None, None) => {
                if seed.len() != seq_in.len() {
                    eprintln!(
                        "[seed warning] local seed length {} does not match slice length {}; \
                        using local input sequence instead",
                        seed.len(),
                        seq_in.len()
                    );

                    seq_in
                } else {
                    seed
                }
            }

            _ => {
                eprintln!(
                    "[seed warning] incomplete slice bounds; using local input sequence instead"
                );

                seq_in
            }
        }
    } else {
        if seed.len() != seq_in.len() {
            eprintln!(
                "[seed warning] seed length {} does not match target length {}; \
                using input sequence instead",
                seed.len(),
                seq_in.len()
            );

            seq_in
        } else {
            seed
        }
    };

    let all_pools: Vec<Vec<DesignResult>> = (0..n_runs)
        .into_par_iter()
        .map(|_run| {
            hill_climb_design(
                seq_in,
                target_structure,
                n_position.clone(),
                max_steps,
                wobble_frequency,
                n_keep_per_run,
                temp,
                cooling_rate,
                reheat_after,
                reheat_temp,
                if_slices,
                last_global,
                &seed_seq,
            )
        })
        .collect();

    for pool in all_pools {
        all_candidates.extend(pool);
    }

    let _n_keep_global: usize = 20;
    all_candidates.sort_by(|a, b| {
        a.bp_distance.cmp(&b.bp_distance).then_with(|| {
            a.mfe
                .partial_cmp(&b.mfe)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    all_candidates
}
