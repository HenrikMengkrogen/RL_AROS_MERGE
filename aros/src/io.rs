use std::fs::{self};
use std::io::{Write};
use crate::fs::OpenOptions;

use std::path::Path;





pub fn read_input_file(path: &str) -> Result<(String, String), std::io::Error> {
    let content = fs::read_to_string(path)?;

    let mut seq = String::new();
    let mut target = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("Sequence") {
            if let Some(idx) = line.find(':') {
                seq = line[idx + 1..].trim().to_string();
            }
        } else if line.starts_with("Structure") {
            if let Some(idx) = line.find(':') {
                target = line[idx + 1..].trim().to_string();
            }
        }
    }

    Ok((seq, target))
}



pub fn gc_content(sequence: &str) -> Option<f64> {
    let mut gc_count = 0usize;
    let mut base_count = 0usize;

    for base in sequence.bytes() {
        match base.to_ascii_uppercase() {
            b'G' | b'C' => {
                gc_count += 1;
                base_count += 1;
            }
            b'A' | b'U' => {
                base_count += 1;
            }
            _ => {}
        }
    }

    if base_count == 0 {
        return None;
    }

    let percentage = (gc_count as f64 / base_count as f64) * 100.0;
    Some((percentage * 100.0).round() / 100.0)
}

pub fn write_data(sequence: &str, structure: &str) -> Result<(), std::io::Error> {
    let positive: &str = "misc/temp_folder/temp_file.txt";
    let negative: &str = "Python/Training_data/training_data_negative.txt";
    

    if let Some(dir) = Path::new(positive).parent() {
        fs::create_dir_all(dir)?;
    }
    if let Some(dir) = Path::new(negative).parent() {
        fs::create_dir_all(dir)?;
    }

    
        let mut file = OpenOptions::new().create(true).append(true).open(positive)?;
    

        writeln!(file, "Sequence: {}", sequence)?;
        writeln!(file, "Structure: {}", structure)?;
        writeln!(file)?;
        Ok(())
    

    
}

