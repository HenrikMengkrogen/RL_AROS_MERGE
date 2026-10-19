use crate::types::Substructure;
use std::collections::{HashMap, HashSet};
use rand::RngExt;

pub fn get_pair_map(structure: &str) -> HashMap<usize, usize> {
    let mut pair_map = HashMap::new();
    let mut paren_stack: Vec<usize> = Vec::new();
    let mut bracket_stack: Vec<usize> = Vec::new();

    for (i, c) in structure.chars().enumerate() {
        match c {
            '(' => paren_stack.push(i),
            ')' => {
                let j = paren_stack.pop().expect("Unmatched closing parenthesis");
                pair_map.insert(i, j);
                pair_map.insert(j, i);
            }
            '[' => bracket_stack.push(i),
            ']' => {
                let j = bracket_stack.pop().expect("Unmatched closing bracket");
                pair_map.insert(i, j);
                pair_map.insert(j, i);
            }
            _ => {}
        }
    }

    pair_map
}

pub fn strip_pseudoknots(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '[' | ']' => '.',
            _ => c,
        })
        .collect()
}

pub fn get_pk_pairs(structure: &str) -> Vec<(usize, usize)> {
    let pair_map = get_pair_map(structure);
    let b = structure.as_bytes();
    let mut pk_pairs = Vec::new();
    let mut seen = HashSet::new();
    for (&i, &j) in pair_map.iter() {
        if i >= j || seen.contains(&i) {
            continue;
        }
        if b[i] == b'[' || b[i] == b']' {
            seen.insert(i);
            seen.insert(j);
            pk_pairs.push((i.min(j), i.max(j)));
        }
    }
    pk_pairs
}

pub fn identify_mismatches(mfe_structure: &str, target: &str) -> Vec<usize> {
    let target_no_pk = strip_pseudoknots(target);
    let mfe_bytes = mfe_structure.as_bytes();
    let target_bytes = target_no_pk.as_bytes();

    (0..target_bytes.len())
        .filter(|&i| mfe_bytes.get(i).copied() != Some(target_bytes[i]))
        .collect()
}

pub fn is_adjacent_to_loop(pos: usize, structure: &str) -> bool {
    let bytes = structure.as_bytes();
    let n = bytes.len();
    (pos > 0 && bytes[pos - 1] == b'.') || (pos + 1 < n && bytes[pos + 1] == b'.')
}

pub fn find_hard_loops(structure: &str) -> bool {
    let motifs: &[&str] = &["()"];
    let mut motif_found: bool = false;
    for n in motifs {
        for i in 0..structure.len() - 1 {
            let j = i + n.len();
            let window = &structure[i..j];
            if window == *n {
                motif_found = true;
                break;
            } else {
                continue;
            };
        }
    }
    motif_found
}

pub fn find_tetraloops(structure: &str) -> bool {
    let motifs: &[&str] = &["(....)"];
    let mut motif_found: bool = false;
    for n in motifs {
        for i in 0..structure.len() - 5 {
            let j = i + n.len();
            let window = &structure[i..j];
            if window == *n {
                motif_found = true;
                break;
            } else {
                continue;
            };
        }
    }
    motif_found
}

pub fn find_mismatched_positions(
    current_structure: &str,
    target: &str,
    n_positions: &[usize],
) -> Vec<usize> {
    let current_bytes = current_structure.as_bytes();
    let tgt_no_pk = strip_pseudoknots(target);
    let target_bytes = tgt_no_pk.as_bytes();
    n_positions
        .iter()
        .filter(|&&i| target_bytes[i] != b'.' && current_bytes[i] != target_bytes[i])
        .copied()
        .collect()
}

