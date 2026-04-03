# CSV Convert

[![Rust](https://img.shields.io/badge/rust-1.56%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust-based Terminal User Interface (TUI) application for data conversion and processing between CSV and JSON, YAML, and TOML formats. Provides an intuitive interactive interface with support for file browsing, data preview, and format conversion.

## ✨ Features

- **Multi-format Support**: Supports conversion from CSV to JSON, YAML, and TOML formats
- **Interactive TUI Interface**: Modern terminal user interface based on Ratatui
- **File Browser**: Built-in file browser for easy selection of input/output files
- **Real-time Preview**: Supports data preview with syntax highlighting
- **Keyboard-Driven**: Fully keyboard-operated, no mouse required
- **Cross-Platform**: Supports macOS, Linux, and Windows
- **Efficient Processing**: Supports large file processing with memory-mapped optimization for performance

## 🛠 Technology Stack

- **Rust** - Systems programming language providing memory safety and high performance
- **Ratatui** - Modern Rust TUI framework
- **Crossterm** - Cross-platform terminal handling library
- **Serde** - Rust serialization framework supporting multiple data formats
- **Syntect** - Syntax highlighting library

## 📁 Project Structure

```
csv_convert/
├── src/
│   ├── main.rs      # Application entry point and event loop
│   ├── app.rs       # Application logic and state management
│   ├── ui.rs        # UI rendering logic
│   └── utils.rs     # Utility functions and data processing
├── assets/
│   ├── input.csv    # CSV format sample data
│   ├── input.json   # JSON format sample data
│   ├── input.toml   # TOML format sample data
│   └── input.yaml   # YAML format sample data
├── Cargo.toml       # Project dependency configuration
├── LICENSE          # MIT License
└── README.md        # Project documentation
```

## 🚀 Installation and Running

### Prerequisites

- Rust 1.56 or higher
- Unicode-supporting terminal (Nerd Font recommended for best visual experience)

### Installing Rust

If Rust is not installed, visit [rustup.rs](https://rustup.rs/) for installation:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building the Project

```bash
# Clone the repository
git clone https://github.com/gushiii/csv_convert.git
cd csv_convert

# Build release version
cargo build --release
```

### Running the Application

```bash
# Run directly
cargo run

# Or run the built binary
./target/release/csv_convert
```

## 📖 Usage

### Basic Operations

After starting the application, you will see a split-pane interface:

1. **File Browser** (left): Browse and select input files
2. **Data Preview** (right): Display selected file content with syntax highlighting
3. **Output Panel** (popup): Display conversion results

### Keyboard Shortcuts

- `↑/↓` - Navigate in file list
- `[Enter]/F` - Select file or confirm operation
- `Q` - Quit application
- `J/K` - Scroll file preview

## 📋 Examples

The project includes sample data files in the `assets/` directory:

### CSV Example (input.csv)
```csv
Name,Position,DOB,Nationality,KitNumber
Wojciech Szczesny,Goalkeeper,"Apr 18, 1990 (29)",Poland,1
Mattia Perin,Goalkeeper,"Nov 10, 1992 (26)",Italy,37
```

### JSON Example (input.json)
```json
[
  {
    "Name": "Wojciech Szczesny",
    "Position": "Goalkeeper",
    "DOB": "Apr 18,1990 (29)",
    "Nationality": "Poland",
    "KitNumber": 1
  }
]
```

### YAML Example (input.yaml)
```yaml
- Name: Wojciech Szczesny
  Position: Goalkeeper
  DOB: "Apr 18,1990 (29)"
  Nationality: Poland
  KitNumber: 1
```

### TOML Example (input.toml)
```toml
[[players]]
Name = "Wojciech Szczesny"
Position = "Goalkeeper"
DOB = "Apr 18,1990 (29)"
Nationality = "Poland"
KitNumber = 1
```

### Development Environment Setup

```bash
# Install development dependencies
cargo install cargo-edit cargo-watch

# Run development server (auto-recompile)
cargo watch -x run
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👤 Author

**gushiii** - [zrjie001@gmail.com](mailto:zrjie001@gmail.com)

## 🙏 Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Excellent Rust TUI framework
- [Serde](https://github.com/serde-rs/serde) - Rust serialization framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal handling library

---

⭐ If this project helps you, please give it a star!