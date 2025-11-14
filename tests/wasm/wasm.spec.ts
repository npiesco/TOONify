import { test, expect } from '@playwright/test';

test.describe.configure({ mode: 'serial' });

test.describe('TOONify WASM Integration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/tests/wasm/test.html');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM module to load
    await page.waitForFunction(() => {
      return document.body.getAttribute('data-all-tests-complete') === 'true';
    }, { timeout: 15000 });
  });

  test('should load WASM module successfully', async ({ page }) => {
    const status = await page.textContent('#status');
    expect(status).toBe('All tests completed');
  });

  test('should convert JSON to TOON', async ({ page }) => {
    const success = await page.getAttribute('body', 'data-json-to-toon-success');
    expect(success).toBe('true');
    
    const result = await page.getAttribute('body', 'data-toon-result');
    expect(result).toBeTruthy();
    expect(result).toContain('name');
    expect(result).toContain('age');
    expect(result).toContain('Alice');
  });

  test('should convert TOON to JSON', async ({ page }) => {
    const success = await page.getAttribute('body', 'data-toon-to-json-success');
    expect(success).toBe('true');
    
    const result = await page.getAttribute('body', 'data-json-result');
    expect(result).toBeTruthy();
    
    const parsed = JSON.parse(result!);
    expect(parsed.name).toBe('Bob');
    expect(parsed.age).toBe(25);
  });

  test('should handle roundtrip conversion correctly', async ({ page }) => {
    const success = await page.getAttribute('body', 'data-roundtrip-success');
    expect(success).toBe('true');
  });

  test('should handle errors gracefully', async ({ page }) => {
    const success = await page.getAttribute('body', 'data-error-handling-success');
    expect(success).toBe('true');
  });

  test('should handle complex nested objects', async ({ page }) => {
    const success = await page.getAttribute('body', 'data-complex-success');
    expect(success).toBe('true');
    
    const result = await page.getAttribute('body', 'data-complex-result');
    expect(result).toBeTruthy();
    
    // Just verify it parses as valid JSON
    const parsed = JSON.parse(result!);
    expect(parsed).toBeTruthy();
  });

  test('should perform conversions via page.evaluate', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const wasmModule = await import('../../pkg/toonify.js');
      await wasmModule.default();
      const { json_to_toon, toon_to_json } = wasmModule;
      
      const testJson = JSON.stringify({ foo: 'bar', count: 42 });
      const toon = json_to_toon(testJson);
      const backToJson = toon_to_json(toon);
      
      // Check that values are preserved
      const originalParsed = JSON.parse(testJson);
      const roundtripParsed = JSON.parse(backToJson);
      
      return {
        toon: toon,
        originalFoo: originalParsed.foo,
        roundtripFoo: roundtripParsed.foo,
        originalCount: originalParsed.count,
        roundtripCount: roundtripParsed.count
      };
    });
    
    expect(result.toon).toContain('foo');
    expect(result.toon).toContain('bar');
    expect(result.originalFoo).toBe(result.roundtripFoo);
    expect(result.originalCount).toBe(result.roundtripCount);
  });

  test('should handle 100 conversions quickly', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const wasmModule = await import('../../pkg/toonify.js');
      await wasmModule.default();
      const { json_to_toon } = wasmModule;
      
      const start = Date.now();
      for (let i = 0; i < 100; i++) {
        const json = JSON.stringify({ id: i, name: `User${i}`, active: true });
        json_to_toon(json);
      }
      const elapsed = Date.now() - start;
      
      return { count: 100, elapsed };
    });
    
    expect(result.count).toBe(100);
    expect(result.elapsed).toBeLessThan(2000); // Should complete in under 2 seconds
  });

  test('should handle large payloads', async ({ page }) => {
    const result = await page.evaluate(async () => {
      const wasmModule = await import('../../pkg/toonify.js');
      await wasmModule.default();
      const { json_to_toon, toon_to_json } = wasmModule;
      
      // Create a large object with 1000 fields
      const largeObj: any = {};
      for (let i = 0; i < 1000; i++) {
        largeObj[`field${i}`] = `value${i}`;
      }
      
      const json = JSON.stringify(largeObj);
      const toon = json_to_toon(json);
      const backToJson = toon_to_json(toon);
      
      // Check that all fields are preserved (values match)
      const originalParsed = JSON.parse(json);
      const roundtripParsed = JSON.parse(backToJson);
      let allFieldsMatch = true;
      for (let i = 0; i < 1000; i++) {
        if (originalParsed[`field${i}`] !== roundtripParsed[`field${i}`]) {
          allFieldsMatch = false;
          break;
        }
      }
      
      return {
        originalSize: json.length,
        toonSize: toon.length,
        allFieldsMatch: allFieldsMatch
      };
    });
    
    expect(result.allFieldsMatch).toBe(true);
    expect(result.originalSize).toBeGreaterThan(0);
    expect(result.toonSize).toBeGreaterThan(0);
  });

  test('should have no console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.reload();
    await page.waitForLoadState('networkidle');
    await page.waitForFunction(() => {
      return document.body.getAttribute('data-all-tests-complete') === 'true';
    }, { timeout: 15000 });
    
    expect(errors).toHaveLength(0);
  });

  test.afterEach(async ({ page }) => {
    // Cleanup
    await page.evaluate(() => {
      // Clear any test state if needed
    }).catch(() => {});
  });
});

