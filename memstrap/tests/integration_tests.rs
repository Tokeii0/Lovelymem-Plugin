use memstrap::{Config, StringExtractor, CsvOutput, FoundString};
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_end_to_end_extraction() {
    // Create a temporary test file
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_data = b"Hello World! This is a test file with various strings.\nEmail: test@example.com\nPassword: secret123\nPath: C:\\Windows\\System32";
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();
    
    // Create extractor
    let extractor = StringExtractor::new(
        4,
        vec![memstrap::config::EncodingType::Ascii, memstrap::config::EncodingType::Utf8],
        None,
        false,
    ).unwrap();
    
    // Read file and extract strings
    let file_content = fs::read(temp_file.path()).unwrap();
    let results = extractor.extract_strings(&file_content, 0);
    
    // Verify results
    assert!(!results.is_empty());
    assert!(results.iter().any(|s| s.content.contains("Hello World!")));
    assert!(results.iter().any(|s| s.content.contains("test@example.com")));
    assert!(results.iter().any(|s| s.content.contains("secret123")));
}

#[test]
fn test_csv_output() {
    // Create test data
    let found_strings = vec![
        FoundString {
            offset: 0,
            content: "Hello World".to_string(),
            encoding: memstrap::Encoding::Utf8,
            byte_length: 11,
        },
        FoundString {
            offset: 20,
            content: "Test String".to_string(),
            encoding: memstrap::Encoding::Ascii,
            byte_length: 11,
        },
    ];
    
    // Create temporary output file
    let temp_file = NamedTempFile::new().unwrap();
    let test_path = PathBuf::from("test.txt");
    
    // Write CSV
    CsvOutput::write_to_file(temp_file.path(), &found_strings, &test_path).unwrap();
    
    // Read and verify CSV content
    let csv_content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(csv_content.contains("FilePath,Offset(Hex),Offset(Dec),Encoding,Length,Content"));
    assert!(csv_content.contains("Hello World"));
    assert!(csv_content.contains("Test String"));
    assert!(csv_content.contains("0x0"));
    assert!(csv_content.contains("0x14"));
}

#[test]
fn test_search_filtering() {
    let test_data = b"Hello World! Email: user@domain.com. Password: secret123. Another string.";
    
    // Test plain text search
    let extractor = StringExtractor::new(
        4,
        vec![memstrap::config::EncodingType::Ascii],
        Some("Email".to_string()),
        false,
    ).unwrap();
    
    let results = extractor.extract_strings(test_data, 0);
    assert!(!results.is_empty());
    assert!(results.iter().all(|s| s.content.contains("Email")));
    
    // Test regex search for email pattern
    let extractor = StringExtractor::new(
        4,
        vec![memstrap::config::EncodingType::Ascii],
        Some(r"\w+@\w+\.\w+".to_string()),
        true,
    ).unwrap();
    
    let results = extractor.extract_strings(test_data, 0);
    assert!(!results.is_empty());
    // Should find strings containing email addresses
    assert!(results.iter().any(|s| s.content.contains("user@domain.com")));
}

#[test]
fn test_utf16_extraction() {
    // Create UTF-16LE test data: "Hello" in UTF-16LE
    let utf16le_data = b"H\x00e\x00l\x00l\x00o\x00 \x00W\x00o\x00r\x00l\x00d\x00\x00\x00";
    
    let extractor = StringExtractor::new(
        4,
        vec![memstrap::config::EncodingType::Utf16Le],
        None,
        false,
    ).unwrap();
    
    let results = extractor.extract_strings(utf16le_data, 0);
    assert!(!results.is_empty());
    assert!(results.iter().any(|s| s.content.contains("Hello World")));
    assert!(results.iter().any(|s| s.encoding == memstrap::Encoding::Utf16Le));
}

#[test]
fn test_minimum_length_filtering() {
    let test_data = b"Hi! This is a much longer string that should be found.";
    
    // Test with minimum length of 20
    let extractor = StringExtractor::new(
        20,
        vec![memstrap::config::EncodingType::Ascii],
        None,
        false,
    ).unwrap();
    
    let results = extractor.extract_strings(test_data, 0);
    
    // Should only find strings with at least 20 bytes
    assert!(results.iter().all(|s| s.byte_length >= 20));
    // Should not find "Hi!" as it's too short
    assert!(!results.iter().any(|s| s.content == "Hi!"));
}
