use crate::extractor::FoundString;
use crate::error::Result;
use csv::Writer;
use std::io::{self, Write};
use std::path::Path;

/// CSV output handler
pub struct CsvOutput;

impl CsvOutput {
    /// Write found strings to CSV format
    pub fn write_results<W: Write>(
        writer: W,
        results: &[FoundString],
        file_path: &Path,
    ) -> Result<()> {
        let mut csv_writer = Writer::from_writer(writer);

        // Write header
        csv_writer.write_record(&[
            "FilePath",
            "Offset(Hex)",
            "Offset(Dec)",
            "Encoding",
            "Length",
            "Content",
        ])?;

        // Write data rows
        for found_string in results {
            csv_writer.write_record(&[
                file_path.to_string_lossy().as_ref(),
                &format!("0x{:X}", found_string.offset),
                &found_string.offset.to_string(),
                &found_string.encoding.to_string(),
                &found_string.byte_length.to_string(),
                &found_string.content,
            ])?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    /// Write results to a file
    pub fn write_to_file(
        output_path: &Path,
        results: &[FoundString],
        file_path: &Path,
    ) -> Result<()> {
        let file = std::fs::File::create(output_path)?;
        Self::write_results(file, results, file_path)
    }

    /// Write results to stdout
    pub fn write_to_stdout(results: &[FoundString], file_path: &Path) -> Result<()> {
        let stdout = io::stdout();
        let handle = stdout.lock();
        Self::write_results(handle, results, file_path)
    }
}
