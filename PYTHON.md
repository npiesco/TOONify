# Python Bindings for TOONify

This document describes how to use TOONify from Python using UniFFI-generated bindings.

## What is UniFFI?

[UniFFI](https://mozilla.github.io/uniffi-rs/) is Mozilla's tool for generating foreign language bindings for Rust code. It allows you to write your library once in Rust and automatically generate idiomatic bindings for Python, Swift, Kotlin, and other languages.

## Prerequisites

- Python 3.8+
- Rust toolchain (for building the library)
- `pip` for installing Python packages

## Quick Start

### 1. Build the library

```bash
cargo build --lib --release
```

### 2. Generate Python Bindings

```bash
cargo run --bin uniffi-bindgen -- generate \
    --library target/release/libtoonify.dylib \
    --language python \
    --out-dir bindings/python
```

**On Linux, use `.so` instead of `.dylib`**

### 3. Copy the Native Library

```bash
# macOS
cp target/release/libtoonify.dylib bindings/python/

# Linux
cp target/release/libtoonify.so bindings/python/
```

### 4. Install the Package

```bash
pip install -e bindings/python/
```

### 5. Run the Example

```bash
python3 examples/python_example.py
```

## How It Works

TOONify uses UniFFI 0.29 with proc-macros to automatically generate Python bindings:

1. **Compile Time**: Rust code with `#[uniffi::export]` macros is compiled, embedding metadata in the library
2. **Generation**: Custom `uniffi-bindgen` binary extracts metadata and generates Python code  
3. **Installation**: Python package wraps the generated code and native library
4. **Runtime**: Python calls Rust functions via ctypes with automatic marshaling

## Usage Examples

### Basic JSON to TOON Conversion

```python
from toonify import json_to_toon, ToonError

json_data = '{"users": [{"id": 1, "name": "Alice"}]}'

try:
    toon_data = json_to_toon(json_data)
    print(toon_data)
    # Output:
    # users[1]{id,name}:
    # 1,Alice
except ToonError as e:
    print(f"Conversion failed: {e}")
```

### Basic TOON to JSON Conversion

```python
from toonify import toon_to_json, ToonError

toon_data = """users[1]{id,name}:
1,Alice"""

try:
    json_data = toon_to_json(toon_data)
    print(json_data)
    # Output: {"users": [{"id": 1, "name": "Alice"}]}
except ToonError as e:
    print(f"Conversion failed: {e}")
```

### Round-trip Conversion

```python
import json
from toonify import json_to_toon, toon_to_json

# Original data
original = {"users": [{"id": 1, "name": "Bob"}]}
json_str = json.dumps(original)

# Convert to TOON and back
toon = json_to_toon(json_str)
final_json = toon_to_json(toon)

# Verify equivalence
final = json.loads(final_json)
assert original == final  # Should pass!
```

### Error Handling

```python
from toonify import json_to_toon, ToonError

invalid_json = '{"broken": json}'

try:
    toon = json_to_toon(invalid_json)
except ToonError as e:
    print(f"Error: {e}")
    # Output: Error: Conversion error: Invalid JSON: ...
```

## API Reference

### Functions

#### `json_to_toon(json_data: str) -> str`

Converts a JSON string to TOON format.

**Parameters:**
- `json_data` (str): A valid JSON string

**Returns:**
- `str`: The converted TOON format string

**Raises:**
- `ToonError`: If the JSON is invalid or conversion fails

#### `toon_to_json(toon_data: str) -> str`

Converts a TOON format string to JSON.

**Parameters:**
- `toon_data` (str): A valid TOON format string

**Returns:**
- `str`: The converted JSON string (pretty-printed)

**Raises:**
- `ToonError`: If the TOON format is invalid or conversion fails

### Exceptions

#### `ToonError`

Base exception for all TOON conversion errors.

**Variants:**
- `ConversionError`: Raised when conversion fails

## Distribution

To distribute your Python package with TOONify:

### Option 1: Include Pre-built Binaries

1. Build for target platforms (macOS, Linux, Windows)
2. Include the native library (.dylib, .so, .dll) in your package
3. Include the generated Python binding code
4. Use `setuptools` to package everything

### Option 2: Build from Source

Create a `setup.py` that:
1. Uses `setuptools-rust` to compile the Rust code
2. Runs UniFFI generation during build
3. Includes the generated bindings

Example `setup.py`:

```python
from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="toonify",
    version="0.1.0",
    rust_extensions=[
        RustExtension(
            "toonify.libtoonify",
            binding=Binding.UniFFI,
        )
    ],
    packages=["toonify"],
    zip_safe=False,
)
```

## Performance

The Python bindings call directly into native Rust code, providing:
- Near-native performance
- Zero-copy operations where possible
- Efficient memory usage

Typical conversion times:
- Small JSON (< 1KB): < 1ms
- Medium JSON (1-100KB): 1-10ms
- Large JSON (> 100KB): 10-100ms

## Troubleshooting

### ImportError: cannot import name 'toonify'

**Solution:** Make sure the package is installed:
```bash
pip install -e bindings/python/
```

If that doesn't work, regenerate bindings:
```bash
cargo build --lib --release
cargo run --bin uniffi-bindgen -- generate --library target/release/libtoonify.dylib --language python --out-dir bindings/python
cp target/release/libtoonify.dylib bindings/python/
pip install -e bindings/python/
```

### Library load error on Linux

**Solution:** Ensure the `.so` file is in `bindings/python/`:
```bash
cp target/release/libtoonify.so bindings/python/
pip install --force-reinstall -e bindings/python/
```

### Binding generation fails

**Solution:** The custom `uniffi-bindgen` binary is built from the project:
```bash
cargo build --bin uniffi-bindgen
cargo run --bin uniffi-bindgen -- generate --library target/release/libtoonify.dylib --language python --out-dir bindings/python
```

## Further Reading

- [UniFFI User Guide](https://mozilla.github.io/uniffi-rs/)
- [TOON Format Specification](README.md#toon-format-specification)
- [Rust Documentation](https://docs.rs/uniffi/latest/uniffi/)

## Contributing

To contribute to the Python bindings:

1. Make changes to `src/lib.rs` (UniFFI exports use `#[uniffi::export]`)
2. Rebuild: `cargo build --lib --release`
3. Regenerate bindings: `cargo run --bin uniffi-bindgen -- generate --library target/release/libtoonify.dylib --language python --out-dir bindings/python`
4. Copy library: `cp target/release/libtoonify.dylib bindings/python/`
5. Reinstall: `pip install --force-reinstall -e bindings/python/`
6. Test: `python3 examples/python_example.py`
7. Run Rust tests: `cargo test`

## License

MIT

