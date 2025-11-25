#!/bin/bash
# bump-version.sh - Update version across all package files

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/bump-version.sh 0.1.0"
    exit 1
fi

echo "🔄 Bumping version to $VERSION..."

# Update Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml && rm Cargo.toml.bak
echo "✅ Updated Cargo.toml"

# Update Python setup.py
sed -i.bak "s/version=\".*\"/version=\"$VERSION\"/" bindings/python/setup.py && rm bindings/python/setup.py.bak
echo "✅ Updated bindings/python/setup.py"

# Update npm package.json
cd pkg
npm version $VERSION --no-git-tag-version --allow-same-version
cd ..
echo "✅ Updated pkg/package.json"

# Update VS Code extension package.json
cd vscode-extension
npm version $VERSION --no-git-tag-version --allow-same-version
cd ..
echo "✅ Updated vscode-extension/package.json"

echo ""
echo "✨ Version bumped to $VERSION"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am 'chore: bump version to $VERSION'"
echo "  3. Tag: git tag v$VERSION"
echo "  4. Push: git push origin main --tags"
echo ""
echo "GitHub Actions will automatically publish to PyPI and npm when you push the tag."

