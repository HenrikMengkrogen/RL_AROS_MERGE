use std::collections::HashSet;
use std::path::PathBuf;

use rand::RngExt;

use rayon::prelude::*;

use crate::cleanup::{diversify_sequence, gc_cleanup};
use crate::config::{GC_TEST, GC_THRESHOLD, RIBOSOMAL_RNA, GC_LOWER_THRESHOLD};
use crate::decomposition::decompose;
use crate::folding::{bp_distance_to_target, ensemble_diversity};
use crate::hill_climb::{hill_climb_design, multi_start_hill_climb_design};
use crate::mutation::{mutate_seq};

use crate::pseudoknot::{
    annotate_pk, fold_with_pkplex, pk_aware_bp_distance_to_target, pk_pair_mismatches,
};
use crate::ribosome::ribosome_similarity;
use crate::structure::{
    find_conserved_positions, get_local_protected_positions, get_pair_map, get_pk_pairs,
    identify_mismatches, important_motifs, strip_pseudoknots,replace_selected_positions,
};
use crate::types::{DesignResult, GlobalDesignResult, PkPrediction};

pub fn decomposed_hill_climb_design(
    start_seq: &str,
    target: &str,
    n_starts: i64,
    max_steps: i64,
    wobble_frequency: f64,
    ribo_positions: Option<&[usize]>,
) -> Result<GlobalDesignResult, String> {
    assert_eq!(
        start_seq.len(),
        target.len(),
        "start_seq and target must have equal length"
    );

    let ribo_set: HashSet<usize> = ribo_positions
        .map(|p| p.iter().copied().collect())
        .unwrap_or_default();
    if RIBOSOMAL_RNA {
        println!(
            "Restricting mutations to {} ribosomal-inserted positions",
            ribo_set.len()
        );
    }
    let _project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    

    let conserved_motifs = important_motifs();
    let conserved_motifs = find_conserved_positions(&start_seq, &conserved_motifs);

    let if_slices: bool = true;
    let slices = decompose(target).map_err(|e| e.to_string())?;

    let mut seq = start_seq.as_bytes().to_vec();
    if RIBOSOMAL_RNA {
        let mut rng = rand::rng();
        let nucleotides = ['A', 'U', 'G', 'C'];
        for pos in 0..seq.len() {
            if !ribo_set.contains(&pos) && matches!(seq[pos], b'N' | b'K' | b'S') {
                seq[pos] = match seq[pos] {
                    b'K' => {
                        if rng.random::<f64>() < 0.5 {
                            b'G'
                        } else {
                            b'U'
                        }
                    }
                    b'S' => {
                        if rng.random::<f64>() < 0.5 {
                            b'G'
                        } else {
                            b'C'
                        }
                    }
                    _ => nucleotides[rng.random_range(0..4)] as u8,
                };
            }
        }
    }

    println!(
        "decomposed {} slices (post-order: children before parents)",
        slices.len()
    );

    let global_protected_positions: HashSet<usize> = conserved_motifs.iter().copied().collect();

    let mut global_designable: Vec<usize> = if RIBOSOMAL_RNA {
        ribo_set
            .iter()
            .copied()
            .filter(|&global_pos| global_pos < start_seq.len())
            .filter(|global_pos| !global_protected_positions.contains(global_pos))
            .collect()
    } else {
        start_seq
            .as_bytes()
            .iter()
            .enumerate()
            .filter_map(|(global_pos, &base)| {
                matches!(base, b'N' | b'K' | b'S').then_some(global_pos)
            })
            .filter(|global_pos| !global_protected_positions.contains(global_pos))
            .collect()
    };

    global_designable.sort_unstable();
    global_designable.dedup();

    if global_designable.is_empty() {
        println!("[global repair] no designable positions found; skipping global repair");
    } else {
        println!(
            "[global repair] {} designable positions available",
            global_designable.len()
        );
    }

    
    let seed_seq = mutate_seq(&start_seq, target, &global_designable, wobble_frequency, false);
    let global_seed_seq = seed_seq.clone();

    let mut unresolved: HashSet<usize> = HashSet::new();
    for (idx, sub) in slices.iter().enumerate() {
        let local_len = sub.end - sub.start;
        let local_protected_positions =
            get_local_protected_positions(&conserved_motifs, sub.start, sub.end);
        let local_designable: Vec<usize> = if RIBOSOMAL_RNA {
            sub.designable
                .iter()
                .filter(|&&g| ribo_set.contains(&g))
                .map(|&g| g - sub.start)
                .filter(|local_pos| !local_protected_positions.contains(local_pos))
                .collect()
        } else {
            sub.designable
                .iter()
                .filter(|&&g| matches!(start_seq.as_bytes()[g], b'N' | b'K' | b'S'))
                .map(|&g| g - sub.start)
                .collect()
        };

        let child_mismatches: Vec<usize> = unresolved
            .iter()
            .filter(|&&g| g >= sub.start && g < sub.end)
            .map(|&g| g - sub.start)
            .filter(|p| !RIBOSOMAL_RNA || !local_protected_positions.contains(p))
            .collect();

        if local_designable.is_empty() && child_mismatches.is_empty() {
            let slice_seq = String::from_utf8(seq[sub.start..sub.end].to_vec()).unwrap();
            let scored = bp_distance_to_target(&slice_seq, &sub.structure);
            println!(
                "[slice {}/{}] fixed slice, dist={}",
                idx + 1,
                slices.len(),
                scored.bp_distance
            );
            for p in identify_mismatches(&scored.structure, &strip_pseudoknots(&sub.structure)) {
                unresolved.insert(p + sub.start);
            }
            continue;
        }

        let refine_set: HashSet<usize> = local_designable
            .iter()
            .chain(child_mismatches.iter())
            .copied()
            .collect();

        let mut slice_start_bytes = seq[sub.start..sub.end].to_vec();
        for &loc in &refine_set {
            slice_start_bytes[loc] = b'N';
        }

        let slice_start = String::from_utf8(slice_start_bytes).unwrap();

        let n_designable = refine_set.len();
        let n_total = sub.end - sub.start;

        println!(
            "[slice {}/{}] range=[{}, {}] len={} designable={:?} (local={:?})",
            idx + 1,
            slices.len(),
            sub.start,
            sub.end,
            local_len,
            sub.designable,
            local_designable,
        );
        println!("  target:  {}", sub.structure);
        println!("  input:   {}", slice_start);

        if refine_set.is_empty() {
            println!(
                "[slice {}/{}] range=[{}, {}] — no designable or unresolved positions, skipping",
                idx + 1,
                slices.len(),
                sub.start,
                sub.end
            );
            continue;
        }

        

        

        let local_seed = global_seed_seq[sub.start..sub.end].to_string();

        let active_designable: Vec<usize> = refine_set.iter().copied().collect();

        const LARGE_SLICE_LIMIT: usize = 150;
        const MIN_FREE_FRACTION: f64 = 0.20;

        let free_fraction = active_designable.len() as f64 / local_len as f64;

        if local_len > LARGE_SLICE_LIMIT && free_fraction < MIN_FREE_FRACTION {
            println!(
                "  large constrained slice: len={}, free={}; using limited repair",
                local_len,
                active_designable.len(),
            );

            
        }

        let slice_steps = if local_len > 150 && active_designable.len() < 40 {
            50
        } else {
            max_steps
        };


        let mut best = if active_designable.is_empty() {
            bp_distance_to_target(&slice_start, &sub.structure)
        } else {
            multi_start_hill_climb_design(
                &slice_start,
                &sub.structure,
                active_designable,
                n_starts,
                slice_steps,
                wobble_frequency,
                if_slices,
                false,
                None,
                None,
                &local_seed,
            )
            .into_iter()
            .min_by_key(|r| r.bp_distance)
            .ok_or_else(|| format!("slice {} returned empty pool", idx))?
        };

        const MAX_REFINE_ROUNDS: usize = 25;
        const MAX_STALLED_ROUNDS: usize = 2;
        let mut stalled = 0;


        for round in 1..=MAX_REFINE_ROUNDS {
            if best.bp_distance == 0 {
                break;
            }

            let remaining: Vec<usize> = identify_mismatches(&best.structure, &sub.structure)
                .into_iter()
                .filter(|p| refine_set.contains(p))
                .collect();

            if remaining.is_empty() {
                break;
            }

            println!(
                "  [slice {}] refine round {}: dist={}, {} free positions",
                idx + 1,
                round,
                best.bp_distance,
                remaining.len()
            );

            let candidate = multi_start_hill_climb_design(
                &best.sequence,
                &sub.structure,
                remaining,
                n_starts,
                50,
                wobble_frequency,
                true,
                true,
                None,
                None,
                &local_seed,
            )
            .into_iter()
            .min_by_key(|r| r.bp_distance)
            .ok_or_else(|| format!("slice {} returned empty pool", idx))?;

            if candidate.bp_distance < best.bp_distance {
                best = candidate;
                stalled = 0;
            } else {
                stalled += 1;
                if stalled >= MAX_STALLED_ROUNDS {
                    break;
                }
            }
        }

        

        

        let fixed_fraction = 1.0 - (n_designable as f64 / n_total as f64);

        if best.bp_distance > 0 && fixed_fraction > 0.8 {
            println!(
                "  NOTE: slice {} best dist={} ({}% fixed), proceeding to next slice",
                idx + 1,
                best.bp_distance,
                (fixed_fraction * 100.0) as u32
            );
        }

        println!(
            "  best:    bp_distance={}, mfe={:.2}",
            best.bp_distance, best.mfe,
        );
        println!("  seq:     {}", best.sequence);
        println!("  struct:  {}", best.structure);

        unresolved.retain(|&g| g < sub.start || g >= sub.end);

        for p in identify_mismatches(&best.structure, &sub.structure) {
            unresolved.insert(p + sub.start);
        }
        let designed_bytes = best.sequence.as_bytes();

        if designed_bytes.len() != local_len {
            return Err(format!(
                "slice {} returned sequence of length {}, expected {}",
                idx,
                designed_bytes.len(),
                local_len,
            ));
        }
    }

    let mut rng = rand::rng();
    let nucleotides = ['A', 'U', 'G', 'C'];
    let mut full_seq_chars: Vec<char> = String::from_utf8(seq.clone()).unwrap().chars().collect();

    for c in full_seq_chars.iter_mut() {
        if *c == 'N' {
            *c = nucleotides[rng.random_range(0..nucleotides.len())];
        }
        if *c == 'K' {
            *c = if rng.random::<f64>() < 0.5 { 'G' } else { 'U' };
        }
        if *c == 'S' {
            *c = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };
        }
    }

    let pk_pairs = get_pk_pairs(target);
    if !pk_pairs.is_empty() {
        let valid_pairs = [('A', 'U'), ('U', 'A'), ('G', 'C'), ('C', 'G')];
        let mut n_pk_fixed = 0;
        for &(i, j) in &pk_pairs {
            if RIBOSOMAL_RNA && !ribo_set.contains(&i) && !ribo_set.contains(&j) {
                continue;
            }

            let (bi, bj) = (full_seq_chars[i], full_seq_chars[j]);
            if !valid_pairs.contains(&(bi, bj)) {
                let (ni, nj) = valid_pairs[rng.random_range(0..valid_pairs.len())];
                full_seq_chars[i] = ni;
                full_seq_chars[j] = nj;
                n_pk_fixed += 1;
            }
        }
        if n_pk_fixed > 0 {
            println!("PK repair: fixed {} pseudoknotted pair(s)", n_pk_fixed);
        }
    }

    let full_seq = full_seq_chars.into_iter().collect::<String>();

    println!("Proceeding to global check");

    let global_designable_set: HashSet<usize> = global_designable.iter().copied().collect();

    let pre_repair_result = bp_distance_to_target(&full_seq, target);

    let mut global_repair_pool: Vec<DesignResult> = vec![pre_repair_result.clone()];

    let full_seq = if pre_repair_result.bp_distance > 0 {
        let repair_max_steps: i64 =
            ((pre_repair_result.bp_distance as f64 / 0.008).round() as i64).clamp(1, 50);

        println!(
            "attempting global repair via multi_start_hill_climb_design, \
                bp_distance={}, global designable positions={}, max_steps={}",
            pre_repair_result.bp_distance,
            global_designable.len(),
            repair_max_steps
        );

        let seed_seq = {
            let mismatches: HashSet<usize> =
                identify_mismatches(&pre_repair_result.structure, target).into_iter().collect();

            let masked_full_seq = String::from_utf8(
                full_seq
                    .bytes()
                    .enumerate()
                    .map(|(position, base)| {
                        if mismatches.contains(&position) {
                            b'N'
                        } else {
                            base
                        }
                    })
                    .collect(),
            )
            .map_err(|error| format!("Masked sequence was not valid UTF-8: {error}"))?;

            let mut seed = global_seed_seq.clone();
                merge_slice_into_global_seed(&mut seed, 0, masked_full_seq.len(), &masked_full_seq)?;
                seed
            };
        

        let mut repair_pool = multi_start_hill_climb_design(
            &full_seq,
            target,
            global_designable.clone(),
            n_starts,
            repair_max_steps,
            wobble_frequency,
            false,
            false,
            None,
            None,
            &seed_seq,
        );

        repair_pool.push(pre_repair_result.clone());

        repair_pool.sort_by(|a, b| {
            a.bp_distance
                .cmp(&b.bp_distance)
                .then_with(|| a.sequence.cmp(&b.sequence))
        });

        repair_pool.dedup_by(|a, b| a.sequence == b.sequence);

        const MAX_GLOBAL_REPAIR_SEEDS: usize = 16;
        repair_pool.truncate(MAX_GLOBAL_REPAIR_SEEDS);

        global_repair_pool = repair_pool;

        match global_repair_pool
            .iter()
            .map(|r| {
                let (div_nested, div_pk) = ensemble_diversity(&r.sequence, target);
                (r, div_nested + div_pk)
            })
            .min_by(|(a, sa), (b, sb)| {
                a.bp_distance
                    .cmp(&b.bp_distance)
                    .then(sa.total_cmp(sb))
            })
            .map(|(r, _)| r)
        {
            Some(repaired) if repaired.bp_distance < pre_repair_result.bp_distance =>  {
                println!(
                    "global repair improved: {} -> {}",
                    pre_repair_result.bp_distance, repaired.bp_distance
                );

                repaired.sequence.clone()
            }
            _ => {
                println!("global repair made no improvement, keeping pre-repair sequence");

                full_seq
            }
        }
    } else {
        full_seq
    };

    


    let first_focused_check = bp_distance_to_target(&full_seq, target);

    let mut first_focused_repair_pool: Vec<DesignResult> = vec![first_focused_check.clone()];

        if first_focused_check.bp_distance > 0 {
                let first_focused_max_steps: i64 =
                    ((first_focused_check.bp_distance as f64 / 0.008).round() as i64).clamp(1, 50);

                println!(
                    "attempting first focused global repair from {} global-pool candidates",
                    global_repair_pool.len()
                );

                let mut focused_pool: Vec<DesignResult> = global_repair_pool
            .par_iter()
            .flat_map_iter(|seed_result| {
                let mismatched_designable: Vec<usize> =
                    identify_mismatches(&seed_result.structure, target)
                        .into_iter()
                        .filter(|position| global_designable_set.contains(position))
                        .collect();

                let candidate_seed = replace_selected_positions(
                    &seed_result.sequence,
                    &global_seed_seq,
                    mismatched_designable,
                )
                .expect("candidate seed construction failed");

                hill_climb_design(
                    &seed_result.sequence,
                    target,
                    global_designable.clone(),
                    first_focused_max_steps,
                    wobble_frequency,
                    1,
                    1.0,
                    0.99,
                    75,
                    1.0,
                    false,
                    true,
                    &candidate_seed,
                )
                .into_iter()
            })
            .collect();


        focused_pool.push(first_focused_check.clone());

        focused_pool.sort_by(|a, b| {
            a.bp_distance
                .cmp(&b.bp_distance)
                .then_with(|| a.sequence.cmp(&b.sequence))
        });

        focused_pool.dedup_by(|a, b| a.sequence == b.sequence);

        const MAX_FIRST_FOCUSED_SEEDS: usize = 16;
        focused_pool.truncate(MAX_FIRST_FOCUSED_SEEDS);

        first_focused_repair_pool = focused_pool;
    }

    let full_seq = match first_focused_repair_pool
        .iter()
        .map(|r| {
            let (div_nested, div_pk) = ensemble_diversity(&r.sequence, target);
            (r, div_nested + div_pk)
        })
        .min_by(|(a, sa), (b, sb)| {
            a.bp_distance
                .cmp(&b.bp_distance)
                .then(sa.total_cmp(sb))
        })
        .map(|(r, _)| r)
    {
        Some(repaired) if repaired.bp_distance < first_focused_check.bp_distance => {
            println!(
                "first focused global repair improved: {} -> {}",
                first_focused_check.bp_distance, repaired.bp_distance
            );

            repaired.sequence.clone()
        }
        _ => {
            println!("first focused global repair made no improvement, keeping current sequence");

            full_seq
        }
    };

    let final_check_result = bp_distance_to_target(&full_seq, target);

    let full_seq = if final_check_result.bp_distance > 0 {
        let final_repair_max_steps: i64 =
            ((final_check_result.bp_distance as f64 / 0.008).round() as i64).clamp(1, 50);

        let initial_temp: f64 = 1.0;
        let final_temp: f64 = 0.01;

        let cooling_rate: f64 =
            (final_temp / initial_temp).powf(1.0 / final_repair_max_steps as f64);

        let mut repair_seeds: Vec<String> = first_focused_repair_pool
            .iter()
            .map(|result| result.sequence.clone())
            .collect();

        repair_seeds.push(full_seq.clone());

        repair_seeds.sort_unstable();
        repair_seeds.dedup();

        println!(
            "running final mismatch-only repair for {} pool sequences in parallel",
            repair_seeds.len() - 2
        );

        let final_repair_pool: Vec<DesignResult> = repair_seeds
            .par_iter()
            .flat_map_iter(|seed| {
                let seed_result = bp_distance_to_target(seed, target);
                let mismatched_designable: Vec<usize> =
                    identify_mismatches(&seed_result.structure, target)
                        .into_iter()
                        .filter(|position| global_designable_set.contains(position))
                        .collect();

                let candidate_seed = replace_selected_positions(
                    &seed_result.sequence,
                    &global_seed_seq,
                    mismatched_designable.clone(),
                )
                .expect("candidate seed construction failed");

                if mismatched_designable.is_empty() {
                    return vec![seed_result].into_iter();
                }

                hill_climb_design(
                    seed,
                    target,
                    mismatched_designable,
                    final_repair_max_steps,
                    wobble_frequency,
                    1,
                    initial_temp,
                    cooling_rate,
                    75,
                    1.0,
                    false,
                    true,
                    &candidate_seed,
                )
                .into_iter()
            })
            .collect();

        match final_repair_pool
            .iter()
            .map(|r| {
                let (div_nested, div_pk) = ensemble_diversity(&r.sequence, target);
                (r, div_nested + div_pk)
            })
            .min_by(|(a, sa), (b, sb)| {
                a.bp_distance
                    .cmp(&b.bp_distance)
                    .then(sa.total_cmp(sb))
            })
            .map(|(r, _)| r)
        {
            Some(repaired) if repaired.bp_distance < first_focused_check.bp_distance => {
                println!(
                    "final mismatch-only repair improved: {} -> {}",
                    first_focused_check.bp_distance, repaired.bp_distance
                );

                repaired.sequence.clone()
            }
            _ => {
                println!("final mismatch-only repair made no improvement, keeping current sequence");

                full_seq
            }
        }
    } else {
        full_seq
    };

    

    

    let full_seq = if RIBOSOMAL_RNA || GC_TEST {
        println!("Skipping GC cleanup");
        full_seq
    } else {
        gc_cleanup(
            &diversify_sequence(&full_seq, start_seq, target, GC_THRESHOLD, GC_LOWER_THRESHOLD,),
            start_seq,
            target,
            GC_THRESHOLD,
            GC_LOWER_THRESHOLD,
        )
    };

    let result_no_pk = bp_distance_to_target(&full_seq, target);

    let (mfe_struct_with_pk, n_pk_reinserted) =
        annotate_pk(&full_seq, target, &result_no_pk.structure);

    let result = pk_aware_bp_distance_to_target(&full_seq, target);

    let pair_map = get_pair_map(target);
    let pk_penalty = pk_pair_mismatches(&full_seq, &pair_map, target) as i64;

    let (div_nested, div_pk) = ensemble_diversity(&full_seq, target);
    let ensemble_diversity_score = div_nested + div_pk;

    let pk_predictions = fold_with_pkplex(&full_seq);
    let pk_pairs = get_pk_pairs(target);

    let _hit_matches_target = |pk: &PkPrediction| -> bool {
        pk_pairs.iter().any(|&(i, j)| {
            (i >= pk.start_5 && i <= pk.end_5 && j >= pk.start_3 && j <= pk.end_3)
                || (j >= pk.start_5 && j <= pk.end_5 && i >= pk.start_3 && i <= pk.end_3)
        })
    };

    //let pk_energy: f64 = pk_predictions
    //    .iter()
    //    .filter(|pk| hit_matches_target(pk))
    //    .map(|pk| pk.dgint) // or dgpk, see the earlier note
    //    .sum();

    //let mfe_with_pk = result.mfe + pk_energy;

    println!("---- global verification ----");
    println!("designed:                 {}", full_seq);
    println!("target:                   {}", target);
    println!("mfe struct (VRNA-only):   {}", result.structure);
    println!("mfe struct (with pk):     {}", pk_predictions[0].structure);
    println!("pk pairs reinserted:      {}", n_pk_reinserted);
    println!("bp_distance (VRNA only):  {}", result_no_pk.bp_distance);
    println!("pk penalty (bad pk pairs): {}", pk_penalty);
    println!("bp_distance (pk-aware):   {}", result.bp_distance);
    println!("mfe                    : {:.2}", pk_predictions[0].energy);
    println!("ensemble diversity:       {:.2}", ensemble_diversity_score);

    if !pk_predictions.is_empty() {
        println!("---- pkplex verification ----");
        for (k, pk) in pk_predictions.iter().enumerate() {
            println!(
                "  pk {}: dGpk={:.2}, dG1={:.2}, dG2={:.2}, dGint={:.2}, range 5'=[{},{}], 3'=[{},{}]",
                k, pk.dgpk, pk.dg1, pk.dg2, pk.dgint, pk.start_5, pk.end_5, pk.start_3, pk.end_3
            );
        }
    } else {
        println!("---- pkplex: no pseudoknot predicted ----");
    }

    let (ribosome_identity, ribosome_mismatches) = if RIBOSOMAL_RNA {
        let ribo_positions_vec: Vec<usize> = ribo_positions.unwrap_or(&[]).to_vec();
        let sim = ribosome_similarity(&full_seq, &ribo_positions_vec);
        println!(
            "ribosome identity: {}/{} ({:.1}%)",
            sim.matches,
            sim.total,
            sim.identity * 100.0
        );
        (Some(sim.identity), Some(sim.mismatched_positions))
    } else {
        (None, None)
    };

    Ok(GlobalDesignResult {
        sequence: full_seq,
        mfe_structure: mfe_struct_with_pk,
        bp_distance: result.bp_distance,
        mfe: pk_predictions[0].energy,
        n_slices: slices.len(),
        ribosome_identity,
        ribosome_mismatches,
        ensemble_diversity: ensemble_diversity_score,
    })
}

