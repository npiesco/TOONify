# VS Code Extension Setup Guide

Complete guide for building, packaging, and installing the TOONify VS Code extension.

## Prerequisites

- **Node.js** 18+ and npm
- **VS Code**, **Windsurf**, **Cursor**, or another VS Code fork
- No Rust compilation required (uses published `@npiesco/toonify` npm package)

## Quick Start

```bash
cd vscode-extension
npm install
npm run compile
npm run package
```

This creates `toonify-1.1.1.vsix` ready for installation.

---

## Step-by-Step Setup

### 1. Install Dependencies

```bash
cd vscode-extension
npm install
```

**What this does:**
- Installs 303+ packages
- Downloads `@npiesco/toonify` v1.1.0 (WASM package with pre-compiled binary)
- Installs TypeScript, ESLint, and build tools

**Important:** The extension uses the **published npm package** (`@npiesco/toonify`), which already contains the compiled WASM binary. You don't need to build Rust code unless you want to use a local development version.

### 2. Compile TypeScript

```bash
npm run compile
```

**What this does:**
- Compiles `src/extension.ts` → `out/extension.js`
- Validates TypeScript types
- Creates source maps for debugging

**Watch Mode (for development):**
```bash
npm run watch
```

### 3. Package Extension

```bash
npm run package
```

**What this does:**
- Runs `vsce package` (Visual Studio Code Extension packager)
- Bundles extension code + dependencies + WASM binary
- Creates `toonify-1.1.1.vsix` (1.1 MB)
- Includes:
  - Extension manifest (`package.json`)
  - Compiled JavaScript (`out/extension.js`)
  - WASM binary (`node_modules/@npiesco/toonify/toonify_bg.wasm` - 144KB)
  - Syntax grammar (`syntaxes/toon.tmLanguage.json`)
  - Icon (`icon.png`)

**Output:**
```
toonify-1.1.1.vsix (1.14 MB, 13 files)
```

---

## Installation Methods

### For VS Code

#### Method 1: Command Palette (Recommended)
1. Open VS Code
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
3. Type: `Extensions: Install from VSIX...`
4. Navigate to `vscode-extension/toonify-1.1.1.vsix`
5. Click **Open**
6. Reload VS Code when prompted

#### Method 2: Extensions View
1. Open Extensions view (`Cmd+Shift+X`)
2. Click `...` (three dots) at top of Extensions panel
3. Select **"Install from VSIX..."**
4. Choose `toonify-1.1.1.vsix`

#### Method 3: Command Line
```bash
code --install-extension vscode-extension/toonify-1.1.1.vsix
```

**Note:** If `code` command not found, add it via:
- VS Code → Command Palette → "Shell Command: Install 'code' command in PATH"

### For Windsurf

Windsurf is a VS Code fork that uses the same extension system.

#### Method 1: Command Palette (Recommended)
1. Open Windsurf
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
3. Type: `Extensions: Install from VSIX...`
4. Navigate to `vscode-extension/toonify-1.1.1.vsix`
5. Click **Open**
6. Reload Windsurf when prompted

#### Method 2: Windsurf CLI (if installed)
```bash
windsurf --install-extension vscode-extension/toonify-1.1.1.vsix
```

**Note:** Windsurf uses Open VSX Registry by default, but VSIX installation works identically to VS Code.

### For Cursor

#### Method 1: Command Palette
1. Open Cursor
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
3. Type: `Extensions: Install from VSIX...`
4. Select `vscode-extension/toonify-1.1.1.vsix`

#### Method 2: Cursor CLI
```bash
cursor --install-extension vscode-extension/toonify-1.1.1.vsix
```

### For VSCodium

VSCodium is an open-source build of VS Code.

```bash
codium --install-extension vscode-extension/toonify-1.1.1.vsix
```

Or use the Command Palette method (same as VS Code).

---

## Verifying Installation

After installation, verify the extension is active:

1. **Check Extensions List:**
   - Open Extensions view (`Cmd+Shift+X`)
   - Search for "TOONify"
   - Should show version 1.1.1 installed

2. **Test Commands:**
   - Open Command Palette (`Cmd+Shift+P`)
   - Type "TOONify"
   - Should see 4 commands:
     - TOONify: Convert JSON to TOON
     - TOONify: Convert TOON to JSON
     - TOONify: Validate TOON
     - TOONify: Format TOON

