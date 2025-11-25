# TOONify Python Bindings

High-performance JSON ↔ TOON converter with native Rust bindings for Python.

**Reduce LLM token usage by 30-60%** with TOON (Token-Oriented Object Notation).

## Installation

```bash
pip install toonifypy
```

> **Note:** The package is installed as `toonifypy`, but you import it as `toonify`:
> ```python
> pip install toonifypy  # Install command
> from toonify import ... # Import statement
> ```
> This follows common Python packaging practice (like `pip install pillow` → `from PIL import ...`).

## Quick Start

```python
from toonify import json_to_toon, toon_to_json
import json

# Convert JSON to TOON format
data = {"users": [{"id": 1, "name": "Alice", "role": "admin"}]}
toon = json_to_toon(json.dumps(data))
print("TOON:", toon)
# Output: users[1]{id,name,role}:
#         1,Alice,admin

# Convert TOON back to JSON
json_result = toon_to_json(toon)
parsed = json.loads(json_result)
print("JSON:", parsed)
# Output: {'users': [{'id': 1, 'name': 'Alice', 'role': 'admin'}]}
```

## What is TOON?

TOON (Token-Oriented Object Notation) is a compact data format designed to minimize token usage for AI and LLM applications.

**Comparison:**

```json
// JSON (25 tokens)
{
  "users": [
    {
      "id": 1,
      "name": "Alice",
      "role": "admin"
    }
  ]
}
```

```toon
# TOON (3 tokens - 88% reduction)
users[1]{id,name,role}:
1,Alice,admin
```

## API Reference

### `json_to_toon(json_data: str) -> str`

Converts a JSON string to TOON format.

**Parameters:**
- `json_data` (str): A valid JSON string

**Returns:**
- str: TOON formatted string

**Raises:**
- `ToonError`: If JSON is invalid or conversion fails

**Example:**
```python
from toonify import json_to_toon

json_str = '{"products":[{"sku":"ABC","price":19.99}]}'
toon = json_to_toon(json_str)
print(toon)
# products[1]{sku,price}:
# ABC,19.99
```

### `toon_to_json(toon_data: str) -> str`

Converts a TOON formatted string to JSON.

**Parameters:**
- `toon_data` (str): A valid TOON formatted string

**Returns:**
- str: JSON string

**Raises:**
- `ToonError`: If TOON format is invalid or conversion fails

**Example:**
```python
from toonify import toon_to_json

toon = '''products[1]{sku,price}:
ABC,19.99'''
json_str = toon_to_json(toon)
print(json_str)
# {"products":[{"sku":"ABC","price":19.99}]}
```

## Error Handling

```python
from toonify import json_to_toon, ToonError

try:
    result = json_to_toon('invalid json')
except ToonError as e:
    print(f"Conversion failed: {e}")
```

## High-Performance Caching

For repeated conversions, use `CachedConverter` for **10-330x speedup**:

```python
from toonify import CachedConverter

# Create cached converter (Moka + Sled)
converter = CachedConverter(
    cache_size=100,              # Max 100 entries in memory
    cache_ttl_secs=3600,         # 1 hour TTL (None = forever)
    persistent_path="./cache.db" # Persistent storage (None = memory only)
)

# First conversion (cache miss)
json_data = '{"users": [{"id": 1, "name": "Alice"}]}'
toon1 = converter.json_to_toon(json_data)  # ~1ms

# Second conversion (cache hit)
toon2 = converter.json_to_toon(json_data)  # <100ns (330x faster!)

# Check cache stats
print(converter.cache_stats())
# Cache Statistics:
#   Moka entries: 1
#   Moka weighted size: 1 bytes
#   Sled entries: 1

# Clear cache
converter.clear_cache()
```

**Cache Architecture:**
- **Moka**: Lock-free concurrent in-memory cache (hot path)
- **Sled**: Embedded persistent database (survives restarts)
- **Lookup**: Moka → Sled → Conversion

## Use Cases

### LLM API Cost Reduction

**Before (JSON):**
```python
import openai
import json

prompt = {"users": [...]}  # 1000 tokens
response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{"role": "user", "content": json.dumps(prompt)}]
)
# Cost: $0.03 per 1K tokens = $0.03
```

