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
    from toonify import json_to_toon, toon_to_json, CachedConverter, ToonError
except ImportError as e:
    print(f"✗ Failed to import toonify: {e}")
    print("\nMake sure the package is installed:")
    print("   pip install toonifypy")
    print("\nOr install from source:")
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
    
    # Example 4: Using CachedConverter for performance
    print("Example 4: CachedConverter (Moka + Sled)")
    print("-" * 60)
    
    try:
        import time
        import tempfile
        import os
        
        # Create temporary directory for Sled cache
        temp_dir = tempfile.mkdtemp()
        sled_path = os.path.join(temp_dir, "toon_cache.db")
        
        print(f"Using Sled database: {sled_path}")
        print()
        
        # Create cached converter with Moka (100 entries) + Sled (persistent)
        converter = CachedConverter(
            cache_size=100,
            cache_ttl_secs=None,  # No TTL (cache forever)
            persistent_path=sled_path
        )
        
        test_json = '{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}'
        
        # First conversion (cache miss)
        print("First conversion (cache miss):")
        start = time.time()
        result1 = converter.json_to_toon(test_json)
        time1 = (time.time() - start) * 1000
        print(f"  Time: {time1:.2f}ms")
        print(f"  Result: {result1[:50]}...")
        print()
        
        # Second conversion (cache hit from Moka)
        print("Second conversion (cache hit from Moka):")
        start = time.time()
        result2 = converter.json_to_toon(test_json)
        time2 = (time.time() - start) * 1000
        print(f"  Time: {time2:.2f}ms")
        print(f"  Speedup: {time1/time2:.1f}x faster!")
        print()
        
        # Show cache statistics
        print("Cache statistics:")
        print(converter.cache_stats())
        
        # Clear Moka cache (but keep Sled)
        print("\nClearing Moka cache...")
        converter.clear_cache()
        print(converter.cache_stats())
        
        # Third conversion (cache hit from Sled, warms up Moka)
        print("\nThird conversion (cache hit from Sled):")
        start = time.time()
        result3 = converter.json_to_toon(test_json)
        time3 = (time.time() - start) * 1000
        print(f"  Time: {time3:.2f}ms")
        print(f"  Result: {result3[:50]}...")
        print()
        
        print("Final cache statistics:")
        print(converter.cache_stats())
        
        # Cleanup
        import shutil
        shutil.rmtree(temp_dir)
        
        print("\n✓ Cached converter example completed!")
        
    except Exception as e:
        print(f"✗ Cached converter example failed: {e}")
        import traceback
        traceback.print_exc()
    
    print()
    print("=" * 60)
    print("✓ All examples completed successfully!")
    print("=" * 60)


if __name__ == "__main__":
    main()

