//! memstrap - High-performance CLI tool for memory forensics string extraction
//! 
//! This library provides functionality for extracting strings from memory images
//! and large files with support for multiple encodings and parallel processing.

pub mod config;
pub mod extractor;
pub mod output;
pub mod error;

pub use config::Config;
pub use extractor::{StringExtractor, FoundString, Encoding};
pub use output::CsvOutput;
pub use error::{MemstrapError, Result};
