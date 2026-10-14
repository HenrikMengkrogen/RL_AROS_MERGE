use rand::seq::SliceRandom;

use crate::folding::{bp_distance_to_target, ensemble_diversity};
use crate::io::gc_content;
use crate::structure::get_pair_map;
use crate::types::DesignResult;

pub fn gc_cleanup(
    seq_in: &str,
    original_annotation: &str,
    target: &str,
    max_gc_percent: f64,
    min_gc_percent : f64,
) -> String {
    assert_eq!(seq_in.len(), original_annotation.len());
    assert_eq!(seq_in.len(), target.len());

    let pair_map = get_pair_map(target);
    let annotation = original_annotation.as_bytes();
    let mut seq: Vec<u8> = seq_in.as_bytes().to_vec();
    let mut rng = rand::rng();

    let mut current_result =
        bp_distance_to_target(&String::from_utf8(seq.clone()).unwrap(), target);
    let mut current_dist = current_result.bp_distance;

    let mut pairs: Vec<(usize, usize)> = pair_map
        .iter()
        .filter_map(|(&i, &j)| (i < j).then_some((i, j)))
        .filter(|&(i, j)| annotation[i] == b'N' && annotation[j] == b'N')
        .filter(|&(i, j)| matches!((seq[i], seq[j]), (b'G', b'C') | (b'C', b'G')))
        .collect();
    pairs.sort_unstable();
    pairs.dedup();

    pairs.shuffle(&mut rng);

    
    let mut gc = gc_content(&String::from_utf8(seq.clone()).unwrap()).unwrap_or(0.0);

    let mut total_replacements = 0usize;

    for &(i, j) in &pairs {
        if gc > min_gc_percent && gc < max_gc_percent{
            break;
        }

        let mut options = [(b'A', b'U'), (b'U', b'A')];
        options.shuffle(&mut rng);
        let (div_nested, div_pk) = ensemble_diversity(&seq_in, target);
        let ensemble_diversity_score = div_nested + div_pk;
        let mut current_div = ensemble_diversity_score;

        let mut best_trial: Option<(Vec<u8>, DesignResult)> = None;
        for (left, right) in options {
            let mut trial = seq.clone();
            trial[i] = left;
            trial[j] = right;

            let (div_nested, div_pk) = ensemble_diversity(&seq_in, target);
            let ensemble_diversity_score = div_nested + div_pk;
            let trial_result =
                bp_distance_to_target(&String::from_utf8(trial.clone()).unwrap(), target);
            if trial_result.bp_distance > current_dist {
                continue;
            }

            let replace_best = match &best_trial {
                None => true,
                Some((_, b)) => {
                    trial_result.bp_distance < b.bp_distance
                        && ensemble_diversity_score < current_div
                        || (trial_result.bp_distance == b.bp_distance && trial_result.mfe < b.mfe)
                            && ensemble_diversity_score < current_div
                }
            };
            if replace_best {
                best_trial = Some((trial, trial_result));
                current_div = ensemble_diversity_score;
            }
        }

        if let Some((trial, trial_result)) = best_trial {
            seq = trial;
            current_dist = trial_result.bp_distance;
            current_result = trial_result;
            gc = gc_content(&current_result.sequence).unwrap_or(0.0);
            
            total_replacements += 1;
        }
    }

    println!(
        "GC cleanup complete: {} GC pair(s) converted; final bp_distance={}",
        total_replacements, current_result.bp_distance
    );

    String::from_utf8(seq).expect("sequence must be valid UTF-8")
}

pub fn diversify_sequence(
    seq_in: &str,
    original_annotation: &str,
    target: &str,
    max_gc_percent: f64,
    min_gc_percent : f64,
) -> String {
    assert_eq!(seq_in.len(), original_annotation.len());
    assert_eq!(seq_in.len(), target.len());

    let pair_map = get_pair_map(target);
    let annotation = original_annotation.as_bytes();
    let mut seq: Vec<u8> = seq_in.as_bytes().to_vec();
    let mut rng = rand::rng();

    let mut current_result = bp_distance_to_target(&String::from_utf8(seq.clone()).unwrap(), target);
    let mut current_dist = current_result.bp_distance;

    let mut pairs: Vec<(usize, usize)> = pair_map
        .iter()
        .filter_map(|(&i, &j)| (i < j).then_some((i, j)))
        .filter(|&(i, j)| annotation[i] == b'N' && annotation[j] == b'N')
        .filter(|&(i, j)| {
            matches!(
                (seq[i], seq[j]),
                (b'G', b'C') | (b'C', b'G') | (b'A', b'U') | (b'U', b'A')
            )
        })
        .collect();
    pairs.sort_unstable();
    pairs.dedup();

    pairs.shuffle(&mut rng);

    
    let mut gc = gc_content(&String::from_utf8(seq.clone()).unwrap()).unwrap_or(0.0);

    for &(i, j) in &pairs {
        if gc > min_gc_percent && gc < max_gc_percent{
            break;
        }

        let mut options = [(b'A', b'U'), (b'U', b'A'), (b'G', b'C'), (b'C', b'G')];
        options.shuffle(&mut rng);
        let (div_nested, div_pk) = ensemble_diversity(&seq_in, target);
        let ensemble_diversity_score = div_nested + div_pk;
        let mut current_div = ensemble_diversity_score;

        let mut best_trial: Option<(Vec<u8>, DesignResult)> = None;
        for (left, right) in options {
            let mut trial = seq.clone();
            trial[i] = left;
            trial[j] = right;
            let (div_nested, div_pk) = ensemble_diversity(&seq_in, target);
            let ensemble_diversity_score = div_nested + div_pk;
            let trial_result =
                bp_distance_to_target(&String::from_utf8(trial.clone()).unwrap(), target);
            if trial_result.bp_distance > current_dist {
                continue;
            }

            let replace_best = match &best_trial {
                None => true,
                Some((_, b)) => {
                    trial_result.bp_distance < b.bp_distance
                        && ensemble_diversity_score < current_div
                        || (trial_result.bp_distance == b.bp_distance && trial_result.mfe < b.mfe)
                            && ensemble_diversity_score < current_div
                }
            };
            if replace_best {
                best_trial = Some((trial, trial_result));
                current_div = ensemble_diversity_score;
            }
        }

        if let Some((trial, trial_result)) = best_trial {
            seq = trial;
            current_dist = trial_result.bp_distance;
            current_result = trial_result;
            gc = gc_content(&current_result.sequence).unwrap_or(0.0);
        }
    }

    String::from_utf8(seq).expect("sequence must be valid UTF-8")
}