3. **Test Conversion:**
   - Create a new file with JSON:
     ```json
     {"users": [{"id": 1, "name": "Alice", "role": "admin"}]}
     ```
   - Select the text
   - Press `Cmd+Alt+T` (Mac) or `Ctrl+Alt+T` (Windows/Linux)
   - Should convert to TOON:
     ```toon
     users[1]{id,name,role}:
     1,Alice,admin
     ```

4. **Check Syntax Highlighting:**
   - Create a file named `test.toon`
   - Should have syntax highlighting enabled

---

## Usage

### Keyboard Shortcuts

| Action | Mac | Windows/Linux |
|--------|-----|---------------|
| JSON → TOON | `Cmd+Alt+T` | `Ctrl+Alt+T` |
| TOON → JSON | `Cmd+Alt+J` | `Ctrl+Alt+J` |

### Commands

Access via Command Palette (`Cmd+Shift+P`):

1. **TOONify: Convert JSON to TOON**
   - Converts selected JSON text or entire document
   - Auto-validates JSON before conversion
   - Replaces selection with TOON output

2. **TOONify: Convert TOON to JSON**
   - Converts selected TOON text or entire document
   - Pretty-prints JSON output
   - Replaces selection with formatted JSON

3. **TOONify: Validate TOON**
   - Validates TOON syntax
   - Shows error messages in editor

4. **TOONify: Format TOON**
   - Roundtrip format (TOON → JSON → TOON)
   - Normalizes spacing and structure

### Context Menu

Right-click on selected text:
- **TOONify: Convert JSON to TOON**
- **TOONify: Convert TOON to JSON**

### Language Support

The extension provides full language support for `.toon` files:
- Syntax highlighting
- Auto-closing brackets and quotes
- Line comments with `#`
- Code folding for multi-line structures

---

## Performance Features

### Caching

The extension uses `WasmCachedConverter` with 100-entry in-memory cache:
- **First conversion**: ~0.5ms (cache miss)
- **Repeated conversion**: <1μs (cache hit) - **500x faster!**

### Cache Management Commands

**Show Cache Statistics:**
```
Command Palette → TOONify: Show Cache Statistics
```

**Clear Cache:**
```
Command Palette → TOONify: Clear Cache
```

---

## Architecture

```
VS Code Extension (TypeScript)
    ↓
Loads @npiesco/toonify (npm package)
    ↓
WasmCachedConverter (100-entry cache)
    ↓
WASM Binary (toonify_bg.wasm - 144KB)
    ↓
Rust Core (compiled to WebAssembly)
```

**Key Points:**
- **No network requests**: All conversion happens locally
- **No external dependencies**: WASM binary is self-contained
- **Fast initialization**: Pre-loads on activation
- **Memory efficient**: ~1MB footprint

---

## Development Workflow

### Local Development

1. **Open in VS Code:**
   ```bash
   cd vscode-extension
   code .
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Start watch mode:**
   ```bash
   npm run watch
   ```

4. **Launch Extension Development Host:**
   - Press `F5` in VS Code
   - Opens new window with extension loaded
   - Make changes → Reload window (`Cmd+R`)

### Using Local WASM Build

If you need to use a locally-built WASM package instead of the published npm version:

1. **Build WASM package:**
   ```bash
   cd .. # Back to TOONify root
   wasm-pack build --target web --out-dir pkg --no-default-features
   ```

2. **Link local package:**
   ```bash
   cd pkg
   npm link

   cd ../vscode-extension
   npm link @npiesco/toonify
   ```

3. **Rebuild extension:**
   ```bash
   npm run compile
   npm run package
   ```

### Debugging

**Enable Developer Tools:**
- Help → Toggle Developer Tools
- View extension console logs
- Debug WASM module

**Check Extension Logs:**
- Output panel → Select "Extension Host"

---

## Publishing to Marketplace

### VS Code Marketplace

1. **Install vsce:**
   ```bash
   npm install -g @vscode/vsce
   ```

2. **Create publisher account:**
   - Go to https://marketplace.visualstudio.com/manage
   - Create Personal Access Token (PAT)

3. **Login:**
   ```bash
   vsce login NicholasPiesco
   ```

4. **Publish:**
   ```bash
   vsce publish
   ```

### Open VSX Registry

For Windsurf, VSCodium, and other VS Code forks:

1. **Install ovsx:**
   ```bash
   npm install -g ovsx
   ```

2. **Create account:**
   - Go to https://open-vsx.org/

3. **Publish:**
   ```bash
   ovsx publish toonify-1.1.1.vsix -p <your-token>
   ```

---

## Troubleshooting

### Extension Not Loading

**Symptoms:**
- Commands not appearing in Command Palette
- No syntax highlighting for `.toon` files

**Solutions:**
1. Check extension is enabled:
   - Extensions view → Search "TOONify"
   - Ensure not disabled
2. Reload window: `Cmd+R` (Mac) or `Ctrl+R` (Windows)
3. Check Developer Console for errors

### WASM Module Load Failure

**Symptoms:**
- "Failed to load WASM module" error

**Solutions:**
1. Verify `node_modules/@npiesco/toonify/toonify_bg.wasm` exists
2. Reinstall dependencies:
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   npm run compile
   npm run package
   ```

