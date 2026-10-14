use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;


use crate::ffi::*;
use crate::pseudoknot::pk_pair_mismatches;
use crate::structure::{get_pair_map, get_pk_pairs, strip_pseudoknots};
use crate::types::{CandidateScore, DesignResult};

pub fn bp_distance_to_target(seq: &str, target: &str) -> DesignResult {
    let n = seq.len();
    let target_no_pk = strip_pseudoknots(target);

    let pair_map = get_pair_map(&target_no_pk);
    let _max_span = pair_map
        .iter()
        .map(|(&i, &j)| i.abs_diff(j))
        .max()
        .unwrap_or(0);

    unsafe {
        let mut md: vrna_md_t = std::mem::zeroed();
        vrna_md_set_default(&mut md);
        md.temperature = 37.0;
        md.dangles = 1;

        //md.max_bp_span = (max_span + 30) as i32;

        let seq_bytes = seq.as_bytes();
        let target_bytes = target_no_pk.as_bytes();
        let mut fold_seq_bytes = seq.as_bytes().to_vec();
        for i in 0..n {
            if seq_bytes[i] != b'N' && target_bytes[i] == b'.' {
                fold_seq_bytes[i] = b'N'; // Mask for folding only
            }
        }
        let fold_seq = String::from_utf8(fold_seq_bytes).unwrap();

        /*
         * Pass a borrowed byte slice to CString::new rather than moving fold_seq.
         * This is portable and means fold_seq remains available if needed later.
         */
        let seq_c = CString::new(fold_seq.as_bytes()).expect("seq has interior NUL");

        let fc = vrna_fold_compound(seq_c.as_ptr(), &md, VRNA_OPTION_MFE as u32);

        assert!(!fc.is_null(), "vrna_fold_compound returned null");

        let mut structure: Vec<c_char> = vec![0; n + 1];
        let mfe = vrna_mfe(fc, structure.as_mut_ptr());

        let structure_str = CStr::from_ptr(structure.as_ptr())
            .to_string_lossy()
            .into_owned();

        let target_c =
            CString::new(target_no_pk.as_bytes()).expect("target structure has interior NUL");

        let structure_c =
            CString::new(structure_str.as_bytes()).expect("returned structure has interior NUL");

        let distance = vrna_bp_distance(target_c.as_ptr(), structure_c.as_ptr());

        vrna_fold_compound_free(fc);

        DesignResult {
            bp_distance: distance as i64,
            structure: structure_str,
            mfe: mfe as f64,
            sequence: seq.to_string(), // Return the REAL sequence, not the masked one
        }
    }
}

pub fn energy_of_target_structure(seq: &str, target: &str) -> f64 {
    let target_no_pk = strip_pseudoknots(target);
    unsafe {
        let mut md: vrna_md_t = std::mem::zeroed();
        vrna_md_set_default(&mut md);
        md.temperature = 37.0;
        md.dangles = 1;

        let seq_c = CString::new(seq).unwrap();
        let fc = vrna_fold_compound(seq_c.as_ptr(), &md, VRNA_OPTION_MFE as u32);
        let target_c = CString::new(target_no_pk).unwrap();
        let energy = vrna_eval_structure(fc, target_c.as_ptr());
        vrna_fold_compound_free(fc);
        energy as f64
    }
}

pub fn score_candidate(
    seq: &str,
    target_structure: &str,
    pair_map: &HashMap<usize, usize>,
) -> CandidateScore {
    let fold = bp_distance_to_target(seq, target_structure);

    let pk_mismatches = pk_pair_mismatches(seq, pair_map, target_structure);

    let target_energy = energy_of_target_structure(seq, target_structure);

    let energy_gap = (target_energy - fold.mfe).max(0.0);

    CandidateScore {
        structure: fold.structure,
        mfe: fold.mfe,
        total_distance: fold.bp_distance + 2 * pk_mismatches as i64,
        energy_gap,
    }
}

