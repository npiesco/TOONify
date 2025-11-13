#!/usr/bin/env python3
"""
TOONify Python Example

This example demonstrates how to use the TOONify library from Python
using UniFFI-generated bindings.

Installation:
1. Build: cargo build --lib --release
2. Generate bindings: cargo run --bin uniffi-bindgen -- generate --library target/release/libtoonify.dylib --language python --out-dir bindings/python
3. Copy library: cp target/release/libtoonify.dylib bindings/python/
4. Install: pip install -e bindings/python/

Running:
  python3 examples/python_example.py
"""

try:
    from toonify import json_to_toon, toon_to_json, ToonError
except ImportError as e:
    print(f"✗ Failed to import toonify: {e}")
    print("\nMake sure the package is installed:")
    print("   pip install -e bindings/python/")
    print("\nOr if not installed, ensure bindings are generated:")
    print("   1. cargo build --lib --release")
    print("   2. cargo run --bin uniffi-bindgen -- generate --library target/release/libtoonify.dylib --language python --out-dir bindings/python")
    print("   3. cp target/release/libtoonify.dylib bindings/python/")
    print("   4. pip install -e bindings/python/")
    import sys
    sys.exit(1)


def main():
    print("=" * 60)
    print("TOONify Python Example")
    print("=" * 60)
    print()
    
    # Example 1: Simple JSON to TOON conversion
    print("Example 1: JSON to TOON")
    print("-" * 60)
    
    json_data = """{
  "users": [
    {
      "id": 1,
      "name": "Sreeni",
      "role": "admin",
      "email": "sreeni@example.com"
    },
    {
      "id": 2,
      "name": "Krishna",
      "role": "admin",
      "email": "krishna@example.com"
    }
  ]
}"""
    
    print("Input JSON:")
    print(json_data)
    print()
    
    try:
        toon_data = json_to_toon(json_data)
        print("✓ Converted to TOON:")
        print(toon_data)
        print()
        
        # Calculate token savings (approximate)
        json_tokens = len(json_data.split())
        toon_tokens = len(toon_data.split())
        savings = ((json_tokens - toon_tokens) / json_tokens) * 100
        print(f"Approximate token savings: {savings:.1f}%")
        print(f"   JSON: ~{json_tokens} tokens")
        print(f"   TOON: ~{toon_tokens} tokens")
        
    except ToonError as e:
        print(f"✗ Conversion failed: {e}")
        return
    
    print()
    print("=" * 60)
    
    # Example 2: TOON to JSON conversion
    print("Example 2: TOON to JSON")
    print("-" * 60)
    
    toon_input = """products[2]{id,name,price,inStock}:
1,Laptop,999.99,true
2,Mouse,29.99,false"""
    
    print("Input TOON:")
    print(toon_input)
    print()
    
    try:
        json_output = toon_to_json(toon_input)
        print("✓ Converted to JSON:")
        print(json_output)
    except ToonError as e:
        print(f"✗ Conversion failed: {e}")
        return
    
    print()
    print("=" * 60)
    
    # Example 3: Round-trip conversion
    print("Example 3: Round-trip (JSON -> TOON -> JSON)")
    print("-" * 60)
    
    original_json = """{
  "metadata": {
    "version": "1.0.0",
    "timestamp": "2024-11-13T10:30:00Z"
  }
}"""
    
    print("Original JSON:")
    print(original_json)
    print()
    
    try:
        # Convert to TOON
        toon = json_to_toon(original_json)
        print("TOON intermediate:")
        print(toon)
        print()
        
        # Convert back to JSON
        final_json = toon_to_json(toon)
        print("✓ Final JSON:")
        print(final_json)
        
        # Verify semantic equivalence (would need json module for proper comparison)
        import json
        original_obj = json.loads(original_json)
        final_obj = json.loads(final_json)
        
        if original_obj == final_obj:
            print("\n✓ Round-trip successful! Data preserved.")
        else:
            print("\n! Round-trip completed but data differs (this may be due to formatting)")
            
    except ToonError as e:
        print(f"✗ Conversion failed: {e}")
        return
    except json.JSONDecodeError as e:
        print(f"✗ JSON parsing failed: {e}")
        return
    
    print()
    print("=" * 60)
    print("✓ All examples completed successfully!")
    print("=" * 60)


if __name__ == "__main__":
    main()

