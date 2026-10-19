pub const RIBOSOMAL_RNA: bool = false;
// RIBOSOMA_SEQUENCE can be changed to any start sequence of desire. If it is longer than the structure the sequence will be sliced accordingly.
pub const RIBOSOME_SEQUENCE: &str = "GGCGCCCGCCCCGCGCCGGGGGCCGCCGGCCGGCGGCCCCGGGGGCCCCCCGCCCCCCCGCCCCGCGCGCCCCGCGCGGCGCGGGCCGCGGGCCGGGGGCGGCGCCGGGGCCCCCCCGCCCCCGGCCGGGCCCGGGGCCCGCGCCGGCCCCCCCCGGCGCCGGCGGGCGGGGCCCGCGCCCCGCCCCCGCGCGGCCCGCCCGCCGCGCCCCCCGCCGCCCGGC";
pub const GC_TEST: bool = false; // This is just if you want your start sequence to be purely paired GC-pairs
pub const GC_THRESHOLD: f64 = 55.00; // must be a float
pub const GC_LOWER_THRESHOLD: f64 = 30.00; // must be a float
pub const DEFAULT_N_RUNS: usize = 20; // Number of times each file should be run -> Default: 3
pub const DEFAULT_N_STARTS: i64 = 10; // How many times in paralell the software should process each sequence -> Default: 5
pub const MAX_STEPS: i64 = 150;
pub const WOBBLE_FREQUENCY: f64 = 0.0;


pub const USE_ML_MODEL: bool = false;