pub fn merge_slice_into_global_seed(
    global_seed: &mut String,
    start: usize,
    end: usize,
    generated: &str,
) -> Result<(), String> {
    if start > end {
        return Err(format!(
            "Invalid merge range: start {} is greater than end {}",
            start, end
        ));
    }

    if end > global_seed.len() {
        return Err(format!(
            "Merge range {}..{} exceeds global seed length {}",
            start,
            end,
            global_seed.len()
        ));
    }

    let expected_len = end - start;

    if generated.len() != expected_len {
        return Err(format!(
            "Generated slice length {} does not match range {}..{} (expected {})",
            generated.len(),
            start,
            end,
            expected_len
        ));
    }

    let mut global_bases = global_seed.as_bytes().to_vec();

    for (offset, &base) in generated.as_bytes().iter().enumerate() {
        let global_position = start + offset;

        match base {
            b'A' | b'U' | b'G' | b'C' => {
                global_bases[global_position] = base;
            }

            b'N' => {}

            other => {
                return Err(format!(
                    "Invalid base '{}' in generated slice at local position {} \
                     (global position {})",
                    other as char, offset, global_position
                ));
            }
        }
    }

    *global_seed = String::from_utf8(global_bases)
        .map_err(|error| format!("Global seed became invalid UTF-8: {error}"))?;

    Ok(())
}
