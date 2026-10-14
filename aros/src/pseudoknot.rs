use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};

use std::os::raw::c_void;

use crate::ffi::{
    VRNA_OPTION_MFE, vrna_fold_compound, vrna_fold_compound_free, vrna_md_set_default, vrna_md_t,
    vrna_pk_plex, vrna_pk_plex_opt_defaults,
};
use crate::folding::bp_distance_to_target;
use crate::structure::get_pair_map;
use crate::types::{DesignResult, PkPrediction};

pub fn pk_pair_mismatches(seq: &str, pair_map: &HashMap<usize, usize>, target: &str) -> usize {
    let seq_bytes = seq.as_bytes();
    let target_bytes = target.as_bytes();
    let mut mismatches = 0;
    let mut seen = HashSet::new();

    for (&i, &j) in pair_map.iter() {
        if i >= j || seen.contains(&i) {
            continue;
        }

        if target_bytes[i] != b'[' && target_bytes[i] != b']' {
            continue;
        }
        seen.insert(i);
        seen.insert(j);

        let ok = matches!(
            (seq_bytes[i], seq_bytes[j]),
            (b'A', b'U') | (b'U', b'A') | (b'G', b'C') | (b'C', b'G') | (b'G', b'U') | (b'U', b'G')
        );
        if !ok {
            mismatches += 1;
        }
    }
    mismatches
}

pub fn annotate_pk(seq: &str, target: &str, mfe_struct: &str) -> (String, usize) {
    let pair_map = get_pair_map(target);
    let sb = seq.as_bytes();
    let tb = target.as_bytes();
    let mut s: Vec<char> = mfe_struct.chars().collect();
    let mut n_reinserted = 0;

    for (&i, &j) in pair_map.iter() {
        if tb[i] != b'[' && tb[i] != b']' {
            continue;
        }
        if i >= j {
            continue;
        }

        let ok = matches!(
            (sb[i], sb[j]),
            (b'A', b'U') | (b'U', b'A') | (b'G', b'C') | (b'C', b'G') | (b'G', b'U') | (b'U', b'G')
        );
        if ok {
            if s[i] == '.' {
                s[i] = tb[i] as char;
            }
            if s[j] == '.' {
                s[j] = tb[j] as char;
            }
            if s[i] != '.' && s[j] != '.' {
                n_reinserted += 1;
            }
        }
    }
    (s.into_iter().collect(), n_reinserted)
}

pub fn pk_aware_bp_distance_to_target(seq: &str, target: &str) -> DesignResult {
    let result = bp_distance_to_target(seq, target);
    let pair_map = get_pair_map(target);
    let pk_penalty = pk_pair_mismatches(seq, &pair_map, target) as i64;
    DesignResult {
        bp_distance: result.bp_distance + pk_penalty,
        structure: result.structure,
        mfe: result.mfe,
        sequence: result.sequence,
    }
}

pub fn fold_with_pkplex(seq: &str) -> Vec<PkPrediction> {
    unsafe {
        let mut md: vrna_md_t = std::mem::zeroed();
        vrna_md_set_default(&mut md);
        md.temperature = 37.0;
        md.dangles = 1;

        let seq_c = CString::new(seq).expect("seq has interior NUL");
        let fc = vrna_fold_compound(seq_c.as_ptr(), &md, VRNA_OPTION_MFE as u32);
        assert!(!fc.is_null(), "fold_compound returned null");

        let options = vrna_pk_plex_opt_defaults();
        assert!(
            !options.is_null(),
            "vrna_pk_plex_opt_defaults returned null"
        );

        let accessibility: *mut *const std::os::raw::c_int = std::ptr::null_mut();
        let result_ptr = vrna_pk_plex(fc, accessibility, options);

        let mut out = Vec::new();
        if !result_ptr.is_null() {
            let mut p = result_ptr;
            while !(*p).structure.is_null() {
                let structure = CStr::from_ptr((*p).structure)
                    .to_string_lossy()
                    .into_owned();

                out.push(PkPrediction {
                    structure,
                    energy: (*p).energy,
                    dgpk: (*p).dGpk,
                    dgint: (*p).dGint,
                    dg1: (*p).dG1,
                    dg2: (*p).dG2,
                    start_5: (*p).start_5 as usize,
                    end_5: (*p).end_5 as usize,
                    start_3: (*p).start_3 as usize,
                    end_3: (*p).end_3 as usize,
                });

                libc::free((*p).structure as *mut c_void);
                p = p.add(1);
            }
            libc::free(result_ptr as *mut c_void);
        }

        libc::free(options as *mut c_void);

        vrna_fold_compound_free(fc);
        out
    }
}
