import * as assert from 'node:assert/strict';
import test from 'node:test';

import { normalizeControllerIdentitySeed } from './controllerIdentity.ts';

test('accepts and copies an exact 32-byte identity seed', () => {
  const seed = Array.from({ length: 32 }, (_, index) => index);
  const normalized = normalizeControllerIdentitySeed(seed);

  assert.deepEqual(normalized, seed);
  assert.notEqual(normalized, seed);
});

test('rejects missing and incorrectly sized identity seeds', () => {
  assert.equal(normalizeControllerIdentitySeed(null), null);
  assert.equal(normalizeControllerIdentitySeed(new Array(31).fill(0)), null);
  assert.equal(normalizeControllerIdentitySeed(new Array(33).fill(0)), null);
});

test('rejects non-byte identity seed entries', () => {
  const invalidValues = [-1, 256, 1.5, '1', Number.NaN];
  for (const invalid of invalidValues) {
    const seed: unknown[] = new Array(32).fill(0);
    seed[10] = invalid;
    assert.equal(normalizeControllerIdentitySeed(seed), null);
  }
});