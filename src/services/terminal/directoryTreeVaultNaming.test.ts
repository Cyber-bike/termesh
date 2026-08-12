import * as assert from 'node:assert/strict';
import test from 'node:test';

import { resolveUniqueVaultPath } from './directoryTreeVaultNaming.ts';

test('returns the desired path unchanged when free', () => {
  const path = resolveUniqueVaultPath('notes/readme.md', () => false);
  assert.equal(path, 'notes/readme.md');
});

test('appends " (2)" before the extension on first conflict', () => {
  const taken = new Set(['notes/readme.md']);
  const path = resolveUniqueVaultPath('notes/readme.md', (p) => taken.has(p));
  assert.equal(path, 'notes/readme (2).md');
});

test('keeps incrementing until a free suffix is found', () => {
  const taken = new Set([
    'notes/readme.md',
    'notes/readme (2).md',
    'notes/readme (3).md',
  ]);
  const path = resolveUniqueVaultPath('notes/readme.md', (p) => taken.has(p));
  assert.equal(path, 'notes/readme (4).md');
});

test('handles paths with no directory component', () => {
  const taken = new Set(['readme.md']);
  const path = resolveUniqueVaultPath('readme.md', (p) => taken.has(p));
  assert.equal(path, 'readme (2).md');
});

test('handles extensionless names (e.g. a folder or a dotfile)', () => {
  const taken = new Set(['assets/diagram']);
  const path = resolveUniqueVaultPath('assets/diagram', (p) => taken.has(p));
  assert.equal(path, 'assets/diagram (2)');

  const takenDotfile = new Set(['.gitignore']);
  const dotfilePath = resolveUniqueVaultPath('.gitignore', (p) => takenDotfile.has(p));
  assert.equal(dotfilePath, '.gitignore (2)');
});
