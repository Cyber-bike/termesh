import * as assert from 'node:assert/strict';
import test from 'node:test';

import { collectRecursive } from './noteCollectorRecursive.ts';
import type { LinkSource, ResolvedLink } from './noteCollector.ts';

/** A tiny fake vault: each note has its own links; sizes cover notes and attachments alike. */
function vault(notes: Record<string, ResolvedLink[]>, sizes: Record<string, number>) {
  return (path: string): LinkSource | null => {
    if (!(path in notes)) return null;
    return {
      rootNotePath: path,
      links: () => notes[path],
      sizeOf: (p) => (p in sizes ? sizes[p] : null),
    };
  };
}

test('a note with no links collects just itself', () => {
  const sourceFor = vault({ 'root.md': [] }, { 'root.md': 10 });
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, true);
  assert.deepEqual(result.files.map((f) => f.relativePath), ['root.md']);
});

test('a linked note is recursed into, including its own attachments', () => {
  const sourceFor = vault(
    {
      'root.md': [{ raw: 'b', resolved: 'b.md' }],
      'b.md': [{ raw: 'img', resolved: 'img.png' }],
    },
    { 'root.md': 10, 'b.md': 20, 'img.png': 5 }
  );
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, true);
  assert.deepEqual(result.files.map((f) => f.relativePath), ['root.md', 'b.md', 'img.png']);
});

test('recursion has no depth limit', () => {
  const chainLength = 20;
  const notes: Record<string, ResolvedLink[]> = {};
  const sizes: Record<string, number> = {};
  for (let i = 0; i < chainLength; i++) {
    const path = `n${i}.md`;
    const next = i + 1 < chainLength ? [{ raw: `n${i + 1}`, resolved: `n${i + 1}.md` }] : [];
    notes[path] = next;
    sizes[path] = 1;
  }
  const sourceFor = vault(notes, sizes);
  const result = collectRecursive(sourceFor('n0.md')!, sourceFor);
  assert.equal(result.ok, true);
  assert.equal(result.files.length, chainLength);
});

test('a cycle between linked notes does not loop forever and each note appears once', () => {
  const sourceFor = vault(
    {
      'root.md': [{ raw: 'a', resolved: 'a.md' }],
      'a.md': [{ raw: 'b', resolved: 'b.md' }],
      'b.md': [{ raw: 'a', resolved: 'a.md' }],
    },
    { 'root.md': 1, 'a.md': 1, 'b.md': 1 }
  );
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, true);
  assert.deepEqual(result.files.map((f) => f.relativePath).sort(), ['a.md', 'b.md', 'root.md']);
});

test('a note that no longer exists is skipped with a reason, not fatal', () => {
  const sourceFor = vault(
    {
      'root.md': [
        { raw: 'gone', resolved: 'gone.md' },
        { raw: 'b', resolved: 'b.md' },
      ],
      'b.md': [],
    },
    { 'root.md': 1, 'b.md': 1 }
  );
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, true);
  assert.deepEqual(result.files.map((f) => f.relativePath).sort(), ['b.md', 'root.md']);
  assert.equal(result.skippedNotes.length, 1);
  assert.equal(result.skippedNotes[0].path, 'gone.md');
  assert.match(result.skippedNotes[0].reason, /no longer exists/);
});

test('a linked note with a broken attachment is skipped with a reason, the rest keeps going', () => {
  const sourceFor = vault(
    {
      'root.md': [
        { raw: 'broken', resolved: 'broken.md' },
        { raw: 'b', resolved: 'b.md' },
      ],
      'broken.md': [{ raw: 'missing.png', resolved: null }],
      'b.md': [],
    },
    { 'root.md': 1, 'broken.md': 1, 'b.md': 1 }
  );
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, true);
  assert.deepEqual(result.files.map((f) => f.relativePath).sort(), ['b.md', 'root.md']);
  assert.equal(result.skippedNotes.length, 1);
  assert.equal(result.skippedNotes[0].path, 'broken.md');
  assert.match(result.skippedNotes[0].reason, /missing\.png/);
});

test('an attachment referenced by two different linked notes is sent once', () => {
  const sourceFor = vault(
    {
      'root.md': [
        { raw: 'a', resolved: 'a.md' },
        { raw: 'b', resolved: 'b.md' },
      ],
      'a.md': [{ raw: 'shared', resolved: 'shared.png' }],
      'b.md': [{ raw: 'shared', resolved: 'shared.png' }],
    },
    { 'root.md': 1, 'a.md': 1, 'b.md': 1, 'shared.png': 5 }
  );
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, true);
  const sharedCount = result.files.filter((f) => f.relativePath === 'shared.png').length;
  assert.equal(sharedCount, 1);
});

test("the root note's own broken attachment is still fatal, unchanged from v2.0", () => {
  const sourceFor = vault({ 'root.md': [{ raw: 'missing.png', resolved: null }] }, { 'root.md': 1 });
  const result = collectRecursive(sourceFor('root.md')!, sourceFor);
  assert.equal(result.ok, false);
  assert.match(result.error ?? '', /missing\.png/);
});