pub fn reduce_positions_by_mismatch_proximity(
    current_structure: &str,
    target_structure: &str,
    current_positions: &[usize],
    minimum_positions: usize,
) -> Vec<usize> {
    let mismatches = identify_mismatches(current_structure, target_structure);

    if mismatches.is_empty() {
        return current_positions.to_vec();
    }

    let mut ranked_positions: Vec<(usize, usize)> = current_positions
        .iter()
        .copied()
        .map(|position| {
            let nearest_mismatch_distance = mismatches
                .iter()
                .map(|&mismatch| position.abs_diff(mismatch))
                .min()
                .unwrap_or(usize::MAX);

            (position, nearest_mismatch_distance)
        })
        .collect();

    ranked_positions.sort_by_key(|&(_, distance)| distance);

    let keep_count = minimum_positions.min(ranked_positions.len());

    let mut reduced_positions: Vec<usize> = ranked_positions
        .into_iter()
        .take(keep_count)
        .map(|(position, _)| position)
        .collect();

    reduced_positions.sort_unstable();
    reduced_positions
}

pub fn expand_positions_near_mismatches(
    current_structure: &str,
    target_structure: &str,
    current_active_positions: &[usize],
    original_designable_positions: &HashSet<usize>,
    pair_map: &HashMap<usize, usize>,
    radius: usize,
) -> Vec<usize> {
    let current_bytes = current_structure.as_bytes();
    let target_bytes = target_structure.as_bytes();
    let sequence_length = target_bytes.len();

    assert_eq!(
        current_bytes.len(),
        sequence_length,
        "Current structure and target structure must have equal lengths"
    );

    let mut expanded: HashSet<usize> = current_active_positions.iter().copied().collect();

    let mut add_neighborhood = |center: usize| {
        let start = center.saturating_sub(radius);
        let end = (center + radius).min(sequence_length.saturating_sub(1));

        for position in start..=end {
            if original_designable_positions.contains(&position) {
                expanded.insert(position);
            }
        }
    };

    for position in 0..sequence_length {
        if current_bytes[position] == target_bytes[position] {
            continue;
        }

        add_neighborhood(position);

        if let Some(&partner) = pair_map.get(&position) {
            add_neighborhood(partner);
        }
    }

    let mut expanded_positions: Vec<usize> = expanded.into_iter().collect();

    expanded_positions.sort_unstable();
    expanded_positions
}

pub fn find_conserved_positions(sequence: &str, motifs: &[&str]) -> Vec<usize> {
    let mut protected = Vec::new();

    for motif in motifs {
        let motif_len = motif.len();

        if motif_len > sequence.len() {
            continue;
        }

        for start in 0..=sequence.len() - motif_len {
            let window = &sequence[start..start + motif_len];

            if window == *motif {
                for i in start..start + motif_len {
                    protected.push(i);
                }
            }
        }
    }

    protected.sort_unstable();
    protected.dedup();

    protected
}

pub fn get_local_protected_positions(
    conserved_positions: &[usize],
    slice_start: usize,
    slice_end: usize,
) -> Vec<usize> {
    conserved_positions
        .iter()
        .filter(|&&pos| pos >= slice_start && pos < slice_end)
        .map(|&pos| pos - slice_start)
        .collect()
}

pub fn assert_slices_balanced(slices: &[Substructure]) -> Result<(), String> {
    for (idx, sub) in slices.iter().enumerate() {
        let opens = sub.structure.matches('(').count();
        let closes = sub.structure.matches(')').count();
        if opens != closes {
            return Err(format!(
                "slice {} range=[{}, {}) is unbalanced: {} '(' vs {} ')'\n  struct: {}",
                idx, sub.start, sub.end, opens, closes, sub.structure
            ));
        }

        let stripped = strip_pseudoknots(&sub.structure);
        let mut depth = 0i32;
        for (pos, c) in stripped.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(format!(
                            "slice {} has closing paren before opening at local pos {}",
                            idx, pos
                        ));
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(format!("slice {} not balanced after stripping pk", idx));
        }
    }
    Ok(())
}

