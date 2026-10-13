use crate::config::RIBOSOME_SEQUENCE;
use crate::types::RibosomeSimilarity;

pub fn ribosome_similarity(seq_in: &str, n_positions: &[usize]) -> RibosomeSimilarity {
    let seq_bytes = seq_in.as_bytes();
    let ref_bytes = RIBOSOME_SEQUENCE.as_bytes();

    assert!(
        seq_bytes.len() <= ref_bytes.len(),
        "seq_in ({} nt) is longer than the reference ribosome sequence ({} nt)",
        seq_bytes.len(),
        ref_bytes.len()
    );

    let mut mismatched_positions = Vec::new();
    for &pos in n_positions {
        assert!(
            pos < seq_bytes.len(),
            "position {} out of bounds for seq_in",
            pos
        );
        if seq_bytes[pos] != ref_bytes[pos] {
            mismatched_positions.push(pos);
        }
    }

    let total = n_positions.len();
    let matches = total - mismatched_positions.len();
    let identity = if total == 0 {
        1.0
    } else {
        matches as f64 / total as f64
    };

    RibosomeSimilarity {
        matches,
        total,
        identity,
        mismatched_positions,
    }
}
