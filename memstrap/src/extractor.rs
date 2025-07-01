use crate::config::EncodingType;
use crate::error::Result;
use regex::Regex;
use std::collections::HashSet;
use encoding_rs::GBK;

/// Represents the encoding of a found string
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Ascii,
    Utf8,
    Utf16Le,
    Utf16Be,
    Gbk,
}

impl From<EncodingType> for Encoding {
    fn from(encoding_type: EncodingType) -> Self {
        match encoding_type {
            EncodingType::Ascii => Encoding::Ascii,
            EncodingType::Utf8 => Encoding::Utf8,
            EncodingType::Utf16Le => Encoding::Utf16Le,
            EncodingType::Utf16Be => Encoding::Utf16Be,
            EncodingType::Gbk => Encoding::Gbk,
        }
    }
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoding::Ascii => write!(f, "ASCII"),
            Encoding::Utf8 => write!(f, "UTF-8"),
            Encoding::Utf16Le => write!(f, "UTF-16LE"),
            Encoding::Utf16Be => write!(f, "UTF-16BE"),
            Encoding::Gbk => write!(f, "GBK"),
        }
    }
}

/// Represents a found string with its metadata
#[derive(Debug, Clone)]
pub struct FoundString {
    pub offset: u64,
    pub content: String,
    pub encoding: Encoding,
    pub byte_length: usize,
    pub context_before: Option<Vec<u8>>,
    pub context_after: Option<Vec<u8>>,
}

/// Configuration for string extraction
pub struct ExtractionConfig {
    pub min_len: usize,
    pub encodings: HashSet<Encoding>,
    pub search_pattern: Option<String>,
    pub regex_pattern: Option<Regex>,
    pub context_bytes: Option<usize>,
}

/// Main string extractor
pub struct StringExtractor {
    config: ExtractionConfig,
}

impl StringExtractor {
    /// Create a new string extractor with the given configuration
    pub fn new(
        min_len: usize,
        encodings: Vec<EncodingType>,
        search_pattern: Option<String>,
        use_regex: bool,
        context_bytes: Option<usize>,
    ) -> Result<Self> {
        let encodings: HashSet<Encoding> = encodings.into_iter().map(Encoding::from).collect();
        
        let regex_pattern = if use_regex && search_pattern.is_some() {
            Some(Regex::new(search_pattern.as_ref().unwrap())?)
        } else {
            None
        };

        let config = ExtractionConfig {
            min_len,
            encodings,
            search_pattern,
            regex_pattern,
            context_bytes,
        };

        Ok(StringExtractor { config })
    }

    /// Extract strings from a byte slice with a given base offset
    pub fn extract_strings(&self, data: &[u8], base_offset: u64) -> Vec<FoundString> {
        let mut results = Vec::with_capacity(1024); // Pre-allocate capacity

        // Extract ASCII/UTF-8 strings
        if self.config.encodings.contains(&Encoding::Ascii)
            || self.config.encodings.contains(&Encoding::Utf8) {
            results.extend(self.extract_ascii_utf8(data, base_offset));
        }

        // Extract UTF-16LE strings
        if self.config.encodings.contains(&Encoding::Utf16Le) {
            results.extend(self.extract_utf16le(data, base_offset));
        }

        // Extract UTF-16BE strings
        if self.config.encodings.contains(&Encoding::Utf16Be) {
            results.extend(self.extract_utf16be(data, base_offset));
        }

        // Extract GBK strings
        if self.config.encodings.contains(&Encoding::Gbk) {
            results.extend(self.extract_gbk(data, base_offset));
        }

        results
    }

