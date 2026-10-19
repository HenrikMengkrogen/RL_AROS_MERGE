use std::collections::{HashMap, HashSet};

use rand::RngExt;

use crate::config::{GC_TEST, RIBOSOME_SEQUENCE};
use crate::structure::{find_hard_loops, find_tetraloops, get_pair_map, is_adjacent_to_loop};

use crate::structure::{find_conserved_positions, important_motifs};

pub fn mutate_seq(
    seq_in: &str,
    structure: &str,
    n_positions: &[usize],
    wobble_frequency: f64,
    last_global: bool,
) -> String {
    use std::collections::HashSet;

    let mut rng = rand::rng();

    let pair_map = get_pair_map(structure);

    let comp_dict = HashMap::from([('A', 'U'), ('U', 'A'), ('G', 'C'), ('C', 'G')]);

    let hard_loops = find_hard_loops(structure);
    let tetra_loops = find_tetraloops(structure);

    let paired_nucleotides: &[char] = if GC_TEST || last_global {
        &['G', 'C']
    } else {
        &['A', 'U', 'G', 'G', 'G', 'C', 'C', 'C', 'G', 'C'] // 80% chance for GC pair
        //&['A', 'U', 'C', 'G'] // 50% chance for GC pair
    };

    let purines: &[char] = if GC_TEST { &['G', 'C'] } else { &['A', 'U', 'C', 'G'] };

    let mut mut_seq: Vec<char> = seq_in.chars().collect();
    let mut_struct: Vec<char> = structure.chars().collect();

    let mutable_positions: HashSet<usize> = n_positions.iter().copied().collect();

    for (big, small) in [('R', 'r'), ('X', 'x')] {
        let big_pos: Vec<usize> = (0..mut_seq.len()).filter(|&p| mut_seq[p] == big).collect();
        let small_pos: Vec<usize> = (0..mut_seq.len()).filter(|&p| mut_seq[p] == small).collect();

        if big_pos.is_empty() && small_pos.is_empty() {
            continue;
        }

        if big_pos.len() == small_pos.len() {
            // One shared domain sequence; the small domain is its reverse complement.
            let domain: Vec<char> = (0..big_pos.len())
                .map(|_| paired_nucleotides[rng.random_range(0..paired_nucleotides.len())])
                .collect();
            let n = domain.len();
            for (k, &p) in big_pos.iter().enumerate() {
                mut_seq[p] = domain[k];
            }
            for (k, &p) in small_pos.iter().enumerate() {
                mut_seq[p] = comp_dict[&domain[n - 1 - k]];
            }
        } else {
            eprintln!(
                "mutate_seq: domain {big}/{small} has {} vs {} positions; \
                falling back to pairwise assignment",
                big_pos.len(),
                small_pos.len()
            );
            for &p in big_pos.iter().chain(small_pos.iter()) {
                mut_seq[p] = paired_nucleotides[rng.random_range(0..paired_nucleotides.len())];
            }
            // pair-map pass below does not touch these, so fix partners here
            for &p in big_pos.iter().chain(small_pos.iter()) {
                if let Some(&j) = pair_map.get(&p) {
                    if let Some(&c) = comp_dict.get(&mut_seq[p]) {
                        mut_seq[j] = c;
                    }
                }
            }
        }
    }

    let mut processed_pairs: HashSet<(usize, usize)> = HashSet::new();

    for &i in n_positions {
        if i >= mut_seq.len() {
            continue;
        }

        if let Some(&j) = pair_map.get(&i) {
            if j >= mut_seq.len() {
                continue;
            }

            let pair_key = (i.min(j), i.max(j));

            if !mutable_positions.contains(&j) {
                let fixed_partner = mut_seq[j];

                if let Some(&complement) = comp_dict.get(&fixed_partner) {
                    mut_seq[i] = complement;
                } else {
                    eprintln!(
                        "mutate_seq: position {j} is fixed but has invalid base '{fixed_partner}'"
                    );
                }

                continue;
            }

            if processed_pairs.contains(&pair_key) {
                continue;
            }

            processed_pairs.insert(pair_key);

            if tetra_loops && mutable_positions.contains(&i) && mutable_positions.contains(&(i+1)) && mutable_positions.contains(&(i+2)) 
            && mutable_positions.contains(&(i+3)) && mutable_positions.contains(&(i+4)) && mutable_positions.contains(&(i+5))
                && i + 5 < mut_struct.len()
                && mut_struct[i] == '('
                && mut_struct[i + 1] == '.'
                && mut_struct[i + 2] == '.'
                && mut_struct[i + 3] == '.'
                && mut_struct[i + 4] == '.'
                && mut_struct[i + 5] == ')'
            {
                mut_seq[i] = 'G';
                mut_seq[i + 1] = 'U';
                mut_seq[i + 2] = 'U';
                mut_seq[i + 3] = 'C';
                mut_seq[i + 4] = 'G';
                mut_seq[i + 5] = 'C';
            }

            let original_i = mut_seq[i];

            if original_i == 'K' {
                let choice = if rng.random::<f64>() < 0.5 { 'G' } else { 'U' };

                mut_seq[i] = choice;

                if rng.random::<f64>() < wobble_frequency && choice == 'G' {
                    mut_seq[j] = 'U';
                } else {
                    mut_seq[j] = comp_dict[&choice];
                }
            } else if original_i == 'S' {
                let choice = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };

                mut_seq[i] = choice;
                mut_seq[j] = comp_dict[&choice];
            } else if hard_loops
                && i + 1 < mut_struct.len()
                && mut_struct[i] == '('
                && mut_struct[i + 1] == ')'
            {
                let choice = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };

                mut_seq[i] = choice;
                mut_seq[j] = comp_dict[&choice];

                println!("HARD LOOP WARNING -> GC-PAIR!");

                if i + 2 < j {
                    if let Some(&partner_of_next) = pair_map.get(&(i + 2)) {
                        if partner_of_next == j - 2
                            && mutable_positions.contains(&(i + 2))
                            && mutable_positions.contains(&(j - 2))
                        {
                            let choice2 = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };

                            mut_seq[i + 2] = choice2;
                            mut_seq[j - 2] = comp_dict[&choice2];
                        }
                    }
                }

                if i + 3 < j {
                    if let Some(&partner_of_next) = pair_map.get(&(i + 3)) {
                        if partner_of_next == j - 3
                            && mutable_positions.contains(&(i + 3))
                            && mutable_positions.contains(&(j - 3))
                        {
                            let choice3 = if rng.random::<f64>() < 0.5 { 'G' } else { 'U' };

                            mut_seq[i + 3] = choice3;
                            mut_seq[j - 3] = comp_dict[&choice3];
                        }
                    }
                }
            } else {
                let near_loop =
                    is_adjacent_to_loop(i, structure) || is_adjacent_to_loop(j, structure);

                if near_loop {
                    let choice = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };

                    mut_seq[i] = choice;
                    mut_seq[j] = comp_dict[&choice];

                    if i + 1 < j {
                        if let Some(&partner_of_next) = pair_map.get(&(i + 1)) {
                            if partner_of_next == j - 1
                                && mutable_positions.contains(&(i + 1))
                                && mutable_positions.contains(&(j - 1))
                            {
                                let choice2 = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };

                                mut_seq[i + 1] = choice2;
                                mut_seq[j - 1] = comp_dict[&choice2];
                            }
                        }
                    }

                    if i + 2 < j {
                        if let Some(&partner_of_next) = pair_map.get(&(i + 2)) {
                            if partner_of_next == j - 2
                                && mutable_positions.contains(&(i + 2))
                                && mutable_positions.contains(&(j - 2))
                            {
                                let choice2 = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };

                                mut_seq[i + 2] = choice2;
                                mut_seq[j - 2] = comp_dict[&choice2];
                            }
                        }
                    }
                } else {
                    let choice = paired_nucleotides[rng.random_range(0..paired_nucleotides.len())];

                    mut_seq[i] = choice;

                    if rng.random::<f64>() < wobble_frequency && choice == 'G' {
                        mut_seq[j] = 'U';
                    } else {
                        mut_seq[j] = comp_dict[&choice];
                    }
                }
            }
        } else {
            mut_seq[i] = purines[rng.random_range(0..purines.len())];
        }
    }

    mut_seq.into_iter().collect()
}

