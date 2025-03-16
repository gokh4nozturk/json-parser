# Rust JSON Parser

This project is a simple JSON parser application developed as part of the Rust programming language learning process.

## Features

- Parse and display JSON data
- Read JSON from file or standard input
- Support for basic JSON data types:
  - Null
  - Boolean (true/false)
  - Number (integer and float)
  - String
  - Array
  - Object

## Usage

### Parse JSON from File

```bash
cargo run file.json
```

### Parse JSON from Standard Input

```bash
cargo run
# Enter JSON and end with Ctrl+D
```

## Project Structure

- `JsonValue`: Enum representing JSON data
- `Lexer`: Structure that separates JSON text into tokens
- `Parser`: Structure that converts tokens to JSON data structure

## Rust Concepts Learned

- Using Enums and Structs
- Pattern Matching
- Result and Error Handling
- Lifetime and Borrowing
- Trait implementation
- Generics
- Iterator usage

## Tests

To test the project:

```bash
cargo test
```

## Development

This project was developed as part of the Rust learning process. Features that could be added in the future:

- JSON data formatting (pretty printing)
- Writing JSON data to file
- More comprehensive error messages
- Performance improvements 