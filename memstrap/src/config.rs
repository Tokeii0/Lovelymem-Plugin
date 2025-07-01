use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Supported string encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EncodingType {
    #[value(name = "ascii")]
    Ascii,
    #[value(name = "utf8")]
    Utf8,
    #[value(name = "utf16le")]
    Utf16Le,
    #[value(name = "utf16be")]
    Utf16Be,
    #[value(name = "gbk")]
    Gbk,
}

/// Command line configuration
#[derive(Parser, Debug)]
#[command(name = "memstrap")]
#[command(about = "High-performance CLI tool for memory forensics string extraction")]
#[command(version = "0.1.0")]
pub struct Config {
    /// Path to the memory image or file to scan
    #[arg(value_name = "FILE_PATH")]
    pub file_path: PathBuf,

    /// Output CSV file path (defaults to stdout)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Minimum string length to extract
    #[arg(short = 'n', long = "min-len", default_value = "4", value_name = "LENGTH")]
    pub min_len: usize,

    /// Number of threads to use (defaults to CPU core count)
    #[arg(short = 'j', long = "threads", value_name = "NUM")]
    pub threads: Option<usize>,

    /// Search pattern (can be plain text or regex)
    #[arg(short = 's', long = "search", value_name = "PATTERN")]
    pub search: Option<String>,

    /// Interpret search pattern as regex
    #[arg(short = 'r', long = "regex")]
    pub regex: bool,

    /// Disable progress bar
    #[arg(long = "no-progress")]
    pub no_progress: bool,

    /// Encoding types to search for
    #[arg(short = 'e', long = "encoding", value_enum)]
    pub encodings: Vec<EncodingType>,

    /// Show context bytes around found strings (hex format)
    #[arg(short = 'C', long = "context", value_name = "NUM_BYTES")]
    pub context_bytes: Option<usize>,
}

impl Config {
    /// Get the list of encodings to search for, defaulting to original four if none specified
    pub fn get_encodings(&self) -> Vec<EncodingType> {
        if self.encodings.is_empty() {
            // Default to original four encodings (GBK is optional)
            vec![
                EncodingType::Ascii,
                EncodingType::Utf8,
                EncodingType::Utf16Le,
                EncodingType::Utf16Be,
            ]
        } else {
            self.encodings.clone()
        }
    }

    /// Get the number of threads to use, defaulting to CPU core count
    pub fn get_threads(&self) -> usize {
        self.threads.unwrap_or_else(|| {
            // Default to CPU core count but cap at 8 for better performance
            std::cmp::min(num_cpus::get(), 8)
        })
    }
}