pub fn mutate_ks(seq: &str, pair_map: &HashMap<usize, usize>) -> String {
    let chars: Vec<char> = seq.chars().collect();
    let mut out = chars.clone();
    let mut resolved = vec![false; chars.len()];
    let mut rng = rand::rng();

    for i in 0..chars.len() {
        if resolved[i] {
            continue;
        }
        if !matches!(chars[i], 'K' | 'S') {
            continue;
        }

        let Some(&j) = pair_map.get(&i) else {
            continue;
        };
        if resolved[j] {
            continue;
        }

        match (chars[i], chars[j]) {
            ('K', 'K') => {
                let (a, b) = if rng.random::<f64>() < 0.5 {
                    ('G', 'U')
                } else {
                    ('U', 'G')
                };
                out[i] = a;
                out[j] = b;
            }
            ('S', 'S') => {
                let (a, b) = if rng.random::<f64>() < 0.5 {
                    ('G', 'C')
                } else {
                    ('C', 'G')
                };
                out[i] = a;
                out[j] = b;
            }
            _ => {
                continue;
            }
        }

        resolved[i] = true;
        resolved[j] = true;
    }

    out.into_iter().collect()
}

pub fn mutate_ribosome(seq_in: &str, structure: &str, n_positions: &[usize]) -> String {
    let mut rng = rand::rng();
    let conserved_motifs = important_motifs();
    let pair_map = get_pair_map(structure);
    let conserved_positions = find_conserved_positions(seq_in, &conserved_motifs);
    let hard_loops: bool = find_hard_loops(structure);

    // O(1) membership test for the designable set
    let designable: HashSet<usize> = n_positions.iter().copied().collect();

    let mut comp_dict = HashMap::new();
    comp_dict.insert('A', 'U');
    comp_dict.insert('U', 'A');
    comp_dict.insert('G', 'C');
    comp_dict.insert('C', 'G');

    let _nucleotides = ['A', 'U', 'G', 'C'];
    let paired_nucleotides = ['G', 'C'];
    let purines = ['A', 'G'];

    let mut mut_seq: Vec<char> = seq_in.chars().collect();
    let mut_struct: Vec<char> = structure.chars().collect();

    for i in 0..mut_seq.len() {
        if !designable.contains(&i) {
            continue;
        }

        if conserved_positions.contains(&i) {
            continue;
        }

        if let Some(&j) = pair_map.get(&i) {
            if !designable.contains(&j) {
                // Only fix i from the 5' side to avoid double work
                if i < j {
                    mut_seq[i] = *comp_dict.get(&mut_seq[j]).unwrap();
                }
                continue;
            }

            if i >= j {
                continue;
            }

            if rng.random::<f64>() < 0.20 {
                let choice = paired_nucleotides[rng.random_range(0..paired_nucleotides.len())];
                mut_seq[i] = choice;
                mut_seq[j] = *comp_dict.get(&choice).unwrap();
            }

            if hard_loops && mut_struct[i] == '(' && mut_struct[i + 1] == ')' {
                let choice = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };
                mut_seq[i] = choice;
                mut_seq[i + 1] = *comp_dict.get(&choice).unwrap();
                println!("HARD LOOP WARNING -> GC-PAIR!");
                if i + 2 < j {
                    if let Some(&partner_of_next) = pair_map.get(&(i + 2)) {
                        if partner_of_next == j - 2
                            && designable.contains(&(i + 2))
                            && designable.contains(&(j - 2))
                        {
                            let choice2 = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };
                            mut_seq[i + 2] = choice2;
                            mut_seq[j - 2] = *comp_dict.get(&choice2).unwrap();
                        }
                    }
                }
                if i + 3 < j {
                    if let Some(&partner_of_next) = pair_map.get(&(i + 3)) {
                        if partner_of_next == j - 3
                            && designable.contains(&(i + 3))
                            && designable.contains(&(j - 3))
                        {
                            let choice3 = if rng.random::<f64>() < 0.5 { 'G' } else { 'U' };
                            mut_seq[i + 3] = choice3;
                            mut_seq[j - 3] = *comp_dict.get(&choice3).unwrap();
                        }
                    }
                }
            }

            let near_loop = is_adjacent_to_loop(i, structure) || is_adjacent_to_loop(j, structure);

            if near_loop {
                let choice = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };
                mut_seq[i] = choice;
                mut_seq[j] = *comp_dict.get(&choice).unwrap();

                if i + 1 < j {
                    if let Some(&partner_of_next) = pair_map.get(&(i + 1)) {
                        if partner_of_next == j - 1
                            && designable.contains(&(i + 1))
                            && designable.contains(&(j - 1))
                            && !conserved_positions.contains(&(i + 1))
                            && !conserved_positions.contains(&(j - 1))
                        {
                            let choice2 = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };
                            mut_seq[i + 1] = choice2;
                            mut_seq[j - 1] = *comp_dict.get(&choice2).unwrap();
                        }
                    }
                }

                if i + 2 < j {
                    if let Some(&partner_of_next) = pair_map.get(&(i + 2)) {
                        if partner_of_next == j - 2
                            && designable.contains(&(i + 2))
                            && designable.contains(&(j - 2))
                            && !conserved_positions.contains(&(i + 2))
                            && !conserved_positions.contains(&(j - 2))
                        {
                            let choice3 = if rng.random::<f64>() < 0.5 { 'G' } else { 'C' };
                            mut_seq[i + 2] = choice3;
                            mut_seq[j - 2] = *comp_dict.get(&choice3).unwrap();
                        }
                    }
                }
            }
        } else {
            // Unpaired designable position
            if rng.random::<f64>() < 0.20 {
                mut_seq[i] = purines[rng.random_range(0..purines.len())];
            }
        }
    }

    mut_seq.into_iter().collect()
}

pub fn insert_ribosome_sequence(seq: &str) -> String {
    let mut result = seq.as_bytes().to_vec(); // Only use as much of the ribosome sequence as we need 
    let ribosome = &RIBOSOME_SEQUENCE.as_bytes()[..seq.len()];
    for i in 0..result.len() {
        if result[i] == b'N' {
            result[i] = ribosome[i];
        }
    }
    String::from_utf8(result).unwrap()
}