pub fn important_motifs() -> &'static [&'static str] {
    let conserved_motifs: &[&str] = &[
        "GGCCAAAU", // Alpha KL
        "UAAACCGG",
        "UUAAUCGAUACCUGGGCUGGCAGAGCGUGCCGGCAUGCUCGGGUGUGAGAUGAGCUGUAUUGAUUGC", // Broccoli
        "CCGUGCGAGACGGUCGGGUCCAUAGCUAAUUCGUUAGUUAUGGAGGCUCGUACGG",             // Broccoli
        "UGAAGCCUCCACG",
        "GCACCUCCGAAGU", // Regular KL
        "AUCACGGGAGACACACGGCGGGUGNNNNNNNNNNNNNNNNNNNNGCAUGCUGGAGGUAUCAGAAGUGCGAAUGCUGACAUAAGUAACGAUAAAGCGGGUGAAAAGCCCGCUCGCCGGAAGACCAAGGGUUCCUGUCCAACGUUAAUCGGGGCAGGGUGAGUCGACCCCUAAGGCGAGGCCGAAAGGCGUAGUCGAUGGGAAACA",
    ];

    return conserved_motifs;
}

pub fn replace_selected_positions(
    sequence: &str,
    replacement_source: &str,
    positions: impl IntoIterator<Item = usize>,
) -> Result<String, String> {
    if sequence.len() != replacement_source.len() {
        return Err(format!(
            "Sequence length {} does not match source length {}",
            sequence.len(),
            replacement_source.len(),
        ));
    }

    let mut output = sequence.as_bytes().to_vec();
    let source = replacement_source.as_bytes();

    for position in positions {
        if position >= output.len() {
            return Err(format!(
                "Position {} exceeds sequence length {}",
                position,
                output.len()
            ));
        }

        output[position] = source[position];
    }

    String::from_utf8(output)
        .map_err(|error| format!("Generated invalid UTF-8 sequence: {error}"))
}

pub fn is_placeholder(c: char) -> bool {
    matches!(c, 'R' | 'r' | 'X' | 'x')
}

pub fn complement(c: char) -> Option<char> {
    match c {
        'A' => Some('U'),
        'U' => Some('A'),
        'G' => Some('C'),
        'C' => Some('G'),
        'R' => Some('r'),
        'r' => Some('R'),
        'X' => Some('x'),
        'x' => Some('X'),
        _ => None,
    }
}


pub fn to_foldable(seq: &str) -> String {
    seq.chars()
        .map(|c| match c {
            'R' => 'G',
            'r' => 'C',
            'X' => 'A',
            'x' => 'U',
            other => other,
        })
        .collect()
}


pub fn enforce_placeholders(
    chars: &mut [char],
    seq_in: &[char],
    pair_map: &HashMap<usize, usize>,
) {
    for (i, &c) in seq_in.iter().enumerate() {
        if is_placeholder(c) {
            chars[i] = c;
            if let Some(&j) = pair_map.get(&i) {
                if let Some(comp) = complement(c) {
                    chars[j] = comp;
                }
            }
        }
    }
}

pub fn resolve_placeholders(seq: &str, structure: &str, gc_only: bool) -> String {
    let mut rng = rand::rng();
    let pair_map = get_pair_map(structure);
    let mut s: Vec<char> = seq.chars().collect();

    let pool: &[char] = if gc_only { &['G', 'C'] } else { &['A', 'U', 'G', 'G', 'G', 'C', 'C', 'C', 'G', 'C'] };
    let comp = |c: char| match c { 'A'=>'U','U'=>'A','G'=>'C','C'=>'G', o=>o };

    // 1. Big letters (R, X) get random bases
    for p in 0..s.len() {
        if s[p] == 'R' || s[p] == 'X' {
            s[p] = pool[rng.random_range(0..pool.len())];
        }
    }
    // 2. Small letters (r, x): complement of their structural partner if it was R/X
    let orig: Vec<char> = seq.chars().collect();
    for p in 0..s.len() {
        if orig[p] == 'r' || orig[p] == 'x' {
            let want_big = if orig[p] == 'r' { 'R' } else { 'X' };
            s[p] = match pair_map.get(&p) {
                Some(&q) if orig[q] == want_big => comp(s[q]),
                _ => pool[rng.random_range(0..pool.len())], // unpaired or paired elsewhere
            };
        }
    }
    s.into_iter().collect()
}