pub fn compute_pf_defect(seq: &str, target: &str) -> (f64, Vec<f64>, f64, f64) {
    let n = seq.len();
    let target_bytes = target.as_bytes();
    let target_no_pk = strip_pseudoknots(target);

    unsafe {
        let mut md: vrna_md_t = std::mem::zeroed();
        vrna_md_set_default(&mut md);
        md.temperature = 37.0;
        md.dangles = 1;

        let seq_c = CString::new(seq).expect("seq has interior NUL");
        let fc = vrna_fold_compound(
            seq_c.as_ptr(),
            &md,
            (VRNA_OPTION_MFE | VRNA_OPTION_PF) as u32,
        );
        assert!(!fc.is_null(), "fold_compound returned null");

        let mut mfe_struct: Vec<c_char> = vec![0; n + 1];
        let mfe = vrna_mfe(fc, mfe_struct.as_mut_ptr());

        let mut mfe_scaled: f64 = mfe as f64;
        vrna_exp_params_rescale(fc, &mut mfe_scaled);

        let mut pf_struct: Vec<c_char> = vec![0; n + 1];
        let _pf_energy = vrna_pf(fc, pf_struct.as_mut_ptr());

        let exp_matrices = (*fc).exp_matrices;
        assert!(!exp_matrices.is_null(), "exp_matrices is null");

        let probs_ptr = (*exp_matrices).__bindgen_anon_1.__bindgen_anon_1.probs;
        let iindx_ptr = (*fc).iindx;
        assert!(!probs_ptr.is_null(), "probs is null after vrna_pf");
        assert!(!iindx_ptr.is_null(), "iindx is null");

        let mut defects = vec![0.0_f64; n];
        let mut cost = 0.0_f64;

        for i in 0..n {
            let mut p_paired = 0.0_f64;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let (a, b) = if i < j {
                    (i + 1, j + 1)
                } else {
                    (j + 1, i + 1)
                };
                let iindx_val = *iindx_ptr.add(a) as isize;
                let idx = iindx_val - b as isize;
                let pr_val = *probs_ptr.offset(idx) as f64;
                p_paired += pr_val;
            }
            p_paired = p_paired.min(1.0);

            let should_be_paired = target_bytes[i] != b'.';
            defects[i] = if should_be_paired {
                (1.0 - p_paired).max(0.0)
            } else {
                p_paired
            };
            cost += defects[i];
        }

        let target_c = CString::new(target_no_pk).expect("target has interior NUL");
        let e_target = vrna_eval_structure(fc, target_c.as_ptr()) as f64;

        vrna_fold_compound_free(fc);

        (cost / n as f64, defects, mfe as f64, e_target)
    }
}

pub fn ensemble_diversity(seq: &str, target: &str) -> (f64, f64) {
    let n = seq.len();
    let seq_b = seq.as_bytes();
    let target_no_pk = strip_pseudoknots(target);
    let pair_map = get_pair_map(&target_no_pk);
    let pk_pairs = get_pk_pairs(target); // your existing helper: Vec<(usize, usize)>

    unsafe {
        let mut md: vrna_md_t = std::mem::zeroed();
        vrna_md_set_default(&mut md);
        md.temperature = 37.0;
        md.dangles = 1;

        let seq_c = CString::new(seq).expect("seq has interior NUL");
        let fc = vrna_fold_compound(seq_c.as_ptr(), &md, VRNA_OPTION_PF as u32);
        assert!(!fc.is_null(), "fold_compound returned null");

        let mut pf_struct: Vec<c_char> = vec![0; n + 1];
        
        


        let _ = vrna_pf(fc, pf_struct.as_mut_ptr());

        let exp_matrices = (*fc).exp_matrices;
        assert!(!exp_matrices.is_null(), "exp_matrices is null");
        let probs_ptr = (*exp_matrices).__bindgen_anon_1.__bindgen_anon_1.probs;
        let iindx_ptr = (*fc).iindx;
        assert!(!probs_ptr.is_null() && !iindx_ptr.is_null());

        let mut nested_div = 0.0_f64;
        let mut p_paired = vec![0.0_f64; n]; // total pairing probability per base

        for i in 0..n {
            for j in (i + 1)..n {
                let idx = *iindx_ptr.add(i + 1) as isize - (j + 1) as isize;
                let p_ij = *probs_ptr.offset(idx) as f64;

                p_paired[i] += p_ij;
                p_paired[j] += p_ij;

                nested_div += if pair_map.get(&i) == Some(&j) {
                    1.0 - p_ij
                } else {
                    p_ij
                };
            }
        }
        vrna_fold_compound_free(fc);

        let mut pk_div = 0.0_f64;
        for &(i, j) in &pk_pairs {
            let canonical = matches!(
                (seq_b[i], seq_b[j]),
                (b'A', b'U')
                    | (b'U', b'A')
                    | (b'G', b'C')
                    | (b'C', b'G')
                    | (b'G', b'U')
                    | (b'U', b'G')
            );
            let q = if canonical {
                (1.0 - p_paired[i]).max(0.0) * (1.0 - p_paired[j]).max(0.0)
            } else {
                0.0
            };
            pk_div += 1.0 - q;
        }

        (nested_div, pk_div)
    }
}
