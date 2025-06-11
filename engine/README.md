# ChessFlow Engine

ChessFlow is a high-performance chess engine implemented in Rust, capable of running both natively and on the web via WebAssembly.

## Features

- Native and WebAssembly builds
- [UCI protocol](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf) support
- Advanced move generation and evaluation
- Multi-threaded search capabilities
- Web worker support for browser environments

## Dependencies

### Runtime:
- [Rust](https://www.rust-lang.org/tools/install) via [rustup](https://rust-lang.github.io/rustup/)

### Development
- Python 3 (for running the test web server)
- [Nix : the package manager](https://nixos.org/download/) for `shell-nix`

## Installation

1. Clone the repository:
```bash
git clone https://github.com/Dragos-Florin-Pojoga/ChessFlow.git
cd ChessFlow/engine
```

2. Install the required toolchain and components:
```bash
make toolchain
```

This will:
- Install the nightly Rust toolchain
- Add the `rust-src` component
- Install `wasm-bindgen-cli`
- Install `wasm-pack`
- Add the WebAssembly target

## Building

### Building Everything
To build both native and web versions:
```bash
make
```

### Native Build
To build only the native engine:
```bash
make build_native
```
This will create a release build in `target/release/native_engine`

### Web Build
To build the WebAssembly version:
```bash
make build_web
```
This will generate the following files in `public/pkg/`:
- `wasm_engine_bg.wasm`
- `wasm_engine.js`
- Associated TypeScript definition files

## Running

### Native Engine
Run the native engine in debug mode:
```bash
make run_native_debug
```

Run the native engine in release mode:
```bash
make run_native
```

### Web Version
To start the test web server:
```bash
make run_web
```
This will start a Python-based test server.
Open your browser to `http://localhost:8000` and open the dev console

## Development Features

### Profiling and Debugging
The engine supports several development features:

- Tracy profiler integration (via `tracy` feature flag)
- Heap profiling with dhat (via `dhat-heap` feature flag)
- Comprehensive logging and tracing capabilities

### Feature Flags
- `tracy` - Enable Tracy profiler integration
- `dhat-heap` - Enable heap profiling
- Default features are minimal for optimal performance