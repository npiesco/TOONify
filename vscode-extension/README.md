# TOONify for VS Code

**JSON ↔ TOON format converter for Visual Studio Code. Reduce LLM token usage by 30-60%.**

Convert between JSON and TOON (Token-Oriented Object Notation) directly in your editor. TOON is a compact, human-readable data format optimized for AI/LLM applications, offering significant token savings compared to JSON.

## Features

- **JSON → TOON Conversion**: Convert JSON documents to compact TOON format
- **TOON → JSON Conversion**: Convert TOON back to JSON with perfect fidelity
- **Selection Support**: Convert selected text or entire document
- **Keyboard Shortcuts**: Quick access via `Cmd+Alt+T` (JSON→TOON) and `Cmd+Alt+J` (TOON→JSON)
- **Context Menu**: Right-click to convert selections
- **Syntax Highlighting**: Full syntax support for `.toon` files
- **Format Validation**: Validate TOON syntax on-the-fly

## Installation

1. Open VS Code
2. Press `Cmd+Shift+X` (or `Ctrl+Shift+X` on Windows/Linux)
3. Search for "TOONify"
4. Click **Install**

## Usage

### Convert JSON to TOON

**Method 1: Command Palette**
1. Open a JSON file or select JSON text
2. Press `Cmd+Shift+P` (or `Ctrl+Shift+P`)
3. Type "TOONify: Convert JSON to TOON"
4. Press Enter

**Method 2: Keyboard Shortcut**
- Mac: `Cmd+Alt+T`
- Windows/Linux: `Ctrl+Alt+T`

**Method 3: Context Menu**
1. Select JSON text
2. Right-click
3. Choose "TOONify: Convert JSON to TOON"

### Convert TOON to JSON

**Method 1: Command Palette**
1. Open a TOON file or select TOON text
2. Press `Cmd+Shift+P` (or `Ctrl+Shift+P`)
3. Type "TOONify: Convert TOON to JSON"
4. Press Enter

**Method 2: Keyboard Shortcut**
- Mac: `Cmd+Alt+J`
- Windows/Linux: `Ctrl+Alt+J`

**Method 3: Context Menu**
1. Select TOON text
2. Right-click
3. Choose "TOONify: Convert TOON to JSON"

## Example

**JSON (25 tokens):**
```json
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

**TOON (3 tokens - 88% reduction):**
```toon
users[1]{id,name,role}:
1,Alice,admin
```

## What is TOON?

TOON (Token-Oriented Object Notation) is a modern data format optimized for AI and LLM applications. It uses a compact, tabular format that dramatically reduces token counts when working with language models like GPT-4, Claude, and others.

### Benefits

- **30-60% Token Reduction**: Significantly lower LLM API costs
- **Human Readable**: Easy to read and understand, unlike binary formats
- **Bidirectional**: Perfect JSON ↔ TOON roundtrip conversions
- **No Schema Required**: Works without predefined schemas
- **Fast**: Powered by high-performance Rust backend

### Use Cases

- Reducing costs for LLM API calls
- Optimizing prompt engineering workflows
- Data pipeline optimization
- AI agent communication
- ETL processing for ML training data

## Requirements

- Visual Studio Code 1.75.0 or higher
- **WASM Support**: This extension uses WebAssembly for high-performance conversions

## Extension Settings

This extension contributes the following settings:

- `toonify.autoFormat`: Automatically format TOON files on save (default: `true`)
- `toonify.validateOnType`: Validate TOON syntax as you type (default: `true`)

## Known Issues

- Large files (>10MB) may take a few seconds to convert
- Complex nested structures with deep nesting (>20 levels) are not yet optimized

## Release Notes

### 0.1.0

Initial release:
- JSON to TOON conversion
- TOON to JSON conversion
- Syntax highlighting for `.toon` files
- Keyboard shortcuts and context menu integration
- WASM-powered conversions

## Support

- **Issues**: https://github.com/npiesco/TOONify/issues
- **Documentation**: https://github.com/npiesco/TOONify
- **Discussions**: https://github.com/npiesco/TOONify/discussions

## License

MIT License - see [LICENSE](https://github.com/npiesco/TOONify/blob/main/LICENSE) for details

## About

TOONify is built with Rust and WebAssembly for maximum performance. The core conversion logic is battle-tested and used in production environments for LLM token optimization.

**Like this extension?** Star the repo on [GitHub](https://github.com/npiesco/TOONify)!