    /// Extract ASCII and UTF-8 strings
    fn extract_ascii_utf8(&self, data: &[u8], base_offset: u64) -> Vec<FoundString> {
        let mut results = Vec::with_capacity(256);
        let mut i = 0;
        let data_len = data.len();

        while i < data_len {
            // Look for potential string start (printable ASCII)
            if self.is_printable_ascii(data[i]) {
                let start = i;
                let mut has_non_ascii = false;

                // Fast path: scan for ASCII printable characters
                while i < data_len {
                    let byte = data[i];

                    // Stop at null terminator or control characters (except space and tab)
                    if byte == 0 || (byte < 0x20 && byte != 0x09) {
                        break;
                    }

                    // For ASCII printable characters, continue
                    if self.is_printable_ascii(byte) || byte == 0x09 { // Include tab
                        i += 1;
                        continue;
                    }

                    // Mark that we found non-ASCII and break
                    if (byte & 0x80) != 0 {
                        has_non_ascii = true;
                        // Try to skip this UTF-8 sequence
                        if byte & 0xE0 == 0xC0 && i + 1 < data_len { // 2-byte sequence
                            i += 2;
                        } else if byte & 0xF0 == 0xE0 && i + 2 < data_len { // 3-byte sequence
                            i += 3;
                        } else if byte & 0xF8 == 0xF0 && i + 3 < data_len { // 4-byte sequence
                            i += 4;
                        } else {
                            break; // Invalid UTF-8
                        }
                    } else {
                        // Non-printable ASCII, stop
                        break;
                    }
                }

                let byte_length = i - start;
                if byte_length >= self.config.min_len {
                    let string_bytes = &data[start..i];

                    // Only validate UTF-8 if we found non-ASCII bytes
                    let (content, encoding) = if has_non_ascii {
                        match std::str::from_utf8(string_bytes) {
                            Ok(s) => (s.to_string(), Encoding::Utf8),
                            Err(_) => {
                                // Convert to ASCII, replacing invalid bytes
                                let ascii_string: String = string_bytes
                                    .iter()
                                    .map(|&b| if b.is_ascii_graphic() || b == b' ' || b == b'\t' {
                                        b as char
                                    } else {
                                        '?'
                                    })
                                    .collect();
                                (ascii_string, Encoding::Ascii)
                            }
                        }
                    } else {
                        // Pure ASCII, no need to validate UTF-8
                        let ascii_string = unsafe {
                            std::str::from_utf8_unchecked(string_bytes).to_string()
                        };
                        (ascii_string, Encoding::Ascii)
                    };

                    if self.matches_search_criteria(&content) {
                        let (context_before, context_after) = self.extract_context(data, start, i);
                        results.push(FoundString {
                            offset: base_offset + start as u64,
                            content,
                            encoding,
                            byte_length,
                            context_before,
                            context_after,
                        });
                    }
                }
            } else {
                i += 1;
            }
        }

        results
    }

