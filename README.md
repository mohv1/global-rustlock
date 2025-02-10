# Global RustLock 🌐🔒

A cross-platform CapsLock synchronization tool written in Rust, allowing you to keep your CapsLock state in sync across multiple machines via WebSocket.

[![Rust](https://img.shields.io/badge/Rust-1.65%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

## Features ✨

- Real-time CapsLock state synchronization
- Secure WebSocket (wss://) communication
- Low latency (<100ms updates)
- Cross-platform support (Linux/Windows/macOS)
- Automatic reconnection logic

## Installation 📦

### Linux Requirements

#### Arch-based
```bash
sudo pacman -S openssl xdotool rust
```

#### Debian/Ubuntu
```bash
sudo apt install openssl xdotool rustc cargo
```

### macOS Requirements
```bash
brew install openssl rustup-init
rustup default stable
```

### Windows Requirements
- Install Rust
- Install OpenSSL

## Usage 🚀
Clone repository:
```bash
git clone https://github.com/mohv1/global-rustlock
cd global-rustlock
```

Build and run:
```bash
cargo run --release
```

The client will automatically:
- Connect to the synchronization server
- Share initial CapsLock state
- Send/receive state updates

## Building from Source 🔨
```bash
# Install dependencies
cargo install --path .

# Build optimized binary
cargo build --release

# Output will be in target/release/global-rustlock
```


## Dependencies 📚
| Component      | Linux | Windows | macOS |
|----------------|-------|---------|-------|
| OpenSSL        | ✅    | ✅      | ✅    |
| xdotool        | ✅    | ❌      | ❌    |
| Rust 1.65+     | ✅    | ✅      | ✅    |

## Contributing 🤝
1. Fork the repository
2. Create feature branch:
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. Commit changes
4. Push to branch
5. Open Pull Request

## License 📄
MIT License - See LICENSE for details

## Acknowledgments 🙏
- Original Python concept by nolenroyalty
- Rust community for awesome libraries
- Tokio team for async runtime
