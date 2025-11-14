# TOONify WASM

High-performance JSON ↔ TOON converter compiled to WebAssembly.

**Reduce LLM token usage by 30-60%** with TOON (Token-Oriented Object Notation).

## Installation

```bash
npm install toonify
```

## Quick Start

### Browser (ES Modules)

```javascript
import init, { json_to_toon, toon_to_json } from 'toonify';

async function main() {
    // Initialize the WASM module
    await init();
    
    // Convert JSON to TOON
    const json = JSON.stringify({ users: [{ id: 1, name: "Alice" }] });
    const toon = json_to_toon(json);
    console.log("TOON:", toon);
    // Output: users[1]{id,name}:\n1,Alice
    
    // Convert TOON back to JSON
    const jsonResult = toon_to_json(toon);
    console.log("JSON:", jsonResult);
    // Output: {"users":[{"id":1,"name":"Alice"}]}
}

main();
```

### Node.js

```javascript
import { json_to_toon, toon_to_json } from 'toonify';

// Convert JSON to TOON
const json = JSON.stringify({ users: [{ id: 1, name: "Bob" }] });
const toon = json_to_toon(json);
console.log("TOON:", toon);

// Convert back to JSON
const jsonResult = toon_to_json(toon);
console.log("JSON:", jsonResult);
```

### HTML

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>TOONify Demo</title>
</head>
<body>
    <script type="module">
        import init, { json_to_toon, toon_to_json } from './node_modules/toonify/toonify.js';
        
        async function run() {
            await init();
            
            const data = { users: [{ id: 1, name: "Alice", role: "admin" }] };
            const toon = json_to_toon(JSON.stringify(data));
            console.log("TOON format:", toon);
            
            const json = toon_to_json(toon);
            console.log("Back to JSON:", json);
        }
        
        run();
    </script>
</body>
</html>
```

## API

### `json_to_toon(json: string): string`

Converts a JSON string to TOON format.

**Parameters:**
- `json` - A valid JSON string

**Returns:**
- TOON formatted string

**Throws:**
- Error if JSON is invalid

**Example:**
```javascript
const toon = json_to_toon('{"users":[{"id":1,"name":"Alice"}]}');
// Returns: users[1]{id,name}:\n1,Alice
```

### `toon_to_json(toon: string): string`

Converts a TOON formatted string back to JSON.

**Parameters:**
- `toon` - A valid TOON formatted string

**Returns:**
- JSON string

**Throws:**
- Error if TOON format is invalid

**Example:**
```javascript
const json = toon_to_json('users[1]{id,name}:\n1,Alice');
// Returns: {"users":[{"id":1,"name":"Alice"}]}
```

### `init(): Promise<void>`

Initializes the WASM module. Must be called before using any conversion functions in browser environments.

**Example:**
```javascript
await init();
// Now you can use json_to_toon and toon_to_json
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

## Features

- **Zero dependencies** - Pure WASM module
- **Browser native** - Works in Chrome, Firefox, Safari
- **Node.js compatible** - Works in Node.js 12+
- **TypeScript support** - Auto-generated type definitions
- **Fast** - Near-native Rust performance
- **Small** - ~132KB WASM binary

## Performance

| Payload Size | Conversion Time |
|--------------|-----------------|
| < 1KB        | < 0.5ms        |
| 10KB         | 1-3ms          |
| 100KB        | 10-30ms        |

## Use Cases

- **LLM Cost Reduction** - Save 30-60% on API costs for GPT-4, Claude, etc.
- **Data Processing** - Client-side data transformation in web apps
- **Serverless Functions** - Run in Cloudflare Workers, Fastly Compute@Edge
- **Browser Extensions** - Convert data formats on the fly
- **Electron/Tauri Apps** - Desktop applications with data optimization

## Browser Compatibility

- Chrome/Edge 57+
- Firefox 52+
- Safari 11+
- Node.js 12+

## Error Handling

```javascript
import { json_to_toon } from 'toonify';

try {
    const toon = json_to_toon('invalid json');
} catch (error) {
    console.error('Conversion failed:', error.message);
}
```

## TypeScript

Type definitions are included automatically:

```typescript
import { json_to_toon, toon_to_json, init } from 'toonify';

// TypeScript will provide full type checking and autocomplete
const result: string = json_to_toon('{"key":"value"}');
```

## Examples

### Roundtrip Conversion

```javascript
import { json_to_toon, toon_to_json } from 'toonify';

const original = {
    products: [
        { sku: "ABC123", name: "Widget", price: 19.99 },
        { sku: "DEF456", name: "Gadget", price: 29.99 }
    ]
};

// JSON → TOON
const toon = json_to_toon(JSON.stringify(original));
console.log("TOON:", toon);
// products[2]{sku,name,price}:
// ABC123,Widget,19.99
// DEF456,Gadget,29.99

// TOON → JSON
const json = toon_to_json(toon);
const parsed = JSON.parse(json);
console.log("Roundtrip:", parsed);
// Identical to original
```

### Processing Large Datasets

```javascript
import { json_to_toon } from 'toonify';

// Convert large JSON dataset
const largeData = { records: Array(1000).fill({ id: 1, name: "Test" }) };
const toon = json_to_toon(JSON.stringify(largeData));

console.log(`Original JSON size: ${JSON.stringify(largeData).length} bytes`);
console.log(`TOON size: ${toon.length} bytes`);
console.log(`Savings: ${((1 - toon.length / JSON.stringify(largeData).length) * 100).toFixed(1)}%`);
```

## License

MIT License - see [LICENSE](https://github.com/npiesco/TOONify/blob/main/LICENSE)

## Links

- [GitHub Repository](https://github.com/npiesco/TOONify)
- [Documentation](https://github.com/npiesco/TOONify#readme)
- [Issue Tracker](https://github.com/npiesco/TOONify/issues)
- [Full Project README](https://github.com/npiesco/TOONify/blob/main/README.md)

## Contributing

Contributions welcome! Please see the [main repository](https://github.com/npiesco/TOONify) for contribution guidelines.

---

Built with Rust and compiled to WebAssembly for maximum performance.