    /// Extract UTF-16LE strings
    fn extract_utf16le(&self, data: &[u8], base_offset: u64) -> Vec<FoundString> {
        let mut results = Vec::new();
        let mut i = 0;

        while i + 1 < data.len() {
            // Look for potential UTF-16LE pattern (ASCII char followed by 0x00)
            if self.is_printable_ascii(data[i]) && data[i + 1] == 0x00 {
                let start = i;
                let mut utf16_bytes = Vec::new();

                // Collect UTF-16LE bytes
                while i + 1 < data.len() {
                    let low = data[i];
                    let high = data[i + 1];
                    
                    // Check for null terminator
                    if low == 0x00 && high == 0x00 {
                        break;
                    }
                    
                    // Check if it's a valid UTF-16LE character
                    if high == 0x00 && self.is_printable_ascii(low) {
                        utf16_bytes.push(low as u16);
                        i += 2;
                    } else {
                        // Try to decode as full UTF-16
                        let code_unit = u16::from_le_bytes([low, high]);
                        utf16_bytes.push(code_unit);
                        i += 2;
                        
                        // If it's not a simple ASCII pattern, be more conservative
                        if high != 0x00 {
                            break;
                        }
                    }
                }

                let byte_length = i - start;
                if utf16_bytes.len() >= self.config.min_len {
                    if let Ok(content) = String::from_utf16(&utf16_bytes) {
                        if self.matches_search_criteria(&content) {
                            let (context_before, context_after) = self.extract_context(data, start, i);
                            results.push(FoundString {
                                offset: base_offset + start as u64,
                                content,
                                encoding: Encoding::Utf16Le,
                                byte_length,
                                context_before,
                                context_after,
                            });
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        results
    }

    /// Extract UTF-16BE strings
    fn extract_utf16be(&self, data: &[u8], base_offset: u64) -> Vec<FoundString> {
        let mut results = Vec::new();
        let mut i = 0;

        while i + 1 < data.len() {
            // Look for potential UTF-16BE pattern (0x00 followed by ASCII char)
            if data[i] == 0x00 && self.is_printable_ascii(data[i + 1]) {
                let start = i;
                let mut utf16_bytes = Vec::new();

                // Collect UTF-16BE bytes
                while i + 1 < data.len() {
                    let high = data[i];
                    let low = data[i + 1];
                    
                    // Check for null terminator
                    if high == 0x00 && low == 0x00 {
                        break;
                    }
                    
                    // Check if it's a valid UTF-16BE character
                    if high == 0x00 && self.is_printable_ascii(low) {
                        utf16_bytes.push(low as u16);
                        i += 2;
                    } else {
                        // Try to decode as full UTF-16
                        let code_unit = u16::from_be_bytes([high, low]);
                        utf16_bytes.push(code_unit);
                        i += 2;
                        
                        // If it's not a simple ASCII pattern, be more conservative
                        if high != 0x00 {
                            break;
                        }
                    }
                }

                let byte_length = i - start;
                if utf16_bytes.len() >= self.config.min_len {
                    if let Ok(content) = String::from_utf16(&utf16_bytes) {
                        if self.matches_search_criteria(&content) {
                            let (context_before, context_after) = self.extract_context(data, start, i);
                            results.push(FoundString {
                                offset: base_offset + start as u64,
                                content,
                                encoding: Encoding::Utf16Be,
                                byte_length,
                                context_before,
                                context_after,
                            });
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        results
    }

    /// Extract GBK strings
    fn extract_gbk(&self, data: &[u8], base_offset: u64) -> Vec<FoundString> {
        let mut results = Vec::new();
        let mut i = 0;
        let data_len = data.len();

        while i < data_len {
            // Look for potential GBK string start
            // GBK first byte ranges: 0x81-0xFE
            if data[i] >= 0x81 && data[i] <= 0xFE {
                let start = i;
                let mut gbk_bytes = Vec::new();
                let mut consecutive_invalid = 0;
                const MAX_INVALID_BYTES: usize = 3; // Stop after too many invalid bytes
                const MAX_STRING_LENGTH: usize = 1024; // Prevent extremely long strings

                // Collect potential GBK bytes with limits
                while i < data_len && gbk_bytes.len() < MAX_STRING_LENGTH {
                    let byte = data[i];

                    // Check for null terminator or control characters
                    if byte == 0 || (byte < 0x20 && byte != 0x09) {
                        break;
                    }

                    // ASCII printable characters are valid in GBK
                    if byte >= 0x20 && byte <= 0x7E {
                        gbk_bytes.push(byte);
                        consecutive_invalid = 0;
                        i += 1;
                        continue;
                    }

                    // GBK double-byte character
                    if byte >= 0x81 && byte <= 0xFE && i + 1 < data_len {
                        let second_byte = data[i + 1];
                        // GBK second byte ranges: 0x40-0x7E, 0x80-0xFE
                        if (second_byte >= 0x40 && second_byte <= 0x7E) ||
                           (second_byte >= 0x80 && second_byte <= 0xFE) {
                            gbk_bytes.push(byte);
                            gbk_bytes.push(second_byte);
                            consecutive_invalid = 0;
                            i += 2;
                            continue;
                        }
                    }

                    // Invalid byte - increment counter and stop if too many
                    consecutive_invalid += 1;
                    if consecutive_invalid >= MAX_INVALID_BYTES {
                        break;
                    }

                    // Skip this invalid byte and continue
                    i += 1;
                }

                let byte_length = i - start;
                if gbk_bytes.len() >= self.config.min_len {
                    // Try to decode as GBK - allow some errors for robustness
                    let (decoded, _encoding, _had_errors) = GBK.decode(&gbk_bytes);
                    // Only reject if the string is mostly errors or empty
                    if !decoded.trim().is_empty() && decoded.chars().count() >= self.config.min_len / 2 {
                        let content = decoded.into_owned();
                        if self.matches_search_criteria(&content) {
                            let (context_before, context_after) = self.extract_context(data, start, i);
                            results.push(FoundString {
                                offset: base_offset + start as u64,
                                content,
                                encoding: Encoding::Gbk,
                                byte_length,
                                context_before,
                                context_after,
                            });
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        results
    }

    /// Check if a byte is a printable ASCII character
    fn is_printable_ascii(&self, byte: u8) -> bool {
        byte >= 0x20 && byte <= 0x7E
    }

    /// Check if a string matches the search criteria
    fn matches_search_criteria(&self, content: &str) -> bool {
        if let Some(ref regex) = self.config.regex_pattern {
            regex.is_match(content)
        } else if let Some(ref pattern) = self.config.search_pattern {
            content.contains(pattern)
        } else {
            true
        }
    }

    /// Extract context bytes around a found string
    fn extract_context(&self, data: &[u8], start: usize, end: usize) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
        if let Some(context_size) = self.config.context_bytes {
            let before_start = start.saturating_sub(context_size);
            let after_end = std::cmp::min(end + context_size, data.len());

            let context_before = if before_start < start {
                Some(data[before_start..start].to_vec())
            } else {
                None
            };

            let context_after = if end < after_end {
                Some(data[end..after_end].to_vec())
            } else {
                None
            };

            (context_before, context_after)
        } else {
            (None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EncodingType;

    #[test]
    fn test_ascii_extraction() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Ascii],
            None,
            false,
            None,
        ).unwrap();

        let data = b"Hello World! This is a test.";
        let results = extractor.extract_strings(data, 0);

        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.content.contains("Hello World!")));
        assert!(results.iter().any(|s| s.content.contains("This is a test.")));
    }

    #[test]
    fn test_utf8_extraction() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Utf8],
            None,
            false,
            None,
        ).unwrap();

        let data = "Hello 世界! Test string.".as_bytes();
        let results = extractor.extract_strings(data, 0);

        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.content.contains("Hello 世界!")));
    }

    #[test]
    fn test_utf16le_extraction() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Utf16Le],
            None,
            false,
            None,
        ).unwrap();

        // "Hello" in UTF-16LE
        let data = b"H\x00e\x00l\x00l\x00o\x00\x00\x00";
        let results = extractor.extract_strings(data, 0);

        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.content == "Hello"));
        assert!(results.iter().any(|s| s.encoding == Encoding::Utf16Le));
    }

    #[test]
    fn test_gbk_extraction() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Gbk],
            None,
            false,
            None,
        ).unwrap();

