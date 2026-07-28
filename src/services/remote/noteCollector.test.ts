import * as assert from 'node:assert/strict';
import test from 'node:test';

import {
  checkQuotas,
  collect,
  extensionOf,
  isExternal,
  looksLikeAttachment,
  MAX_FILE_BYTES,
  type LinkSource,
  type ResolvedLink,
} from './noteCollector.ts';

function source(
  rootNotePath: string,
  links: ResolvedLink[],
  sizes: Record<string, number>
): LinkSource {
  return {
    rootNotePath,
    sizeOf: (p) => (p in sizes ? sizes[p] : null),
    links: () => links,
  };
}

test('the root note is always the first entry', () => {
  const result = collect(source('notes/demo.md', [], { 'notes/demo.md': 42 }));
  assert.equal(result.ok, true);
  assert.deepEqual(result.files, [{ index: 0, relativePath: 'notes/demo.md', size: 42 }]);
});

test('resolved attachments follow the root note in order', () => {
  const result = collect(
    source(
      'notes/demo.md',
      [
        { raw: 'diagram.png', resolved: 'assets/diagram.png' },
        { raw: 'data.csv', resolved: 'assets/data.csv' },
      ],
      { 'notes/demo.md': 10, 'assets/diagram.png': 200, 'assets/data.csv': 30 }
    )
  );

  assert.equal(result.ok, true);
  assert.deepEqual(
    result.files.map((f) => f.relativePath),
    ['notes/demo.md', 'assets/diagram.png', 'assets/data.csv']
  );
  assert.deepEqual(result.files.map((f) => f.index), [0, 1, 2]);
});

test('an unresolved link to a note is ignored, not fatal', () => {
  // This is the rule that makes the feature usable at all: linking to a note
  // that does not exist yet is normal in Obsidian.
  const result = collect(
    source(
      'notes/demo.md',
      [{ raw: 'Some Future Note', resolved: null }],
      { 'notes/demo.md': 10 }
    )
  );

  assert.equal(result.ok, true);
  assert.equal(result.files.length, 1);
  assert.ok(result.skipped.some((s) => s.includes('Some Future Note')));
});

test('an unresolved attachment is fatal', () => {
  const result = collect(
    source('notes/demo.md', [{ raw: 'missing.png', resolved: null }], { 'notes/demo.md': 10 })
  );

  assert.equal(result.ok, false);
  assert.match(result.error ?? '', /missing\.png/);
});

test('links to other markdown files are not recursed', () => {
  const result = collect(
    source(
      'notes/demo.md',
      [{ raw: 'other', resolved: 'notes/other.md' }],
      { 'notes/demo.md': 10, 'notes/other.md': 20 }
    )
  );

  assert.equal(result.ok, true);
  assert.equal(result.files.length, 1, 'only the root note travels');
});

test('external targets are skipped', () => {
  const result = collect(
    source(
      'notes/demo.md',
      [
        { raw: 'https://example.com/a.png', resolved: null },
        { raw: 'http://example.com/b.png', resolved: null },
        { raw: 'data:image/png;base64,AAAA', resolved: null },
      ],
      { 'notes/demo.md': 10 }
    )
  );

  assert.equal(result.ok, true, result.error);
  assert.equal(result.files.length, 1);
  assert.equal(result.skipped.length, 3);
});

test('the same attachment referenced twice is sent once', () => {
  const result = collect(
    source(
      'notes/demo.md',
      [
        { raw: 'a.png', resolved: 'assets/a.png' },
        { raw: 'assets/a.png', resolved: 'assets/a.png' },
      ],
      { 'notes/demo.md': 10, 'assets/a.png': 5 }
    )
  );

  assert.equal(result.ok, true);
  assert.equal(result.files.length, 2);
});

test('an unsafe attachment path fails the whole batch', () => {
  const result = collect(
    source(
      'notes/demo.md',
      [{ raw: 'x', resolved: '../outside.png' }],
      { 'notes/demo.md': 10, '../outside.png': 5 }
    )
  );

  assert.equal(result.ok, false);
  assert.match(result.error ?? '', /climb out/);
});

test('a vanished root note is reported clearly', () => {
  const result = collect(source('notes/gone.md', [], {}));
  assert.equal(result.ok, false);
  assert.match(result.error ?? '', /no longer exists/);
});

test('attachment detection keys on the extension', () => {
  assert.equal(looksLikeAttachment('diagram.png'), true);
  assert.equal(looksLikeAttachment('report.pdf'), true);
  assert.equal(looksLikeAttachment('notes/other.md'), false);
  assert.equal(looksLikeAttachment('Some Note'), false);
  assert.equal(looksLikeAttachment('image.png#fragment'), true);
  assert.equal(looksLikeAttachment('weird.unknownext'), false);
});

test('external detection does not misfire on windows drive letters', () => {
  assert.equal(isExternal('https://example.com'), true);
  assert.equal(isExternal('data:text/plain,hi'), true);
  assert.equal(isExternal('notes/demo.md'), false);
  assert.equal(isExternal('C:/Windows/x.png'), false, 'a drive letter is a path, not a scheme');
});

test('extensions are read past fragments and queries', () => {
  assert.equal(extensionOf('a/b/c.PNG'), 'png');
  assert.equal(extensionOf('a/b/c.png#anchor'), 'png');
  assert.equal(extensionOf('a/b/c.png?v=2'), 'png');
  assert.equal(extensionOf('noext'), '');
  assert.equal(extensionOf('.hidden'), '', 'a dotfile has no extension');
});

test('quotas are enforced before anything is sent', () => {
  assert.equal(checkQuotas([{ index: 0, relativePath: 'a.md', size: 10 }]).ok, true);

  const tooBig = checkQuotas([
    { index: 0, relativePath: 'a.md', size: 10 },
    { index: 1, relativePath: 'big.bin', size: MAX_FILE_BYTES + 1 },
  ]);
  assert.equal(tooBig.ok, false);
  assert.match(tooBig.error ?? '', /per-file limit/);

  const tooMany = checkQuotas(
    Array.from({ length: 257 }, (_, i) => ({ index: i, relativePath: `f${i}.png`, size: 1 }))
  );
  assert.equal(tooMany.ok, false);
  assert.match(tooMany.error ?? '', /limit is 256/);

  const tooLarge = checkQuotas(
    Array.from({ length: 5 }, (_, i) => ({
      index: i,
      relativePath: `f${i}.bin`,
      size: MAX_FILE_BYTES,
    }))
  );
  assert.equal(tooLarge.ok, false);
  assert.match(tooLarge.error ?? '', /256 MiB/);
});
