#!/usr/bin/env python3
"""
bump_version.py - Update version across all package files
"""

import re
import sys
import json
from pathlib import Path


def update_cargo_toml(version: str, root: Path) -> None:
    """Update version in Cargo.toml"""
    cargo_path = root / "Cargo.toml"
    content = cargo_path.read_text()
    updated = re.sub(
        r'^version = "[^"]*"',
        f'version = "{version}"',
        content,
        count=1,
        flags=re.MULTILINE
    )
    cargo_path.write_text(updated)
    print(f"✓ Updated {cargo_path}")


def update_setup_py(version: str, root: Path) -> None:
    """Update version in bindings/python/setup.py"""
    setup_path = root / "bindings" / "python" / "setup.py"
    content = setup_path.read_text()
    updated = re.sub(
        r'version="[^"]*"',
        f'version="{version}"',
        content
    )
    setup_path.write_text(updated)
    print(f"✓ Updated {setup_path}")


def update_package_json(version: str, root: Path, path: Path) -> None:
    """Update version in a package.json file"""
    pkg_path = root / path / "package.json"
    with open(pkg_path, 'r') as f:
        data = json.load(f)
    
    data['version'] = version
    
    with open(pkg_path, 'w') as f:
        json.dump(data, f, indent=2)
        f.write('\n')  # Add trailing newline
    
    print(f"✓ Updated {pkg_path}")


def main():
    if len(sys.argv) != 2:
        print("Usage: python scripts/bump_version.py 0.1.0")
        sys.exit(1)
    
    version = sys.argv[1]
    
    # Validate version format (basic semver)
    if not re.match(r'^\d+\.\d+\.\d+$', version):
        print(f"x Invalid version format: {version}")
        print("Expected format: X.Y.Z (e.g., 0.1.0)")
        sys.exit(1)
    
    root = Path(__file__).parent.parent
    
    print(f"\nBumping version to {version}...")
    print()
    
    try:
        # Update Cargo.toml
        update_cargo_toml(version, root)
        
        # Update Python setup.py
        update_setup_py(version, root)
        
        # Update npm package.json
        update_package_json(version, root, Path("pkg"))
        
        # Update VS Code extension package.json
        update_package_json(version, root, Path("vscode-extension"))
        
        print()
        print(f"Version bumped to {version}")
        print()
        print("Next steps:")
        print("  1. Review changes: git diff")
        print(f"  2. Commit: git commit -am 'chore: bump version to {version}'")
        print(f"  3. Tag: git tag v{version}")
        print("  4. Push: git push origin main --tags")
        print()
        print("GitHub Actions will automatically publish to PyPI and npm when you push the tag.")
        
    except FileNotFoundError as e:
        print(f"x Error: {e}")
        print("Make sure you're running this from the repository root.")
        sys.exit(1)
    except Exception as e:
        print(f"x Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

