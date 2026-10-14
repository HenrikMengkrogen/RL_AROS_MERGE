#[derive(Debug, Clone)]
pub struct GlobalDesignResult {
    pub sequence: String,
    pub mfe_structure: String,
    pub bp_distance: i64,
    pub mfe: f64,
    pub n_slices: usize,
    pub ribosome_identity: Option<f64>,
    pub ribosome_mismatches: Option<Vec<usize>>,
    pub ensemble_diversity: f64,
}

#[derive(Debug, Clone)]
pub struct Substructure {
    pub start: usize,
    pub end: usize,
    pub structure: String,
    pub designable: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CandidateScore {
    pub structure: String,
    pub mfe: f64,
    pub total_distance: i64,
    pub energy_gap: f64,
}

#[derive(Debug, Clone)]
pub struct DesignResult {
    pub bp_distance: i64,
    pub structure: String,
    pub mfe: f64,
    pub sequence: String,
}

#[derive(Debug)]
pub struct RibosomeSimilarity {
    pub matches: usize,
    pub total: usize,
    pub identity: f64,
    pub mismatched_positions: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct PkPrediction {
    pub structure: String,
    pub energy: f64,
    pub dgpk: f64,
    pub dgint: f64,
    pub dg1: f64,
    pub dg2: f64,
    pub start_5: usize,
    pub end_5: usize,
    pub start_3: usize,
    pub end_3: usize,
}
