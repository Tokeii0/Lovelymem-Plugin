use memstrap::{StringExtractor, CsvOutput, config::EncodingType};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample data
    let sample_data = b"Hello World! This is a test file.\nEmail: user@example.com\nPassword: secret123\nPath: C:\\Windows\\System32\\notepad.exe";
    
    println!("=== Basic String Extraction Example ===\n");
    
    // Example 1: Extract all strings
    println!("1. Extracting all strings (minimum length 4):");
    let extractor = StringExtractor::new(
        4,
        vec![EncodingType::Ascii, EncodingType::Utf8],
        None,
        false,
    )?;
    
    let results = extractor.extract_strings(sample_data, 0);
    println!("Found {} strings:", results.len());
    for (i, result) in results.iter().take(5).enumerate() {
        println!("  {}. Offset: 0x{:X}, Content: \"{}\"", i + 1, result.offset, result.content);
    }
    
    // Example 2: Search for specific patterns
    println!("\n2. Searching for email addresses:");
    let email_extractor = StringExtractor::new(
        4,
        vec![EncodingType::Ascii, EncodingType::Utf8],
        Some(r"\w+@\w+\.\w+".to_string()),
        true, // Use regex
    )?;
    
    let email_results = email_extractor.extract_strings(sample_data, 0);
    println!("Found {} email patterns:", email_results.len());
    for result in &email_results {
        println!("  - Offset: 0x{:X}, Content: \"{}\"", result.offset, result.content);
    }
    
    // Example 3: Search for passwords
    println!("\n3. Searching for password-related strings:");
    let password_extractor = StringExtractor::new(
        4,
        vec![EncodingType::Ascii, EncodingType::Utf8],
        Some("password".to_string()),
        false, // Plain text search
    )?;
    
    let password_results = password_extractor.extract_strings(sample_data, 0);
    println!("Found {} password-related strings:", password_results.len());
    for result in &password_results {
        println!("  - Offset: 0x{:X}, Content: \"{}\"", result.offset, result.content);
    }
    
    // Example 4: UTF-16 string extraction
    println!("\n4. UTF-16 string extraction:");
    let utf16_data = b"H\x00e\x00l\x00l\x00o\x00 \x00W\x00o\x00r\x00l\x00d\x00\x00\x00";
    let utf16_extractor = StringExtractor::new(
        4,
        vec![EncodingType::Utf16Le],
        None,
        false,
    )?;
    
    let utf16_results = utf16_extractor.extract_strings(utf16_data, 0);
    println!("Found {} UTF-16 strings:", utf16_results.len());
    for result in &utf16_results {
        println!("  - Offset: 0x{:X}, Encoding: {}, Content: \"{}\"", 
                result.offset, result.encoding, result.content);
    }
    
    // Example 5: CSV output
    println!("\n5. CSV output example:");
    let csv_file = "example_output.csv";
    CsvOutput::write_to_file(
        std::path::Path::new(csv_file),
        &results,
        std::path::Path::new("sample_data.bin"),
    )?;
    
    println!("Results written to: {}", csv_file);
    
    // Read and display first few lines of CSV
    let csv_content = fs::read_to_string(csv_file)?;
    let lines: Vec<&str> = csv_content.lines().take(4).collect();
    println!("CSV content preview:");
    for line in lines {
        println!("  {}", line);
    }
    
    // Clean up
    fs::remove_file(csv_file).ok();
    
    println!("\n=== Example completed successfully! ===");
    
    Ok(())
}
