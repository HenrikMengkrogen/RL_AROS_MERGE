mod cleanup;
mod config;
mod decomposition;
mod design;
mod ffi;
mod folding;
mod hill_climb;
mod io;
mod mutation;
mod pseudoknot;
mod ribosome;
mod structure;
mod types;

use crate::config::{
    DEFAULT_N_RUNS, DEFAULT_N_STARTS, GC_TEST, MAX_STEPS, RIBOSOMAL_RNA,
    WOBBLE_FREQUENCY,
};
use crate::design::decomposed_hill_climb_design;
use crate::io::{
    gc_content, read_input_file, write_data,
};

use crate::mutation::{insert_ribosome_sequence, mutate_ks};
use crate::structure::get_pair_map;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("RNA design configuration");
    println!("Press Enter to accept a default value.");
    println!("========================================");

    let n_runs = DEFAULT_N_RUNS;

    
    let n_starts = DEFAULT_N_STARTS;

    println!(
        "\nConfiguration selected: N_RUNS={}, N_STARTS={}, MAX_STEPS={}\n",
        n_runs, n_starts, MAX_STEPS
    );

    let input_root: PathBuf = if Path::new("misc").is_dir() {
        PathBuf::from("misc")
    } else if Path::new("../misc").is_dir() {
        PathBuf::from("../misc")
    } else {
        panic!(
            "Could not find a misc directory. Looked in:\n\
            - {}\n\
            - {}",
            Path::new("misc").display(),
            Path::new("../misc").display(),
        );
    };

    println!(
        "Current working directory: {}",
        std::env::current_dir()
            .expect("could not determine current directory")
            .display()
    );

    println!("Input directory: {}", input_root.display());

    let input_dir = fs::read_dir(&input_root)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", input_root.display()));

    let mut input_files: Vec<PathBuf> = input_dir
        .filter_map(|entry| {
            let path = entry.ok()?.path();

            let is_input_file = path.is_file()
                && path.file_name()?.to_str()?.starts_with("input_")
                && path.extension()?.to_str()? == "txt";

            is_input_file.then_some(path)
        })
        .collect();

    input_files.sort();

    println!("Found {} input files.", input_files.len());

    if input_files.is_empty() {
        eprintln!(
            "No files matching input_*.txt were found in {}",
            input_root.display()
        );
    }

    let output_base = input_root.join("output");

    fs::create_dir_all(&output_base).unwrap_or_else(|e| {
        panic!(
            "Could not create output directory {}: {e}",
            output_base.display()
        )
    });

    println!("Output base directory: {}", output_base.display());

    let mut final_results = String::new();

    for run in 1..=n_runs {
        println!("\n########################################");
        println!("RUN {} of {}", run, n_runs);
        println!("########################################");

        let run_dir = output_base.join(format!("run_{}", run));
        fs::create_dir_all(&run_dir).expect("failed to create run directory");

        for input_path in &input_files {
            let input_path_str = input_path.to_str().expect("Invalid input file path");

            println!("\n========================================");
            println!("PROCESSING: {}", input_path_str);
            println!("========================================");

            
            
            

            let (seq, target_structure) = match read_input_file(input_path_str) {
                Ok((seq, target)) => {
                    println!("Sequence : {}", seq);
                    println!("Structure: {}", target);
                    (seq, target)
                }

                Err(e) => {
                    eprintln!("Failed to read {}: {}", input_path_str, e);
                    continue;
                }
            };

            if RIBOSOMAL_RNA {
                println!("====RIBOSOMAL SEQUENCE USED====");
            }

            if GC_TEST {
                println!("====INITIAL CANDIDATE WILL HAVE OVERLOAD OF GC-PAIRS====");
            }

            let ribo_positions: Vec<usize> = if RIBOSOMAL_RNA {
                seq.char_indices()
                    .filter(|(_, c)| *c == 'N')
                    .map(|(i, _)| i)
                    .collect()
            } else {
                Vec::new()
            };

            let result = if RIBOSOMAL_RNA {
                decomposed_hill_climb_design(
                    &mutate_ks(
                        &insert_ribosome_sequence(&seq),
                        &get_pair_map(&target_structure),
                    ),
                    &target_structure,
                    n_starts,
                    MAX_STEPS,
                    WOBBLE_FREQUENCY,
                    Some(&ribo_positions),
                )
            } else {
                decomposed_hill_climb_design(
                    &mutate_ks(&seq, &get_pair_map(&target_structure)),
                    &target_structure,
                    n_starts,
                    MAX_STEPS,
                    WOBBLE_FREQUENCY,
                    None,
                )
            };
            match &result {
                Ok(_design_result) => {
                    if let Ok(design_result) = &result {
                        if let Err(e) = write_data(&design_result.sequence, &target_structure) {
                            eprintln!("Could not write output: {e}");
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Design failed: {err}");
                }
            }

            
            
            match result {
                Ok(r) => {
                    println!("\n==== FINAL (run {}) ====", run);
                    println!("sequence     : {}", r.sequence);
                    println!("target       : {}", target_structure);
                    println!("mfe structure: {}", r.mfe_structure);
                    println!("bp_distance  : {}", r.bp_distance);
                    println!("mfe          : {:.2}", r.mfe);
                    println!("Ensemble Diversity: {:.2}", r.ensemble_diversity);
                    println!("slices       : {}", r.n_slices);
                    println!("Ribosomal RNA used: {}", RIBOSOMAL_RNA);

                    match gc_content(&r.sequence) {
                        Some(gc) => println!("GC Content: {:.2}%", gc),
                        None => println!("GC Content: no valid DNA bases found"),
                    }

                    if let Some(identity) = r.ribosome_identity {
                        let n_mismatched =
                            r.ribosome_mismatches.as_ref().map(|v| v.len()).unwrap_or(0);

                        println!(
                            "ribosome identity  : {:.1}% ({} mismatched positions)",
                            identity * 100.0,
                            n_mismatched
                        );

                        if let Some(mismatches) = &r.ribosome_mismatches {
                            if !mismatches.is_empty() {
                                println!("ribosome mismatches: {:?}", mismatches);
                            }
                        }
                    }

                    final_results.push_str(&format!(
                        "\n========================================\n\
                         INPUT: {}\n\
                         ==== FINAL (run {}) ====\n",
                        input_path_str, run
                    ));

                    final_results.push_str(&format!("sequence     : {}\n", r.sequence));
                    final_results.push_str(&format!("target       : {}\n", target_structure));
                    final_results.push_str(&format!("mfe structure: {}\n", r.mfe_structure));
                    final_results.push_str(&format!("bp_distance  : {}\n", r.bp_distance));
                    final_results.push_str(&format!("mfe          : {:.2}\n", r.mfe));
                    final_results.push_str(&format!(
                        "Ensemble Diversity: {:.2}\n",
                        r.ensemble_diversity
                    ));
                    final_results.push_str(&format!("slices       : {}\n", r.n_slices));
                    final_results.push_str(&format!("Ribosomal RNA used: {}\n", RIBOSOMAL_RNA));

                    match gc_content(&r.sequence) {
                        Some(gc) => {
                            final_results.push_str(&format!("GC Content: {:.2}%\n", gc));
                        }
                        None => {
                            final_results.push_str("GC Content: no valid DNA bases found\n");
                        }
                    }

                    if let Some(identity) = r.ribosome_identity {
                        let n_mismatched =
                            r.ribosome_mismatches.as_ref().map(|v| v.len()).unwrap_or(0);

                        final_results.push_str(&format!(
                            "ribosome identity  : {:.1}% ({} mismatched positions)\n",
                            identity * 100.0,
                            n_mismatched
                        ));

                        if let Some(mismatches) = &r.ribosome_mismatches {
                            if !mismatches.is_empty() {
                                final_results
                                    .push_str(&format!("ribosome mismatches: {:?}\n", mismatches));
                            }
                        }
                    }

                    

                    let input_filename = input_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output");

                    let output_filename = input_filename
                        .strip_prefix("input_")
                        .map(|n| format!("output_{}.txt", n))
                        .unwrap_or_else(|| "output.txt".to_string());

                    let output_path = run_dir.join(&output_filename);
                    

                    let mut file =
                        File::create(&output_path).expect("failed to create output file");

                    writeln!(file, "==== FINAL (run {}) ====", run).unwrap();
                    writeln!(file, "sequence      : {}", r.sequence).unwrap();
                    writeln!(file, "target        : {}", target_structure).unwrap();
                    writeln!(file, "mfe structure : {}", r.mfe_structure).unwrap();
                    writeln!(file, "bp_distance   : {}", r.bp_distance).unwrap();
                    writeln!(file, "mfe           : {:.2}", r.mfe).unwrap();
                    writeln!(file, "Ensemble Diversity : {:.2}", r.ensemble_diversity).unwrap();
                    writeln!(file, "slices        : {}", r.n_slices).unwrap();
                    writeln!(file, "Ribosomal RNA used: {}", RIBOSOMAL_RNA).unwrap();

                    match gc_content(&r.sequence) {
                        Some(gc) => writeln!(file, "GC Content: {:.2}%", gc).unwrap(),
                        None => writeln!(file, "GC Content: no valid DNA bases found").unwrap(),
                    }

                    if let Some(identity) = r.ribosome_identity {
                        let n_mismatched =
                            r.ribosome_mismatches.as_ref().map(|v| v.len()).unwrap_or(0);

                        writeln!(
                            file,
                            "ribosome identity  : {:.1}% ({} mismatched positions)",
                            identity * 100.0,
                            n_mismatched
                        )
                        .unwrap();

                        if let Some(mismatches) = &r.ribosome_mismatches {
                            if !mismatches.is_empty() {
                                writeln!(file, "ribosome mismatches: {:?}", mismatches).unwrap();
                            }
                        }
                    }

                    println!("\nOutput written to {}", output_path.display());
                    
                    
                }

                Err(e) => {
                    eprintln!("Design failed for {} (run {}): {e}", input_path_str, run);

                    // Optional: include failures in the final pager output too.
                    final_results.push_str(&format!(
                        "\n========================================\n\
                         INPUT: {}\n\
                         ==== FAILED (run {}) ====\n\
                         Error: {}\n",
                        input_path_str, run, e
                    ));
                }
            }
        }
    }

    
    

    Ok(())
}