        // "你好世界" (Hello World) in GBK encoding
        let gbk_data = &[0xC4, 0xE3, 0xBA, 0xC3, 0xCA, 0xC0, 0xBD, 0xE7];
        let results = extractor.extract_strings(gbk_data, 0);

        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.content.contains("你好世界")));
        assert!(results.iter().any(|s| s.encoding == Encoding::Gbk));
    }

    #[test]
    fn test_search_functionality() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Ascii],
            Some("test".to_string()),
            false,
            None,
        ).unwrap();

        let data = b"Hello World! This is a test. Another string.";
        let results = extractor.extract_strings(data, 0);

        // Should only find strings containing "test"
        assert!(!results.is_empty());
        assert!(results.iter().all(|s| s.content.contains("test")));
    }

    #[test]
    fn test_regex_search() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Ascii],
            Some(r"\d+".to_string()),
            true,
            None,
        ).unwrap();

        let data = b"String with number 123 and another 456.";
        let results = extractor.extract_strings(data, 0);

        // Should only find strings containing numbers
        assert!(!results.is_empty());
        assert!(results.iter().all(|s| s.content.chars().any(|c| c.is_ascii_digit())));
    }

    #[test]
    fn test_minimum_length_filter() {
        let extractor = StringExtractor::new(
            10, // Minimum length of 10
            vec![EncodingType::Ascii],
            None,
            false,
            None,
        ).unwrap();

        let data = b"Hi! This is a longer string that should be found.";
        let results = extractor.extract_strings(data, 0);

        // All results should have at least 10 characters
        assert!(results.iter().all(|s| s.byte_length >= 10));
    }

    #[test]
    fn test_offset_calculation() {
        let extractor = StringExtractor::new(
            4,
            vec![EncodingType::Ascii],
            None,
            false,
            None,
        ).unwrap();

        let data = b"Start Hello World End";
        let base_offset = 100;
        let results = extractor.extract_strings(data, base_offset);

        // Check that offsets are calculated correctly
        assert!(results.iter().any(|s| s.offset >= base_offset));
        assert!(results.iter().any(|s| s.content.contains("Hello World")));
    }
}
