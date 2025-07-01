# memstrap

A high-performance CLI tool for memory forensics string extraction.

## Features

- **Fast**: Multi-threaded processing with efficient I/O operations for GB to TB sized memory images
- **Specialized**: Focused on memory forensics scenarios with support for multiple encodings
- **Powerful**: Extract strings and perform high-speed searches (plain text and regex)
- **Practical**: Structured CSV output for analysis in Excel, Python/Pandas, etc.

## Supported Encodings

- ASCII (7-bit)
- UTF-8
- UTF-16 Little Endian
- UTF-16 Big Endian
- GBK (Chinese character encoding)

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Basic usage - extract all strings
memstrap memory_dump.raw

# Extract strings with minimum length of 8 characters
memstrap memory_dump.raw -n 8

# Search for specific patterns
memstrap memory_dump.raw -s "password"

# Use regex search
memstrap memory_dump.raw -s "\w+@\w+\.\w+" -r

# Output to CSV file
memstrap memory_dump.raw -o results.csv

# Use specific number of threads
memstrap memory_dump.raw -j 8

# Search only ASCII strings
memstrap memory_dump.raw -e ascii

# Disable progress bar
memstrap memory_dump.raw --no-progress
```

## Command Line Options

```
Usage: memstrap [OPTIONS] <FILE_PATH>

Arguments:
  <FILE_PATH>  Path to the memory image or file to scan

Options:
  -o, --output <FILE>         Output CSV file path (defaults to stdout)
  -n, --min-len <LENGTH>      Minimum string length to extract [default: 4]
  -j, --threads <NUM>         Number of threads to use (defaults to CPU core count)
  -s, --search <PATTERN>      Search pattern (can be plain text or regex)
  -r, --regex                 Interpret search pattern as regex
      --no-progress           Disable progress bar
  -e, --encoding <ENCODINGS>  Encoding types to search for [possible values: ascii, utf8, utf16le, utf16be, gbk]
  -h, --help                  Print help
  -V, --version               Print version
```

## Output Format

The tool outputs results in CSV format with the following columns:

- **FilePath**: Path to the input file
- **Offset(Hex)**: Hexadecimal offset where the string was found
- **Offset(Dec)**: Decimal offset where the string was found
- **Encoding**: Detected encoding (ASCII, UTF-8, UTF-16LE, UTF-16BE, GBK)
- **Length**: Length of the string in bytes
- **Content**: The extracted string content

## Performance

The tool uses memory mapping and parallel processing to handle large files efficiently:

- Memory mapping avoids loading entire files into RAM
- Parallel processing utilizes multiple CPU cores
- Chunk overlap prevents string splitting at boundaries
- Progress bar shows processing status for large files

## Examples

### Extract all strings from a memory dump
```bash
memstrap memory.raw -o strings.csv
```

### Find email addresses
```bash
memstrap memory.raw -s "\w+@\w+\.\w+" -r -o emails.csv
```

### Find passwords (case-insensitive)
```bash
memstrap memory.raw -s "(?i)password" -r -o passwords.csv
```

### Extract only UTF-16 strings
```bash
memstrap memory.raw -e utf16le -e utf16be -o utf16_strings.csv
```

### Extract Chinese text (GBK encoding)
```bash
memstrap memory.raw -e gbk -o chinese_strings.csv
```

## License

This project is licensed under the MIT License.