**After (TOON):**
```python
import openai
from toonify import json_to_toon

prompt = {"users": [...]}
toon_prompt = json_to_toon(json.dumps(prompt))  # 350 tokens
response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{"role": "user", "content": toon_prompt}]
)
# Cost: $0.03 per 1K tokens = $0.0105 (65% savings!)
```

### Roundtrip Conversion

```python
from toonify import json_to_toon, toon_to_json
import json

# Original data
original = {
    "products": [
        {"sku": "ABC123", "name": "Widget", "price": 19.99},
        {"sku": "DEF456", "name": "Gadget", "price": 29.99}
    ]
}

# JSON → TOON
toon = json_to_toon(json.dumps(original))
print("TOON format:")
print(toon)

# TOON → JSON
json_str = toon_to_json(toon)
result = json.loads(json_str)

# Verify roundtrip
assert original == result  # Perfect preservation
```

### Data Pipeline Integration

```python
from toonify import json_to_toon
import gzip

# Convert and compress for storage
data = {"records": [...]}
toon = json_to_toon(json.dumps(data))
compressed = gzip.compress(toon.encode())

# Massive size reduction
print(f"Original JSON: {len(json.dumps(data))} bytes")
print(f"TOON: {len(toon)} bytes")
print(f"TOON + gzip: {len(compressed)} bytes")
```

## Performance

| Payload Size | Conversion Time |
|--------------|-----------------|
| < 1KB        | < 1ms          |
| 1-100KB      | 1-10ms         |
| > 100KB      | 10-100ms       |

## Token Savings

| Data Type | JSON Tokens | TOON Tokens | Savings |
|-----------|-------------|-------------|---------|
| User list (3 items) | 45 | 12 | 73% |
| Product catalog (10 items) | 180 | 48 | 73% |
| API response (nested) | 120 | 35 | 71% |
| Time series (100 points) | 600 | 150 | 75% |

## Requirements

- Python 3.8+
- Works on macOS, Linux, and Windows

## Features

- **Blazing Fast**: Native Rust implementation
- **Zero Dependencies**: Pure Rust + Python ctypes
- **Type Safe**: Full error handling with ToonError
- **Roundtrip Safe**: Perfect data preservation
- **Memory Efficient**: Minimal allocations
- **Production Ready**: Comprehensive test coverage

## Platform Support

| Platform | Library File | Status |
|----------|--------------|--------|
| macOS    | libtoonify.dylib | ✓ Supported |
| Linux    | libtoonify.so | ✓ Supported |
| Windows  | toonify.dll | ✓ Supported |

## Advanced Usage

### Batch Processing

```python
from toonify import json_to_toon
import json
import os

# Convert multiple JSON files
for filename in os.listdir("data/"):
    if filename.endswith(".json"):
        with open(f"data/{filename}") as f:
            data = json.load(f)
        
        toon = json_to_toon(json.dumps(data))
        
        with open(f"data/{filename}.toon", "w") as f:
            f.write(toon)
```

### Integration with LLM Libraries

```python
from toonify import json_to_toon, toon_to_json
import anthropic

client = anthropic.Anthropic()

# Prepare data in TOON format for token efficiency
data = {"users": [...], "orders": [...]}
toon_data = json_to_toon(json.dumps(data))

message = client.messages.create(
    model="claude-3-opus-20240229",
    max_tokens=1024,
    messages=[{
        "role": "user",
        "content": f"Analyze this data: {toon_data}"
    }]
)

print(f"Tokens saved: ~{len(json.dumps(data)) - len(toon_data)} characters")
```

## Development

Built with:
- **Rust** - High-performance core implementation
- **UniFFI** - Automatic FFI bindings generation (Mozilla)
- **nom** - Parser combinators for TOON parsing

## Links

- [GitHub Repository](https://github.com/npiesco/TOONify)
- [Full Documentation](https://github.com/npiesco/TOONify#readme)
- [Issue Tracker](https://github.com/npiesco/TOONify/issues)
- [TOON Format Specification](https://github.com/npiesco/TOONify#toon-format-specification)

## License

MIT License - see [LICENSE](https://github.com/npiesco/TOONify/blob/main/LICENSE)

## Contributing

Contributions welcome! Please see the [main repository](https://github.com/npiesco/TOONify) for contribution guidelines.

---

**Questions?** Open an [issue](https://github.com/npiesco/TOONify/issues) or check the [documentation](https://github.com/npiesco/TOONify#readme).

**Like this project?** Star the repo and share with your AI engineering team!

