# CLI Search Utility

A command-line text search tool built in Rust, replicating core `grep` functionality with additional features for pattern matching and file traversal.

## Features

- **Pattern Matching**: Search for text patterns across single or multiple files
- **Case-Insensitive Search** (`-i`): Match patterns regardless of case
- **Line Numbers** (`-n`): Display line numbers for matched results
- **Inverted Match** (`-v`): Show lines that don't match the pattern
- **Recursive Search** (`-r`): Traverse directories and subdirectories
- **Colored Output** (`-c`): Highlight matched text in red for better visibility
- **Filename Display** (`-f`): Show which files contain matches
- **Help Menu** (`-h`,`--help`): Show help information

## Usage
```
cargo run -- [OPTIONS] <pattern> <files...>
```

## Examples

**Basic search:**
```bash
cargo run -- "pattern" file.txt
```

**Case-insensitive search with line numbers:**
```bash
cargo run -- -i -n "pattern" file.txt
```

**Recursive search in directory with colored output:**
```bash
cargo run -- -r -c -f "pattern" ./src
```

**Search multiple files:**
```bash
cargo run -- "pattern" file1.txt file2.txt
```

**Inverted match (lines NOT containing pattern):**
```bash
cargo run -- -v "pattern" file.txt
```

