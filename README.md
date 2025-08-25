# tl - Tool Installer

A simple, fast, and user-friendly tool installer and manager that downloads binaries directly from GitHub releases.

## ( Features

- =æ **Easy Installation** - Install tools with a single command
- =Ñ **Clean Uninstall** - Remove tools just as easily
- = **Smart Detection** - Automatically finds the right binary for your platform
-   **Safety First** - Prompts before overwriting existing installations
- =ã **PATH Integration** - Smart PATH detection with helpful setup instructions
- <¨ **Beautiful Output** - Clean, emoji-enhanced terminal output

## =€ Quick Install

Install `tl` with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/eyalev/tl/master/install.sh | bash
```

Or download the binary manually from the [latest release](https://github.com/eyalev/tl/releases/latest).

## =Ë Usage

### Install a tool
```bash
tl install ports-tool
```

### List available tools
```bash
tl list
```

### Uninstall a tool
```bash
tl uninstall ports-tool
```

### Get help
```bash
tl --help
```

## =à Available Tools

Currently supported tools:

- **ports-tool** - A tool for checking which ports are in use on your system

Want to add more tools? Edit the `tools.json` file and submit a pull request!

## =' How It Works

1. **Registry-based** - Tools are defined in a simple JSON registry (`tools.json`)
2. **GitHub Releases** - Downloads binaries directly from GitHub releases
3. **Platform Detection** - Automatically detects your OS and architecture
4. **Local Installation** - Installs to `~/.local/bin` by default (configurable)

## =Á Tool Registry

Tools are defined in `tools.json`:

```json
{
  "tools": {
    "ports-tool": {
      "name": "ports-tool",
      "description": "A tool for checking which ports are in use on your system",
      "github_repo": "eyalev/ports-tool",
      "install_method": "github_release",
      "binary_name": "ports-tool",
      "install_path": "~/.local/bin"
    }
  }
}
```

## <× Building from Source

Requirements:
- Rust 1.70+ 
- Cargo

```bash
git clone https://github.com/eyalev/tl.git
cd tl
cargo build --release
```

The binary will be available at `target/release/tl`.

## > Contributing

1. Fork the repository
2. Add your tool to `tools.json`
3. Test the installation
4. Submit a pull request

### Adding a New Tool

To add a new tool, add an entry to `tools.json`:

```json
{
  "your-tool": {
    "name": "your-tool",
    "description": "Description of what your tool does",
    "github_repo": "username/repo-name",
    "install_method": "github_release",
    "binary_name": "binary-name",
    "install_path": "~/.local/bin"
  }
}
```

## =Ä License

MIT License - see [LICENSE](LICENSE) file for details.

## =O Acknowledgments

- Built with [Rust](https://rust-lang.org/) >€
- Uses [clap](https://clap.rs/) for CLI parsing
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP requests