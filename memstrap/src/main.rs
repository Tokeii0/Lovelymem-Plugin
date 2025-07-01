use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use memstrap::{Config, StringExtractor, CsvOutput, FoundString, Result};

fn main() -> Result<()> {
    let config = Config::parse();

    // Validate input file
    if !config.file_path.exists() {
        eprintln!("Error: File '{}' does not exist", config.file_path.display());
        std::process::exit(1);
    }

    if !config.file_path.is_file() {
        eprintln!("Error: '{}' is not a regular file", config.file_path.display());
        std::process::exit(1);
    }

    // Open and memory-map the file
    let file = File::open(&config.file_path).map_err(|e| {
        eprintln!("Error opening file '{}': {}", config.file_path.display(), e);
        std::process::exit(1);
    }).unwrap();

    let mmap = unsafe {
        Mmap::map(&file).map_err(|e| {
            eprintln!("Error mapping file '{}': {}", config.file_path.display(), e);
            std::process::exit(1);
        }).unwrap()
    };

    println!("Processing file: {}", config.file_path.display());
    println!("File size: {} bytes ({:.2} MB)", mmap.len(), mmap.len() as f64 / 1024.0 / 1024.0);

    // Create string extractor
    let extractor = StringExtractor::new(
        config.min_len,
        config.get_encodings(),
        config.search.clone(),
        config.regex,
    ).map_err(|e| {
        eprintln!("Error creating string extractor: {}", e);
        std::process::exit(1);
    }).unwrap();

    // Calculate chunks for parallel processing
    let max_threads = config.get_threads();
    // For large files, limit threads to avoid excessive overhead
    let optimal_threads = if mmap.len() > 100 * 1024 * 1024 { // > 100MB
        std::cmp::min(max_threads, 8) // Limit to 8 threads for large files
    } else {
        max_threads
    };

    // Use larger chunk sizes for better performance
    let min_chunk_size = 16 * 1024 * 1024; // 16MB minimum chunk size
    let num_threads = if mmap.len() < min_chunk_size {
        1
    } else {
        std::cmp::min(optimal_threads, mmap.len() / min_chunk_size)
    };

    let chunk_size = if num_threads == 1 { mmap.len() } else { mmap.len() / num_threads };
    let overlap_size = 4096; // Larger overlap for better string detection

    println!("Using {} threads", num_threads);
    println!("Chunk size: {:.2} MB", chunk_size as f64 / 1024.0 / 1024.0);
    println!("Minimum string length: {}", config.min_len);
    if let Some(ref pattern) = config.search {
        println!("Search pattern: {} ({})", pattern, if config.regex { "regex" } else { "plain text" });
    }
    println!("Encodings: {:?}", config.get_encodings());

    // Create progress bar
    let progress = if !config.no_progress {
        let pb = ProgressBar::new(num_threads as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks processed ({eta}) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Extracting strings...");
        Some(pb)
    } else {
        None
    };

    // Create chunks with overlap
    let chunks: Vec<(usize, usize, u64)> = (0..num_threads)
        .map(|i| {
            let start = i * chunk_size;
            let end = if i == num_threads - 1 {
                mmap.len()
            } else {
                std::cmp::min((i + 1) * chunk_size + overlap_size, mmap.len())
            };
            (start, end, start as u64)
        })
        .collect();

    // Process chunks in parallel
    let processed_count = Arc::new(AtomicUsize::new(0));
    let progress_ref = Arc::new(progress);

    let results: Vec<FoundString> = chunks
        .par_iter()
        .enumerate()
        .flat_map(|(chunk_idx, (start, end, base_offset))| {
            let chunk_data = &mmap[*start..*end];
            let chunk_results = extractor.extract_strings(chunk_data, *base_offset);

            // Update progress less frequently to reduce overhead
            let count = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
            if let Some(ref pb) = progress_ref.as_ref() {
                // Only update progress every few chunks or for the last chunk
                if chunk_idx % std::cmp::max(1, num_threads / 4) == 0 || count == num_threads {
                    pb.set_position(count as u64);
                }
            }

            chunk_results
        })
        .collect();

    if let Some(pb) = progress_ref.as_ref() {
        pb.finish_with_message("Processing complete!");
    }

    // Remove duplicates (can happen due to overlap)
    let mut unique_results: Vec<FoundString> = results;
    unique_results.sort_by_key(|s| s.offset);
    let original_count = unique_results.len();
    unique_results.dedup_by_key(|s| s.offset);
    let final_count = unique_results.len();

    println!("\nResults:");
    println!("  Total strings found: {}", final_count);
    if original_count != final_count {
        println!("  Duplicates removed: {}", original_count - final_count);
    }

    // Output results
    if let Some(output_path) = &config.output {
        CsvOutput::write_to_file(output_path, &unique_results, &config.file_path).map_err(|e| {
            eprintln!("Error writing to file '{}': {}", output_path.display(), e);
            std::process::exit(1);
        }).unwrap();
        println!("  Results written to: {}", output_path.display());
    } else {
        CsvOutput::write_to_stdout(&unique_results, &config.file_path).map_err(|e| {
            eprintln!("Error writing to stdout: {}", e);
            std::process::exit(1);
        }).unwrap();
    }

    Ok(())
}
