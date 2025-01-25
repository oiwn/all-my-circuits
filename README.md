[![codecov](https://codecov.io/gh/oiwn/all-my-circuits/graph/badge.svg?token=RQJ86YJPL0)](https://codecov.io/gh/oiwn/all-my-circuits)
[![dependency status](https://deps.rs/repo/github/oiwn/all-my-circuits/status.svg)](https://deps.rs/repo/github/oiwn/all-my-circuits)

# File Concatenator with Git Metadata

🚀 BLAZINGLY FAST 🚀 file concatenator built with 🦀 Rust 🦀! ⚡️ Combines your
files into a single output while preserving Git history metadata with
ZERO-COST ABSTRACTIONS and MEMORY SAFETY guarantees! ⚡️ Designed to
create rich, contextual file dumps with Git metadata that serve as perfect
context windows for Large Language Models — because your AI assistant
deserves to know not just WHAT your code is,
but WHEN and WHY it was written!

## Features

- 🚶‍♂️ Recursively walks through directories
- 🎯 Filters files by extension
- 📝 Adds Git metadata headers (commit hash and timestamp)
- ⚙️ Configurable via TOML file
- 🛠️ Customizable delimiter for file sections

## Installation

```bash
cargo install all-my-circuits
```

## Quick Start

1. Create a configuration file `.amc.toml` (in current directory):

```toml
delimiter = "---"
extensions = ["rs", "ts", "py"]
```

2. Run the tool:

```bash
# Scan current directory with default config
amc

# Scan specific directory with custom config
amc --dir ./src --config custom-config.toml
```

## Output Format

The tool generates output in the following format:

```
---
File: src/main.rs
Last commit: 623a9e4b9dbdfa9367232ba67e7abe90245c2948
Last update: 1729838996
---
<file contents>

---
File: src/walk.rs
Last commit: 623a9e4b9dbdfa9367232ba67e7abe90245c2948
Last update: 1729838996
---
<file contents>
```

## CLI Options

```
Usage: amc [OPTIONS]

Options:
  -d, --dir <DIR>      Directory to scan [default: .]
  -c, --config <FILE>  Config file path [default: .amc.toml]
  -h, --help          Print help
  -V, --version       Print version
```

## Configuration

Create a `.amc.toml` file with the following options:

```toml
# String used to separate file sections
delimiter = "---"

# List of file extensions to process (without dots)
extensions = ["rs", "ts", "py", "md"]
```

## Use Cases

- Generate documentation with context
- Create annotated source code compilations
- Prepare code for review with Git history
- Archive project snapshots with metadata
- Create meaningful diffs with context

## Error Handling

The tool provides friendly error messages for common issues:
- Missing configuration file
- Invalid directory paths
- Git repository access problems
- File reading permissions

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
