use crate::structure::{assert_slices_balanced, strip_pseudoknots};
use crate::types::Substructure;

pub fn compute_partners(structure: &str) -> Option<Vec<Option<usize>>> {
    let b = structure.as_bytes();
    let n = b.len();
    let mut partner = vec![None; n];
    let mut paren_stack = Vec::new();
    let mut bracket_stack = Vec::new();

    for (i, &c) in b.iter().enumerate() {
        match c {
            b'(' => paren_stack.push(i),
            b')' => {
                let j = paren_stack.pop()?;
                partner[i] = Some(j);
                partner[j] = Some(i);
            }
            b'[' => bracket_stack.push(i),
            b']' => {
                let j = bracket_stack.pop()?;
                partner[i] = Some(j);
                partner[j] = Some(i);
            }
            b'.' | b'_' => {}
            _ => return None,
        }
    }
    if paren_stack.is_empty() && bracket_stack.is_empty() {
        Some(partner)
    } else {
        None
    }
}

pub fn find_blocks(partner: &[Option<usize>], b: &[u8]) -> Vec<(usize, usize)> {
    let n = partner.len();
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < n {
        let mut reach = i;
        let mut k = i;
        loop {
            if b[k] != b'[' && b[k] != b']' {
                if let Some(p) = partner[k] {
                    if p > reach {
                        reach = p;
                    }
                }
            }
            if k == reach {
                break;
            }
            k += 1;
        }
        blocks.push((i, reach));
        i = reach + 1;
    }
    blocks
}

pub fn decompose(structure: &str) -> Result<Vec<Substructure>, &'static str> {
    let partner = compute_partners(structure).ok_or("unbalanced or unsupported structure")?;
    let b = structure.as_bytes();
    let mut out = Vec::new();

    for (start, reach) in find_blocks(&partner, b) {
        if start == reach {
            continue; // single unpaired position
        }
        if b[start] == b'(' && partner[start] == Some(reach) {
            process_stem_loop(b, &partner, start, reach, &mut out);
        } else {
            let designable: Vec<usize> = (start..=reach).collect();
            out.push(Substructure {
                start,
                end: reach + 1,
                structure: std::str::from_utf8(&b[start..=reach]).unwrap().to_string(),
                designable,
            });
        }
    }

    for sub in out.iter_mut() {
        sub.structure = strip_pseudoknots(&sub.structure);
    }

    assert_slices_balanced(&out).map_err(|e| {
        eprintln!("{e}");
        "unbalanced slice produced by decompose"
    })?;

    Ok(out)
}

pub fn _process_stem_loop(
    b: &[u8],
    partner: &[Option<usize>],
    i: usize,
    j: usize,
    out: &mut Vec<Substructure>,
) {
    let mut i_inner = i;
    let mut j_inner = j;
    while i_inner + 1 < j_inner
        && b[i_inner + 1] == b'('
        && b[j_inner - 1] == b')'
        && partner[i_inner + 1] == Some(j_inner - 1)
    {
        i_inner += 1;
        j_inner -= 1;
    }

    let mut designable = Vec::new();
    for k in i..=i_inner {
        designable.push(k);
    }
    for k in j_inner..=j {
        designable.push(k);
    }

    let mut k = i_inner + 1;
    while k < j_inner {
        match b[k] {
            b'(' => {
                let l = partner[k].expect("unmatched '(' inside slice");
                process_stem_loop(b, partner, k, l, out);
                k = l + 1;
            }
            b'[' => {
                if let Some(l) = partner[k] {
                    let lo = k.min(l);
                    let hi = k.max(l);
                    let designable_pk: Vec<usize> = (lo..=hi).collect();
                    out.push(Substructure {
                        start: lo,
                        end: hi + 1,
                        structure: std::str::from_utf8(&b[lo..=hi]).unwrap().to_string(),
                        designable: designable_pk,
                    });
                    k = hi + 1;
                } else {
                    designable.push(k);
                    k += 1;
                }
            }

            b')' => {
                designable.push(k);
                k += 1;
            }
            _ => {
                designable.push(k);
                k += 1;
            }
        }
    }

    out.push(Substructure {
        start: i,
        end: j + 1,
        structure: std::str::from_utf8(&b[i..=j]).unwrap().to_string(),
        designable,
    });
}

pub fn process_stem_loop(
    b: &[u8],
    partner: &[Option<usize>],
    i: usize,
    j: usize,
    out: &mut Vec<Substructure>,
) {
    let mut i_inner = i;
    let mut j_inner = j;
    while i_inner + 1 < j_inner && partner[i_inner + 1] == Some(j_inner - 1) {
        i_inner += 1;
        j_inner -= 1;
    }

    let mut designable = Vec::new();

    for k in i..=i_inner {
        designable.push(k);
    }

    for k in j_inner..=j {
        designable.push(k);
    }

    let mut k = i_inner + 1;
    while k < j_inner {
        if b[k] == b'(' {
            let l = partner[k].unwrap();
            process_stem_loop(b, partner, k, l, out);
            k = l + 1;
        } else {
            designable.push(k);
            k += 1;
        }
    }

    out.push(Substructure {
        start: i,
        end: j + 1,
        structure: std::str::from_utf8(&b[i..=j]).unwrap().to_string(),
        designable,
    });
}