### Commands Not Working

**Symptoms:**
- Keyboard shortcuts don't trigger
- Commands appear but do nothing

**Solutions:**
1. Check file is focused in editor (`when="editorTextFocus"`)
2. Verify JSON is valid before converting
3. Check Output panel for error messages

### "command not found: vsce"

**Symptoms:**
- `npm run package` fails

**Solutions:**
1. Install vsce locally:
   ```bash
   npm install --save-dev @vscode/vsce
   ```
2. Or globally:
   ```bash
   npm install -g @vscode/vsce
   ```

### Package.json Version Mismatch

**Symptoms:**
- Extension version incorrect after packaging

**Solutions:**
1. Update version in `package.json`:
   ```json
   {
     "version": "1.1.1"
   }
   ```
2. Rebuild:
   ```bash
   npm run package
   ```

---

## File Structure

```
vscode-extension/
├── package.json              # Extension manifest
├── tsconfig.json            # TypeScript config
├── .vscodeignore            # Files excluded from package
├── icon.png                 # Extension icon (1.06 MB)
├── README.md                # Extension documentation
├── language-configuration.json  # Language features
├── src/
│   └── extension.ts         # Main extension code
├── out/                     # Compiled JavaScript
│   └── extension.js
├── syntaxes/
│   └── toon.tmLanguage.json # TOON syntax grammar
└── node_modules/
    └── @npiesco/toonify/    # WASM package
        ├── toonify_bg.wasm  # 144KB binary
        ├── toonify.js       # JS glue code
        └── toonify.d.ts     # TypeScript types
```

---

## Scripts Reference

| Script | Command | Purpose |
|--------|---------|---------|
| `compile` | `tsc -p ./` | Compile TypeScript |
| `watch` | `tsc -watch -p ./` | Watch mode compilation |
| `package` | `vsce package` | Create VSIX bundle |
| `lint` | `eslint src --ext ts` | Run ESLint |
| `pretest` | `npm run compile` | Pre-test hook |
| `vscode:prepublish` | `npm run compile` | Pre-publish hook |

---

## FAQs

### Do I need to compile Rust?

**No.** The extension uses the published `@npiesco/toonify` npm package (v1.1.0), which already contains the pre-compiled WASM binary. You only need Node.js and npm.

### Does this work offline?

**Yes.** All conversions happen locally using the bundled WASM binary. No network requests are made.

### What's the difference between this and the Python package?

| Feature | VS Code Extension | Python Package |
|---------|------------------|----------------|
| Language | TypeScript + WASM | Python + Rust (UniFFI) |
| Caching | HashMap (100 entries) | Moka + Sled (dual-tier) |
| Use Case | Editor integration | Scripts, APIs, pipelines |
| Performance | <1μs (cached) | <100ns (Moka cached) |
| Distribution | VSIX | PyPI (toonifypy) |

### Can I use this in Cursor/Windsurf?

**Yes!** Both support VSIX installation via Command Palette → "Extensions: Install from VSIX...".

### How do I update the extension?

1. Build new version: `npm run package`
2. Uninstall old version from Extensions view
3. Install new VSIX file
4. Reload editor

### Where is the cache stored?

In-memory only (HashMap). Cache is cleared when editor restarts. No persistent storage.

---

## Resources

- **TOONify Repository**: https://github.com/npiesco/TOONify
- **VS Code Extension API**: https://code.visualstudio.com/api
- **WASM Package**: https://www.npmjs.com/package/@npiesco/toonify
- **VS Code Marketplace**: https://marketplace.visualstudio.com/items?itemName=NicholasPiesco.toonify
- **Open VSX**: https://open-vsx.org/extension/NicholasPiesco/toonify

---

## Support

- **Issues**: https://github.com/npiesco/TOONify/issues
- **Discussions**: https://github.com/npiesco/TOONify/discussions
- **Documentation**: https://github.com/npiesco/TOONify

---

**Happy converting!** 🚀
