# Publishing Guide

This document describes how to publish TOONify packages to PyPI and npm, both manually and via GitHub Actions.

## Quick Release (Automated)

### Option 1: Tag-based Release (Recommended)

1. Create and push a version tag:
```bash
git tag v0.1.0
git push origin v0.1.0
```

2. GitHub Actions will automatically:
   - Create a GitHub release
   - Build binaries for Linux, macOS, Windows
   - Build Python wheels for all platforms
   - Build WASM package
   - Publish to PyPI
   - Publish to npm

### Option 2: Manual Workflow Trigger

1. Go to [GitHub Actions](https://github.com/npiesco/TOONify/actions)
2. Select "Release and Publish" workflow
3. Click "Run workflow"
4. Enter version (e.g., `0.1.0`)
5. Choose whether to publish to PyPI and/or npm

## Manual Publishing

### Prerequisites

#### PyPI
```bash
pip install build twine
```

You'll need a PyPI API token:
1. Go to https://pypi.org/manage/account/token/
2. Create a new token
3. Save it in `~/.pypirc`:
```ini
[pypi]
username = __token__
password = pypi-YourTokenHere
```

#### npm
```bash
npm login
```

### Publish Python Package to PyPI

```bash
# 1. Build Rust library
cargo build --lib --release --features cache,persistent-cache

# 2. Generate UniFFI bindings
cargo run --bin uniffi-bindgen -- generate \
    --library target/release/libtoonify.dylib \
    --language python \
    --out-dir bindings/python

# 3. Copy native library
cp target/release/libtoonify.dylib bindings/python/

# 4. Build Python package
cd bindings/python
python -m build

# 5. Publish to PyPI
python -m twine upload dist/*

# Or for TestPyPI first:
python -m twine upload --repository testpypi dist/*
```

### Publish WASM Package to npm

```bash
# 1. Install wasm-pack (if not already installed)
cargo install wasm-pack

# 2. Build WASM package
wasm-pack build --target web --out-dir pkg --no-default-features

# 3. Update version in package.json if needed
cd pkg
npm version 0.1.0 --no-git-tag-version

# 4. Publish to npm
npm publish

# Or for dry-run first:
npm publish --dry-run
```

## GitHub Secrets Setup

For automated publishing, configure these secrets in your GitHub repository:

### Required Secrets

1. **PYPI_API_TOKEN**
   - Go to https://pypi.org/manage/account/token/
   - Create a new token with upload permissions
   - Add to GitHub: Settings → Secrets → Actions → New repository secret
   - Name: `PYPI_API_TOKEN`
   - Value: `pypi-YourTokenHere`

2. **NPM_TOKEN**
   - Run `npm token create`
   - Add to GitHub: Settings → Secrets → Actions → New repository secret
   - Name: `NPM_TOKEN`
   - Value: Your npm token

## Version Bumping

### Update Version in All Locations

Before publishing, update the version in:

1. **Cargo.toml**
```toml
[package]
version = "0.1.0"
```

2. **bindings/python/setup.py**
```python
setup(
    name="toonify",
    version="0.1.0",
    ...
)
```

3. **pkg/package.json**
```json
{
  "version": "0.1.0",
  ...
}
```

4. **vscode-extension/package.json**
```json
{
  "version": "0.1.0",
  ...
}
```

### Automated Version Bump Script

```bash
#!/bin/bash
# bump-version.sh
VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./bump-version.sh 0.1.0"
    exit 1
fi

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update Python setup.py
sed -i '' "s/version=\".*\"/version=\"$VERSION\"/" bindings/python/setup.py

# Update npm package.json
cd pkg && npm version $VERSION --no-git-tag-version && cd ..

# Update VS Code extension
cd vscode-extension && npm version $VERSION --no-git-tag-version && cd ..

echo "✅ Version bumped to $VERSION"
echo "Run: git commit -am 'chore: bump version to $VERSION' && git tag v$VERSION && git push --tags"
```

## Testing Before Publishing

### Test Python Package

```bash
# Build and install locally
cd bindings/python
python -m build
pip install dist/toonify-0.1.0.tar.gz

# Test in Python
python -c "from toonify import json_to_toon; print(json_to_toon('{\"test\": 1}'))"
```

### Test npm Package

```bash
# Build WASM
wasm-pack build --target web --out-dir pkg --no-default-features

# Test locally
cd pkg
npm link

# In another project
npm link toonify
```

### Test VS Code Extension

```bash
cd vscode-extension
npm install
npm run compile
npm run package

# Install locally
code --install-extension toonify-0.1.0.vsix
```

## Rollback a Release

### PyPI
PyPI does not allow deleting or replacing packages. You must:
1. Yank the bad version: `twine upload --skip-existing --repository pypi dist/*`
2. Publish a new patch version

### npm
```bash
# Unpublish within 72 hours
npm unpublish toonify@0.1.0

# Or deprecate
npm deprecate toonify@0.1.0 "This version has issues, use 0.1.1"
```

## Publishing Checklist

Before releasing:

- [ ] All tests pass (`cargo test`)
- [ ] Version bumped in all files
- [ ] CHANGELOG.md updated
- [ ] README.md updated with new features
- [ ] Documentation is up to date
- [ ] Built and tested locally
- [ ] Git tag created
- [ ] GitHub Secrets configured (PYPI_API_TOKEN, NPM_TOKEN)

## Post-Release

After publishing:

1. **Verify packages**:
   - PyPI: https://pypi.org/project/toonify/
   - npm: https://www.npmjs.com/package/toonify

2. **Test installation**:
   ```bash
   pip install toonify==0.1.0
   npm install toonify@0.1.0
   ```

3. **Announce release**:
   - GitHub Discussions
   - Twitter/X
   - Reddit (r/rust, r/Python)
   - Hacker News

4. **Update documentation sites**:
   - docs.rs (automatic for Rust crates)
   - GitHub Pages (if you have one)

## Troubleshooting

### "Package already exists" on PyPI
- PyPI does not allow re-uploading the same version
- Bump the version and publish again

### "Authentication failed" for npm
- Check that `NPM_TOKEN` secret is set correctly
- Verify token hasn't expired: `npm token list`

### WASM build fails
- Ensure `wasm32-unknown-unknown` target is installed:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### UniFFI bindings generation fails
- Check that the Rust library builds: `cargo build --lib --release`
- Verify `uniffi-bindgen` is installed: `cargo install uniffi-bindgen`

## Support

For issues with publishing, open an issue: https://github.com/npiesco/TOONify/issues

